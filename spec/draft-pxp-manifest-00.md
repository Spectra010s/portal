# PXP-MANIFEST — Transfer Manifest

**Parent:** [PXP](draft-pxp-overview-00.md)  
**Transport:** TCP  
**Phase:** 3 of 4

---

## 1. Purpose

Before streaming any file data, the sender MUST transmit a manifest that describes the transfer as a whole. This allows the receiver to:

- Know how many items to expect.
- Know the total transfer size.
- Determine whether the data stream will be compressed.
- Display transfer metadata to the user before data begins arriving.

---

## 2. Wire Format

The manifest is sent on the same TCP connection used for the handshake, immediately after identity verification completes.

```
+-------------------------------+-------------------------------+
|  Length (4 bytes, big-endian) |  Manifest Payload (Bincode)   |
+-------------------------------+-------------------------------+
```

### 2.1 Fields

| Field | Size | Encoding | Description |
|---|---|---|---|
| Length | 4 bytes | Unsigned 32-bit, big-endian | Byte length of the Bincode-encoded manifest payload that follows. |
| Payload | Variable | Bincode | The serialized `GlobalTransferManifest` structure. |

---

## 3. Manifest Structure

The manifest is a fixed-schema structure with the following fields:

| Field | Type | Required | Description |
|---|---|---|---|
| `total_files` | u32 | MUST | Number of top-level files in this transfer. |
| `total_directories` | u32 | MUST | Number of top-level directories in this transfer. |
| `total_bytes` | u64 | MUST | Total uncompressed size of all items in bytes. |
| `description` | string or null | MAY | Optional human-readable description provided by the sender. |
| `sender_username` | string or null | MAY | The sender's configured username. |
| `compressed` | bool | MUST | If `true`, the data stream in [PXP-STREAMING](draft-pxp-streaming-00.md) is Gzip-compressed. If `false`, raw TAR. |

### 3.1 Item Count

The total number of top-level items is `total_files + total_directories`. This value determines how many top-level metadata contracts the receiver should expect in the data stream.

Nested files within directories are NOT counted in `total_files`. They are tracked separately via nested metadata contracts within the TAR stream.

---

## 4. Serialization

The manifest MUST be serialized using [Bincode](https://github.com/bincode-org/bincode) with default configuration (little-endian, variable-length integers, trailing bytes rejected).

Implementations MUST NOT use JSON, MessagePack, or any other serialization format for the manifest.

---

## 5. Receiver Behavior

Upon receiving the manifest, the receiver:

1. MUST deserialize the payload using Bincode.
2. MUST read the `compressed` field to determine how to decode the subsequent data stream.
3. SHOULD display the transfer summary (item count, total size, sender username, description) to the user.
4. MUST proceed to [PXP-STREAMING](draft-pxp-streaming-00.md) to begin receiving data.

There is no acceptance/rejection message. The receiver cannot decline a transfer at the protocol level — it either reads the stream or drops the connection.

---

## 6. Failure Modes

| Condition | Behavior |
|---|---|
| Length prefix indicates payload > 10 MB | Receiver SHOULD reject as malformed. |
| Bincode deserialization fails | Receiver MUST close the connection and report a protocol error. |
| `total_files` and `total_directories` are both 0 | Valid but degenerate. The data stream phase will contain no items. |
