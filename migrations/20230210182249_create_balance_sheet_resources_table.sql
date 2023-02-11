-- Create Balance Sheet Resources Table
CREATE TABLE balance_sheet_resources(
  id uuid NOT NULL,
  name TEXT NOT NULL,
  category TEXT NOT NULL,
  type TEXT NOT NULL,
  balance BIGINT NOT NULL,
  editable BOOLEAN,
  month_id uuid NOT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (month_id) REFERENCES balance_sheet_months(id)
);
