-- Create Budgeters Config Table
CREATE TABLE budgeters_config(
  id uuid NOT NULL,
  name VARCHAR(256) NOT NULL,
  payee_ids uuid[] NOT NULL,
  PRIMARY KEY (id)
);
