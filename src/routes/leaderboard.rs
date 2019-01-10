use rocket::get;
use rocket_contrib::json::Json;

use crate::db::DbConn;
use crate::error::ApiResult;
use crate::models::leaderboard::Entry;

#[get("/leaderboard")]
pub fn leaderboard(conn: DbConn) -> ApiResult<Json<Vec<Entry>>> {
    let rows = Entry::all(&conn)?;

    Ok(Json(rows))
}
