use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Seed {
    pub timestamp: i32,
    pub seed: String,
    pub challenge_seed: String,
}
