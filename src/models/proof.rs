use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use storage_proofs::hasher::pedersen::PedersenDomain;
use storage_proofs::hasher::PedersenHasher;
use storage_proofs::layered_drgporep::LayerChallenges;
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
    pub zigzag: Option<ZigZagParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigZagParams {
    pub expansion_degree: usize,
    pub layers: usize,
    pub is_tapered: bool,
    pub taper_layers: usize,
    pub taper: f64,
}

impl Params {
    pub fn as_zigzag_params(&self) -> Option<(usize, LayerChallenges)> {
        self.zigzag.as_ref().map(|zigzag| {
            let layer_challenges = if zigzag.is_tapered {
                LayerChallenges::new_tapered(
                    zigzag.layers,
                    self.challenge_count,
                    zigzag.taper_layers,
                    zigzag.taper as f64,
                )
            } else {
                LayerChallenges::new_fixed(zigzag.layers, self.challenge_count)
            };
            (zigzag.expansion_degree, layer_challenges)
        })
    }
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
