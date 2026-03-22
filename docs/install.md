# Install

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
- Install Rust.
- `cargo build -p hiverra-portal`

**First-Time Setup**
- `portal config setup`

This will set your username and default port.
