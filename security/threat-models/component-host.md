# Threat model: Component host (per extension origin, Stage 2; app, app-replay and install-verify configurations, Stage 3)

Owner: Extensions (Stage 2); app host (Stage 3)      Reviewed: not yet      Surface inventory: not generated yet (SEC-6)

> **Not yet filled.** Owner milestone: **M6**. Security scope 6.5 and SEC-26: every file is created at M1 and filled by M6, or earlier if the owning track's process ships earlier. This file was created at M1 from the 6.5 template. Until it is filled, the working model is the first pass for this process in security scope 6.5.

**Security scope 6.3a row**

| ATK | TB | Specific rules | Sandbox (6.7 row) | Specific targets | Track |
|---|---|---|---|---|---|
| ATK-2, ATK-4 | TB-2, TB-8 | IPC-R17 harness (extension-API and native-messaging messages), G171–G175, G178, G179; for apps: G220–G229, the manifest ceiling and `CallContext`, and the app-profile section of the component-host model (package/ZIP, component, WIT and canonical-ABI subsections, each with its fuzz target) | Utility | AH-4's `hg-app-fuzz` targets | Extensions (Stage 2); app host (Stage 3) |

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

- Fill this model (owner: Extensions (Stage 2); app host (Stage 3); by M6).
