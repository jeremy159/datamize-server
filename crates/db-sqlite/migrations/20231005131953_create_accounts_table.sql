-- Create Accounts Table
CREATE TABLE accounts(
  id BLOB NOT NULL,
  name TEXT NOT NULL,
  type TEXT NOT NULL,
  on_budget BOOLEAN NOT NULL,
  closed BOOLEAN NOT NULL,
  note TEXT,
  balance BIGINT NOT NULL,
  cleared_balance BIGINT NOT NULL,
  uncleared_balance BIGINT NOT NULL,
  transfer_payee_id BLOB NOT NULL,
  direct_import_linked BOOLEAN,
  direct_import_in_error BOOLEAN,
  deleted BOOLEAN NOT NULL,
  last_reconciled_at DATETIME,
  debt_original_balance BIGINT,
  debt_interest_rates TEXT NOT NULL,
  debt_minimum_payments TEXT NOT NULL,
  debt_escrow_amounts TEXT NOT NULL,
  PRIMARY KEY (id)
);
