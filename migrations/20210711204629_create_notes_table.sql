CREATE TABLE notes 
( 
  id SERIAL PRIMARY KEY,
  token varchar(32) NOT NULL UNIQUE,
  user_id integer NOT NULL REFERENCES users(id),
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  deleted_at TIMESTAMPTZ,
  encrypted_key varchar(200) NOT NULL,
  metainfo varchar(1000) NOT NULL,
  content text NOT NULL
);