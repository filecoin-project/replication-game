CREATE TABLE params (
  id BIG INTEGER PRIMARY KEY,
  typ INTEGER NOT NULL,
  size INTEGER NOT NULL,
  challenge_count INTEGER NOT NULL,
  vde INTEGER NOT NULL,
  degree INTEGER NOT NULL,
  expansion_degree INTEGER,
  layers INTEGER
)
