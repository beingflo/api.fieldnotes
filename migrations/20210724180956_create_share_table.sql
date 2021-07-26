CREATE TABLE shares 
( 
  id SERIAL PRIMARY KEY,
  token varchar(128) NOT NULL,
  note_id integer NOT NULL REFERENCES notes(id),
  created_at TIMESTAMPTZ NOT NULL
);