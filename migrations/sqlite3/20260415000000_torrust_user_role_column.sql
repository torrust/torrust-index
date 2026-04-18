-- ADR-T-008 Phase 1: Add `role` TEXT column, populate from `administrator`.
ALTER TABLE torrust_users ADD COLUMN role TEXT NOT NULL DEFAULT 'registered';
UPDATE torrust_users SET role = 'admin' WHERE administrator = TRUE;
