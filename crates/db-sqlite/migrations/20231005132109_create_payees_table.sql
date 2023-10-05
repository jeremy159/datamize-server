-- Create Payees Table
CREATE TABLE payees(
  id BLOB NOT NULL,
  name TEXT NOT NULL,
  transfer_account_id BLOB,
  deleted BOOLEAN NOT NULL,
  PRIMARY KEY (id)
);
