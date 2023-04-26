ALTER TABLE categories
  ALTER COLUMN goal_percentage_complete TYPE INT,
  ALTER COLUMN goal_months_to_budget TYPE INT,
  ADD COLUMN goal_day INT,
  ADD COLUMN goal_cadence INT,
  ADD COLUMN goal_cadence_frequency INT;
