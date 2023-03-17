-- Create Balance Sheet Resources Table
CREATE TABLE balance_sheet_resources(
  id uuid NOT NULL,
  name VARCHAR(256) NOT NULL,
  category VARCHAR(32) NOT NULL,
  type VARCHAR(32) NOT NULL,
  editable BOOLEAN NOT NULL,
  PRIMARY KEY (id)
);
