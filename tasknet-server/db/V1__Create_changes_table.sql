CREATE TABLE changes (
  doc_id BYTEA NOT NULL,
  hash BYTEA NOT NULL,
  data BYTEA NOT NULL,
  PRIMARY KEY(doc_id, hash)
)
