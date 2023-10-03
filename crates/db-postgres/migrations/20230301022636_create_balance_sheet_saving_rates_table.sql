CREATE TYPE ids_and_balance AS (
    ids uuid[],
    extra_balance BIGINT
);

-- Create Balance Sheet Saving Rates Table
CREATE TABLE balance_sheet_saving_rates(
  id uuid NOT NULL,
  name VARCHAR(256) NOT NULL,
  savings ids_and_balance NOT NULL,
  employer_contribution BIGINT NOT NULL,
  employee_contribution BIGINT NOT NULL,
  mortgage_capital BIGINT NOT NULL,
  incomes ids_and_balance NOT NULL,
  year_id uuid NOT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (year_id) REFERENCES balance_sheet_years(id) ON DELETE CASCADE
);
