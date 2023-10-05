-- Create Balance Sheet Net Totals Years Table
CREATE TABLE balance_sheet_net_totals_years(
  id BLOB NOT NULL,
  type TEXT NOT NULL,
  total BIGINT NOT NULL,
  percent_var REAL NOT NULL,
  balance_var BIGINT NOT NULL,
  year_id BLOB NOT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (year_id) REFERENCES balance_sheet_years(id) ON DELETE CASCADE
);
