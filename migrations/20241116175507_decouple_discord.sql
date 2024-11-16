-- Add migration script here

ALTER TABLE accounts RENAME COLUMN user_id TO discord_id;
ALTER TABLE accounts ALTER COLUMN discord_id DROP NOT NULL;
ALTER TABLE accounts ADD COLUMN user_id VARCHAR(100); /* long term this should be NOT NULL */
ALTER TABLE accounts ALTER COLUMN first_name DROP NOT NULL; /* long term this should be deleted. */
/*ALTER TABLE accounts ADD CONSTRAINT one_main UNIQUE (user_id, is_main) - this should be when user_id is enforced */