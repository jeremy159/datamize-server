-- Create Balance Sheet Months Table
CREATE TABLE balance_sheet_months(
  id uuid NOT NULL,
  month SMALLINT NOT NULL,
  year_id uuid NOT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (year_id) REFERENCES balance_sheet_years(id)
);
