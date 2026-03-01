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

4. Multiple File Support (New) Why: This is a structural change to the protocol. Before we can show a progress bar or retry a connection, the pipe needs to know if it's carrying one file or a hundred.

- Goal: Allow the sender to pass a list of paths.
- Impact: Requires a "Manifest" header (JSON) sent over TCP before the data starts, so the receiver knows how many files to expect and their sizes. [X]

5. Folder Recursive Sending (New)
   Why: Sending a folder is just "Multiple Files" with directory logic.

- Goal: portal send ./my_folder should automatically walk the directory tree.
- Impact: The receiver needs to recreate the folder structure on its end based on the paths provided in the Manifest.

6. Progress Bars with indicatif [(Issue 18)](https://github.com/Spectra010s/portal/issues/18)
   Why: This is "UI Polish." It’s much easier to implement once we file transfer logic (the handshake and stream) is finalized.

- Goal: Show a smooth [===> ] 45% bar during the transfer. []

7. Logging System [(Issue 4)](https://github.com/Spectra010s/portal/issues/5)
   Why: As the app gets complex (retries, handshakes, directory creation), we need a way to see what went wrong without using println!.

- Goal: Implement tracing or log crates to save errors to a file. []

8. Polling and Retry Logic [(Issue 26)](https://github.com/Spectra010s/portal/issues/26)
   Why: This is the most complex "Quality of Life" feature. It handles dirty networks.

- Goal: If the connection drops during the handshake or transfer, the sender doesn't just quit; it tries again 3 times. []

