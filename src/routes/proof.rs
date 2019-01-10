use std::time::{SystemTime, UNIX_EPOCH};

use blake2::crypto_mac::Mac;
use blake2::Blake2b;
use rocket::http::Status;
use rocket::post;
use rocket::request::Form;

use crate::db::DbConn;
use crate::models::leaderboard::Entry;
use crate::models::proof::ProofResponse;

#[post("/proof", data = "<proof>")]
pub fn proof(conn: DbConn, proof: Form<ProofResponse>) -> Status {
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
        Ok(_) => match Entry::insert(&proof.prover_id, repl_time, &conn) {
            Ok(_) => Status::Ok,
            Err(err) => {
                println!("upsert error: {:?}", err);
                Status::InternalServerError
            }
        },
    }
}
