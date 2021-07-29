CREATE TABLE auth_tokens 
( 
  id SERIAL PRIMARY KEY,
  user_id integer NOT NULL REFERENCES users(id),
  token varchar(128) NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE UNIQUE INDEX auth_token_index ON auth_tokens (token);