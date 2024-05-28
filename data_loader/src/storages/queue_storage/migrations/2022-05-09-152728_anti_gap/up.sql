-- Your SQL goes here
ALTER TABLE signatures DROP CONSTRAINT signatures_signature_key;
ALTER TABLE signatures ADD CONSTRAINT signatures_program_signature_key UNIQUE(program, signature);
ALTER TABLE signatures ADD COLUMN potential_gap_start boolean;
