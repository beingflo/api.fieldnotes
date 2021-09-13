CREATE TABLE auth_tokens 
( 
  id SERIAL PRIMARY KEY,
  user_id integer NOT NULL REFERENCES users(id),
  token text NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL
);