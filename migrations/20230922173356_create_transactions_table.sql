-- Create Transactions Table
CREATE TABLE transactions(
  id uuid NOT NULL,
  PRIMARY KEY (id),
  date DATE NOT NULL,
  amount BIGINT NOT NULL,
  memo TEXT,
  cleared TEXT NOT NULL,
  approved BOOLEAN NOT NULL,
  flag_color TEXT,
  account_id uuid NOT NULL,
  payee_id uuid,
  category_id uuid,
  transfer_account_id uuid,
  transfer_transaction_id uuid,
  matched_transaction_id uuid,
  import_id uuid,
  deleted BOOLEAN NOT NULL,
  account_name TEXT NOT NULL,
  payee_name TEXT,
  category_name TEXT,
  subtransactions JSONB NOT NULL
);
