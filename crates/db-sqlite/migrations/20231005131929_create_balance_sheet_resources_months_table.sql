-- Create Balance Sheet Resources Months Table
CREATE TABLE balance_sheet_resources_months(
  resource_id BLOB REFERENCES balance_sheet_resources(id) ON DELETE CASCADE,
  month_id BLOB REFERENCES balance_sheet_months(id) ON DELETE CASCADE,
  balance BIGINT NOT NULL DEFAULT 0,
  PRIMARY KEY (resource_id, month_id)
);
