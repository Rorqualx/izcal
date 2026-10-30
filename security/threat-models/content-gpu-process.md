# Threat model: Content GPU process (per renderer key)

Owner: Graphics      Reviewed: not yet      Surface inventory: not generated yet (SEC-6)

> **Not yet filled.** Owner milestone: **M6**. Security scope 6.5 and SEC-26: every file is created at M1 and filled by M6, or earlier if the owning track's process ships earlier. This file was created at M1 from the 6.5 template. Until it is filled, the working model is the first pass for this process in security scope 6.5.

**Security scope 6.3a row**

| ATK | TB | Specific rules | Sandbox (6.7 row) | Specific targets | Track |
|---|---|---|---|---|---|
| ATK-2, ATK-3 | TB-2, TB-3, TB-5 | GPU-R1 to GPU-R15; glyph atlases partitioned per renderer key and partition (FP-D14) | GPU | SEC-10, GR-13, GR-14, GR-17 | Graphics |

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

- Fill this model (owner: Graphics; by M6).
