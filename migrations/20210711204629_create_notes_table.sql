-- Add migration script here
CREATE TABLE notes 
( 
  id SERIAL PRIMARY KEY,
  token varchar(32) NOT NULL,
  user_id integer REFERENCES users(id) NOT NULL,
  created_at BIGINT NOT NULL,
  modified_at BIGINT NOT NULL,
  deleted_at BIGINT,
  title varchar(100) NOT NULL,
  tags varchar NOT NULL,
  content varchar NOT NULL
);