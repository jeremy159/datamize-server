-- Create Categories Table
CREATE TABLE categories(
  id uuid NOT NULL,
  PRIMARY KEY (id),
  category_group_id uuid NOT NULL,
  name TEXT NOT NULL,
  hidden BOOLEAN NOT NULL,
  original_category_group_id uuid,
  note TEXT,
  budgeted BIGINT NOT NULL,
  activity BIGINT NOT NULL,
  balance BIGINT NOT NULL,
  goal_type TEXT,
  goal_creation_month DATE,
  goal_target BIGINT NOT NULL,
  goal_target_month DATE,
  goal_percentage_complete BIGINT,
  goal_months_to_budget BIGINT,
  goal_under_funded BIGINT,
  goal_overall_funded BIGINT,
  goal_overall_left BIGINT,
  deleted BOOLEAN NOT NULL
);
