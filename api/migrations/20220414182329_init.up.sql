-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS citext;
CREATE TYPE enum_org_role AS ENUM ('admin', 'owner');
CREATE TABLE IF NOT EXISTS orgs (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  name citext UNIQUE NOT NULL,
  is_personal BOOLEAN NOT NULL DEFAULT 't',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_orgs_is_personal on orgs (is_personal);
CREATE TABLE IF NOT EXISTS users (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  first_name TEXT NOT NULL,
  last_name TEXT NOT NULL,
  email citext UNIQUE NOT NULL,
  hashword TEXT NOT NULL,
  salt TEXT NOT NULL,
  token TEXT UNIQUE,
  refresh TEXT UNIQUE DEFAULT uuid_generate_v4(),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email on users (email);
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_refresh on users (refresh);
CREATE TABLE IF NOT EXISTS orgs_users (
  orgs_id UUID NOT NULL REFERENCES orgs(id),
  users_id UUID NOT NULL REFERENCES users(id),
  role enum_org_role,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (orgs_id, users_id)
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email on users (email);
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email on users (email);