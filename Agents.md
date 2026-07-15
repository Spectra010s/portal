# AGENTS.md

## What Hiverra Portal Is

Hiverra Portal is a local-first file transfer tool built in Rust. Its current production path is CLI-to-CLI transfer across devices on the same network. The goal is simple: move files and folders between devices without needing a hosted relay, cloud storage, accounts, or a third-party transfer service.

Portal started as a personal phone-to-computer file transfer workflow. It is now a cross-platform CLI intended to work on Linux, macOS, Windows, and Android through Termux, with browser, desktop, and mobile flows planned around the same local-first model.

Use the product name as **Hiverra Portal** in formal docs, releases, and public-facing text. **Portal** is fine in normal command docs and casual usage.

## What Portal Can Do Today

- Send files from one device to another over the local network.
- Send folders recursively with `portal send -r`.
- Receive files with `portal receive`.
- Discover receivers by username when both devices are on the same network.
- Connect directly with `--address` and `--port` when discovery is unreliable or manual setup is preferred.
- Verify receiver identity in discovery mode using the receiver's advertised session node ID.
- Keep local transfer history in `~/.portal/history.jsonl`.
- Export transfer history as JSON.
- Store session logs in `~/.portal/_logs/`.
- Update itself with `portal update`.
- Run with quieter or more verbose terminal logging through `-q` and `-v`.
- Control file log verbosity with `PORTAL_LOG` or `RUST_LOG`.

## Main User Flows

### First-Time Setup

```bash
portal config setup
```

This sets up the local username and default network settings. Usernames commonly use the `@portal` suffix.

### Receive

```bash
portal receive
```

The receiver opens a TCP listener, starts advertising a discovery beacon, and waits for a sender. If Portal can detect a friendly local IP, it shows that address. If it cannot, it still listens on all interfaces and prints manual connection tips.

### Send With Discovery

```bash
portal send --to <username> path/to/file
```

Discovery mode is the preferred user experience when both devices are on the same network. The sender searches for the named receiver, opens a TCP connection after discovery succeeds, then verifies that the receiver's claimed node ID matches the beacon.

### Send With Direct IP

```bash
portal send --address <receiver-ip> --port <port> path/to/file
```

Direct IP mode bypasses discovery. It is useful for hotspots, controlled networks, debugging, and cases where discovery traffic is blocked. This mode does not provide the discovery-based node ID verification step.

### Send Folders

```bash
portal send -r path/to/folder
```

Directories are intentionally rejected unless `-r` is passed. Do not change this behavior casually; it prevents accidental large transfers.

### History

```bash
portal history
portal history --json
portal history export --detailed --output portal_history.json
portal history clear
portal history delete <id>
```

History is transfer-oriented, not log-oriented. It is useful for users and agents because it records what happened across send and receive operations, including failures and partial transfer details.

## How Portal Helps People

- It moves files without uploading them to a cloud service.
- It works in local network situations where internet access is unavailable or undesirable.
- It supports direct device-to-device workflows for personal productivity.
- It can help small teams transfer files on the same LAN without setting up shared drives.
- It gives users a manual fallback through direct IP mode when discovery fails.
- It keeps a local history so users can audit what was transferred.
- It is scriptable because the core interface is a CLI.

## How Portal Can Help Agents

Portal is useful to coding agents and automation agents because it provides a local, scriptable file transfer path.

Agents can use Portal to:

- Move generated artifacts from one device to another without requiring cloud upload.
- Transfer logs, screenshots, builds, archives, or test outputs across local machines.
- Receive files from a user-controlled device during debugging.
- Send release artifacts to another local device for manual testing.
- Export transfer history as JSON and inspect it programmatically.
- Use direct IP mode for deterministic automation when discovery is not needed.

When writing agent workflows around Portal, prefer explicit commands and predictable paths:

```bash
portal receive --dir /path/to/inbox --port 7878
portal send --address <ip> --port 7878 /path/to/artifact
portal history --json
```

For human-friendly workflows, prefer discovery:

```bash
portal receive
portal send --to <username> /path/to/file
```

## Architecture Notes For Agents

- Portal is a Rust CLI package named `hiverra-portal`.
- The binary name is `portal`.
- The CLI entrypoint is `src/main.rs`.
- Command definitions live in `src/commands.rs`.
- Sender logic lives under `src/sender/`.
- Receiver logic lives under `src/receiver/`.
- Discovery logic lives under `src/discovery/`.
- History logic lives under `src/history/`.
- Logging setup lives in `src/logger.rs`.
- Progress rendering lives in `src/progress.rs`.
- Update logic lives in `src/update.rs`.
- Web docs and landing content live under `apps/web/`.

The current stable transfer path is:

1. Receiver runs `portal receive`.
2. Receiver binds a TCP listener.
3. Receiver sends discovery beacons.
4. Sender runs `portal send --to <username> <path>` or direct IP mode.
5. Sender connects over TCP.
6. Discovery mode verifies receiver node ID.
7. Sender sends a transfer manifest.
8. File stream starts.
9. Both sides write history records.

## Discovery And Networking

Discovery is local-network only. The documented discovery path uses multicast beacons, and current implementation work has also explored subnet broadcast fallback for networks where multicast is unreliable.

Important behavior:

- Discovery is for finding a receiver by username.
- Direct IP mode is for bypassing discovery.
- Discovery mode can verify receiver identity through node ID matching.
- Direct IP mode intentionally skips that identity verification because there is no beacon-derived expected node ID.
- Hotspot and phone-to-laptop networks may behave differently across operating systems and network privacy settings.

When changing discovery, keep user messaging clear. If discovery fails, users should know they can use:

```bash
portal send --address <receiver-ip> --port <port> <file-or-folder>
```

## Logging

Portal separates terminal logging from file logging.

- Default terminal log level is `warn`.
- `portal -v ...` raises terminal logs to `info`.
- `portal -q ...` reduces terminal logs to `error`.
- File logs are written to `~/.portal/_logs/`.
- File logs default to `debug`.
- `PORTAL_LOG` is preferred over `RUST_LOG`.
- `RUST_LOG` is the fallback environment variable.

Use file logs for diagnostics. Do not make normal terminal output noisy just because a detail is useful for debugging.

## Documentation

Public docs live in the web app under `apps/web/content/`.

Important docs:

- `apps/web/content/install.mdx`
- `apps/web/content/usage.mdx`
- `apps/web/content/cli-cli.mdx`
- `apps/web/content/troubleshooting.mdx`
- `apps/web/content/faq.mdx`

The old top-level `docs/` directory was removed because the website docs should be the source of truth.

## Package And Release Notes

- Rust crate version is in `Cargo.toml`.
- The CLI package published to npm is `@hiverra/portal`.
- The web app is under `apps/web`.
- Release artifacts are created by GitHub Actions.
- Windows releases include an MSI.
- Shell and PowerShell installers are release assets.
- npm publishing in the release workflow is for the CLI package, not for the docs app.

Do not assume all package manager changes affect the CLI package. The web app package manager and the release workflow's `npm publish` step serve different purposes.

## Repo Working Rules For Agents

- Do not delete generated dependencies or caches unless the user explicitly asks. This repository may be used from Termux with limited mobile data.
- Do not commit `node_modules/`.
- Do not run installs, builds, or checks unless the user asks or approves.
- Use `/data/data/com.termux/files/usr/tmp` for temporary files that should not be committed.
- Keep PR bodies concise. Avoid a `Summary` heading unless the user asks for one.
- Do not delete remote or local branches unless explicitly asked.
- Avoid unrelated edits in the same commit.
- Preserve the user's wording choices in docs unless there is a concrete correctness issue.

## Product Direction

Planned Portal directions include:

- CLI-to-browser transfer flow.
- Browser-to-CLI transfer flow.
- Desktop and mobile wrappers around the same local-first transfer model.
- Better hotspot and direct-network reliability.
- Offline-friendly documentation or bundled help for future app surfaces.

The long-term product idea is that Portal should remain useful even without internet access. Cloud transfer may exist later, but it is not the core design center today.
