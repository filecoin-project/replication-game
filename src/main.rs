#![feature(proc_macro_hygiene, decl_macro)]
#![feature(duration_as_u128)]

#[macro_use] extern crate rocket;
use blake2::Blake2b;
use blake2::crypto_mac::Mac;
use hex;
use rocket::http::Status;
use std::io;
use std::time::{SystemTime, UNIX_EPOCH};
use rocket::request::{Form};

#[derive(FromForm)]
struct ProofResponse {
    ts: u128,
    seed: String,
    // TODO proof
}


#[get("/")]
fn index() -> &'static str {
    "
    USAGE
      GET /seed
          Returns a timestamp and a seed separated by a line
          EXAMPLE: curl  http://localhost:8000/seed

      POST /proof
          Send your `ts` (timestamp), `seed` and `proof`,
          it verifies it and stores it, returns status 400 or 406
          EXAMPLE: curl -X POST -d \"ts=123123&seed=rhoch83q\"  http://localhost:8000/proof
    "
}

#[get("/seed")]
fn seed() -> io::Result<String> {
    
    // Get current timestamp
    let ts = {
        let start = SystemTime::now();
        let timestamp = start.duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        format!("{:?}", timestamp.as_millis())
    };

    // take the mac of the timestamp
    let mut hasher = Blake2b::new_varkey(b"my key").unwrap();
    hasher.input(ts.as_bytes());
    let result = hasher.result();
    let code_bytes = result.code().to_vec();
    let mac = hex::encode(&code_bytes);

    // return ts and mac
    let result = format!("{}\n{}", &ts, &mac);
    Ok(result)
}

#[post("/proof", data = "<proof>")]
fn proof(proof: Form<ProofResponse>) -> Status {
    // Get timestamp
    let ts =  format!("{}", proof.ts);

    // Get mac code
    let ch = &proof.seed.clone();
    let mac = hex::decode(ch).unwrap();
    let mut hasher = Blake2b::new_varkey(b"my key").unwrap();
    hasher.input(&ts.as_bytes());

    // verifies the code
    let verification = hasher.verify(&mac);

    // TODO verify the proof

    match verification {
        Err(_) => Status::NotAcceptable,
        Ok(_) => Status::Ok,
    }
}

fn main() {
    rocket::ignite().mount("/", routes![index, seed, proof]).launch();
}