#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel;
#[cfg(feature = "postgres")]
#[macro_use]
extern crate diesel_migrations;

pub mod models;
pub mod proofs;

#[cfg(feature = "postgres")]
mod db;
mod error;
#[cfg(feature = "postgres")]
mod gzip;
#[cfg(feature = "postgres")]
mod routes;
#[cfg(feature = "postgres")]
mod schema;

#[cfg(test)]
mod tests;

#[cfg(feature = "postgres")]
use rocket::fairing::AdHoc;
#[cfg(feature = "postgres")]
use rocket::{catchers, routes, Rocket};
#[cfg(feature = "postgres")]
use rocket_contrib::serve::StaticFiles;

#[cfg(feature = "postgres")]
use crate::db::DbConn;

// This macro from `diesel_migrations` defines an `embedded_migrations` module
// containing a function named `run`. This allows the example to be run and
// tested without any outside setup of the database.
#[cfg(feature = "postgres")]
embed_migrations!();

#[cfg(feature = "postgres")]
pub fn rocket() -> (Rocket, Option<DbConn>) {
    let rocket = rocket::ignite()
        .attach(DbConn::fairing())
        .attach(gzip::GzipFairing)
        .attach(AdHoc::on_attach("Database Migrations", |rocket| {
            let conn = DbConn::get_one(&rocket).expect("database connection");
            match embedded_migrations::run(&*conn) {
                Ok(()) => Ok(rocket),
                Err(e) => {
                    println!("Error: Failed to run database migrations: {:?}", e);
                    Err(rocket)
                }
            }
        }))
        .register(catchers![routes::catchers::not_found])
        .mount(
            "/api",
            routes![
                routes::index::index,
                routes::seed::seed,
                routes::proof::proof,
                routes::proof::proof_gz,
                routes::leaderboard::leaderboard
            ],
        )
        .mount("/", StaticFiles::from("./static"));

    let conn = if cfg!(test) {
        DbConn::get_one(&rocket)
    } else {
        None
    };

    (rocket, conn)
}
