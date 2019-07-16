use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use blake2::crypto_mac::Mac;
use blake2::Blake2b;
use rocket::post;
use rocket_contrib::json::Json;
use storage_proofs::hasher::Domain;

use crate::error::ApiResult;
use crate::models::seed::{Seed, SeedInput};

#[post("/seed", format = "json", data = "<res>")]
pub fn seed(res: Json<SeedInput>) -> ApiResult<Json<Seed>> {
    // Get current timestamp
    let ts = {
        let start = SystemTime::now();
        let timestamp = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        timestamp.as_secs() as i32
    };

    // mac the timestamp with the data
    let key = env::var("GAME_KEY").unwrap_or_else(|_| "my cool key".into());
    let mut hasher = Blake2b::new_varkey(key.as_bytes())?;
    hasher.input(format!("{}", ts).as_bytes());
    hasher.input(&res.data.into_bytes());
    let result = hasher.result();

    Ok(Json(Seed {
        timestamp: ts,
        mac: hex::encode(result.code().as_ref()),
        data: res.data,
    }))
}
