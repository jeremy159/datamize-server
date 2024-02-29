-- Create Balance Sheet Net Totals Months Table
CREATE TABLE balance_sheet_net_totals_months(
  net_total_id BLOB NOT NULL,
  type TEXT NOT NULL,
  total BIGINT NOT NULL,
  percent_var REAL NOT NULL,
  balance_var BIGINT NOT NULL,
  month_id BLOB NOT NULL,
  last_updated DATETIME,
  PRIMARY KEY (net_total_id),
  UNIQUE (type, month_id), -- Only one type per month can exist
  FOREIGN KEY (month_id) REFERENCES balance_sheet_months(month_id) ON DELETE CASCADE
);
