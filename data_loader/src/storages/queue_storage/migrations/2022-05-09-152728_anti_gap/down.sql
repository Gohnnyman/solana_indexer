-- This file should undo anything in `up.sql`
ALTER TABLE signatures DROP COLUMN potential_gap_start;
ALTER TABLE signatures DROP CONSTRAINT signatures_program_signature_key;
ALTER TABLE signatures ADD CONSTRAINT signatures_signature_key UNIQUE(signature);