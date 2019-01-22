CREATE TYPE proof_type as ENUM ('zigzag', 'drgporep');

CREATE TABLE params (
  id BIGINT PRIMARY KEY,
  typ proof_type NOT NULL,
  size INT NOT NULL,
  challenge_count INT NOT NULL,
  vde INT NOT NULL,
  degree INT NOT NULL,
  expansion_degree INT,
  layers INT
);
