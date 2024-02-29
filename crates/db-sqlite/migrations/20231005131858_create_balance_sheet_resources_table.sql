-- Create Balance Sheet Resources Table
CREATE TABLE balance_sheet_resources(
  resource_id BLOB NOT NULL,
  name TEXT NOT NULL UNIQUE,
  resource_type TEXT NOT NULL,
  ynab_account_ids TEXT,
  external_account_ids TEXT,
  PRIMARY KEY (resource_id)
);
