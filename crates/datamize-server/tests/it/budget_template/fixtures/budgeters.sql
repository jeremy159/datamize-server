INSERT INTO public.budgeters_config (id, name, payee_ids)
VALUES
  ('e399da25-8807-4f5b-850b-6c70b66f529c', 'Budgeter_Test1', ARRAY['6921cdaa-ff86-4973-b4f8-9e5bb422475b', 'a8d26d21-b8d7-4b7f-9cf0-b2d9994ba6a9']::UUID[]), -- no ynab or external accounts ids
  ('3b162522-e282-4e15-8da3-797e18d47f8d', 'Budgeter_Test2', ARRAY['9c727ee0-f556-4636-94aa-ee4778bccbf2']::UUID[]),
  ('19418aba-cd89-446f-9c53-db526c4a1baf', 'Budgeter_Test3', ARRAY[]::UUID[]);
