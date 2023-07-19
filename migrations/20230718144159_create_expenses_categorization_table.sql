-- Create Expenses Categorization Table
CREATE TABLE expenses_categorization(
  id uuid NOT NULL,
  PRIMARY KEY (id),
  name TEXT NOT NULL,
  type VARCHAR(128) NOT NULL,
  sub_type VARCHAR(128) NOT NULL
);
