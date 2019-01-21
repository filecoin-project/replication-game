CREATE TABLE leaderboard (
  id SERIAL PRIMARY KEY,
  prover TEXT NOT NULL,
  repl_time INT NOT NULL,
  params_id BIGINT NOT NULL
)
