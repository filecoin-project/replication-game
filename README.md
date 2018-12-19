# replication-game-server
WIP: Participants compete on fastest replication algorithms

## Design
- GET `/seed`:
  - Returns a `timestamp` (unix time) and a `seed` to be used as `replica_id` in the proof of replication
- POST `/proof`
  - Inputs: `timestamp`, `seed`, `prover_id` and `proof`
  - Checks authenticity of the seed (using the timestamp and a secret on the server)
  - Checks that the `proof` is correct
  - Computes `replication_time = timestamp - current_time`
  - If `replication_time < times[prover_id]`, then `times[prover_id] = replication_time`
- GET `/`:
  - Shows a leaderboard of all the miners sorted by replication time
