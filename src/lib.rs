#![feature(proc_macro_hygiene, decl_macro, duration_as_u128)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod db;
mod error;
mod models;
mod routes;

#[cfg(test)]
mod tests;

use rocket::fairing::AdHoc;
use rocket::{catchers, routes, Rocket};

use crate::db::DbConn;

// This macro from `diesel_migrations` defines an `embedded_migrations` module
// containing a function named `run`. This allows the example to be run and
// tested without any outside setup of the database.
embed_migrations!();

pub fn rocket() -> (Rocket, Option<DbConn>) {
    let rocket = rocket::ignite()
        .attach(DbConn::fairing())
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
            "/",
            routes![
                routes::index::index,
                routes::seed::seed,
                routes::proof::proof,
                routes::leaderboard::leaderboard
            ],
        );

    let conn = if cfg!(test) {
        DbConn::get_one(&rocket)
    } else {
        None
    };

    (rocket, conn)
}
