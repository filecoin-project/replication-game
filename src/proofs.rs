use std::fs::File;
use std::io::Write;

use byteorder::{BigEndian, ByteOrder};
use ff::PrimeField;
use memmap::MmapMut;
use memmap::MmapOptions;
use paired::bls12_381::Bls12;
use paired::bls12_381::Fr;
use rand::{thread_rng, Rng};

use storage_proofs::drgporep::{self, *};
use storage_proofs::drgraph::*;
use storage_proofs::fr32::fr_into_bytes;
use storage_proofs::hasher::blake2s::Blake2sDomain;
use storage_proofs::hasher::pedersen::PedersenDomain;
use storage_proofs::hasher::{Blake2sHasher, Domain, Hasher, PedersenHasher};
use storage_proofs::layered_drgporep::{self, LayerChallenges};
use storage_proofs::porep::PoRep;
use storage_proofs::proof::ProofScheme;
use storage_proofs::zigzag_drgporep::*;

use crate::models::proof;
use crate::models::seed::Seed;

fn file_backed_mmap_from_random_bytes(rng: &mut impl Rng, n: usize) -> MmapMut {
    let mut tmpfile: File = tempfile::tempfile().unwrap();

    for _ in 0..n {
        tmpfile
            .write_all(&fr_into_bytes::<Bls12>(&rng.gen()))
            .unwrap();
    }

    unsafe { MmapOptions::new().map_mut(&tmpfile).unwrap() }
}

pub fn zigzag_work<F>(prover: String, params: proof::Params, get_seed: F) -> String
where
    F: Fn(Blake2sDomain) -> Seed,
{
    eprintln!("{:?}", &params);
    let seed = get_seed(Default::default());
    let replica_id = id_from_str::<<Blake2sHasher as Hasher>::Domain>(&seed.mac);

    let data_size = params.size;
    let m = params.degree;
    let challenge_count = params.challenge_count;
    let sloth_iter = params.vde;

    let (expansion_degree, layer_challenges) = params
        .as_zigzag_params()
        .unwrap_or_else(|| (6, LayerChallenges::new_fixed(10, challenge_count)));

    let partitions = 1;

    let mut rng = thread_rng();

    eprintln!("generating fake data, {:?}", std::time::SystemTime::now());

    let nodes = data_size / 32;
    let mut data = file_backed_mmap_from_random_bytes(&mut rng, nodes);

    let sp = layered_drgporep::SetupParams {
        drg: drgporep::DrgParams {
            nodes,
            degree: m,
            expansion_degree,
            // TODO: where should this come from?
            seed: [0u32; 7],
        },
        sloth_iter,
        layer_challenges,
    };

    eprintln!("running setup, {:?}", std::time::SystemTime::now());
    let pp = ZigZagDrgPoRep::<Blake2sHasher>::setup(&sp).unwrap();

    eprintln!("running replicate, {:?}", std::time::SystemTime::now());

    let (tau, aux) =
        ZigZagDrgPoRep::<Blake2sHasher>::replicate(&pp, &replica_id, &mut data, None).unwrap();

    eprintln!("generating one proof, {:?}", std::time::SystemTime::now());

    let seed_challenge = get_seed(tau.layer_taus[tau.layer_taus.len() - 1].comm_r);
    let seed_fr = derive_seed_fr(&seed_challenge);

    let pub_inputs = layered_drgporep::PublicInputs::<<Blake2sHasher as Hasher>::Domain> {
        replica_id,
        seed: Some(seed_fr.into()),
        tau: Some(tau.simplify()),
        comm_r_star: tau.comm_r_star,
        k: Some(0),
    };

    let priv_inputs = layered_drgporep::PrivateInputs {
        aux,
        tau: tau.layer_taus.clone(),
    };

    let pr = ZigZagDrgPoRep::<Blake2sHasher>::prove_all_partitions(
        &pp,
        &pub_inputs,
        &priv_inputs,
        partitions,
    )
    .expect("failed to prove");

    eprintln!("verifying proof, {:?}", std::time::SystemTime::now());
    eprintln!("inputs: {:?}", &pub_inputs);
    let verified = ZigZagDrgPoRep::<Blake2sHasher>::verify_all_partitions(&pp, &pub_inputs, &pr)
        .expect("failed to verify");

    assert!(verified, "verification failed");

    eprintln!("verfication done, {:?}", std::time::SystemTime::now());
    serde_json::to_string(&proof::Response {
        prover,
        seed_start: seed,
        seed_challenge,
        proof_params: params,
        proof: proof::Proof::Zigzag(pr),
        comm_r_star: Some(tau.comm_r_star),
        tau: tau.simplify(),
    })
    .expect("failed to serialize")
}

pub fn id_from_str<T: Domain>(raw: &str) -> T {
    let replica_id_raw = hex::decode(raw).expect("invalid hex for replica id seed");
    let mut replica_id_bytes = vec![0u8; 32];
    let len = ::std::cmp::min(32, replica_id_raw.len());
    replica_id_bytes[..len].copy_from_slice(&replica_id_raw[..len]);
    T::try_from_bytes(&replica_id_bytes).expect("invalid replica id")
}

pub fn porep_work<F>(prover: String, params: proof::Params, get_seed: F) -> String
where
    F: Fn(Blake2sDomain) -> Seed,
{
    let seed = get_seed(Default::default());
    let replica_id = id_from_str::<<Blake2sHasher as Hasher>::Domain>(&seed.mac);

    let data_size = params.size;
    let m = params.degree;
    let sloth_iter = params.vde;
    let challenge_count = params.challenge_count;

    let mut rng = thread_rng();

    eprintln!("generating fake data");

    let nodes = data_size / 32;

    let mut data: Vec<u8> = (0..nodes)
        .flat_map(|_| fr_into_bytes::<Bls12>(&rng.gen()))
        .collect();

    // TODO: proper challenge generation
    let challenges = vec![2; challenge_count];

    let sp = SetupParams {
        drg: DrgParams {
            nodes,
            degree: m,
            expansion_degree: 0,
            // TODO: where should this come from?
            seed: [0u32; 7],
        },
        challenges_count: challenge_count,
        private: false,
        sloth_iter,
    };

    eprintln!("running setup");
    let pp = DrgPoRep::<Blake2sHasher, BucketGraph<Blake2sHasher>>::setup(&sp).unwrap();

    eprintln!("running replicate");
    let (tau, aux) =
        DrgPoRep::<Blake2sHasher, _>::replicate(&pp, &replica_id, data.as_mut_slice(), None)
            .unwrap();

    let pub_inputs = PublicInputs {
        replica_id: Some(replica_id),
        challenges,
        tau: Some(tau),
    };

    let priv_inputs = PrivateInputs::<Blake2sHasher> {
        tree_d: &aux.tree_d,
        tree_r: &aux.tree_r,
    };

    eprintln!("sampling proving & verifying");

    let challenge_seed = get_seed(tau.comm_r);

    let pr = DrgPoRep::<Blake2sHasher, _>::prove(&pp, &pub_inputs, &priv_inputs)
        .expect("failed to prove");

    DrgPoRep::<Blake2sHasher, _>::verify(&pp, &pub_inputs, &pr).expect("failed to verify");

    serde_json::to_string(&proof::Response {
        prover,
        seed_start: seed.clone(),
        seed_challenge: challenge_seed,
        proof_params: params,
        proof: proof::Proof::DrgPoRep(pr),
        comm_r_star: None,
        tau,
    })
    .expect("failed to serialize")
}

pub fn derive_seed_fr(seed: &Seed) -> Fr {
    let mac = hex::decode(&seed.mac).expect("invalid mac");

    // turn the mac into a u64
    let code_num = BigEndian::read_u64(&mac);

    // turn the u64 into an Fr
    Fr::from_repr(code_num.into()).unwrap()
}
