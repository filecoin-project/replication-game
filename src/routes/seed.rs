use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use blake2::crypto_mac::Mac;
use blake2::Blake2b;
use rand::Rng;
use rocket::get;
use rocket_contrib::json::Json;

use crate::error::ApiResult;
use crate::models::seed::Seed;

#[get("/seed")]
pub fn seed() -> ApiResult<Json<Seed>> {
    // Get current timestamp
    let ts = {
        let start = SystemTime::now();
        let timestamp = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        timestamp.as_secs() as i32
    };

    // take the mac of the timestamp
    let key = env::var("GAME_KEY").unwrap_or_else(|_| "my cool key".into());
    let mut hasher = Blake2b::new_varkey(key.as_bytes())?;
    hasher.input(format!("{}", ts).as_bytes());
    let result = hasher.result();
    let code_bytes = result.code().to_vec();
    let mut challenge_seed = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut challenge_seed);

    Ok(Json(Seed {
        timestamp: ts,
        seed: hex::encode(&code_bytes),
        challenge_seed: hex::encode(&challenge_seed),
    }))
}
