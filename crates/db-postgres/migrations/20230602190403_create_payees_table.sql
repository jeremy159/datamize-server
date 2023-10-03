-- Create Payees Table
CREATE TABLE payees(
  id uuid NOT NULL,
  PRIMARY KEY (id),
  name TEXT NOT NULL,
  transfer_account_id uuid,
  deleted BOOLEAN NOT NULL
);
