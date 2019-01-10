use lazy_static::lazy_static;
use parking_lot::Mutex;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use rocket::http::{ContentType, Status};
use rocket::local::Client;

use crate::models::leaderboard::Entry;
use crate::models::seed::Seed;

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
        let id: String = rng.sample_iter(&Alphanumeric).take(12).collect();

        // Issue a request to insert a result
        let response = client
            .post("/proof")
            .header(ContentType::Form)
            .body(format!(
                "prover_id={}&ts={}&seed={}",
                &id, seed.timestamp, seed.seed
            ))
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
        let entries: Vec<Entry> = serde_json::from_str(&body).unwrap();

        assert!(entries.iter().find(|entry| &entry.prover == &id).is_some());
    })
}
