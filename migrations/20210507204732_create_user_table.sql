-- Add migration script here
CREATE TABLE users
( 
  id SERIAL PRIMARY KEY,
  username varchar(25) UNIQUE NOT NULL,
  password varchar(100) NOT NULL,
  created_at BIGINT NOT NULL,
  balance BIGINT NOT NULL
);