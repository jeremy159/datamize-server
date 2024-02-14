-- Step 1: Add the new column
ALTER TABLE balance_sheet_resources
  ADD COLUMN resource_type VARCHAR(64);

-- Step 2: Populate the new column with values from category and type
UPDATE balance_sheet_resources
  SET resource_type = CONCAT(category, '_', type);

-- Step 3: Convert column into a not null one now that all rows are populated
ALTER TABLE balance_sheet_resources
  ALTER resource_type SET NOT NULL;

-- Step 4: Drop the old columns
ALTER TABLE balance_sheet_resources
  DROP COLUMN category,
  DROP COLUMN type;
