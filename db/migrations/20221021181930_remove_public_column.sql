-- migrate:up
ALTER TABLE shares
DROP COLUMN public;

-- migrate:down

