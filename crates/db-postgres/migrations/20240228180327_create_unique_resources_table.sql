-- Create Balance Sheet Resources with a Unique contraint on name Table
CREATE TABLE balance_sheet_unique_resources(
  resource_id uuid NOT NULL,
  name VARCHAR(256) NOT NULL UNIQUE,
  resource_type VARCHAR(64) NOT NULL,
  ynab_account_ids uuid[],
  external_account_ids uuid[],
  PRIMARY KEY (resource_id)
);
