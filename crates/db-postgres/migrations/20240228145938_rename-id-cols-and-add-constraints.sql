-- Change id col of Years
ALTER TABLE balance_sheet_years RENAME COLUMN id TO year_id;

-- Change id col of Months
ALTER TABLE balance_sheet_months RENAME COLUMN id TO month_id;

-- no same month of same year should exist twice
ALTER TABLE balance_sheet_months ADD UNIQUE (month, year_id);

-- Change id col of Saving Rates
ALTER TABLE balance_sheet_saving_rates RENAME COLUMN id TO saving_rate_id;

-- Change id col of Year Net Totals
ALTER TABLE balance_sheet_net_totals_years RENAME COLUMN id TO net_total_id;

-- Only one type per year can exist
ALTER TABLE balance_sheet_net_totals_years ADD UNIQUE (type, year_id);

-- Change id col of Month Net Totals
ALTER TABLE balance_sheet_net_totals_months RENAME COLUMN id TO net_total_id;

-- Only one type per month can exist
ALTER TABLE balance_sheet_net_totals_months ADD UNIQUE (type, month_id);

-- Change id col of Resourcess
ALTER TABLE balance_sheet_resources RENAME COLUMN id TO resource_id;
