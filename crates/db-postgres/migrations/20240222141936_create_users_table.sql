-- Create Users Table
CREATE TABLE users(
  id uuid NOT NULL,
  access_token VARCHAR NOT NULL,
  refresh_token VARCHAR,
  expires_at timestamptz,
  PRIMARY KEY (id)
);
