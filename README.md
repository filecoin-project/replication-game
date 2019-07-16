# The Replication Game

> Compete on the fastest replication algorithm - [Participate here!](http://replication-game.herokuapp.com/)

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

Make sure you have all required dependencies installed:

- [rustup](https://www.rust-lang.org/tools/install)
- Rust nightly (usually `rustup install nightly`)
- [PostgreSQL](https://www.postgresql.org/)
- Clang and libclang
- [jq](https://stedolan.github.io/jq/download/) (optional) - prettify json output on the command-line, for viewing the leaderbord
- gzip

From the replication-game/ directory, compile the game binary:

```bash
cargo +nightly build --release --bin replication-game
```

### Play the game from the command line

There are two ways to play:
- **Method 1:** Run the `play` helper script
- **Method 2:** Run each individual command

#### Method 1: Run the `play` helper script

From the replication-game/ directory, run the `play` helper script in `bin/`, specifying:
- `NAME`: your player name
- `SIZE`: the size in KB of the data you want to replicate
- `TYPE`: the type of algorithm you want to run (current options are `zigzag` and `drgporep`)

```bash
# Run like this:
# bin/play NAME SIZE TYPE

# E.g.

# Zigzag 10MiB
bin/play NAME 10240 zigzag

# Zigzag 1GiB
bin/play NAME 1048576 zigzag

# DrgPoRep 10MiB
bin/play NAME 10240 drgporep

# DrgPoRep 1GiB
bin/play NAME 1048576 drgporep
```

The `play` script will retrieve the seed from the game server, replicate the data, generate a proof, and then post that proof to the game server. The script runs each of the commands in **Method 2**, but wraps them in an easy-to-use shell script.

#### Method 2: Run each individual command

Set your player name:

```bash
export REPL_GAME_ID="ReadyPlayerOne"
```

Play the game:

```bash
./target/release/replication-game \
	--prover $REPL_GAME_ID \
	--size 10240 \
	zigzag > proof.json
```

Send your proof:

```bash
curl -X POST -H "Content-Type: application/json" -d @./proof.json https://replication-game.herokuapp.com/api/proof
```

### Check the current leaderboard

There are three ways to check the leaderboard, two from the command line and one from the browser:
- **Method 1:** (From the command line) Run the `show-leaderboard` helper script
- **Method 2:** (From the command line) Curl the leaderboard
- **Method 3:** View the leaderboard in the browser

#### Method 1: Run the `show-leaderboard` helper script

From the replication-game/ directory, run the `show-leaderboard` helper script in `bin/`, specifying `SIZE`, which is the size in KB by which you want to filter the leaderboard results. The leaderboard shows all results across all parameters in a single list, so filtering by `SIZE` allows you to see only those results that match a particular size.

```bash
bin/show-leaderboard SIZE
```

#### Method 2: Curl the leaderboard

To check the current leaderboard using `curl`:

```bash
curl https://replication-game.herokuapp.com/api/leaderboard | jq
```

#### Method 3: View the leaderboard in the browser

You can also directly view the leaderboard in the browser at https://replication-game.herokuapp.com/.

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
$ cargo +nightly run --bin replication-game-server
```

This server requires Postgresql to work. The details of the expected configuration can be found in [`Rocket.toml`](Rocket.toml). The default environment is `development`.

### API

- POST `/api/seed`:
  - Inputs: `data`
  - Returns a `timestamp` (unix time) and a `seed`
- POST `/api/proof`
  - Inputs: `timestamp`, `seed`, `seed_challenge`, `prover_id` and `proof`
  - Checks authenticity of the seed (using the timestamp and a secret on the server)
  - Checks that the `proof` is correct
  - Computes `replication_time = timestamp - current_time`
  - If `replication_time < times[prover_id]`, then `times[prover_id] = replication_time`
- GET `/api/leaderboard`:
  - Shows a leaderboard of all the miners sorted by replication time

## License

The Filecoin Project is dual-licensed under Apache 2.0 and MIT terms:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
