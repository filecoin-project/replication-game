use diesel::PgConnection;
use rocket_contrib::database;

#[database("leaderboard")]
pub struct DbConn(PgConnection);
