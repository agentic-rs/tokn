# Persistence schemas

This directory is the authoritative source for SQLite schema files used by the
`persistence` crate. Reorganizing files here must not change the effective DB
schema unless a migration is intentionally added.

## Layout

- `snapshot/<db>/vX.Y.Z.sql`: the canonical current schema for a fresh database.
- `migrations/<db>/000N_name.sql`: the historical forward-only migrations for an
  existing database.

## Version mapping

- The current `v0.0.0` files are the original released schemas that map to
  migration version `1` in Rust.
- The current `v0.1.1` files are the latest equivalent schemas for fresh
  databases and replace the old `000_bootstrap.sql` files.

## Rules

- Do not edit old migration files after release.
- Keep each snapshot schema equivalent to applying all migrations for that DB.
- Restructure-only changes in this directory must not change SQL semantics.
