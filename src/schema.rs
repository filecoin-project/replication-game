table! {
    leaderboard (id) {
        id -> Integer,
        prover -> Text,
        repl_time -> Integer,
        params_id -> BigInt,
    }
}

table! {
    use crate::models::proof::ProofTypeMapping;
    use diesel::sql_types::{Nullable, BigInt, Integer, Double, Bool};

    params (id) {
        id -> BigInt,
        typ -> ProofTypeMapping,
        size -> Integer,
        challenge_count -> Integer,
        vde -> Integer,
        degree -> Integer,
        expansion_degree -> Nullable<Integer>,
        layers -> Nullable<Integer>,
        is_tapered -> Nullable<Bool>,
        taper_layers -> Nullable<Integer>,
        taper -> Nullable<Double>,
    }
}

allow_tables_to_appear_in_same_query!(leaderboard, params);
joinable!(leaderboard -> params (params_id));
