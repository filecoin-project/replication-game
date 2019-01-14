use rocket::get;
use rocket_contrib::json::Json;

use crate::db::DbConn;
use crate::error::ApiResult;
use crate::models::leaderboard::{Entry, PrintableEntry};

#[get("/leaderboard")]
pub fn leaderboard(conn: DbConn) -> ApiResult<Json<Vec<PrintableEntry>>> {
    let rows = Entry::all(&conn)?;

    Ok(Json(rows))
}
