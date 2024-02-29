-- Create Balance Sheet Resources Balance Per Months Table
CREATE TABLE resources_balance_per_months(
  resource_id uuid REFERENCES balance_sheet_unique_resources(resource_id) ON DELETE CASCADE,
  month_id uuid REFERENCES balance_sheet_months(month_id) ON DELETE CASCADE,
  balance BIGINT NOT NULL,
  PRIMARY KEY (resource_id, month_id)
);
