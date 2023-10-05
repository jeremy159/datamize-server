-- Create External Accounts Table
CREATE TABLE external_accounts(
  id BLOB NOT NULL,
  name TEXT NOT NULL,
  type TEXT NOT NULL,
  balance BIGINT NOT NULL,
  username TEXT NOT NULL,
  encrypted_password BLOB NOT NULL,
  deleted BOOLEAN NOT NULL,
  PRIMARY KEY (id)
);
