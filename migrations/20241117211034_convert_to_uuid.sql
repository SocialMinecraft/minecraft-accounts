-- 1. First add a temporary UUID column
ALTER TABLE accounts
    ADD COLUMN temp_id UUID;

-- 2. Update the temporary column with converted UUIDs from the existing varchar
UPDATE accounts
SET temp_id = CAST(minecraft_uuid AS UUID);

-- 3. Add NOT NULL constraint to the temporary column
ALTER TABLE accounts
    ALTER COLUMN temp_id SET NOT NULL;

-- 4. Drop the old varchar column
ALTER TABLE accounts
DROP COLUMN minecraft_uuid;

-- 5. Rename the temporary column to the original column name
ALTER TABLE accounts
    RENAME COLUMN temp_id TO minecraft_uuid;