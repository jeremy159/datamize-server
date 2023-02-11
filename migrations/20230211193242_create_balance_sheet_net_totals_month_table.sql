-- Create Balance Sheet Net Totals Months Table
CREATE TABLE balance_sheet_net_totals_months(
  id uuid NOT NULL,
  type TEXT NOT NULL,
  total BIGINT NOT NULL,
  percent_var REAL NOT NULL,
  balance_var BIGINT NOT NULL,
  month_id uuid NOT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (month_id) REFERENCES balance_sheet_months(id)
);
