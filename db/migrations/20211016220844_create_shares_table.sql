-- migrate:up
CREATE TABLE shares 
( 
  id SERIAL PRIMARY KEY,
  token text NOT NULL UNIQUE,
  note_id integer NOT NULL REFERENCES notes(id) UNIQUE,
  user_id integer NOT NULL REFERENCES users(id),
  created_at TIMESTAMPTZ NOT NULL,
  expires_at TIMESTAMPTZ,
  view_count integer NOT NULL,
  public text
);

-- migrate:down
DROP TABLE IF EXISTS shares;
