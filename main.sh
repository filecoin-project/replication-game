#!/usr/bin/env bash

echo "building"

RUSTFLAGS="-C codegen-units=1 -C target-cpu=native" cargo build --release --bin replication-game --no-default-features

REPL_GAME_ID="simple"
SIZE=1048576
# SIZE=10240

# export RUST_BACKTRACE=1
export FIL_PROOFS_MAXIMIZE_CACHING=1

time ./target/release/replication-game \
	--prover $REPL_GAME_ID \
	--size $SIZE \
        --vde 0\
        --expansion-degree 8\
        --layers 10\
        --degree 5\
	zigzag > proof.json
