# Install and first run

This guide starts the **local research application**. The GitHub Pages website is only the public documentation.

!!! warning "Alpha deployment boundary"
    The default login uses a shared local secret, not end-user account management.
    Keep the application on localhost. Do not expose the Docker port through a
    reverse proxy, public VM, internet tunnel, or public domain.

## Prerequisites

- Docker Engine and Docker Compose v2
- `git`, `curl`, `jq` and OpenSSL
- Enough disk space for Docker images, Cargo dependencies and PostgreSQL data
- No AI model API key and no Reddit OAuth credentials are required for the fictional demo

## 1. Clone the project

```bash
git clone https://github.com/savisaluwadana/Pain-Point-Analyzer.git
cd Pain-Point-Analyzer
cp .env.example .env
```

## 2. Configure three **independent** secrets

Generate each value separately. Copy each generated value to the matching
entry in your **local** `.env` file:

```bash
openssl rand -hex 32   # TOKEN_ENCRYPTION_KEY_HEX
openssl rand -hex 32   # LOCAL_LOGIN_TOKEN
openssl rand -hex 24   # POSTGRES_PASSWORD
```

Keep the values private. `TOKEN_ENCRYPTION_KEY_HEX` protects stored OAuth tokens;
`LOCAL_LOGIN_TOKEN` protects the local browser login; `POSTGRES_PASSWORD`
is the database password. Never commit `.env` to Git.

Leave `WEB_URL=http://localhost:3000`, `COOKIE_SECURE=false`, and
`REDDIT_API_APPROVED=false` for the local default. Do not set Reddit approval
true unless you have actually received provider approval.

## 3. Start the stack

```bash
docker compose up --build -d --wait
curl --fail http://127.0.0.1:3000/api/health
```

The stack has three services: `web`, `api`, `postgres`. Next.js listens on
`127.0.0.1:3000` by default. Rust applies SQLx migrations at startup and
launches the internal Tokio worker.

Open **http://localhost:3000**, choose local login, and enter the
`LOCAL_LOGIN_TOKEN` value.

## 4. Run the source-free example

```bash
bash scripts/try-demo.sh
```

This script seeds six **fictional** observations and runs a keyword-analysis
report; it does not call Reddit, YouTube, GitHub, or an AI model. Visit
**Evidence Library**, **Analyze & Discover**, and **Project Ideas** to inspect
the results and try the manual validation workflow.

Read the [full demo walkthrough](DEMO.md) for cleanup instructions and expected
behavior.

## 5. Try one permitted source

Open **Community Sources**, choose GitHub Issues or another available source,
enter a specific research phrase, and perform a small collection. Review the
source URL and permission requirements before using the findings. The optional
Reddit archive adapter has separate limitations explained in
[Reddit archive setup](ARCTIC_SHIFT.md).

Source-specific credentials such as `YOUTUBE_API_KEY` and the
`DISCOURSE_ALLOWED_HOSTS` allowlist are optional. Read the
[connector guide](SOURCES.md) before enabling them.

## Common issues

| Symptom | Check |
| --- | --- |
| Local login unavailable | `LOCAL_LOGIN_TOKEN` must contain a generated 64-character hex value; `WEB_URL` must use localhost |
| Startup fails | Verify all three secrets, Docker health, and `docker compose logs --tail=100` |
| API health fails | Check `postgres` health, SQLx migrations, and `api` logs |
| Source reports quota/backoff | Respect the provider's backoff; do not rotate accounts or bypass limits |
| Reddit OAuth controls disabled | Expected without independently approved OAuth access |
| Page loads but no reports | Run the demo or collect permitted evidence, then trigger an analysis |

Read [operations](OPERATIONS.md) for backup, restore and upgrade procedures.

## Stop safely

```bash
docker compose down
```

This retains the named database volume. **Do not run `docker compose down -v`**
unless you intentionally want to delete its PostgreSQL data and have a verified
backup or no need to retain it.
