# AppArmor profile for the full-tier probe (E-Q1)

`izcal-spawner` lets one binary, Izcal's spawner, create user namespaces whose processes keep their capabilities inside the namespace. That is what the full Linux tier needs (security scope 6.7: an empty root by `pivot_root`, id maps). The owner approved installing it on **hueyj only** (E-Q1, 26 Oct), for the spawner path only.

## What changes on the host

Today on hueyj (Ubuntu 24.04, `kernel.apparmor_restrict_unprivileged_userns=1`):
- The spawner runs **unconfined**, like any process the user starts.
- Any user namespace it creates moves its child into the `unprivileged_userns` profile, which denies every capability inside it (launch facts E1, E2). That is the restricted tier the spawner uses now.

With the profile installed:
- The spawner still behaves as unconfined, and it gains one right: `userns`.
- Its children keep their capabilities **inside their own new user namespace**, so they can mount, `pivot_root` and write their id maps there. They gain nothing outside it.
- No other binary on the host changes. The profile attaches by exact path to two files, the debug and release builds of `hg-spawner-main` in the scratch build `/home/joe/izcal-m1e2`, and to nothing else.

## Why this form and not a confined profile

`apparmor.d(5)` gives two ways to grant `userns` to one binary:

1. **The profile below (`flags=(unconfined)` with `userns,`).** The delta from today is exactly the `userns` right. The binary is already unconfined.
2. **An enforce-mode profile with explicit rules.** It would need `userns create`, mount and `pivot_root` rules, and the exec of children from a pre-opened descriptor. In a new mount namespace that exec is a "disconnected path" (launch facts E4, E22), which only the `attach_disconnected` flag admits, and `apparmor.d(5)` calls that flag unsafe ("can result in aliasing and access to objects that should not be allowed").

The man page also says `unconfined` mode is meant for debugging. That fits here: this is a measurement on one test host, not what ships. What ships is DU's metapackage profile (DU 6.3, Q-DU-13), which is out of scope for this probe.

Izcal's own confinement of the children does not depend on AppArmor: every child runs under seccomp, Landlock and new user, PID and network namespaces whatever the profile.

## Install (root, by the owner)

```
sudo install -m 0644 izcal-spawner /etc/apparmor.d/izcal-spawner
sudo apparmor_parser -r /etc/apparmor.d/izcal-spawner
sudo aa-status | grep izcal-spawner      # expect: izcal-spawner (unconfined mode)
```

## Remove

```
sudo apparmor_parser -R /etc/apparmor.d/izcal-spawner
sudo rm /etc/apparmor.d/izcal-spawner
```

## What gets measured after install (no root needed)

Run as user `joe`, under a `rig` booking:
1. The spawner's start-up test reports that mounts work in the child's user namespace, and logs that the full tier is available but not claimed.
2. A full-tier probe in the child: `CLONE_NEWNS`, make the tree private, mount a tmpfs, `pivot_root` into it with the old root detached, then `execveat` of the pre-opened executable descriptor across it.
3. If (2) fails: bind-mount the executable into the empty root, exec it by its new path, and check that its `st_dev`/`st_ino` equal the pre-opened descriptor's before exec.

Results go to `launch_facts.md` as new rows. Until they exist, the full tier stays unclaimed.
