CREATE TABLE notes 
( 
  id SERIAL PRIMARY KEY,
  token text NOT NULL UNIQUE,
  user_id integer NOT NULL REFERENCES users(id),
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  deleted_at TIMESTAMPTZ,
  metadata text NOT NULL,
  key text NOT NULL,
  content text NOT NULL
);