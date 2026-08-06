# PXP — Portal Transfer Protocol

**Version:** 0.1  
**Status:** Draft Specification

---

## What is PXP?

PXP (Portal Transfer Protocol) is an application-layer protocol for transferring files and directories between devices on a local area network. It requires zero configuration — no accounts, no cloud, no pairing codes.

PXP operates over two transports: UDP for peer discovery, and TCP for data transfer.

---

## Protocol Phases

A PXP transaction consists of four sequential phases:

| Phase | Transport | Spec |
|---|---|---|
| 1. Discovery | UDP | [PXP-DISCOVERY](draft-pxp-discovery-00.md) |
| 2. Handshake | TCP | [PXP-HANDSHAKE](draft-pxp-handshake-00.md) |
| 3. Manifest | TCP | [PXP-MANIFEST](draft-pxp-manifest-00.md) |
| 4. Streaming | TCP | [PXP-STREAMING](draft-pxp-streaming-00.md) |

---

## Roles

PXP defines exactly two roles per transaction:

- **Sender** — The peer that initiates a file transfer. It discovers the receiver, opens the TCP connection, and streams the data.
- **Receiver** — The peer that accepts a file transfer. It advertises itself via UDP beacons and listens for incoming TCP connections.

---

## Design Principles

1. **Zero configuration.** Peers discover each other automatically via multicast and broadcast on the local network.
2. **Streaming.** Data is streamed directly from disk to network. The sender does not need to stage or buffer the entire payload before transmission.
3. **Single connection.** The entire transaction — handshake, manifest, and all file data — flows over a single TCP connection.
4. **Unidirectional data flow.** After the handshake, data flows exclusively from sender to receiver. There are no application-level acknowledgments. (See [PXP-STREAMING § Limitations](streaming.md#limitations).)

---

## Conventions

The key words "MUST", "MUST NOT", "SHOULD", "SHOULD NOT", and "MAY" in the spec documents are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

All multi-byte integers are encoded in **big-endian** (network byte order) unless stated otherwise.
