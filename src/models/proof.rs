use diesel_derive_enum::DbEnum;
use serde_derive::{Deserialize, Serialize};
use storage_proofs::hasher::pedersen::PedersenDomain;
use storage_proofs::hasher::PedersenHasher;
use storage_proofs::{drgporep, layered_drgporep, porep};

use crate::models::seed::Seed;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub prover: String,
    pub seed: Seed,
    pub proof_params: Params,
    pub proof: Proof,
    pub tau: porep::Tau<PedersenDomain>,
    // only set for zigzag,
    pub comm_r_star: Option<PedersenDomain>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Params {
    pub typ: ProofType,
    pub size: usize,
    pub challenge_count: usize,
    pub vde: usize,
    pub degree: usize,
    // only set for zigzag
    pub expansion_degree: Option<usize>,
    // only set for zigzag
    pub layers: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, DbEnum)]
pub enum ProofType {
    Zigzag,
    #[db_rename = "drgporep"]
    DrgPoRep,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Proof {
    Zigzag(Vec<layered_drgporep::Proof<PedersenHasher>>),
    DrgPoRep(drgporep::Proof<PedersenHasher>),
}
