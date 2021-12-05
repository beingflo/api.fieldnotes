-- migrate:up
CREATE TABLE users
( 
  id SERIAL PRIMARY KEY,
  username text NOT NULL UNIQUE,
  password text NOT NULL,
  email text,
  salt text,
  metadata text,
  created_at TIMESTAMPTZ NOT NULL,
  deleted_at TIMESTAMPTZ
);

-- migrate:down
DROP TABLE IF EXISTS users;