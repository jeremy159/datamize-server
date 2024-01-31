ALTER TABLE transactions
  ADD COLUMN import_payee_name TEXT,
  ADD COLUMN import_payee_name_original TEXT,
  ADD COLUMN debt_transaction_type TEXT;
