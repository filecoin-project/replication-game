use std::time::{SystemTime, UNIX_EPOCH};

use blake2::crypto_mac::Mac;
use blake2::Blake2b;
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
        timestamp.as_millis() as i32
    };

    // take the mac of the timestamp
    let mut hasher = Blake2b::new_varkey(b"my key")?;
    hasher.input(format!("{}", ts).as_bytes());
    let result = hasher.result();
    let code_bytes = result.code().to_vec();

    Ok(Json(Seed {
        timestamp: ts,
        seed: hex::encode(&code_bytes),
    }))
}
