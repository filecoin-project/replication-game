use std::time::{SystemTime, UNIX_EPOCH};

use blake2::crypto_mac::Mac;
use blake2::Blake2b;
use rocket::post;
use rocket_contrib::json::Json;

use crate::db::DbConn;
use crate::error::ApiResult;
use crate::models::leaderboard::Entry;
use crate::models::proof::ProofResponse;

#[post("/proof", format = "json", data = "<proof>")]
pub fn proof(conn: DbConn, proof: Json<ProofResponse>) -> ApiResult<()> {
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
    let mac = hex::decode(ch)?;
    let mut hasher = Blake2b::new_varkey(b"my key")?;
    hasher.input(&ts.as_bytes());
    hasher.verify(&mac)?;

    // TODO verify the proof

    Entry::insert(&proof.prover_id, repl_time, &conn)?;

    Ok(())
}
