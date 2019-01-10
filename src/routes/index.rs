use rocket::get;

#[get("/")]
pub fn index() -> &'static str {
    "
    USAGE
      GET /leaderboard
          Returns the current leaderboard as JSON

      GET /seed
          Returns a timestamp and a seed separated as JSON
          EXAMPLE: curl  http://localhost:8000/seed

      POST /proof
          Send your `ts` (timestamp), `seed` and `proof`,
          it verifies it and stores it, returns status 400 or 406
          EXAMPLE: curl -X POST -d \"prover_id=myid&ts=123123&seed=rhoch83q\"  http://localhost:8000/proof
    "
}
