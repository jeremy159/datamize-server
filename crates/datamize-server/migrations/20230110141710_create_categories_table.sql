-- Create Categories Table
CREATE TABLE categories(
  id uuid NOT NULL,
  PRIMARY KEY (id),
  category_group_id uuid NOT NULL,
  category_group_name TEXT NOT NULL,
  name TEXT NOT NULL,
  hidden BOOLEAN NOT NULL,
  original_category_group_id uuid,
  note TEXT,
  budgeted BIGINT NOT NULL,
  activity BIGINT NOT NULL,
  balance BIGINT NOT NULL,
  goal_type TEXT,
  goal_day INT,
  goal_cadence INT,
  goal_cadence_frequency INT,
  goal_creation_month DATE,
  goal_target BIGINT NOT NULL,
  goal_target_month DATE,
  goal_percentage_complete INT,
  goal_months_to_budget INT,
  goal_under_funded BIGINT,
  goal_overall_funded BIGINT,
  goal_overall_left BIGINT,
  deleted BOOLEAN NOT NULL
);