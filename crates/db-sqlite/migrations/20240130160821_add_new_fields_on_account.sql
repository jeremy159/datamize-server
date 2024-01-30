ALTER TABLE accounts
  ADD COLUMN last_reconciled_at DATETIME;

ALTER TABLE accounts
  ADD COLUMN debt_original_balance BIGINT;

ALTER TABLE accounts
  ADD COLUMN debt_interest_rates TEXT NOT NULL DEFAULT '';

ALTER TABLE accounts
  ADD COLUMN debt_minimum_payments TEXT NOT NULL DEFAULT '';

ALTER TABLE accounts
  ADD COLUMN debt_escrow_amounts TEXT NOT NULL DEFAULT '';
