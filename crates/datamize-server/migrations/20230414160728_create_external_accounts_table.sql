-- Create External Accounts Table
CREATE TABLE external_accounts(
  id uuid NOT NULL,
  PRIMARY KEY (id),
  name TEXT NOT NULL,
  type VARCHAR(128) NOT NULL,
  balance BIGINT NOT NULL,
  username TEXT NOT NULL,
  encrypted_password BYTEA NOT NULL,
  deleted BOOLEAN NOT NULL
);
