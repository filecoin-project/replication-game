#![feature(proc_macro_hygiene, decl_macro, duration_as_u128)]

#[cfg(test)]
#[macro_use]
extern crate lazy_static;
#[cfg(test)]
extern crate parking_lot;
#[cfg(test)]
extern crate rand;

extern crate serde;
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate rocket_contrib;

use blake2::crypto_mac::Mac;
use blake2::Blake2b;
use diesel::prelude::*;
use diesel::SqliteConnection;
use hex;
use rocket::fairing::AdHoc;
use rocket::http::Status;
use rocket::request::Form;
use rocket::Rocket;
use rocket_contrib::json::{Json, JsonValue};
use std::time::{SystemTime, UNIX_EPOCH};

mod entry;
use self::entry::Entry;

#[cfg(test)]
mod tests;

// This macro from `diesel_migrations` defines an `embedded_migrations` module
// containing a function named `run`. This allows the example to be run and
// tested without any outside setup of the database.
embed_migrations!();

#[database("leaderboard")]
struct LeaderboardDbConn(SqliteConnection);

#[derive(FromForm)]
struct ProofResponse {
    prover_id: String,
    ts: u128,
    seed: String,
    // TODO proof
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Seed {
    timestamp: i32,
    seed: String,
}

#[get("/")]
fn index() -> &'static str {
    "
    USAGE
      GET /leaderboard
          Returns the current leaderboard as JSON

      GET /seed
          Returns a timestamp and a seed separated as JSON
          EXAMPLE: curl  http://localhost:8000/seed

      POST /proof
          Send your `ts` (timestamp), `seed` and `proof`,
          it verifies it and stores it, returns status 400 or 406
          EXAMPLE: curl -X POST -d \"prover_id=myid&ts=123123&seed=rhoch83q\"  http://localhost:8000/proof
    "
}

#[get("/seed")]
fn seed() -> Json<Seed> {
    // Get current timestamp
    let ts = {
        let start = SystemTime::now();
        let timestamp = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        timestamp.as_millis() as i32
    };

    // take the mac of the timestamp
    let mut hasher = Blake2b::new_varkey(b"my key").unwrap();
    hasher.input(format!("{}", ts).as_bytes());
    let result = hasher.result();
    let code_bytes = result.code().to_vec();

    Json(Seed {
        timestamp: ts,
        seed: hex::encode(&code_bytes),
    })
}

#[post("/proof", data = "<proof>")]
fn proof(conn: LeaderboardDbConn, proof: Form<ProofResponse>) -> Status {
    // Get old timestamp
    let ts = format!("{}", proof.ts);

    // Get replication time
    let repl_time = {
        // Get current timestamp
        let start = SystemTime::now();
        let timestamp = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let completion_time = timestamp.as_millis();
        (completion_time - proof.ts) as i32
    };

    // Verify authenticity of seed
    let ch = &proof.seed.clone();
    let mac = hex::decode(ch).unwrap();
    let mut hasher = Blake2b::new_varkey(b"my key").unwrap();
    hasher.input(&ts.as_bytes());
    let verification = hasher.verify(&mac);

    // TODO verify the proof

    match verification {
        Err(_) => Status::NotAcceptable,
        Ok(_) => match upsert_repl_time(conn, &proof.prover_id, repl_time) {
            Ok(_) => Status::Ok,
            Err(err) => {
                println!("upsert error: {:?}", err);
                Status::InternalServerError
            }
        },
    }
}

#[get("/leaderboard")]
fn leaderboard(conn: LeaderboardDbConn) -> QueryResult<Json<Vec<Entry>>> {
    let rows = Entry::all(&conn)?;
    Ok(Json(rows))
}

fn upsert_repl_time(conn: LeaderboardDbConn, prover_id: &str, repl_time: i32) -> QueryResult<()> {
    Entry::insert(prover_id, repl_time, &conn)
}

#[catch(404)]
fn not_found() -> JsonValue {
    json!({
        "status": "error",
        "reason": "Resource was not found."
    })
}

fn rocket() -> (Rocket, Option<LeaderboardDbConn>) {
    let rocket = rocket::ignite()
        .attach(LeaderboardDbConn::fairing())
        .attach(AdHoc::on_attach("Database Migrations", |rocket| {
            let conn = LeaderboardDbConn::get_one(&rocket).expect("database connection");
            match embedded_migrations::run(&*conn) {
                Ok(()) => Ok(rocket),
                Err(e) => {
                    println!("Error: Failed to run database migrations: {:?}", e);
                    Err(rocket)
                }
            }
        }))
        .register(catchers![not_found])
        .mount("/", routes![index, seed, proof, leaderboard]);

    let conn = if cfg!(test) {
        LeaderboardDbConn::get_one(&rocket)
    } else {
        None
    };

    (rocket, conn)
}

fn main() {
    rocket().0.launch();
}
