-- migrate:up
CREATE TYPE event AS ENUM ('starttextli', 'pausetextli', 'addfunds');

CREATE TABLE transactions
( 
  id SERIAL PRIMARY KEY,
  user_id integer NOT NULL REFERENCES users(id),
  event event NOT NULL,
  amount BIGINT,
  date TIMESTAMPTZ NOT NULL
);

-- migrate:down
DROP TABLE app_events;
DROP TYPE event;