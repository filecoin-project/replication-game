use clap::{value_t, App, AppSettings, Arg, SubCommand};

use replication_game::models::proof;
use replication_game::models::seed::SeedInput;
use replication_game::proofs::*;

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
                .default_value("5")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("vde")
                .help("The VDE difficulty")
                .long("vde")
                .default_value("0")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("expansion-degree")
                .help("The expansion degree for Zigzag")
                .long("expansion-degree")
                .default_value("8")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("layers")
                .help("The layers for Zigzag")
                .long("layers")
                .default_value("10")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("url")
                .long("url")
                .help("The url of the replication game api")
                .takes_value(true)
                .default_value("http://localhost:8000/api"),
        )
        .arg(
            Arg::with_name("prover")
                .long("prover")
                .help("The prover name to use for the response")
                .required(true)
                .takes_value(true),
        )
        .setting(AppSettings::SubcommandRequired)
        .subcommand(SubCommand::with_name("drgporep"))
        .subcommand(SubCommand::with_name("zigzag"))
        .get_matches();

    let (typ, zigzag) = match matches.subcommand().0 {
        "drgporep" => (proof::ProofType::DrgPoRep, None),
        "zigzag" => (
            proof::ProofType::Zigzag,
            Some(proof::ZigZagParams {
                expansion_degree: value_t!(matches, "expansion-degree", usize).unwrap(),
                layers: value_t!(matches, "layers", usize).unwrap(),
                is_tapered: true,
                taper_layers: 7,
                taper: 1.0 / 3.0,
            }),
        ),
        _ => panic!("invalid subcommand: {}", matches.subcommand().0),
    };

    let params = proof::Params {
        typ: typ.clone(),
        size: value_t!(matches, "size", usize).unwrap() * 1024,
        degree: value_t!(matches, "degree", usize).unwrap(),
        vde: value_t!(matches, "vde", usize).unwrap(),
        challenge_count: 200,
        zigzag,
        seed: None, // gets filled in manually
    };

    let prover = value_t!(matches, "prover", String).unwrap();
    let host = format!("{}/seed", value_t!(matches, "url", String).unwrap());

    let get_seed = |data| {
        let client = reqwest::Client::new();
        let seed_input = SeedInput { data };

        client
            .post(&host)
            .json(&seed_input)
            .send()
            .expect("failed to get challenge")
            .json()
            .expect("invalid seed challenge response")
    };

    let res = match typ {
        proof::ProofType::DrgPoRep => porep_work(prover, params, get_seed),
        proof::ProofType::Zigzag => zigzag_work(prover, params, get_seed),
    };

    println!("{}", res);
}
