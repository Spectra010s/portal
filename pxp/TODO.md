# pxp — Known Issues & Future Improvements

## 1. Sender crash → Receiver gets opaque EOF

**Problem:** The protocol is unidirectional after the handshake (sender → receiver only). If the sender crashes, encounters a read error, or is killed mid-transfer, the receiver's tar entry iterator hits an unexpected EOF and crashes with a generic IO error like `unexpected end of file` — no indication that the *sender* disconnected.

**Impact:** Poor UX. The receiver has no way to distinguish between "transfer corrupted" and "sender went away."

**Fix:** After the manifest exchange, wrap the TCP stream in a lightweight framing protocol that includes:
- A heartbeat/keepalive mechanism so the receiver can detect sender death within seconds
- Or: prefix each tar chunk with a frame header that includes a "stream continues" flag, so the receiver can detect a clean vs dirty close

---

## 2. Receiver cannot signal sender

**Problem:** There is no reverse channel from receiver → sender. If the receiver:
- Runs out of disk space
- User aborts via conflict resolution (e.g. chooses "cancel all")
- Encounters a write error

...the receiver just drops the TCP connection. The sender sees a broken pipe on the next write, but gets no structured error — just `BrokenPipe` or `ConnectionReset`.

**Impact:** The sender can't tell *why* the transfer failed. Was it a network issue? Did the receiver reject it? Did they run out of space?

**Fix:** Add a simple back-channel protocol:
- Receiver sends status frames (ack/nack) after each top-level item
- On abort, receiver sends a structured error frame before closing
- Sender reads these between items to detect early termination gracefully

---

## 3. No transfer completion acknowledgment

**Problem:** The sender finishes flushing the tar stream and immediately assumes success. It has no confirmation that the receiver:
- Actually received all bytes
- Successfully wrote all files to disk
- Passed all metadata validation checks

**Impact:** The sender's history may record "Success" for a transfer that the receiver considers failed.

**Fix:** After the tar stream is complete:
1. Receiver sends a final `TransferResult` frame (success/failure + summary)
2. Sender reads it before reporting success
3. Both sides have a consistent view of the transfer outcome

---

## 4. No cancellation mechanism

**Problem:** Neither side can cleanly cancel a transfer in progress. The only option is to drop the TCP connection, which triggers the EOF/broken-pipe errors described above.

**Fix:** Define a cancellation frame type that either side can send. The other side should handle it gracefully and record the transfer as "Cancelled" (not "Failed").

---

## Priority Order

1. **#3 — Completion ack** (simplest, biggest UX win)
2. **#2 — Receiver → sender signaling** (enables #4)
3. **#4 — Clean cancellation** (depends on #2)
4. **#1 — EOF detection** (partially solved by #2 and #3)
