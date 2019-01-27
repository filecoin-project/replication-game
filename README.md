# The Replication Game

> Compete on the fastest replication algorithm

![](https://ipfs.io/ipfs/Qmdr2HMghfsknH9nfrRU2fjcdqZK8bjM8xa2JShBkehsCF/giphy.gif)

## Introduction

**What is this "game"?** The Replication Game is a competition where participants compete to outperform the default implementation of Proof-of-Replication. To participate in the game, you can run the current replication algorithm (or your own implementation) and post your proof on our server.

**What is Proof-of-Replication?** Proof of Replication is the proof that: (1) the Filecoin Storage Market is secure: it ensures that miners cannot lie about storing users' data, (2) the Filecoin Blockchain is secure: it ensures that miners cannot lie about the amount of storage they have (remember, miners win blocks based on their storage power!). In Filecoin, we use the Proof of Replication inside "Sealing" during mining.

**How does Proof of Replication work?** The intuition behind Proof of Replication is the following: the data from the Filecoin market is encoded via a slow sequential computation that cannot be parallelized.

**How can I climb up in the leaderboard?** There are some strategies to replicate "faster", some are practical (software and hardware optimizations), some are believe to be impractical or impossible (get ready to win a price and be remembered in the history of cryptography if you do so!)

- *Practical attempts*: Implement a faster replication algorithm with better usage of memory, optimize some parts of the algorithm (e.g. Pedersen, Blake2s) in hardware (e.g. FPGA, GPU, ASICs), performing attacks on Depth Robust Graphs (the best known attacks are [here](https://eprint.iacr.org/2017/443)).
- *Impractical attempts*: Find special datasets that allow for faster replication, break the sequentiality assumption, generate the proof storing less data, break Pedersen hashes.

## Play the Replication Game

This executes an actual game, using [rust-proofs](https://github.com/filecoin-project/rust-proofs), feel free to implement your own version.

### Run our command line

Make sure you have all required dependencies installed:

- [rustup](https://www.rust-lang.org/tools/install)
- Rust nightly (usually `rustup install nightly`)
- [PostgreSQL](https://www.postgresql.org/)
- Clang and libclang

Compile the game binary:

```bash
cargo +nightly build --release --bin replication-game
```

Set your player name:

```bash
export REPL_GAME_ID="ReadyPlayerOne"
```

Get the seed from our server:

```bash
curl https://replication-game.herokuapp.com/seed > seed.json
export REPL_GAME_SEED=$(cat seed.json| jq -r '.seed')
export REPL_GAME_TIMESTAMP=$(cat seed.json| jq -r '.timestamp')
```

Play the game:

```bash
./target/release/replication-game \
	--prover $REPL_GAME_ID \
	--seed $REPL_GAME_SEED \
	--timestamp $REPL_GAME_TIMESTAMP \
	--size 1048576 \
	zigzag > proof.json
```

Send your proof:

```bash
curl -X POST -H "Content-Type: application/json" -d @./proof.json https://replication-game.herokuapp.com/proof
```

### Check the current leaderboard

To check the current leaderboard:

```bash
curl https://replication-game.herokuapp.com/leaderboard | jq
```

## FAQ

>  What parameters should I be using for the replication?

Our leaderboard will track the parameters you will be using, feel free to experiment with many. We are targeting powers of two, in particular: 1GiB (`--size 1048576`), 16GiB (`--size 16777216`), 1TB (`--size 1073741824`)

> How do I know what the parameters mean?

```bash
./target/debug/replication-game --help
```

> What do I win if I am first?

So far, we have no bounty set up for this, but we are planning on doing so. If you beat the replication game (and you can prove it by being in the leaderboard), reach out to [filecoin-research@protocol.ai](mailto:filecoin-research@protocol.ai).



------



## Replication Game Server

```bash
$ cargo +nighlty run --bin replication-game-server
```

This server requires Postgresql to work. The details of the expected configuration can be found in [`Rocket.toml`](Rocket.toml). The default environment is `development`.

### API

- GET `/seed`:
  - Returns a `timestamp` (unix time) and a `seed` to be used as `replica_id` in the proof of replication
- POST `/proof`
  - Inputs: `timestamp`, `seed`, `prover_id` and `proof`
  - Checks authenticity of the seed (using the timestamp and a secret on the server)
  - Checks that the `proof` is correct
  - Computes `replication_time = timestamp - current_time`
  - If `replication_time < times[prover_id]`, then `times[prover_id] = replication_time`
- GET `/leaderboard`:
  - Shows a leaderboard of all the miners sorted by replication time

## License

The Filecoin Project is dual-licensed under Apache 2.0 and MIT terms:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
