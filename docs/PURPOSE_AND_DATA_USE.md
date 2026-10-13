# Purpose, scope, and data use

## Community-driven open-source project discovery

Open Source Discovery Lab is designed to help an individual developer discover recurring challenges, gaps, accessibility needs, and requests described by people in public online communities. The research is intended to inspire ideas for useful software that could be developed and shared as open-source projects.

The current planned operation is **local, individual, and non-commercial**. This is a statement about the operator's present intended use, not a claim that Reddit has approved API access or that the source code is already distributed under an open-source license.

Examples include:
- Finding repeated difficulties with interoperability and exploring a community-maintained integration.
- Understanding accessibility-related software needs and exploring a freely available tool.
- Identifying documentation or workflow barriers that could be addressed through an open-source utility.
- Reviewing existing solutions and contradictory evidence before deciding whether new software would help.

These are possible outcomes, not promises of a particular project or evidence that Reddit users endorse one.

## Intended Reddit Data API behavior

If separately approved, the standalone application would:
1. Use a Reddit OAuth app requesting \`identity\` and \`read\` permissions.
2. Read a bounded sample of public posts from selected communities and explicitly requested thread comments.
3. Store local, short-lived evidence with timestamps, source URLs and relevant discussion metadata.
4. Group related observations through deterministic theme scoring and optional research-host-assisted interpretation.
5. Use the findings to consider open-source contributions, checking original sources and existing solutions.

The collector currently uses selected subreddit top posts for the past month; it does not autonomously perform site-wide search. In the dashboard a scan requests 25 posts. The application is not an embedded Reddit or Devvit app. It requires external local PostgreSQL storage, an independent dashboard, and optional MCP integration for a user-controlled research host.

## Conduct and data practices

- Read-only access. No posting, voting, direct messaging, moderation actions or engagement automation.
- No browser-session impersonation, unauthorized collection, internal Reddit GraphQL endpoint access, or bypass of controls.
- Configured conservative request limits and provider backoff; Reddit's actual permissions, quotas and policies take precedence.
- Default deletion of locally stored Reddit evidence after seven days, plus account deletion and disconnected-account removal.
- No sale or redistribution of Reddit discussions as a dataset; no training of AI models on Reddit content.
- Source material is treated as untrusted input; individual Redditors are not profiled for outreach.
- Publicly visible source code and community discussion **do not** imply authorization to collect or reuse Reddit content.
- Any change in scope, including paid services, commercial use of Reddit-derived analysis, or broader deployment, requires a new rights and policy review, and any separately required permission.

## What the implementation does not currently guarantee

- Approved or tested live Reddit Data API access; Reddit may refuse access or change available interfaces.
- A Reddit-independent local login; Reddit OAuth is still required for the dashboard's authentication flow.
- Fully automated external website scraping. MCP tools coordinate a **connected host** that must independently respect source permissions.
- Complete reconstruction of all historical context or exact compatibility with previous MCP tool outputs.
- Legally open-source distribution of this repository's code: an explicit OSS license has not yet been added.

The research engine retains optional general-purpose validation, planning and market-comparison capabilities from an earlier iteration. **They are not the reason for the requested non-commercial Reddit access and must not be treated as permission for commercial use of the resulting Reddit data.**

See [Architecture](ARCHITECTURE.md) and [MCP](MCP.md).
