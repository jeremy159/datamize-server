-- Create Balance Sheet Net Totals Years Table
CREATE TABLE balance_sheet_net_totals_years(
  net_total_id BLOB NOT NULL,
  type TEXT NOT NULL,
  total BIGINT NOT NULL,
  percent_var REAL NOT NULL,
  balance_var BIGINT NOT NULL,
  year_id BLOB NOT NULL,
  last_updated DATETIME,
  PRIMARY KEY (net_total_id),
  UNIQUE (type, year_id), -- Only one type per year can exist
  FOREIGN KEY (year_id) REFERENCES balance_sheet_years(year_id) ON DELETE CASCADE
);
