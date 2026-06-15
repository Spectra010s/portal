# 🎉 Hiverra Portal v0.12.0 Released!

We're thrilled to announce **Hiverra Portal v0.12.0** with major reliability and infrastructure improvements!

## ✨ What's New

### Discovery Fallback
Portal now keeps multicast discovery as the first path, then falls back to UDP broadcast when multicast doesn't work on the local network. This dramatically improves reliability across different network environments.

### Receiver Startup Reliability
Receive mode no longer needs friendly local IP detection to succeed before the listener can start. This means faster, more reliable receiver startup in challenging network conditions.

### Custom Release Pipeline
Portal releases now use repository-owned workflows instead of cargo-dist-generated installer scripts, giving us better control over the release process across all platforms.

## 🚀 Improvements

- **Receiver beacons** now include broadcast targets alongside the existing multicast beacon
- **Discovery failures** now return direct-address guidance so users can continue with `portal send --address <receiver-ip>`
- **Multi-platform releases** now managed by repository workflows (Windows, Linux, macOS, Android, npm)
- **Documentation** added Git-AIC commit guidance and comprehensive contributor, support, and security guides

## 🐛 Bug Fixes

- Corrected Portal install links on docs/landing site to use `portal.biuld.app`
- Fixed Windows MSI packaging in the release workflow
- Updated Intel macOS release runner (GitHub removed the `macos-13` runner label)
- Fixed release creation checkout sequence for `gh release create`

## 📥 Install Now

### Shell (macOS/Linux)
```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Spectra010s/portal/releases/download/v0.12.0/hiverra-portal-installer.sh | sh
```

### PowerShell (Windows)
```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/Spectra010s/portal/releases/download/v0.12.0/hiverra-portal-installer.ps1 | iex"
```

### npm
```sh
npm install -g @hiverra/portal@0.12.0
```

### Android / Termux
```sh
curl -LsSf https://github.com/Spectra010s/portal/releases/download/v0.12.0/hiverra-portal-android-installer.sh | sh
```

## 📦 Downloads

All platform binaries are available on the [release page](https://github.com/Spectra010s/portal/releases/tag/v0.12.0), including checksums for verification.

## What's Next?

Check out the [open discussions](https://github.com/Spectra010s/portal/discussions) for our upcoming roadmap on:
- Hotspot reliability improvements
- Local web transport mode
- Desktop and mobile app layers

---

**Questions or feedback?** Head over to our [Discussions](https://github.com/Spectra010s/portal/discussions) or [open an issue](https://github.com/Spectra010s/portal/issues)!

Thank you for using Portal! 🚀
