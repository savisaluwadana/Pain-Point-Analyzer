# Local operations and recovery

The default deployment is a three-service **localhost-only alpha**, not a
managed service. This guide helps a local maintainer inspect, back up,
upgrade, troubleshoot and delete their own data.

## Runtime inventory

| Compose service | Purpose | Exposure |
| --- | --- | --- |
| `web` | Next.js dashboard and same-origin API proxy | Bound to `127.0.0.1:3000` by default |
| `api` | Rust/Axum collectors, analysis, MCP execution and Tokio worker | Internal Compose network |
| `postgres` | PostgreSQL 17 and SQLx migration state | Internal Compose network and named volume |

Only the **documentation** is appropriate for publication through GitHub
Pages. Pages never serves the Rust API, your database, cookies, or
collection tokens.

## Verify health

```bash
docker compose ps
curl --fail http://127.0.0.1:3000/api/health
docker compose logs --tail=100 api web postgres
```

A healthy `/api/health` indicates the API can query its PostgreSQL
connection. It does not certify provider credentials, community API
availability, security review or analysis accuracy.

## Protect configuration and secrets

Keep `.env` outside Git, use three separately generated secrets
([setup instructions](GETTING_STARTED.md)), do not share `pil_session` or
MCP bearer values, and use the dashboard to revoke no-longer-needed MCP
keys. Local login is deliberately restricted to localhost `WEB_URL`.

Do not turn on direct Reddit API collection by setting
`REDDIT_API_APPROVED=true` without independent provider authorization.
The archive adapter is a separate third-party interface, not proof of
commercial permission.

## Back up local data

The PostgreSQL named volume persists across `docker compose down`. To
create a **database dump** without dropping anything:

```bash
mkdir -p backups
docker compose exec -T postgres pg_dump -U pain -d pain_intelligence -Fc > backups/pain-$(date +%Y%m%d).dump
```

A database dump can contain tokens in encrypted form, private notes and
licensed excerpts. Store it in a restricted, encrypted location; do not
commit the `backups/` folder. Verify that the file is non-empty and that
`pg_restore -l` can list it before calling it a usable backup.

```bash
pg_restore -l backups/pain-YYYYMMDD.dump | head
```

Do a **test restore into a disposable database**, not the live one:

```bash
docker compose exec -T postgres createdb -U pain pain_restore_test
docker compose exec -T postgres pg_restore -U pain -d pain_restore_test --no-owner \
  < backups/pain-YYYYMMDD.dump
```

Confirm expected tables and row counts before clearing the disposable
database. A complete disaster recovery drill also verifies application
startup against a restored copy and the compatibility of its migration
version. Never automatically drop or overwrite the live volume in a
documentation command.

## Upgrade safely

1. Record the current Git commit, back up the PostgreSQL database and
   review the PR's migration/release notes.
2. Keep applied `db/*.sql` migrations unchanged; new SQLx migrations are
   additive. The historical `002` numbering gap is intentional.
3. Pull a reviewed commit and rebuild the local services:
   ```bash
   git pull --ff-only
   docker compose up --build -d --wait
   ```
4. Verify `/api/health`, open the dashboard and inspect the research
   project, evidence, analysis, opportunity-review and MCP views.
5. If an update fails, inspect the logs and **do not** blindly run
   destructive cleanup. A code rollback may not reverse a newly applied
   database migration; use a tested recovery plan or restore a verified
   backup into a separate environment.

## Known failures and response

| Symptom | Likely cause or next check |
| --- | --- |
| `/api/health` is 503 | PostgreSQL unavailable or migrations failed |
| Local login is 503 | `LOCAL_LOGIN_TOKEN` is unset or origin is not localhost |
| 401 / 403 during mutation | Session is missing or `Origin` does not match `WEB_URL` |
| Analysis returns 429 | Another CPU-heavy analysis is already running |
| Collection returns 429 | Local/provider rate budget or archive pacing |
| Upstream 403 / 502 | Provider permission refusal, changed schema, or connectivity |
| Source appears deleted in an old report | On-read hash hydration intentionally marks it unavailable |
| Reddit job will not run | Direct OAuth is unapproved, disconnected or quota restricted |
| `docker compose down -v` removed data | Volume deletion is destructive; recover only from a tested backup |

An error from a third-party provider is not justification to bypass the
provider's access control or change to unofficial scraping.

## Retention, exports, deletion

Short source evidence is cleaned by the background task according to
provider rules and `SOURCE_RETENTION_DAYS` /
`REDDIT_RETENTION_DAYS` (typically seven days, supported range
1–30 days). Saved historical reports retain scoped IDs and hashes but
not durable copied representative excerpts. The current on-read
hydration resolves quotes only while matching evidence exists.

Account exports have bounded collections and deliberately omit secret
values and raw Reddit post/comment bodies. A `truncated: true` export
is not complete. Account deletion cascades owner records; provider-side
revocation may need manual confirmation. Always observe the original
source's license, terms and deletion requirements.

See [security and reporting](SUPPORT_AND_SECURITY.md),
[data use](PURPOSE_AND_DATA_USE.md) and [database design](DATA_MODEL.md).

## Documentation-only publishing

The [GitHub Pages guide](DOCUMENTATION_SITE.md) covers publishing these
Markdown pages as static HTML. It does **not** change or publicly expose
the locally running web or API services.
