-- Create Balance Sheet Resources Table
CREATE TABLE balance_sheet_resources(
  id BLOB NOT NULL,
  name TEXT NOT NULL,
  resource_type TEXT NOT NULL,
  ynab_account_ids TEXT,
  external_account_ids TEXT,
  PRIMARY KEY (id)
);
