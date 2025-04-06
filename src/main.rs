use std::{env, io};
use std::fs::File;
use std::io::BufReader;
use rustls::ServerConfig;
use rustls_pemfile::{certs, pkcs8_private_keys};

use actix_web::{web, App, HttpServer};
use actix_web::http::header;
use diesel::{r2d2::{self, ConnectionManager}, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenv::dotenv;
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use rustls::{ServerConfig, Certificate, PrivateKey};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::io::BufReader;
use std::fs::File;

mod models;
mod schema;
mod auth;
mod errors;
mod handlers;

type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenv().ok();
    env_logger::init();
    
    // Configuración de la base de datos
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");
    
    // Ejecutar migraciones
    {
        let mut conn = pool.get().expect("Couldn't get db connection from pool");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
    }
    
    // Configuración TLS
    let rustls_config = load_rustls_config();
    
    // Configuración del servidor
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(
                Cors::permissive()
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .supports_credentials()
                    .max_age(3600),
            )
            .service(
                web::scope("/api")
                    .service(
                        web::scope("/auth")
                            .route("/register", web::post().to(handlers::register))
                            .route("/login", web::post().to(handlers::login)),
                    )
                    .service(
                        web::scope("/notes")
                            .wrap(auth::CheckLogin) // Middleware de autenticación
                            .route("", web::get().to(handlers::get_notes))
                            .route("", web::post().to(handlers::create_note))
                            .route("/{id}", web::get().to(handlers::get_note))
                            .route("/{id}", web::put().to(handlers::update_note))
                            .route("/{id}", web::delete().to(handlers::delete_note)),
                    )
            )
    })
    .bind_openssl("0.0.0.0:8443", builder)?
    .bind_rustls("0.0.0.0:8443", tls_config)?
    .run()
    .await
}

fn load_rustls_config() -> std::io::Result<ServerConfig> {
    let cert_file = &mut BufReader::new(File::open("cert.pem")?);
    let key_file = &mut BufReader::new(File::open("key.pem")?);

    let cert_chain = certs(cert_file)?
        .into_iter()
        .map(Certificate)
        .collect();
    let mut keys = pkcs8_private_keys(key_file)?;

    ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, PrivateKey(keys.remove(0)))
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))
}