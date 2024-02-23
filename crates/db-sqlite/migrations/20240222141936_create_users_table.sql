-- Create Users Table
CREATE TABLE users(
  id BLOB NOT NULL,
  access_token TEXT NOT NULL,
  refresh_token TEXT,
  expires_at DATETIME,
  PRIMARY KEY (id)
);
