# Troubleshooting

**Discovery Not Finding Receiver**

- Confirm both devices are on the same network.
- Try direct IP mode.

**Port Issues**

- Use `portal receive --port <port>`.
- Use the same port on sender.

**Partial Transfers**

- Receiver history will show partial items when a transfer fails mid-stream.

**Logs**

- Logs are written to `~/.portal/_logs/`.
- PowerShell: `$env:RUST_LOG="debug"; portal send ...`
- Bash: `RUST_LOG=debug portal send ...`
