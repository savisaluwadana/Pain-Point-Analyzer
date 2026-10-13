# Reddit Data API request — current proposed use

**Project:** Open Source Discovery Lab (formerly Pain Intelligence Lab)

**Source code:** https://github.com/savisaluwadana/Reddit-post-Analyzer

**Operator:** Individual developer; local, currently non-commercial research, subject to Reddit approval.

The following text describes the application's genuine current planned use and existing technical boundaries. It is not a claim that a license, an educational affiliation, or Reddit API approval already exists. If paid/commercial use is planned or occurs, disclose it accurately to Reddit and request any additional permissions.

## What benefit/purpose will the bot/app have for Redditors?

Open Source Discovery Lab is intended to help identify recurring community needs and challenges that may be addressed through freely available open-source software. Its goal is to turn observations from discussions into ideas for useful integrations, accessibility improvements, developer utilities, documentation, and other community-maintained tools.

The app does not post, vote, direct-message, or automatically interact with Redditors. Any projects inspired by this research would be independently reviewed and developed; the app does not imply that Reddit users endorse a particular solution.

## Provide a detailed description of what the Bot/App will be doing on Reddit

Open Source Discovery Lab is a standalone, locally operated research workspace for a single developer. The intended present use is non-commercial and focused on exploring open-source project ideas from recurring issues discussed in online communities.

With separately approved, read-only Reddit API access, the application would retrieve limited samples of public posts from selected subreddits and comments from specific discussion threads. It would retain brief, temporary evidence records and original source references in a local PostgreSQL database. A Rust backend categorizes recurring themes and helps identify possible open-source software contributions. The user can review evidence and verify suggestions before deciding what to build.

For example, repeated discussions about incompatibilities among freely available tools could inspire an open-source integration. Reports of accessibility gaps or repetitive manual tasks could motivate reusable utilities or documentation improvements.

The current collector uses bounded, selected-subreddit top-post requests rather than an unrestricted Reddit-wide crawler. It has conservative request budgets and a default seven-day Reddit evidence retention policy, with source and account deletion controls. An optional MCP integration permits user-controlled research assistance; it does not train models on Reddit content or bypass Reddit's permissions. This app does not automatically publish, vote, contact users, or resell Reddit data.

The repository also contains general-purpose research/validation tools. The requested Reddit access is for the **current local, non-commercial project-discovery use only**; those tools must not be used to commercialize Reddit-derived material without separate authorization.

## What is missing from Devvit that prevents building on that platform?

This application is designed for individual, locally operated research across selected communities, rather than for interactive experiences embedded inside Reddit. It requires a standalone dashboard, a local PostgreSQL evidence database, and optional integration with user-controlled development/research tools through MCP.

As currently understood, Devvit does not directly provide this same external, cross-community local research workspace. I would welcome guidance about an approved Devvit-supported alternative if Reddit offers one.

## Link to source code

https://github.com/savisaluwadana/Reddit-post-Analyzer

## Subreddits for consideration

For example: r/opensource, r/selfhosted, r/programming, r/linux, and r/SaaS. List **only** communities genuinely relevant to the planned scope, and respect each community's rules.

## Other form fields

- Use the actual Reddit username operating the application.
- The Reddit account name is not the email address.
- Attachments are optional; point to source code and scope documentation.
- Choose "I'm a developer" / "an app that does not work in the Devvit ecosystem" only if those statements accurately describe the project.

**Important:** Neither presenting this project as community-oriented nor describing a local, non-commercial use guarantees approval. Reddit controls access and its terms must be followed.
