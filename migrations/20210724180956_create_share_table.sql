CREATE TABLE shares 
( 
  id SERIAL PRIMARY KEY,
  token varchar(128) NOT NULL,
  note_id integer REFERENCES notes(id) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);