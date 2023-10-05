-- Create Expenses Categorization Table
CREATE TABLE expenses_categorization(
  id BLOB NOT NULL,
  name TEXT NOT NULL,
  type TEXT NOT NULL,
  sub_type TEXT NOT NULL,
  PRIMARY KEY (id)
);
