CREATE TABLE users
( 
  id SERIAL PRIMARY KEY,
  username text NOT NULL UNIQUE,
  password text NOT NULL,
  email text,
  salt text,
  created_at TIMESTAMPTZ NOT NULL,
  deleted_at TIMESTAMPTZ,
  balance BIGINT NOT NULL
);