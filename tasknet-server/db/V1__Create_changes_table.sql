CREATE TABLE changes (
  doc_id bytea NOT NULL,
  hash bytea NOT NULL,
  data bytea NOT NULL,
  PRIMARY KEY(doc_id, hash)
)

CREATE TABLE documents (
  doc_id bytea NOT NULL PRIMARY KEY,
  heads bytea NOT NULL,
  data bytea NOT NULL
)

CREATE TABLE sync_states (
  doc_id bytea NOT NULL,
  peer_id bytea NOT NULL,
  data bytea NOT NULL,
  PRIMARY KEY(doc_id, peer_id)
)
