ALTER TABLE instructions ON CLUSTER '{cluster}' ADD COLUMN IF NOT EXISTS
raw_instruction_idx UInt16 MATERIALIZED 
if(
    transaction_instruction_idx IS NULL, 
    instruction_idx * 256, 
    transaction_instruction_idx * 256 + instruction_idx + 1
)
