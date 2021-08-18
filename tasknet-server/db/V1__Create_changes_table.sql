create table changes (
  doc_id bytea not null,
  hash bytea not null,
  data bytea not null,
  primary key(doc_id, hash)
)

create table documents (
  doc_id bytea not null primary key,
  heads bytea not null,
  data bytea not null
)

create table sync_states (
  doc_id bytea not null,
  peer_id bytea not null,
  data bytea not null,
  primary key(doc_id, peer_id)
)
