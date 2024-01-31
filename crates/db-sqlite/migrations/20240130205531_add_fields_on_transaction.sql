ALTER TABLE transactions
  ADD COLUMN import_payee_name TEXT;

ALTER TABLE transactions
  ADD COLUMN import_payee_name_original TEXT;

ALTER TABLE transactions
  ADD COLUMN debt_transaction_type TEXT;
