INSERT INTO public.balance_sheet_saving_rates (saving_rate_id, name, savings, employer_contribution, employee_contribution, mortgage_capital, incomes, year_id)
VALUES
  -- 2022
  ('3fda1be1-35e1-41b7-bb16-7e2a14df6f00', 'SavingRates_Test1', ROW(ARRAY['dc4adb80-775f-4ec4-9f7b-7cf377e23a75', '3542dcc2-8d8f-4fbe-aa0a-3290605346be']::UUID[], 184942), 343350, 831723, 652220, ROW(ARRAY['118860ce-60ea-4fe2-954d-aba17ffec383']::UUID[], 987182), 'a8807643-efcc-446d-8495-8d3c42d5af4a'),
  ('4c26195c-c378-4b5d-aa4e-cc299d19ad43', 'SavingRates_Test2', ROW(ARRAY[]::UUID[], 640687), 353628, 69772, 652220, ROW(ARRAY[]::UUID[], 788484), 'a8807643-efcc-446d-8495-8d3c42d5af4a'),
  -- 2023
  ('231693be-437e-4d6a-b70e-2d74b23d2f39', 'SavingRates_Test1', ROW(ARRAY['dc4adb80-775f-4ec4-9f7b-7cf377e23a75', '3542dcc2-8d8f-4fbe-aa0a-3290605346be']::UUID[], 184942), 76327, 144163, 320703, ROW(ARRAY['118860ce-60ea-4fe2-954d-aba17ffec383']::UUID[], 987182), '36e89291-c088-446b-a320-d9fd44d93743'),
  ('ffb39acd-9d75-4ac9-990a-af7d6798388e', 'SavingRates_Test2', ROW(ARRAY[]::UUID[], 640687), 297436, 387528, 320703, ROW(ARRAY[]::UUID[], 788484), '36e89291-c088-446b-a320-d9fd44d93743');