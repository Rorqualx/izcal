# Threat model: Device-access service (`hg-device-main`: arbiter, session, enumerate)

Owner: Devices      Reviewed: not yet      Surface inventory: not generated yet (SEC-6)

> **Not yet filled.** Owner milestone: **M6**. Security scope 6.5 and SEC-26: every file is created at M1 and filled by M6, or earlier if the owning track's process ships earlier. This file was created at M1 from the 6.5 template. Until it is filled, the working model is the first pass for this process in security scope 6.5.

**Security scope 6.3a row**

| ATK | TB | Specific rules | Sandbox (6.7 row) | Specific targets | Track |
|---|---|---|---|---|---|
| ATK-2, ATK-3, ATK-10 | TB-2, TB-6, TB-14 | CON-R6 to CON-R9; G210–G219; DV-D1, DV-D3, DV-D4, DV-D6; typed transfers only; descriptors, HID reports, GATT, NDEF and MIDI wire data parsed by `hg-device-usb-desc`, `-hid-report`, `-gatt`, `-ndef` and `-midi-wire` at the strictest tier; **no OS device handle or file descriptor crosses IPC** (DV-D8); REC-R7 | Utility (device) | SEC-24's device suite; DV-1 to DV-18 | Devices |

## 1. Assets

Not yet filled.

## 2. Entry points

Not yet filled.

## 3. Attacker capabilities assumed

Not yet filled. Every peer process is assumed compromised (IPC-R17).

## 4. Mitigations

Not yet filled.

## 5. Residual risk

Not yet filled.

## 6. Open items

- Fill this model (owner: Devices; by M6).
