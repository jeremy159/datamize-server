INSERT INTO public.scheduled_transactions (id, date_first, date_next, frequency, amount, memo, flag_color, account_id, payee_id, category_id, transfer_account_id, deleted, account_name, payee_name, category_name, subtransactions)
VALUES
  ( 'd2278d76-2461-4889-834c-e9b68eee8a40',
    '2024-01-15',
    '2024-02-15',
    'monthly',
    -1500000,
    NULL,
    NULL,
    '73c70aef-b932-4077-a75b-ecffe29abce3',
    'd41e2871-1ebf-40de-a2d9-f02fff64a2ce',
    '6df368ec-a553-4fdb-bdab-d4ab6f1cadf5',
    '3f647cf9-83d5-4896-8f8b-9cc01463d88b',
    false,
    'Account_Test1',
    'Payee_Test1',
    'Category_Test2',
    '[]'::jsonb ),
  ( '34d2dc39-380e-4f3d-9e6c-1f4ce1edc8e9',
    '2024-01-24',
    '2024-02-07',
    'everyOtherWeek',
    1050000,
    NULL,
    NULL,
    '73c70aef-b932-4077-a75b-ecffe29abce3',
    'fe01f42c-6061-4e58-a83c-777fdfd42d64',
    'eeb1fc8e-69ec-419a-8677-ea34a36855a4',
    NULL,
    false,
    'Account_Test1',
    'Other_Payee_Test1',
    'Other_Category_Test1',
    '[]'::jsonb ),
  ( '8cd8586f-c706-4da4-9d49-3af3c177ffc4',
    '2024-01-19',
    '2024-02-16',
    'every4Weeks',
    -105000,
    NULL,
    NULL,
    '73c70aef-b932-4077-a75b-ecffe29abce3',
    '5d5abbde-1561-413c-b05e-73aecbaf0047',
    '1d57fd38-934c-42ed-8a67-b8163098b806',
    NULL,
    false,
    'Account_Test1',
    'Other_Payee_Test2',
    'Other_Category_Test2',
    '[]'::jsonb );
