# Database migrations

The gap at `002` is historical. Commit `db4ce09` added `v2/db/002_legacy_records.sql`, a read-only archive table for
legacy MongoDB imports. Commit `c659fc7` removed that migration when promoting the Rust app from `v2/` to the repository
root and retiring the legacy JavaScript stack. The root migration set therefore starts with `001`, `003`, and `004`.

Do not recreate `002` or renumber existing migrations. SQLx applies migrations in version order; version numbers do not
need to be consecutive. Applied SQL files are immutable, including formatting, because SQLx verifies checksums. Add
schema changes as new numbered migrations.

The supported local stack and CI both use PostgreSQL 17.
