# Five-minute demo (fictional evidence; no external APIs)

This is a **reproducible local demonstration**, not a live feed or a claim about any real community member. It seeds six
synthetic research observations into your _own_ local workspace and runs the built-in heuristic analysis. It does not
access Reddit, GitHub, Medium, or any third-party website.

## Prerequisites

- Docker Engine and Docker Compose v2
- `curl`, `jq`, and OpenSSL
- A local checkout of this repository

## 1. Configure secrets

From the repository root:

```bash
cp .env.example .env
openssl rand -hex 32
openssl rand -hex 32
```

Place **different** generated 64-character hex values in the `TOKEN_ENCRYPTION_KEY_HEX` and `LOCAL_LOGIN_TOKEN` fields
of `.env`. Set a strong `POSTGRES_PASSWORD`. Never commit or share `.env`.

No Reddit approval, YouTube key, GitHub token or API account is required.

## 2. Start services

```bash
docker compose up --build -d --wait
curl --fail-with-body http://localhost:3000/api/health
```

If the health check fails, inspect `docker compose logs --tail=100`.

## 3. Run the example

```bash
bash scripts/try-demo.sh
```

The script creates **one new project** labeled `DEMO (fictional)`, loads six records from `examples/demo-evidence.json`,
and saves a research report. Repeating it creates another demo project rather than altering your real projects.

Expected outcome (exact scores are implementation details):

```text
Success: created an explicitly fictional demo project with 6 evidence items.
Theme groups (heuristic, not verified needs):
  - ...
```

Open [http://localhost:3000](http://localhost:3000) and log in with `LOCAL_LOGIN_TOKEN`. Go to **Evidence Library** to
view records, or **Analyze & Discover** to view the saved theme report. Read the references and caveats before
interpreting any scores.

## 4. Then try a real permitted source

Select **Community Sources** and choose **GitHub Issues**. Try the query `"feature request" integration`. Public GitHub
access is subject to rate limits and may fail; an optional GitHub token can be configured in `.env`. The result is an
observation, **not** proof of demand. See [Source connectors](SOURCES.md).

## 5. Clean up

Delete the demo evidence individually in **Evidence Library**. Deleting a project by itself **does not delete
evidence**; it only unassigns the records. If this is a throwaway install and you need to erase _all_ local data, use
the account deletion option in **Privacy & Settings** or stop services and explicitly remove Docker volumes with
`docker compose down -v`. The latter also destroys _real_ local data; never use it on a workspace you want to keep.

## Report your first-run outcome

Please use the repository's [bug report template](../.github/ISSUE_TEMPLATE/bug_report.yml) to report an error, or the
[feedback template](../.github/ISSUE_TEMPLATE/feedback.yml) if the setup works. Do not attach personal tokens or
third-party post text. Helpful feedback includes OS, Docker version, the step that failed, and whether the synthetic
report displayed at least one theme.
