# Usage

This page is a practical, step‑by‑step guide to using Portal.

## Quick Start

Receiver:

```bash
portal receive
```

Sender:

```bash
portal send --to <username> path/to/file
```

## Common Commands

```bash
portal send <path>
portal receive
portal update
```

## Discovery Mode (Recommended)

Use discovery when both devices are on the same network.

Receiver:

```bash
portal receive
```

Sender:

```bash
portal send --to <username> path/to/file
```

## Direct IP Mode

Use this when you already know the receiver’s IP and port.

```bash
portal send --address <IP_ADDRESS> --port <PORT> path/to/file
```

## Send a Folder (Recursive)

```bash
portal send -r path/to/folder
```

## No‑Compress Mode

Use this when CPU is the bottleneck or files are already compressed.

```bash
portal send --no-compress path/to/file
```

## Receive to a Specific Folder

```bash
portal receive --dir /path/to/save
```

## Receive on a Custom Port

```bash
portal receive --port 7878
```

## Transfer History

List history:

```bash
portal history
```

Show a specific record:

```bash
portal history 3
```

JSON output:

```bash
portal history --json
```

Clear history:

```bash
portal history clear
```

Delete a record:

```bash
portal history delete 3
```

Export:

```bash
portal history export --output portal_history.json
```

Export detailed:

```bash
portal history export --detailed --output portal_history.json
```

## History Filters

Filter by mode:

```bash
portal history --mode send
portal history --mode receive
```

Filter by date:

```bash
portal history --since 2026-03-16
```

Limit results:

```bash
portal history --limit 20
```

## Update

```bash
portal update
```

## Configuration

Interactive setup:

```bash
portal config setup
```

Set a value:

```bash
portal config set <key> <value>
```

Show a value:

```bash
portal config show <key>
```

List all:

```bash
portal config list
```
