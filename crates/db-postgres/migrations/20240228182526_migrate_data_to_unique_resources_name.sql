-- Step 1: Create the new resources
CREATE EXTENSION IF NOT EXISTS pgcrypto;
INSERT INTO balance_sheet_unique_resources (resource_id, name, resource_type, ynab_account_ids, external_account_ids)
(SELECT
 	gen_random_uuid(),
	name,
	MIN(resource_type) AS resource_type,
	MAX(ynab_account_ids) AS ynab_account_ids,
	MAX(external_account_ids) AS external_account_ids
FROM balance_sheet_resources
GROUP BY name
ORDER BY name ASC);

-- Step 2: Add balances to resources_balance_per_months from previous resource ids
INSERT INTO resources_balance_per_months (resource_id, month_id, balance)
(SELECT
	new.resource_id,
	month_id,
	balance
FROM public.balance_sheet_resources AS old
JOIN public.balance_sheet_unique_resources AS new ON old.name = new.name
JOIN public.balance_sheet_resources_months AS rm ON old.resource_id = rm.resource_id);
