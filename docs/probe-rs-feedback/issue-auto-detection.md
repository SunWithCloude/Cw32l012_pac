# [Issue draft] `probe-rs info` cannot auto-detect the CW32L012 even though the target is built-in

> 提交处:https://github.com/probe-rs/probe-rs/issues/new
> 建议在正文引用 #2449(Automatic chip detection)。

---

**Environment**

- probe-rs 0.32.0 (crates.io build), Windows
- Probe: ST-Link V2 (0483:3748), SWD
- Target: CMSemicon / Wuhan Xinyuan **CW32L012** (Cortex-M0+, `CW32L012x8` is in the built-in target registry since 0.32)

`probe-rs chip list` shows the family:

```console
$ probe-rs chip list | grep -A3 CW32
CW32L0 Series
        CW32L010x8
        CW32L011x8
        CW32L012x8
```

**Observed**

`probe-rs info` (which ignores `--chip` and tries to auto-determine the target)
fails on this chip:

```console
$ probe-rs info --probe 0483:3748 --protocol swd
Error: Connecting to the chip was unsuccessful.

Caused by:
    0: Unable to load specification for chip
    1: The connected chip could not automatically be determined.
```

With `--verbose` the debug-port side actually works — it just stops at the ARM
core identity, which of course says nothing about the actual SoC:

```console
$ probe-rs info --probe 0483:3748 --protocol swd --verbose
ARM Chip with debug port Default:
Debug Port: DPv1, MINDP, Designer: ARM Ltd
        └── 0xe00ff000 ROM Table (Class 1), Designer: ARM Ltd
Debug port version DPv1 does not support SWD multidrop. Stopping here.
```

Everything else works fine once the chip is named explicitly
(`download` / `reset` / memory access / `run` all verified on hardware).

**Ask**

1. Would `chip_detection` metadata be feasible for the CW32L0 variants, or is
   there nothing vendor-unique reachable through the standard ARM debug
   components on these parts (MINDP, no multidrop)? If a vendor-readable ID
   exists (e.g. a chip-UID register), I'm happy to dig out the address from
   the user manual and test a PR on real hardware.
2. If auto-detection is a dead end for this class of chips, could the error
   message hint that the target may still be supported and suggest
   `--chip`? Right now "could not automatically be determined" reads like
   "unsupported", which cost us some confusion (see also #2449).

Happy to run any test build on the board.
