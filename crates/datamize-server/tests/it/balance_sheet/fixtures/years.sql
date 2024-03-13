INSERT INTO public.balance_sheet_years (year_id, year, refreshed_at)
VALUES
  ('a8807643-efcc-446d-8495-8d3c42d5af4a', 2022, '2023-04-03 15:11:51.512406+00'),
  ('36e89291-c088-446b-a320-d9fd44d93743', 2023, '2023-12-21 19:41:52.478037+00'),
  ('99294fc1-8e32-4508-a91f-061f059400b7', 2024, '2024-01-31 13:23:12.790315+00');

INSERT INTO public.balance_sheet_net_totals_years (net_total_id, type, total, percent_var, balance_var, last_updated, year_id)
VALUES
  ('d546e562-45bb-4248-857a-ff7024695721', 'asset', 884400, 0.1083, 792003, '2023-04-03 15:11:51.512406+00', 'a8807643-efcc-446d-8495-8d3c42d5af4a'),
  ('b94896f7-ad76-40c5-a5d1-7c2592538662', 'portfolio', 978503, 0.0030, 366151, '2023-04-03 15:11:51.512406+00', 'a8807643-efcc-446d-8495-8d3c42d5af4a'),
  ('85131b40-f53a-418a-a676-49ecddedb04d', 'asset', 510550, 0.8672, 251503, '2023-12-21 19:41:52.478037+00', '36e89291-c088-446b-a320-d9fd44d93743'),
  ('ccabf32f-b924-4b01-9015-7e4bfc662234', 'portfolio', 639380, 0.3861, 245506, '2023-12-21 19:41:52.478037+00', '36e89291-c088-446b-a320-d9fd44d93743'),
  ('6f752e28-61bd-4ad1-b1b0-0a519416fecf', 'asset', 203556, 0.2037, 229448, '2024-01-31 13:23:12.790315+00', '99294fc1-8e32-4508-a91f-061f059400b7'),
  ('2150752e-aee1-4f3e-995f-67657be8e8d8', 'portfolio', 764189, 0.4538, 186075, '2024-01-31 13:23:12.790315+00', '99294fc1-8e32-4508-a91f-061f059400b7');