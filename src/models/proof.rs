use rocket::FromForm;
use serde_derive::{Deserialize, Serialize};

#[derive(FromForm, Serialize, Deserialize)]
pub struct ProofResponse {
    pub prover_id: String,
    pub ts: u128,
    pub seed: String,
    // TODO proof
}
