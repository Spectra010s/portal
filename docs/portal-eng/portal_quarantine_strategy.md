# The "Portal Quarantine" Strategy

Instead of writing the file directly to the user's Desktop or Downloads, the `Portal Engine` can follow this high-security workflow:
 * Stage 0: The Hidden Temp Zone
   Portal writes the incoming data into a hidden, temporary directory (e.g., .portal/temp/). On Windows, this might be in AppData/Local/Temp. On Linux/Mac, it’s /tmp.
 * Stage 1: Extension Verification
   While it's in the temp folder, the engine checks: "The sender said this is 'cat.jpg', but the file headers say it's actually an 'executable script'." If there's a mismatch, Portal flags it immediately.
 * Stage 2: External Scan (The Handshake)
   The Closed GUI (which we talked about) can then automatically trigger the computer's built-in antivirus (like Windows Defender) to scan that specific temp folder.
 * Stage 3: The "Final Landing"
   Only after the scan returns "Clean" and the user clicks "Accept" does the engine move the file from the hidden temp zone to the actual destination.
Why this is safer
 * No Auto-Execution: Most viruses need to be "clicked" to work. By keeping the file in a hidden, system-managed temp folder, the user can't accidentally double-click it while it's being scanned.
 * Isolation: If the file is a virus, it’s trapped in a "Waiting Room." It hasn't touched the user's precious files yet.