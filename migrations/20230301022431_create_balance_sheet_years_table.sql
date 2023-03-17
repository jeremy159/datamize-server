-- Create Balance Sheet Years Table
CREATE TABLE balance_sheet_years(
  id uuid NOT NULL,
  year INT NOT NULL UNIQUE,
  refreshed_at timestamptz NOT NULL,
  PRIMARY KEY (id)
);
