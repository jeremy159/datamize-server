-- Create Balance Sheet Saving Rates Table
CREATE TABLE balance_sheet_saving_rates(
  id BLOB NOT NULL,
  name TEXT NOT NULL,
  savings TEXT NOT NULL,
  employer_contribution BIGINT NOT NULL,
  employee_contribution BIGINT NOT NULL,
  mortgage_capital BIGINT NOT NULL,
  incomes TEXT NOT NULL,
  year_id BLOB NOT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (year_id) REFERENCES balance_sheet_years(id) ON DELETE CASCADE
);
