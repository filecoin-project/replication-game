#![feature(proc_macro_hygiene, decl_macro)]
#![feature(duration_as_u128)]

#[macro_use] extern crate rocket;

use blake2::Blake2b;
use blake2::crypto_mac::Mac;
use hex;
use rocket::http::Status;
use std::io;
use std::time::{SystemTime, UNIX_EPOCH};
use rocket::request::{Form};
use rusqlite::{Connection, NO_PARAMS, Result as Res};

#[derive(FromForm)]
struct ProofResponse {
    prover_id: String,
    ts: u128,
    seed: String,
    // TODO proof
}

#[get("/")]
fn index() -> &'static str {
    "
    USAGE
      GET /seed
          Returns a timestamp and a seed separated by a line
          EXAMPLE: curl  http://localhost:8000/seed

      POST /proof
          Send your `ts` (timestamp), `seed` and `proof`,
          it verifies it and stores it, returns status 400 or 406
          EXAMPLE: curl -X POST -d \"ts=123123&seed=rhoch83q\"  http://localhost:8000/proof
    "
}

#[get("/seed")]
fn seed() -> io::Result<String> {
    
    // Get current timestamp
    let ts = {
        let start = SystemTime::now();
        let timestamp = start.duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        format!("{:?}", timestamp.as_millis())
    };

    // take the mac of the timestamp
    let mut hasher = Blake2b::new_varkey(b"my key").unwrap();
    hasher.input(ts.as_bytes());
    let result = hasher.result();
    let code_bytes = result.code().to_vec();
    let mac = hex::encode(&code_bytes);

    // return ts and mac
    let result = format!("{}\n{}", &ts, &mac);
    Ok(result)
}

#[post("/proof", data = "<proof>")]
fn proof(proof: Form<ProofResponse>) -> Status {
    // Get old timestamp
    let ts =  format!("{}", proof.ts);

    // Get replication time
    let repl_time = {
        // Get current timestamp
        let start = SystemTime::now();
        let timestamp = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
        let completion_time = timestamp.as_millis();
        completion_time - proof.ts
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
        Ok(_) => {
            // let prover_id = proof.prover_id.clone();
            // // upsert_repl_time(prover_id, repl_time).unwrap();
            Status::Ok
        },
    }
}

fn upsert_repl_time (prover_id: String, repl_time: u128) -> Res<()> {
    let conn = Connection::open("leaderboard.db")?;
    conn.execute(
        "INSERT INTO leaderboard(prover) VALUES('?1')
        ON CONFLICT(prover) DO UPDATE SET
            repl_time=?2
        WHERE excluded.repl_time > leaderboard.repl_time;",
        &[&prover_id, &format!("{}", repl_time)])?;

    Ok(())
}

fn main() -> Res<()> {
    let conn = Connection::open("leaderboard.db")?;
    conn.execute(
        "create table if not exists leaderboard (
            prover text primary key,
            repl_time integer not null
         )",
        NO_PARAMS,
    )?;

    rocket::ignite()
        .mount("/", routes![index, seed, proof])
        .launch();

    Ok(())
}