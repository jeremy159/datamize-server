-- Create External Expenses Table
CREATE TABLE external_expenses(
  id uuid NOT NULL,
  PRIMARY KEY (id),
  name TEXT NOT NULL,
  type VARCHAR(128) NOT NULL,
  sub_type VARCHAR(128) NOT NULL,
  projected_amount BIGINT NOT NULL
);
