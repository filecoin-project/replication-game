use clap::{value_t, App, AppSettings, Arg, SubCommand};
use pairing::bls12_381::Bls12;
use rand::{thread_rng, Rng};
use serde::Serialize;

use storage_proofs::drgporep::*;
use storage_proofs::drgraph::*;
use storage_proofs::fr32::fr_into_bytes;
use storage_proofs::hasher::{Domain, Hasher, PedersenHasher};
use storage_proofs::porep::PoRep;
use storage_proofs::proof::ProofScheme;

fn zigzag_work<H: Hasher>(params: Params) -> String {
    // TODO: implement me
    unimplemented!("zigzag");
}

fn porep_work<H: Hasher>(params: Params) -> String {
    let replica_id_raw = &params.replica_id;
    let data_size = params.size;
    let m = params.degree;
    let sloth_iter = params.vde;
    let challenge_count = params.challenge_count;

    let mut rng = thread_rng();

    println!("generating fake data");

    let nodes = data_size / 32;

    let mut replica_id_bytes = vec![0u8; 32];
    replica_id_bytes[0..replica_id_raw.len()].copy_from_slice(replica_id_raw);
    let replica_id = H::Domain::try_from_bytes(&replica_id_bytes).expect("invalid replica id");

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
            seed: new_seed(),
        },
        sloth_iter,
    };

    println!("running setup");
    let pp = DrgPoRep::<H, BucketGraph<H>>::setup(&sp).unwrap();

    println!("running replicate");
    let (tau, aux) =
        DrgPoRep::<H, _>::replicate(&pp, &replica_id, data.as_mut_slice(), None).unwrap();

    let pub_inputs = PublicInputs {
        replica_id,
        challenges,
        tau: Some(tau),
    };

    let priv_inputs = PrivateInputs::<H> { aux: &aux };

    println!("sampling proving & verifying");

    let proof = DrgPoRep::<H, _>::prove(&pp, &pub_inputs, &priv_inputs).expect("failed to prove");

    DrgPoRep::<H, _>::verify(&pp, &pub_inputs, &proof).expect("failed to verify");

    format!(
        "{{\"params\": {}, \"proof\": {} }}",
        serde_json::to_string(&params).expect("failed to serialize params"),
        serde_json::to_string(&proof).expect("failed to serialize proof"),
    )
}

#[derive(Debug, Clone, Serialize)]
struct Params {
    size: usize,
    replica_id: Vec<u8>,
    challenge_count: usize,
    vde: usize,
    degree: usize,
    expansion_degree: usize,
}

fn main() {
    let matches = App::new(stringify!("Replication Game CLI"))
        .version("1.0")
        .arg(
            Arg::with_name("size")
                .required(true)
                .long("size")
                .help("The data size in KB")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("degree")
                .help("The degree")
                .long("degree")
                .default_value("6")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("vde")
                .help("The VDE difficulty")
                .long("vde")
                .default_value("1")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("expansion-degree")
                .help("The expansion degree for Zigzag")
                .long("expansion-degree")
                .default_value("6")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("replica-id")
                .long("replica-id")
                .help("The replica ID to use")
                .required(true)
                .takes_value(true),
        )
        .setting(AppSettings::SubcommandRequired)
        .subcommand(SubCommand::with_name("drgporep"))
        .subcommand(SubCommand::with_name("zigzag"))
        .get_matches();

    let params = Params {
        size: value_t!(matches, "size", usize).unwrap() * 1024,
        degree: value_t!(matches, "degree", usize).unwrap(),
        vde: value_t!(matches, "vde", usize).unwrap(),
        replica_id: value_t!(matches, "replica-id", String)
            .unwrap()
            .as_bytes()
            .to_vec(),
        challenge_count: 2, // TODO: use 200
        expansion_degree: value_t!(matches, "expansion-degree", usize).unwrap(),
    };

    let res = match matches.subcommand() {
        ("drgporep", _) => porep_work::<PedersenHasher>(params),
        ("zigzag", _) => zigzag_work::<PedersenHasher>(params),
        (sub, _) => panic!("invalid subcommand: {}", sub),
    };

    println!("\n\n{}", res);
}
