-- Create Balance Sheet Months Table
CREATE TABLE balance_sheet_months(
  id BLOB NOT NULL,
  month INTEGER NOT NULL,
  year_id BLOB NOT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (year_id) REFERENCES balance_sheet_years(id) ON DELETE CASCADE
);
