create table documents (
  doc_id bytea not null primary key,
  heads bytea not null,
  data bytea not null
)
