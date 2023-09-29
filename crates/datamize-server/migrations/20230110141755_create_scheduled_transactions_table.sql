-- Create ScheduledTransactions Table
CREATE TABLE scheduled_transactions(
  id uuid NOT NULL,
  PRIMARY KEY (id),
  date_first DATE NOT NULL,
  date_next DATE NOT NULL,
  frequency TEXT,
  amount BIGINT NOT NULL,
  memo TEXT,
  flag_color TEXT,
  account_id uuid NOT NULL,
  payee_id uuid,
  category_id uuid,
  transfer_account_id uuid,
  deleted BOOLEAN NOT NULL,
  account_name TEXT NOT NULL,
  payee_name TEXT,
  category_name TEXT,
  subtransactions JSONB NOT NULL
);