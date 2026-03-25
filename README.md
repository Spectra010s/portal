# Portal

Portal started as a personal way to move files from phone to computer. It is now a CLI tool designed to make file transfers effortless.

## Overview

Portal (Hiverra Portal) is a local-first file transfer tool built for simple, reliable sharing across devices. Today it focuses on CLI ↔ CLI transfers. Browser flows are planned.

## What Portal Does (Today)

- **CLI ↔ CLI transfers** over local networks
- **Files and folders** (recursive sends supported)
- **Discovery mode** with identity verification
- **Direct IP mode** for quick sends
- **Transfer history** with export and cleanup
- **Optional no-compress** mode (tar only)

## Planned

- **CLI ↔ Browser**
- **Browser ↔ CLI**

## Who It’s For

Portal is for anyone who wants a fast, local, no-fuss way to move files between devices without relying on external services. It is ideal for personal workflows and small team transfers on the same network.

## Install

**Release Installers (Recommended)**

**Shell script (Linux/macOS)**

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Spectra010s/portal/releases/download/v0.10.1/hiverra-portal-installer.sh | sh
```

**PowerShell (Windows)**

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/Spectra010s/portal/releases/download/v0.10.1/hiverra-portal-installer.ps1 | iex"
```

**npm (prebuilt binaries)**

```bash
npm install -g @hiverra/portal@0.10.1
```

**Android / Termux**

```bash
curl -LsSf https://github.com/Spectra010s/portal/releases/download/v0.10.1/hiverra-portal-android-installer.sh | sh
```

**Direct download**

- Download the release asset for your OS from GitHub Releases.

**Build From Source**

- Install Rust

### Build From Source

**Requires the Rust toolchain.**

```bash
git clone https://github.com/Spectra010s/portal.git
cd portal
cargo build --release -p hiverra-portal
```

## Quick Start

1. Run setup

```bash
portal config setup
```

2. Prepare to Recieve

On the destination device

```bash
portal receive
```

1. On sender

```bash
portal send path/to/file
```

## Usage Examples

**Start receiver**

```bash
portal receive
```

**Send via discovery**

```bash
portal send --to <username> path/to/file
```

**Send via direct IP**

```bash
portal send --address <ip> --port <port> path/to/file
```

**Send a folder (recursive)**

```bash
portal send -r path/to/folder
```

**No-compress**

```bash
portal send --no-compress path/to/file
```

**History (list + export)**

```bash
portal history
portal history export --detailed --output portal_history.json
```

**Update**
To update Portal:

```bash
portal update
```

## How to Run or Use It

Portal is a command-line tool. Common commands:

**Send a file**
Use this to send a specific file. If no file is specified, Portal will prompt you to select one.

```bash
portal send <file_path>
```

**Send with discovery (recommended)**
Sends to a user by username and verifies identity.

```bash
portal send --to <username> <file_path>
```

**Send via direct IP**
Use this when you already know the receiver’s IP and port.

```bash
portal send --address <IP_ADDRESS> --port <PORT> <file_path>
```

**Receive**
Puts Portal into listening mode to receive files.

```bash
portal receive
```

**Receive on a custom port**

```bash
portal receive --port <PORT>
```

**Configuration setup**
Interactive setup to configure username and default port.

```bash
portal config setup
```

**Set a configuration value**

```bash
portal config set <key> <value>
```

**Show a configuration value**

```bash
portal config show <key>
```

**List current configuration**

```bash
portal config list
```

## Roadmap

- [ ] CLI ↔ Browser: Send files to a web-based receiver via a temporary link.
- [ ] Browser ↔ CLI: Drag-and-drop from a browser to a listening terminal.
- [ ] Encryption: End-to-end encrypted tunnels for remote transfers.

## Documentation

Detailed guides for every workflow:

- [docs/index.md](https://github.com/Spectra010s/portal/blob/main/docs/index.md)
- [docs/install.md](https://github.com/Spectra010s/portal/blob/main/docs/install.md)
- [docs/usage.md](https://github.com/Spectra010s/portal/blob/main/docs/usage.md)
- [docs/cli-cli.md](https://github.com/Spectra010s/portal/blob/main/docs/cli-cli.md)
- [docs/troubleshooting.md](https://github.com/Spectra010s/portal/blob/main/docs/troubleshooting.md)
- [docs/faq.md](https://github.com/Spectra010s/portal/blob/main/docs/faq.md)

## Author

Github: [Spectra010s](https://github.com/Spectra010s)

## License

This project is licensed under the MIT License. See the [LICENSE](https://github.com/Spectra010s/portal/blob/main/LICENSE) file for details.

> Hiverra Portal: A lightweight CLI tool to transfer files between devices locally or remotely.
