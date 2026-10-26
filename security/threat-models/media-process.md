# Threat model: Media process (per renderer key)

Owner: Images and media      Reviewed: self-approved, M1 phase E (26 October 2026); security owner review pending      Surface inventory: not generated yet (SEC-6); no media endpoint exists at M1

Filled at M1 (security scope 6.5, SEC-26). **No media code exists at M1:** no media process kind, no `.hgidl` interface, no decoder. This model is the security scope's first pass, made concrete where M1's substrate already fixes the mechanism the media process will use. Each item says whether it exists in code today. When the media process lands, the pull request that adds it updates this file (SEC-26).

## 1. Assets

One (site, container)'s encoded media and image bytes, decoded frames before they leave, ICC profiles, WebVTT and HLS state, and opaque cross-site SVG source and its raster (first pass).

## 2. Entry points (designed)

- Encoded image, font-in-SVG and media containers and bitstreams from the network service.
- Demuxers, decoders, encoders, the ICC parser, and the WebVTT and HLS parsers.
- RTP payloads from WebRTC; WebCodecs requests.
- Typed picture parameters it sends to the hardware-decode process.
- The scriptless static document host for opaque SVG images, a shared parse within the partition.
- The opaque-image protocol's channels: format, dimensions, region sizes and the shared regions it names (MD 6.1, G56).
- **Shared-memory regions it parses.** The region kinds exist at M1 (D6): writer, read-only and consumer, each with a declared length that the IDL field's bound caps.

## 3. Attacker capabilities assumed

- ATK-3 through crafted media; ATK-5 through WebRTC payloads.
- ATK-2 from the renderer.
- ATK-1 through channel side effects of the opaque-image protocol (size, timing and region naming).
- Every peer process is assumed compromised (IPC-R17).

## 4. Mitigations

| Mitigation | Status at M1 | Where |
|---|---|---|
| **IPC-R7: every region it parses is sealed or copied** before it parses it | **Exists:** region kinds and acceptance per OS (D6). Linux: memfd seals, checked on receipt. macOS: the read-only entry; `immutable` fields mapped as a held snapshot. Windows: the broker's registry and narrowed masks, with a broker copy for `immutable` fields. Receivers copy before they parse (the API forces it) | [`hg-substrate/src/ipc/region.rs`](../../crates/hg-substrate/src/ipc/region.rs); `hg-substrate-os/src/ipc/shm.rs` and the per-OS acceptance modules; Kani region harnesses |
| The broker allocates every region and charges it to an owner within quotas; a writer comes only from the broker | **Exists** (D6) | as above |
| MD-D5: the strictest sandbox tier; all parsing here; no C codec library and no OpenH264 (MD-D1) | Designed. The strictest tier exists as the empty-process profile on each OS (`sandbox::{linux, macos, windows}`). The media kind is not in `hg-launch-plan` yet | [`sandbox/`](../../crates/hg-substrate-os/src/sandbox/) |
| SEC-12's L2m crates are the only code that parses web formats here | Designed | — |
| One key per process, so a bug reaches one (site, container)'s media | Designed | — |
| Outputs normalized before crossing (canonical RGBA); frames leave as OS handles through the broker; no GPU device access | Designed (the GPU buffer handle kinds are reserved in `HandleKind`; no backend makes them yet) | [`hg-substrate/src/ipc.rs`](../../crates/hg-substrate/src/ipc.rs) |
| The opaque-image protocol carries only receiver-fixed formats and sizes (G56, MD-D20) | Designed | — |
| No socket, no host Unix socket: seccomp denies `socket` and `connect` (Landlock ABI 4 alone would allow `connect` to the D-Bus buses and docker.sock) | Exists for every M1 child; applies when the kind is added | Launch facts E25, E26 |

## 5. Residual risk

- Bugs in Izcal's decoders give away the job's inputs and that key's decoded frames.
- OS framework decoders that run in software for audio (AAC) are OS code Izcal cannot patch. They live in the hardware-decode process.
- An opaque image's dimensions and decode timing are visible to the embedding page, as the format requires.
- **Region residuals inherited from D6:** on macOS and Windows a read-only region is trusted to the process that filled it, unless its IDL field is `immutable`. Media's IDL fields must be `immutable` wherever the media process relies on immutability. On macOS, children can make memory entries themselves (enforcement is at acceptance only).
- Every residual of the M1 child sandbox applies once the kind exists: the `newfstatat` existence leak (launch facts E28), the restricted Linux tier until E-Q1, the deprecated macOS sandbox API (E33), and the Windows M1 tier.

## 6. Open items

- The media process kind, its `.hgidl` interface, its sandbox plan in `hg-launch-plan`, and its fuzz targets (Images and media track).
- MD-S10's measurement of the scriptless SVG host (first pass).
- The generated surface inventory (SEC-6).
