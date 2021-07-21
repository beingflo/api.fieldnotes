-- Add migration script here
CREATE TABLE auth_tokens 
( 
  id SERIAL PRIMARY KEY,
  user_id integer REFERENCES users(id) NOT NULL,
  token varchar(64) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);