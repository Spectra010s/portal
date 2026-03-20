# CLI to CLI

**Receiver**
1. `portal receive`
2. Share your username with the sender.

**Sender**
1. `portal send --to <username> <path>`
2. Confirm the receiver identity prompt.

**Direct IP**
- `portal send --address <ip> --port <port> <path>`

**Notes**
- Discovery mode verifies identity.
- Direct IP mode skips identity verification.
