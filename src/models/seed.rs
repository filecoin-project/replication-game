use serde::{Deserialize, Serialize};
use storage_proofs::hasher::blake2s::Blake2sDomain;
use storage_proofs::hasher::pedersen::PedersenDomain;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Seed {
    /// Timestamp.
    pub timestamp: i32,
    /// Additional data, to be mixed in.
    pub data: Blake2sDomain,
    /// Hex encoded mac(timestamp, data).
    pub mac: String,
}

#[derive(Default, Clone, Debug, Deserialize, Serialize)]
pub struct SeedInput {
    /// Additional input data, to be mixed in.
    pub data: Blake2sDomain,
}
