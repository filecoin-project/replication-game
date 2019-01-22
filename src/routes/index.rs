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
          Send your results here. They should be sent as JSON.
    
    LEARN MORE
      More details on how to play the replication game:
      https://github.com/filecoin-project/replication-game-server
    "
}
