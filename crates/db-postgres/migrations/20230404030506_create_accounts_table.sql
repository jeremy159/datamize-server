-- Create Accounts Table
CREATE TABLE accounts(
  id uuid NOT NULL,
  PRIMARY KEY (id),
  name TEXT NOT NULL,
  type VARCHAR(128) NOT NULL,
  on_budget BOOLEAN NOT NULL,
  closed BOOLEAN NOT NULL,
  note TEXT,
  balance BIGINT NOT NULL,
  cleared_balance BIGINT NOT NULL,
  uncleared_balance BIGINT NOT NULL,
  transfer_payee_id uuid NOT NULL,
  direct_import_linked BOOLEAN,
  direct_import_in_error BOOLEAN,
  deleted BOOLEAN NOT NULL
);
