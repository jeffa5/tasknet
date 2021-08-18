create table sync_states (
  doc_id bytea not null,
  peer_id bytea not null,
  data bytea not null,
  primary key(doc_id, peer_id)
)
