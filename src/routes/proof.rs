use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use blake2::crypto_mac::Mac;
use blake2::Blake2b;
use failure::format_err;
use rocket::post;
use rocket_contrib::json::Json;

use storage_proofs::drgporep::{self, *};
use storage_proofs::drgraph::*;
use storage_proofs::hasher::{Blake2sHasher, Hasher, PedersenHasher};
use storage_proofs::layered_drgporep;
use storage_proofs::proof::ProofScheme;
use storage_proofs::zigzag_drgporep::*;

use crate::db::DbConn;
use crate::error::ApiResult;
use crate::gzip::Gzip;
use crate::models::leaderboard::upsert_entry_with_params;
use crate::models::proof;
use crate::models::seed::Seed;
use crate::proofs::id_from_str;

#[post("/proof", format = "json", data = "<res>")]
pub fn proof_gz(conn: DbConn, res: Gzip<Json<proof::Response>>) -> ApiResult<()> {
    proof(conn, res.into_inner())
}

#[post("/proof", format = "json", data = "<res>", rank = 2)]
pub fn proof(conn: DbConn, res: Json<proof::Response>) -> ApiResult<()> {
    // Get replication time
    let repl_time = {
        // Get current timestamp
        let start = SystemTime::now();
        let timestamp = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let completion_time = timestamp.as_secs();
        (completion_time - res.seed_start.timestamp as u64) as i32
    };

    let key = env::var("GAME_KEY").unwrap_or_else(|_| "my cool key".into());
    validate_seed(key.as_bytes(), &res.seed_start)?;
    validate_seed(key.as_bytes(), &res.seed_challenge)?;

    if !validate(&res) {
        return Err(format_err!("Submitted proofs are invalid").into());
    }

    // check that the challenge data is the root in the proof.
    let expected_root = res.proof.get_replica_root();
    let root = &res.seed_challenge.data;
    if root != expected_root {
        return Err(format_err!("Invalid seed challenge replica").into());
    }

    upsert_entry_with_params(&res, repl_time, &conn)?;

    Ok(())
}

fn validate_seed(key: &[u8], seed: &Seed) -> ApiResult<()> {
    let mac = hex::decode(&seed.mac)?;

    let mut hasher = Blake2b::new_varkey(key)?;
    hasher.input(&format!("{}", seed.timestamp).as_bytes());
    hasher.input(seed.data.as_ref());
    hasher.verify(&mac)?;

    Ok(())
}

fn validate(res: &proof::Response) -> bool {
    let replica_id = id_from_str::<<Blake2sHasher as Hasher>::Domain>(&res.seed_start.mac);
    let params = &res.proof_params;
    let data_size = params.size;
    let m = params.degree;
    let sloth_iter = params.vde;
    let challenge_count = params.challenge_count;
    let nodes = data_size / 32;
    let param_seed = [0u32; 7];

    let challenge_seed = crate::proofs::derive_seed_fr(&res.seed_challenge);

    match res.proof {
        proof::Proof::Zigzag(ref proof) => {
            if params.zigzag.is_none() {
                return false;
            }

            let (expansion_degree, layer_challenges) =
                params.as_zigzag_params().expect("missing zigzag params");
            let comm_r_star = res.comm_r_star.expect("missing comm r star");

            let sp = layered_drgporep::SetupParams {
                drg: drgporep::DrgParams {
                    nodes,
                    degree: m,
                    expansion_degree,
                    seed: param_seed,
                },
                sloth_iter,
                layer_challenges,
            };

            let pp = ZigZagDrgPoRep::<Blake2sHasher>::setup(&sp).unwrap();

            let pub_inputs = layered_drgporep::PublicInputs::<<Blake2sHasher as Hasher>::Domain> {
                replica_id,
                tau: Some(res.tau),
                comm_r_star,
                k: Some(0),
                seed: Some(challenge_seed.into()),
            };

            println!("inputs: {:?}", &pub_inputs);
            ZigZagDrgPoRep::<Blake2sHasher>::verify_all_partitions(&pp, &pub_inputs, proof)
                .unwrap_or_default()
        }
        proof::Proof::DrgPoRep(ref proof) => {
            let sp = SetupParams {
                drg: DrgParams {
                    nodes,
                    degree: m,
                    expansion_degree: 0,
                    seed: param_seed,
                },
                challenges_count: challenge_count,
                private: false,
                sloth_iter,
            };

            println!("running setup");
            let pp = DrgPoRep::<Blake2sHasher, BucketGraph<Blake2sHasher>>::setup(&sp).unwrap();
            let pub_inputs = PublicInputs::<<Blake2sHasher as Hasher>::Domain> {
                replica_id: Some(replica_id),
                challenges: vec![2; challenge_count],
                tau: Some(res.tau),
            };

            DrgPoRep::<Blake2sHasher, _>::verify(&pp, &pub_inputs, proof).unwrap_or_else(|_| false)
        }
    }
}
