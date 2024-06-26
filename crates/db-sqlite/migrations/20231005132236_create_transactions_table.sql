-- Create Transactions Table
CREATE TABLE transactions(
  id BLOB NOT NULL,
  date DATE NOT NULL,
  amount BIGINT NOT NULL,
  memo TEXT,
  cleared TEXT NOT NULL,
  approved BOOLEAN NOT NULL,
  flag_color TEXT,
  account_id BLOB NOT NULL,
  payee_id BLOB,
  category_id BLOB,
  transfer_account_id BLOB,
  transfer_transaction_id BLOB,
  matched_transaction_id BLOB,
  import_id BLOB,
  deleted BOOLEAN NOT NULL,
  account_name TEXT NOT NULL,
  payee_name TEXT,
  category_name TEXT,
  import_payee_name TEXT,
  import_payee_name_original TEXT,
  debt_transaction_type TEXT,
  subtransactions TEXT NOT NULL,
  PRIMARY KEY (id)
);
