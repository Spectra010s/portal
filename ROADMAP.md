# Portal Roadmap (Updated)

There are currently 6 Feature issues, including a bug, if we look at the dependencies between these 6 issues, some are structural (how the app is built) and some are behavioral (how the app acts).

Here is the strategic breakdown of the 6 issues and which will be done first:

1. Hierarchical Config [(Issue 31)](https://github.com/Spectra010s/portal/issues/31) — This will be done first
   Why: This is the foundation. If we do the "Download Dir" or "Logging" first, we’ll be adding them to a flat list that we're just going to break later when we nest them.

- Goal: Move from cfg.username to cfg.user.username.
- Impact: the update_field logic gets organized into "Sections" (user, network, storage). [X]

2. Specifying Download Dir [(Issue 30)](https://github.com/Spectra010s/portal/issues/30)
   Why: Now that we have a storage section in the config, adding storage.download_dir and dir flag is a 5-minute (i hope so haha :smile:) task.

- Goal: Receiver reads this path and saves files there instead of the current working directory or Reciver specifies the path they want to save. [X]

3. Two-Stage Handshake [(Issue 32)](https://github.com/Spectra010s/portal/issues/32)
   Why: This is the biggest logic change. It requires a new "Phase 1" where the sender sends user details and waits for a "YES" from the receiver. Refactoring will be done here

- Goal: Security and Consent. we need the user.username from [Issue 3](https://github.com/Spectra010s/portal/issues/31) to be solid before we start sending it over the wire. [X]

4. Multiple File Support
   Why: This is a structural change to the protocol. Before we can show a progress bar or retry a connection, the pipe needs to know if it's carrying one file or a hundred.

- Goal: Allow the sender to pass a list of paths.
- Impact: Requires a "Manifest" header (JSON) sent over TCP before the data starts, so the receiver knows how many files to expect and their sizes. [X]

5. Folder Recursive Sending
   Why: Sending a folder is just "Multiple Files" with directory logic.

- Goal: portal send ./my_folder should automatically walk the directory tree.
- Impact: The receiver needs to recreate the folder structure on its end based on the paths provided in the Manifest. [X]

6. Progress Bars with indicatif [(Issue 18)](https://github.com/Spectra010s/portal/issues/18)
   Why: This is "UI Polish." It’s much easier to implement once we file transfer logic (the handshake and stream) is finalized.

- Goal: Show a smooth [===> ] 45% bar during the transfer. [X]

7. Logging System [(Issue 4)](https://github.com/Spectra010s/portal/issues/5)
   Why: As the app gets complex (retries, handshakes, directory creation), we need a way to see what went wrong without using println!.

- Goal: Implement tracing or log crates to save errors to a file. [X]

8. Transfer History (Persistent)
   Why: Keep a user-visible history of sends/receives (items, sizes, timestamps) beyond runtime logs.

- Goal: Record transfer summaries to a queryable history file or store. [X]

9. Polling and Retry Logic [(Issue 26)](https://github.com/Spectra010s/portal/issues/26)
   Why: This is the most complex "Quality of Life" feature. It handles dirty networks.

- Goal: If the connection drops during the handshake or transfer, the sender doesn't just quit; it tries again 3 times. []

10. TUI Progress Header (Sticky Top Line)
    Why: A proper TUI is needed to keep the "Sending/Receiving item X of Y" header fixed while file bars and logs scroll beneath it.

- Goal: Implement a `ratatui`-style interface that pins the header and avoids line redraw artifacts. []

11. Receiver Peer Username
    Why: Currently, the receiver never captures the sender's username (`peer_username` is always `None` in receiver history).

- Goal: Have the sender transmit its username during the handshake so the receiver can record who sent the transfer. [X]

12. Intended Bytes on Receive
    Why: The receiver gets item counts from the `GlobalTransferManifest` but not total byte sizes, so `intended_bytes` is always `0` in receive history.

- Goal: Add total byte count to the `GlobalTransferManifest` so the receiver can populate `intended_bytes` accurately. [X]

13. History Clear & Delete
    Why: Users need a way to manage their history — wipe it all or remove a specific record.

- Goal: Add `portal history clear` (removes all records) and `portal history delete <id>` (removes a single record). [X]

14. History Export
    Why: Users may want to save or share their transfer history as a standalone file.

- Goal: Add `portal history export` to write the full history to a user-specified file (JSON or CSV). [X]

15. No-Compression Mode (New)
    Why: Gzip can throttle throughput for large transfers; a switch to disable compression helps performance troubleshooting and fast LAN sends.

- Goal: Add a `--no-compress` flag to bypass gzip for sender/receiver. [X]

16. MSI Installer Images
    Why: The Windows MSI can look more polished with a branded icon and banners.

- Goal: Add WiX image assets (product icon, banner, dialog background). []

