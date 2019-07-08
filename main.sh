#!/usr/bin/env bash

echo "building"

cargo build --release --bin replication-game --no-default-features

echo "getting seed"

curl https://replication-game.herokuapp.com/api/seed > seed.json

REPL_GAME_ID="simple"
REPL_GAME_SEED=$(cat seed.json| jq -r '.seed')
# TODO: use with new server
#REPL_GAME_CHALLENGE=$(cat seed.json| jq -r '.challenge-seed')
REPL_GAME_CHALLENGE="1212121212121212121212121212121212121212121212121212121212121212"
REPL_GAME_TIMESTAMP=$(cat seed.json| jq -r '.timestamp')
SIZE=1048576

export RUST_BACKTRACE=1
export FIL_PROOFS_MAXIMIZE_CACHING=1

time ./target/release/replication-game \
	--prover $REPL_GAME_ID \
	--seed $REPL_GAME_SEED \
	--timestamp $REPL_GAME_TIMESTAMP \
        --challenge-seed $REPL_GAME_CHALLENGE \
	--size $SIZE \
        --vde 0\
        --expansion-degree 8\
        --layers 10\
        --degree = 5\
	zigzag > proof.json
