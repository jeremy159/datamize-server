-- Create Balance Sheet Years Table
CREATE TABLE balance_sheet_years(
  id BLOB NOT NULL,
  year INTEGER NOT NULL UNIQUE,
  refreshed_at DATETIME NOT NULL,
  PRIMARY KEY (id)
);
