ALTER TABLE accounts
  ADD COLUMN last_reconciled_at TIMESTAMPTZ,
  ADD COLUMN debt_original_balance BIGINT,
  ADD COLUMN debt_interest_rates JSONB NOT NULL DEFAULT '{}'::jsonb,
  ADD COLUMN debt_minimum_payments JSONB NOT NULL DEFAULT '{}'::jsonb,
  ADD COLUMN debt_escrow_amounts JSONB NOT NULL DEFAULT '{}'::jsonb;
