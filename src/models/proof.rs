#[cfg(feature = "postgres")]
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
    pub seed_start: Seed,
    pub seed_challenge: Seed,
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
    pub seed: Option<PedersenDomain>,
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

#[cfg_attr(feature = "postgres", derive(DbEnum))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofType {
    Zigzag,
    #[cfg_attr(feature = "postgres", db_rename = "drgporep")]
    DrgPoRep,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Proof {
    Zigzag(Vec<layered_drgporep::Proof<PedersenHasher>>),
    DrgPoRep(drgporep::Proof<PedersenHasher>),
}

impl Proof {
    pub fn get_replica_root(&self) -> &PedersenDomain {
        match self {
            Proof::Zigzag(ref proof) => &proof[0].tau[proof[0].tau.len() - 1].comm_r,
            Proof::DrgPoRep(ref proof) => &proof.replica_root,
        }
    }
}
