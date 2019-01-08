use clap::{value_t, App, Arg};
use pairing::bls12_381::Bls12;
use rand::{thread_rng, Rng};

use storage_proofs::drgporep::*;
use storage_proofs::drgraph::*;
use storage_proofs::example_helper::prettyb;
use storage_proofs::fr32::fr_into_bytes;
use storage_proofs::hasher::{Hasher, PedersenHasher};
use storage_proofs::porep::PoRep;
use storage_proofs::proof::ProofScheme;

fn do_the_work<H: Hasher>(
    data_size: usize,
    m: usize,
    sloth_iter: usize,
    challenge_count: usize,
) -> String {
    let mut rng = thread_rng();
    let challenges = vec![2; challenge_count];

    println!("data_size:  {}", prettyb(data_size));
    println!("challenge_count: {}", challenge_count);
    println!("m: {}", m);
    println!("sloth: {}", sloth_iter);

    println!("generating fake data");

    let nodes = data_size / 32;

    let replica_id: H::Domain = rng.gen();
    let mut data: Vec<u8> = (0..nodes)
        .flat_map(|_| fr_into_bytes::<Bls12>(&rng.gen()))
        .collect();

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

    serde_json::to_string(&proof).expect("failed to serialize proof")
}

fn main() {
    let matches = App::new(stringify!("DrgPoRep Vanilla Bench"))
        .version("1.0")
        .arg(
            Arg::with_name("size")
                .required(true)
                .long("size")
                .help("The data size in KB")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("m")
                .help("The size of m")
                .long("m")
                .default_value("6")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("sloth")
                .help("The number of sloth iterations, defaults to 1")
                .long("sloth")
                .default_value("1")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("challenges")
                .long("challenges")
                .help("How many challenges to execute, defaults to 1")
                .default_value("1")
                .takes_value(true),
        )
        .get_matches();

    let data_size = value_t!(matches, "size", usize).unwrap() * 1024;
    let m = value_t!(matches, "m", usize).unwrap();
    let sloth_iter = value_t!(matches, "sloth", usize).unwrap();
    let challenge_count = value_t!(matches, "challenges", usize).unwrap();

    let res = do_the_work::<PedersenHasher>(data_size, m, sloth_iter, challenge_count);

    println!("\n\n{}", res);
}
