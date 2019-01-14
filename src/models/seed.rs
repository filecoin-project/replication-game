use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Seed {
    pub timestamp: i32,
    pub seed: String,
}
