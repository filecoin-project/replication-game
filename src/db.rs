use diesel::SqliteConnection;
use rocket_contrib::database;

#[database("leaderboard")]
pub struct DbConn(SqliteConnection);
