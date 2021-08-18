create table changes (
  doc_id bytea not null,
  hash bytea not null,
  data bytea not null,
  primary key(doc_id, hash)
)
