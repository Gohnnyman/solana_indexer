-- This file should undo anything in `up.sql`
alter table signatures drop constraint signatures_pkey;
alter table signatures add column id integer default 0;
alter table signatures add primary key(id);
ALTER TABLE signatures ADD CONSTRAINT signatures_program_signature_key UNIQUE(program, signature);

alter table transactions drop constraint transactions_pkey;
alter table transactions drop column signature;
alter table transactions add column id integer default 0;
alter table transactions add primary key(id);
drop index transactions_block_time;
drop index transactions_parsing_status;