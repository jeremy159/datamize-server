-- Create Balance Sheet Saving Rates Table
CREATE TABLE balance_sheet_saving_rates(
  id uuid NOT NULL,
  name VARCHAR(256) NOT NULL,
  savings BIGINT NOT NULL,
  employer_contribution BIGINT NOT NULL,
  employee_contribution BIGINT NOT NULL,
  mortgage_capital BIGINT NOT NULL,
  incomes BIGINT NOT NULL,
  rate REAL NOT NULL,
  year_id uuid NOT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (year_id) REFERENCES balance_sheet_years(id) ON DELETE CASCADE
);
