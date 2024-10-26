use std::path::Path;

use actix_web::App;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use actix_cors::Cors;
use clap::Parser;

// use actix_files as fs;
use actix_web::{
    middleware::{self, DefaultHeaders},
    web,
    web::post,
    HttpResponse, HttpServer, Result,
};

// use std::path::Path;

// use actix_web::{web, App, HttpServer, middleware::DefaultHeaders};
// use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

// use actix_cors::Cors;
// use clap::Parser;

use actix_files::{Files, NamedFile};

use backend::{route_compile, Opts};

pub struct FrontendState {
    pub frontend_folder: String,
}

// pub fn route_frontend(at: &str, dir: &str) -> actix_files::Files {
//     fs::Files::new(at, dir).index_file("index.html")
// }

// pub async fn route_frontend_version(data: web::Data<FrontendState>) -> Result<actix_files::NamedFile> {
//     Ok(fs::NamedFile::open(
//         Path::new(&data.frontend_folder).join("index.html"),
//     )?)
// }

pub fn route_frontend(at: &str, dir: &str) -> Files {
    Files::new(at, dir).index_file("index.html")
}

pub async fn route_frontend_version(data: web::Data<FrontendState>) -> Result<NamedFile> {
    Ok(NamedFile::open(Path::new(&data.frontend_folder).join("index.html"))?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let opts: Opts = Opts::parse();

    let port = opts.port;
    let host = opts.host.clone();

    if let Some(path) = &opts.frontend_folder {
        if !Path::new(path).is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Frontend folder not found: {}", path),
            ));
        }
    }

    async fn health() -> HttpResponse {
        HttpResponse::Ok().finish()
    }
    // Setup OpenSSL
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("/home/ahmadsameh/privkey1.pem", SslFiletype::PEM)
        .unwrap();
    builder
        .set_certificate_chain_file("/home/ahmadsameh/fullchain.pem")
        .unwrap();
    // builder.set_options(openssl::ssl::SslOptions::NO_TLSV1 | openssl::ssl::SslOptions::NO_TLSV1_1);

    HttpServer::new(move || {
        let opts: Opts = opts.clone();
        let frontend_folder = opts.frontend_folder.clone();

        //ahmads edit
        // Configure CORS
        let cors = Cors::default()
            .allowed_origin("https://solangpg.ddnsfree.com") // Replace with your frontend's URL
            .allowed_methods(vec!["GET", "POST", "OPTIONS"])
            .allowed_headers(vec![actix_web::http::header::CONTENT_TYPE,
                //newly added
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::ORIGIN,
                actix_web::http::header::ACCEPT,])
            .allow_any_header()
            .max_age(3600)
            .supports_credentials();

        let mut app = App::new()
            .wrap(cors) // Apply CORS here
            .service(web::resource("/health").to(health))
            // Enable GZIP compression
            .wrap(middleware::Compress::default())
            .wrap(
                DefaultHeaders::new()
                    .add(("Cross-Origin-Opener-Policy", "same-origin"))
                    .add(("Cross-Origin-Embedder-Policy", "require-corp")),
            )
            .route("/compile", post().to(|body| route_compile(body)));

        // Serve frontend files if configured via CLI
        match frontend_folder {
            Some(path) => {
                app = app
                    .app_data(web::Data::new(FrontendState {
                        frontend_folder: path.clone(),
                    }))
                    .route("/v{tail:.*}", web::get().to(route_frontend_version))
                    .service(route_frontend("/", path.as_ref()));
            },
            None => {
                println!(
                    "Warning: Starting backend without serving static frontend files due to missing configuration."
                )
            },
        }

        app
    })
    // .bind(format!("{}:{}", &host, &port))?
    .bind(format!("{}:{}"), "127.0.0.1","9000"),
    .run()
    .await?;

    Ok(())
}
