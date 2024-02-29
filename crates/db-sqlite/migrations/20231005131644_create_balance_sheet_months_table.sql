-- Create Balance Sheet Months Table
CREATE TABLE balance_sheet_months(
  month_id BLOB NOT NULL,
  month INTEGER NOT NULL,
  year_id BLOB NOT NULL,
  PRIMARY KEY (month_id),
  UNIQUE (month, year_id), -- no same month of same year should exist twice
  FOREIGN KEY (year_id) REFERENCES balance_sheet_years(year_id) ON DELETE CASCADE
);
