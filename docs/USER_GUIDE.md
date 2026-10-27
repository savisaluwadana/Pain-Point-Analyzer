# Using the research workspace

The dashboard is for an **individual local operator** who wants to investigate real-world problems
and potential open-source contributions. The current built-in analysis is deterministic and
keyword-based; results are hypotheses to evaluate, not market-demand estimates.

## Navigation

| Area | Purpose | Action |
| --- | --- | --- |
| Community Sources | Collect short, permitted public observations | Select a connector, topic or target, and project |
| Evidence Library | Inspect and manually add evidence | Review source URLs, create permitted manual items, delete unwanted items |
| Analyze & Discover | Run the local `keywords-v2` analyzer | Scope the dataset and choose an analysis record budget |
| Project Ideas | Inspect saved candidates and make a human decision | Read representative evidence, note alternatives and validation findings |
| Research Jobs | Review direct Reddit OAuth collection jobs | Requires independently approved and connected Reddit API |
| Assisted Research | See MCP-hosted research jobs, runs and workspaces | Requires a separately configured compatible MCP host |
| MCP Integrations | Issue or revoke per-account API credentials | Give a secret to the host once; never publish it |
| Privacy & Settings | Inspect personal data controls | Export or delete research data and revoke connected access |

## Recommended workflow

### 1. Define a narrow research question

Good examples describe a user and workflow, such as "manual accessibility checks in
open-source dashboards", rather than "best billion-dollar startup idea". The
research topic helps select relevant evidence lexically and can exclude
paraphrases. An empty or undersampled result should not be treated as proof
that no problem exists.

### 2. Create a project and collect a small sample

Choose a project to keep scope stable across future collections. Start with
GitHub Issues or the six-item fictional demo. Other connectors have source-
specific restrictions: YouTube needs a known video ID and configured API key;
Discourse needs an allowed host. Archive searches are sampled and not
complete crawls. See [source documentation](SOURCES.md).

Every newly saved source record is linked to its origin. The app does not
guarantee a source's completeness, accuracy or continued availability.

### 3. Inspect evidence **before** trusting it

Open **Evidence Library** and read the original source. Check whether the
statement is genuinely a complaint, is a repost, has already been answered,
or describes a missing capability in the particular tool being discussed.

You can enter your own permitted evidence using a title, short explanation and
source URL. Manual import rejects direct Reddit URLs; use independently
approved source paths and respect third-party rights.

### 4. Run analysis

In **Analyze & Discover**, select the project, enter the research question, and
run an analysis with a limit of **1,000, 5,000 or 10,000** candidate records.
The HTTP API defaults to 1,000. Only one heavy analysis runs at a time; a
second concurrent request returns HTTP 429.

Read the output fields carefully:

- **Independent discussions:** conservative grouping by provider thread and normalized long copies.
- **Source families:** number of canonical providers supporting the category, not a credibility assessment.
- **Relative priority:** rank percentile *within that run* only; not severity, probability or spend.
- **Insufficient evidence:** fewer than three independent supporting discussions.
- **Coverage and omitted count:** whether the selected snapshot was bounded or topic rules excluded records.
- **Unavailable supporting evidence:** the original item changed, was deleted, or expired.

Use [analysis quality and limits](ANALYSIS_QUALITY.md) for formulas and reproducibility caveats.

### 5. Manually validate an opportunity

In **Project Ideas**, click **Review evidence and decision**. The brief resolves
currently available representative excerpts from the owning research run.
Record *your own* alternatives comparison and validation observations, then
choose an explicit stage:

`unreviewed` → `investigating` → `needs_validation` →
`ready_to_build` or `not_pursuing`.

These stages are manual, not automatic. The "ready" choice needs a meaningful
written justification, but does **not** mean purchase intent has been verified.
Older runs may not have representative excerpts.

[Full opportunity review reference](OPPORTUNITY_REVIEW.md)

### 6. Optional host-assisted research

A compatible MCP host can plan searches, ingest permitted evidence, annotate
selected records, manage jobs and help draft workspaces. The host controls its
own browsing/model operations; the local API does not autonomously call
arbitrary inference services. You must configure a revocable MCP key and
appropriate host permissions separately.

[MCP host integration](MCP.md) · [MCP tool index](MCP_TOOL_INDEX.md)

## Interpreting results responsibly

A recurring category may combine several distinct root causes. "Pricing
and cost" does not specify how much a buyer will spend. Distinct source
families are not independent representative samples. Scores cannot reliably
compare two separate research runs. Interviewing users or testing a prototype
is a separate process requiring consent and appropriate data handling.

## Export, cleanup and troubleshooting

Use the settings controls to export research before deletion; the export has
per-collection limits and omits Reddit body text. Individual evidence deletion
does not retroactively recalculate old reports; missing quotes are marked
unavailable when reloaded. Deleting an account cascades local research records.
Read [operations](OPERATIONS.md) and [data-use policy](PURPOSE_AND_DATA_USE.md)
before running research with content from other people.
