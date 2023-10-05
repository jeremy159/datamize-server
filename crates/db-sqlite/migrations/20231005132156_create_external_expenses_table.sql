-- Create External Expenses Table
CREATE TABLE external_expenses(
  id BLOB NOT NULL,
  name TEXT NOT NULL,
  type TEXT NOT NULL,
  sub_type TEXT NOT NULL,
  projected_amount BIGINT NOT NULL,
  PRIMARY KEY (id)
);
