-- Create Balance Sheet Saving Rates Table
CREATE TABLE balance_sheet_saving_rates(
  saving_rate_id BLOB NOT NULL,
  name TEXT NOT NULL,
  savings TEXT NOT NULL,
  employer_contribution BIGINT NOT NULL,
  employee_contribution BIGINT NOT NULL,
  mortgage_capital BIGINT NOT NULL,
  incomes TEXT NOT NULL,
  year_id BLOB NOT NULL,
  PRIMARY KEY (saving_rate_id),
  FOREIGN KEY (year_id) REFERENCES balance_sheet_years(year_id) ON DELETE CASCADE
);
