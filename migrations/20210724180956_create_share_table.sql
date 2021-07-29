CREATE TABLE shares 
( 
  id SERIAL PRIMARY KEY,
  token varchar(128) NOT NULL UNIQUE,
  note_id integer NOT NULL REFERENCES notes(id),
  user_id integer NOT NULL REFERENCES users(id),
  created_at TIMESTAMPTZ NOT NULL
);