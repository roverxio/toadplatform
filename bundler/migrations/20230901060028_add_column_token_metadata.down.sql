-- Add down migration script here
alter table if exists token_metadata drop column if exists token_type;
alter table if exists token_metadata drop column if  exists name;
alter table if exists token_metadata drop column if exists is_supported;
