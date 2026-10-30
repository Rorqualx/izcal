# Threat model: Code-cache builder (per job)

Owner: JS, bindings and DOM      Reviewed: not yet      Surface inventory: not generated yet (SEC-6)

> **Not yet filled.** Owner milestone: **M6**. Security scope 6.5 and SEC-26: every file is created at M1 and filled by M6, or earlier if the owning track's process ships earlier. This file was created at M1 from the 6.5 template. Until it is filled, the working model is the first pass for this process in security scope 6.5.

**Security scope 6.3a row**

| ATK | TB | Specific rules | Sandbox (6.7 row) | Specific targets | Track |
|---|---|---|---|---|---|
| ATK-1, ATK-2 | TB-2, TB-6 | JS-D21: one script, one partition, jitless, no network, no storage handle, exits after the job; SEC-11 (d) | Utility | JS-18, SEC-11 | JS, bindings and DOM |

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

- Fill this model (owner: JS, bindings and DOM; by M6).
