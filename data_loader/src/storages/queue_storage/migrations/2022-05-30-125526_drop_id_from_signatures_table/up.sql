-- Your SQL goes here
alter table signatures drop constraint signatures_pkey;
alter table signatures drop constraint signatures_program_signature_key;
alter table signatures add primary key(program, signature);
alter table signatures drop column id;

alter table transactions drop constraint transactions_pkey;
alter table transactions drop column id;
alter table transactions add column signature varchar(88) not null default '';
update transactions set signature = transaction::json->'transaction'->'signatures'->>0;
alter table transactions add primary key(signature);
create index transactions_block_time on transactions (block_time);
create index transactions_parsing_status on transactions (parsing_status);
