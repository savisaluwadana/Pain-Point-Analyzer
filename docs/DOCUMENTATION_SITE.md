# GitHub Pages documentation site

This repository publishes **public, static Markdown documentation** using
MkDocs Material and GitHub Actions. It is **not** a SaaS deployment.
No Docker application, database or user token is included in the static site.

## Expected URL

```text
https://savisaluwadana.github.io/Pain-Point-Analyzer/
```

The link becomes functional **only after** repository Pages settings are
enabled and the first successful Pages deployment finishes. Creating the
workflow or merging a PR alone does not guarantee that Pages is live.

## Maintainer setup (one time)

1. Review and merge the documentation PR into `main`.
2. Open the repository's **Settings → Pages**.
3. Under **Build and deployment → Source**, select **GitHub Actions**.
   Repository/org policy and permissions can affect whether this option
   is available.
4. Open **Actions → Publish project documentation (GitHub Pages)**.
   If the push-triggered run has not succeeded, use **Run workflow** on
   `main` after enabling Pages.
5. Verify the deployment's published URL in the workflow and Pages
   settings before distributing the link publicly.

The workflow requests `pages: write` and `id-token: write` solely
for publishing the static Pages artifact. It does not receive runtime
API/database keys and should not be changed to publish an application
build, `.env`, database dump or private research evidence.

## How it is built

- `mkdocs.yml`: site metadata, navigation, theme, search, and Markdown plugins.
- `docs/index.md`: public project introduction.
- `docs/*.md`: task-based guides and module references.
- `docs/assets/stylesheets/extra.css`: limited custom visual styling.
- `docs/requirements.txt`: pinned documentation-only Python dependency.
- `.github/workflows/pages.yml`: publish from `main`, never from unreviewed PR code.
- `.github/workflows/ci.yml`: docs build on PRs with `--strict`.

Mermaid architecture diagrams are rendered as static-site documentation
content, not by the running API. The site uses Material's built-in
client-side navigation and search.

## Preview locally

```bash
python3 -m venv .venv-docs
. .venv-docs/bin/activate
python -m pip install -r docs/requirements.txt
python -m mkdocs serve
```

Visit **http://127.0.0.1:8000**. Before opening a documentation PR, run:

```bash
python -m mkdocs build --strict
```

Strict mode makes incorrect local links/navigation and most build
warnings fail CI. Review the generated site visually on desktop and
mobile, check keyboard navigation, search, dark mode, diagrams,
contrast and important internal links. A successful CI build is not a
substitute for that manual review.

## Updating documentation

Keep the existing Markdown files as the source of truth. Add new
reference pages to the `nav` in `mkdocs.yml`, link to them from
`docs/index.md`, and document only functionality that is actually
implemented. Mark proposals and unverified provider access explicitly.
Code paths outside `docs/` should link to their GitHub source URL.

Every merge to `main` changing `docs/**`, `mkdocs.yml` or the Pages
workflow triggers a new static publish. The `workflow_dispatch`
trigger supports manual re-publication.

## Troubleshooting

| Issue | Resolution |
| --- | --- |
| Site URL returns 404 | Confirm Pages is enabled, a deployment has succeeded, and the project subpath is retained |
| Actions deployment says Pages is not enabled | Set Source to GitHub Actions in Settings → Pages |
| Build fails on broken links | Fix Markdown links relative to the `docs/` directory or point out-of-repo links to GitHub |
| Site styles missing | Check `site_url` includes the `/Pain-Point-Analyzer/` base path |
| No workflow after merge | Ensure workflow exists on `main`, or run it manually |
| User expects the dashboard | This website is docs only; run Docker Compose locally for the application |

## Security reminder

Do not configure a public host or wildcard CORS for the Rust/Next.js alpha
to make the Pages site interactive. The site must remain a static,
publicly readable reference. Read the [security policy](SUPPORT_AND_SECURITY.md).
