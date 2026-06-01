# Contributing to Hiverra Portal

Hiverra Portal is a local-first file transfer CLI. Keep changes focused, practical, and easy to review.

## How to Contribute

### Reporting Bugs

Before opening a bug report, check existing issues to avoid duplicates.

A good bug report includes:

- Portal version or commit
- operating system
- command used
- steps to reproduce
- expected behavior
- actual behavior
- relevant logs or terminal output

### Suggesting Features

For feature requests, explain the workflow problem first. Portal should stay local-first, predictable, and safe by default.

Useful feature requests include:

- why the feature is needed
- example commands or user flow
- expected behavior
- tradeoffs or security concerns

### Code Changes

Use a focused branch for each change. Keep PRs scoped to one issue or one clear task.

Common Rust commands:

```sh
cargo fmt
cargo check
cargo test
```

For web docs or landing page changes, work inside `apps/web` and use the package scripts there.

## Commit Messages

Hiverra Portal uses Conventional Commits.

Good examples:

```text
fix(update): handle archive layout during self-update
ci(release): replace cargo-dist workflow
chore(release): bump version to 0.11.1
docs(web): update install instructions
build(deps): bump tar from 0.4.45 to 0.4.46
```

Git-AIC is optional. This repository includes `git-aic.config.json` so contributors who use Git-AIC get Portal-specific commit guidance.

```sh
git aic
```

Manual commits are fine. Keep subjects concise, use scopes when helpful, and include a short body with bullet points only when the change is complex or the reason is not obvious.

## Pull Requests

A good PR should:

- have a clear title
- explain the behavior change directly
- link related issues with `Closes #123` when applicable
- mention any tests or checks run
- avoid unrelated formatting, dependency, or generated-file changes

By contributing, you agree that your contributions are provided under this repository's license.

## Documentation

Update docs when behavior, commands, install flow, troubleshooting, or release process changes.

Docs live mainly in:

- `README.md`
- `apps/web/content/`

## Security

Do not report security issues in public issues. See `SECURITY.md` instead.
