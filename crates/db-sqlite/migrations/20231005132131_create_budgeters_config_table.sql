-- Create Budgeters Config Table
CREATE TABLE budgeters_config(
  id BLOB NOT NULL,
  name TEXT NOT NULL,
  payee_ids TEXT NOT NULL,
  PRIMARY KEY (id)
);
