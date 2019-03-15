use blake2::{Blake2b, Digest};
use byteorder::{BigEndian, ByteOrder};
use diesel::{self, prelude::*};
use serde_derive::{Deserialize, Serialize};

use crate::models::proof;
use crate::schema::{leaderboard, params};

#[table_name = "leaderboard"]
#[belongs_to(Params)]
#[derive(Queryable, Insertable, Debug, Clone, Deserialize, Serialize, Associations)]
pub struct Entry {
    pub id: i32,
    pub prover: String,
    pub repl_time: i32,
    pub params_id: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PrintableEntry {
    pub id: i32,
    pub prover: String,
    pub repl_time: i32,
    pub params: Params,
}

impl Entry {
    pub fn all(conn: &PgConnection) -> QueryResult<Vec<PrintableEntry>> {
        let rows = leaderboard::table
            .inner_join(params::table)
            .order(leaderboard::repl_time.asc())
            .load::<(Entry, Params)>(conn)?;

        Ok(rows
            .into_iter()
            .map(|(e, p)| PrintableEntry {
                id: e.id,
                prover: e.prover,
                repl_time: e.repl_time,
                params: p,
            })
            .collect())
    }

    pub fn insert(
        prover: &str,
        repl_time: i32,
        params_id: i64,
        conn: &PgConnection,
    ) -> QueryResult<()> {
        use crate::schema::leaderboard::dsl;

        let record = dsl::leaderboard
            .filter(dsl::prover.eq(prover))
            .filter(dsl::params_id.eq(params_id))
            .first::<(i32, String, i32, i64)>(conn)
            .optional()?;

        if let Some(record) = record {
            if repl_time < record.2 {
                // better time
                diesel::update(dsl::leaderboard.filter(dsl::prover.eq(prover)))
                    .set((dsl::repl_time.eq(repl_time), dsl::params_id.eq(params_id)))
                    .execute(conn)?;
            }
        } else {
            // regular insert
            diesel::insert_into(leaderboard::table)
                .values((
                    dsl::prover.eq(prover),
                    dsl::repl_time.eq(repl_time),
                    dsl::params_id.eq(params_id),
                ))
                .execute(conn)?;
        }

        Ok(())
    }
}

#[table_name = "params"]
#[derive(Queryable, Insertable, Debug, Clone, Serialize, Deserialize)]
pub struct Params {
    pub id: i64,

    pub typ: proof::ProofType,
    pub size: i32,
    pub challenge_count: i32,
    pub vde: i32,
    pub degree: i32,
    pub expansion_degree: Option<i32>,
    pub layers: Option<i32>,
    pub is_tapered: Option<bool>,
    pub taper_layers: Option<i32>,
    pub taper: Option<f64>,
}

impl Params {
    pub fn insert(val: &proof::Params, conn: &PgConnection) -> QueryResult<i64> {
        let serialized_params = serde_json::to_vec(val).expect("invalid params");
        let hash = Blake2b::digest(&serialized_params);
        let id = BigEndian::read_i64(&hash);

        use crate::schema::params::dsl;

        let params_exists = diesel::select(diesel::dsl::exists(dsl::params.filter(dsl::id.eq(id))))
            .get_result::<bool>(conn)?;

        if !params_exists {
            diesel::insert_into(params::table)
                .values(Params {
                    id,
                    typ: val.typ.clone(),
                    size: val.size as i32,
                    challenge_count: val.challenge_count as i32,
                    vde: val.vde as i32,
                    degree: val.degree as i32,
                    expansion_degree: val.zigzag.as_ref().map(|v| v.expansion_degree as i32),
                    layers: val.zigzag.as_ref().map(|v| v.layers as i32),
                    is_tapered: val.zigzag.as_ref().map(|v| v.is_tapered),
                    taper_layers: val.zigzag.as_ref().map(|v| v.taper_layers as i32),
                    taper: val.zigzag.as_ref().map(|v| v.taper),
                })
                .execute(conn)?;
        }

        Ok(id)
    }
}

pub fn upsert_entry_with_params(
    res: &proof::Response,
    repl_time: i32,
    conn: &PgConnection,
) -> QueryResult<()> {
    let params_id = Params::insert(&res.proof_params, conn)?;

    Entry::insert(&res.prover, repl_time, params_id, conn)?;

    Ok(())
}
