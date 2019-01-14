use lazy_static::lazy_static;
use parking_lot::Mutex;
use rand::{thread_rng, Rng};
use rocket::http::{ContentType, Status};
use rocket::local::Client;

use crate::models::leaderboard::{Entry, PrintableEntry};
use crate::models::proof;
use crate::models::seed::Seed;
use crate::proofs;

// We use a lock to synchronize between tests so DB operations don't collide.
// For now. In the future, we'll have a nice way to run each test in a DB
// transaction so we can regain concurrency.
lazy_static! {
    static ref DB_LOCK: Mutex<()> = Mutex::new(());
}

macro_rules! run_test {
    (|$client:ident, $conn:ident| $block:expr) => {{
        let _lock = DB_LOCK.lock();
        let (rocket, db) = super::rocket();
        let $client = Client::new(rocket).expect("Rocket client");
        let $conn = db.expect("failed to get database connection for testing");
        $block
    }};
}

#[test]
fn test_insertion() {
    run_test!(|client, conn| {
        // Get the tasks before making changes.
        let init_leaderboard = Entry::all(&conn).unwrap();
        // Get a seed
        let mut response = client.get("/seed").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let body = response.body_string().unwrap();
        println!("response: {}", &body);

        let seed: Seed = serde_json::from_str(&body).unwrap();

        let mut rng = thread_rng();
        let id: String = rng.gen_ascii_chars().take(12).collect();

        let params = proof::Params {
            typ: proof::ProofType::DrgPoRep,
            size: 1024,
            challenge_count: 1,
            vde: 1,
            degree: 3,
            expansion_degree: None,
            layers: None,
        };

        let proof_value = proofs::porep_work(id.clone(), params, seed.clone());

        // Issue a request to insert a result
        let response = client
            .post("/proof")
            .header(ContentType::JSON)
            .body(proof_value)
            .dispatch();

        assert_eq!(response.status(), Status::Ok);

        // Ensure we have one more entry the database.
        let new_leaderboard = Entry::all(&conn).unwrap();
        assert_eq!(new_leaderboard.len(), init_leaderboard.len() + 1);

        // Ensure the entry exists
        assert!(new_leaderboard
            .iter()
            .find(|entry| &entry.prover == &id)
            .is_some());

        // check that the entry is in the leaderboard

        let mut response = client.get("/leaderboard").dispatch();
        assert_eq!(response.status(), Status::Ok);

        let body = response.body_string().unwrap();
        let entries: Vec<PrintableEntry> = serde_json::from_str(&body).unwrap();

        assert!(entries.iter().find(|entry| &entry.prover == &id).is_some());
    })
}

#[test]
fn test_many_insertions() {
    run_test!(|client, conn| {
        let mut rng = thread_rng();

        // Get the tasks before making changes.
        let init_leaderboard = Entry::all(&conn).unwrap();

        let mut prev_len = init_leaderboard.len();

        for _ in 0..2 {
            // Get a seed
            let mut response = client.get("/seed").dispatch();
            assert_eq!(response.status(), Status::Ok);
            let body = response.body_string().unwrap();
            let seed: Seed = serde_json::from_str(&body).unwrap();

            let id: String = rng.gen_ascii_chars().take(12).collect();

            let params1 = proof::Params {
                typ: proof::ProofType::DrgPoRep,
                size: 1024,
                challenge_count: 1,
                vde: 1,
                degree: 3,
                expansion_degree: None,
                layers: None,
            };
            let params2 = proof::Params {
                typ: proof::ProofType::DrgPoRep,
                size: 1024,
                challenge_count: 2,
                vde: 1,
                degree: 3,
                expansion_degree: None,
                layers: None,
            };

            let proof_value1 = proofs::porep_work(id.clone(), params1.clone(), seed.clone());
            let proof_value2 = proofs::porep_work(id.clone(), params2.clone(), seed.clone());

            // First params
            let old_repl_time = {
                // slower proof
                use std::{thread, time};
                thread::sleep(time::Duration::from_millis(2000));

                let response = client
                    .post("/proof")
                    .header(ContentType::JSON)
                    .body(proof_value1)
                    .dispatch();
                assert_eq!(response.status(), Status::Ok);

                // Ensure we have one more entry the database.
                let new_leaderboard = Entry::all(&conn).unwrap();
                assert_eq!(new_leaderboard.len(), prev_len + 1);
                prev_len = new_leaderboard.len();
                println!("{:?}", new_leaderboard);
                new_leaderboard
                    .iter()
                    .find(|entry| &entry.prover == &id)
                    .unwrap()
                    .repl_time
            };

            // First params, same prover, but faster
            {
                let mut response = client.get("/seed").dispatch();
                assert_eq!(response.status(), Status::Ok);
                let body = response.body_string().unwrap();
                let seed: Seed = serde_json::from_str(&body).unwrap();

                let proof_value = proofs::porep_work(id.clone(), params1, seed.clone());
                let response = client
                    .post("/proof")
                    .header(ContentType::JSON)
                    .body(proof_value)
                    .dispatch();
                assert_eq!(response.status(), Status::Ok);

                // Ensure we don't have another entry
                let new_leaderboard = Entry::all(&conn).unwrap();
                assert_eq!(new_leaderboard.len(), prev_len);

                // check that the entry was updated
                let repl_time = new_leaderboard
                    .iter()
                    .find(|entry| &entry.prover == &id)
                    .unwrap()
                    .repl_time;
                assert!(
                    repl_time < old_repl_time,
                    "replication time was not updated: {} >= {}",
                    repl_time,
                    old_repl_time
                );
            }

            // Second params
            {
                let response = client
                    .post("/proof")
                    .header(ContentType::JSON)
                    .body(&proof_value2)
                    .dispatch();
                assert_eq!(response.status(), Status::Ok);

                // Ensure we have one more entry the database.
                let new_leaderboard = Entry::all(&conn).unwrap();
                assert_eq!(new_leaderboard.len(), prev_len + 1);
                prev_len = new_leaderboard.len();
            }

            // Clone of second params
            {
                let response = client
                    .post("/proof")
                    .header(ContentType::JSON)
                    .body(&proof_value2)
                    .dispatch();
                assert_eq!(response.status(), Status::Ok);

                let new_leaderboard = Entry::all(&conn).unwrap();
                assert_eq!(new_leaderboard.len(), prev_len);
            }
        }
    })
}
