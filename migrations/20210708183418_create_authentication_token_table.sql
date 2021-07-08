-- Add migration script here
CREATE TABLE auth_tokens 
( 
  id SERIAL PRIMARY KEY,
  user_id integer REFERENCES users(id),
  token varchar(100) NOT NULL,
  created_at BIGINT NOT NULL
);