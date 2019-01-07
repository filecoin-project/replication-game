use diesel::{self, prelude::*};

mod schema {
    table! {
        leaderboard (prover) {
            prover -> Text,
            repl_time -> Integer,
        }
    }
}

use self::schema::leaderboard;
use self::schema::leaderboard::dsl::leaderboard as all_leaderboard;

#[table_name = "leaderboard"]
#[derive(Queryable, Insertable, Debug, Clone, Deserialize, Serialize)]
pub struct Entry {
    pub prover: String,
    pub repl_time: i32,
}

impl Entry {
    pub fn all(conn: &SqliteConnection) -> QueryResult<Vec<Entry>> {
        all_leaderboard
            .order(leaderboard::repl_time.asc())
            .load::<Entry>(conn)
    }

    pub fn insert(prover_id: &str, repl_time: i32, conn: &SqliteConnection) -> QueryResult<()> {
        let query = format!(
            "INSERT INTO leaderboard(prover, repl_time) VALUES(\"{prover}\", {repl_time})
            ON CONFLICT(prover) DO UPDATE SET
                repl_time={repl_time}
            WHERE excluded.repl_time < leaderboard.repl_time;",
            prover = prover_id,
            repl_time = repl_time
        );

        diesel::sql_query(query).execute(conn)?;
        Ok(())

        // TODO: Use once https://github.com/diesel-rs/diesel/pull/1884 is merged
        // use self::schema::leaderboard::dsl::*;
        // use pg::upsert::excluded;

        // diesel::insert_into(leaderboard::table)
        //     .values(&entry)
        //     .on_conflict(prover)
        //     .do_update()
        //     .filter(excluded(repl_time) < (entry.repl_time))
        //     .set(repl_time.eq(entry.repl_time))
        //     .execute(conn)
    }
}
