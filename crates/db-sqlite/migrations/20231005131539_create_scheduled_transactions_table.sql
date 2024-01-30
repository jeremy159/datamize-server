-- Create ScheduledTransactions Table
CREATE TABLE scheduled_transactions(
  id BLOB NOT NULL,
  date_first DATE NOT NULL,
  date_next DATE NOT NULL,
  frequency TEXT NOT NULL,
  amount BIGINT NOT NULL,
  memo TEXT,
  flag_color TEXT,
  account_id BLOB NOT NULL,
  payee_id BLOB,
  category_id BLOB,
  transfer_account_id BLOB,
  deleted BOOLEAN NOT NULL,
  account_name TEXT NOT NULL,
  payee_name TEXT,
  category_name TEXT,
  subtransactions TEXT NOT NULL,
  PRIMARY KEY (id)
);
