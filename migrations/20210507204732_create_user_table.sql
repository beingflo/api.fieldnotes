CREATE TABLE users
( 
  id SERIAL PRIMARY KEY,
  username varchar(50) NOT NULL UNIQUE,
  password varchar(100) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  deleted_at TIMESTAMPTZ,
  balance BIGINT NOT NULL
);