-- migrate:up
CREATE TYPE event AS ENUM ('startfieldnotes', 'pausefieldnotes', 'addfunds');

CREATE TABLE transactions
( 
  id SERIAL PRIMARY KEY,
  user_id integer NOT NULL REFERENCES users(id),
  event event NOT NULL,
  amount BIGINT,
  date TIMESTAMPTZ NOT NULL
);

-- migrate:down
DROP TABLE IF EXISTS app_events;
DROP TYPE IF EXISTS event;