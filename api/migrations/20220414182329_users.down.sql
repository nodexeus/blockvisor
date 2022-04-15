-- Add down migration script here
DROP TABLE orgs_users;
DROP TABLE users;
DROP TABLE orgs;
DROP TYPE enum_org_role;