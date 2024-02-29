INSERT INTO public.balance_sheet_unique_resources (resource_id, name, resource_type)
VALUES
  ('ef6454a5-9322-4e92-9bf9-122e53b71fa7', 'Res_Asset_Cash_Test', 'asset_cash'), -- no ynab or external accounts ids
  ('3930790f-db52-405d-b551-32d14c5450ce', 'Res_Asset_Investment_Test', 'asset_investment'),
  ('8ff283e0-3a6a-4a64-b40f-42b834bcf821', 'Res_Asset_Longterm_Test', 'asset_longTerm'),
  ('33debac2-b8a8-440c-86b6-86157113ef62', 'Res_Liability_Cash_Test', 'liability_cash'),
  ('2b0c6f1d-b5c7-4e57-a33b-43d1a0ae8344', 'Res_Liability_Longterm_Test', 'liability_longTerm');

INSERT INTO public.resources_balance_per_months (resource_id, month_id, balance)
VALUES
  -- Jan 2022
  ('ef6454a5-9322-4e92-9bf9-122e53b71fa7', 'fe57c6dc-a520-4cd3-a611-617dc24d050d', 720080),
  ('33debac2-b8a8-440c-86b6-86157113ef62', 'fe57c6dc-a520-4cd3-a611-617dc24d050d', 852032),
  -- Feb 2022
  ('ef6454a5-9322-4e92-9bf9-122e53b71fa7', 'b7558757-6faf-4404-9414-1f707c22eff9', 494498),
  ('33debac2-b8a8-440c-86b6-86157113ef62', 'b7558757-6faf-4404-9414-1f707c22eff9', 616084),
  -- March 2022
  ('ef6454a5-9322-4e92-9bf9-122e53b71fa7', '153b96fe-d325-4000-a781-5e030b54972d', 134408),
  ('33debac2-b8a8-440c-86b6-86157113ef62', '153b96fe-d325-4000-a781-5e030b54972d', 581272),
  -- Jan 2023
  ('ef6454a5-9322-4e92-9bf9-122e53b71fa7', '14124e8d-8420-4097-a0d6-38d761e73d08', 973113),
  ('33debac2-b8a8-440c-86b6-86157113ef62', '14124e8d-8420-4097-a0d6-38d761e73d08', 808122),
  ('3930790f-db52-405d-b551-32d14c5450ce', '14124e8d-8420-4097-a0d6-38d761e73d08', 267405),
  ('8ff283e0-3a6a-4a64-b40f-42b834bcf821', '14124e8d-8420-4097-a0d6-38d761e73d08', 528358),
  ('2b0c6f1d-b5c7-4e57-a33b-43d1a0ae8344', '14124e8d-8420-4097-a0d6-38d761e73d08', 723944),
  -- Nov 2023
  ('ef6454a5-9322-4e92-9bf9-122e53b71fa7', '8ca12f29-93b9-4d06-8d7d-5bcde81efbc9', 428046),
  ('33debac2-b8a8-440c-86b6-86157113ef62', '8ca12f29-93b9-4d06-8d7d-5bcde81efbc9', 568393),
  ('3930790f-db52-405d-b551-32d14c5450ce', '8ca12f29-93b9-4d06-8d7d-5bcde81efbc9', 635068),
  ('8ff283e0-3a6a-4a64-b40f-42b834bcf821', '8ca12f29-93b9-4d06-8d7d-5bcde81efbc9', 514803),
  ('2b0c6f1d-b5c7-4e57-a33b-43d1a0ae8344', '8ca12f29-93b9-4d06-8d7d-5bcde81efbc9', 217716);