-- Add migration script here
CREATE TABLE notes 
( 
  id SERIAL PRIMARY KEY,
  token varchar(32) NOT NULL,
  user_id integer REFERENCES users(id) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  deleted_at TIMESTAMPTZ,
  title varchar(100) NOT NULL,
  tags varchar NOT NULL,
  content varchar NOT NULL
);