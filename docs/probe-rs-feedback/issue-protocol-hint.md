# [Issue draft] Generic connection error when the wire protocol is not specified (SWD-only target)

> 提交处:https://github.com/probe-rs/probe-rs/issues/new
> 可单独提,也可作为 UX 反馈并进自动识别相关讨论。

---

**Environment**

- probe-rs 0.32.0 (crates.io build), Windows
- Probe: ST-Link V2 (0483:3748)
- Target: CMSemicon CW32L012 (SWD-only wiring, `CW32L012x8` built-in)

**Observed**

Without an explicit `--protocol swd`, attaching fails with errors that never
mention the wire protocol, which makes an SWD-only board look "unsupported":

- `probe-rs info --probe 0483:3748`:

  ```console
  Could not attach to target. Try running with --verbose for more information about the target.
  Error: Connecting to the chip was unsuccessful.

  Caused by:
      0: Unable to load specification for chip
      1: The connected chip could not automatically be determined.
  ```

- flashing without the flag (e.g. `probe-rs download --chip CW32L012x8 <elf>`)
  fails during attach with a JTAG-flavored error (`JtagGetIdcodeError` under
  `--verbose`).

Adding `--protocol swd` fixes both. This is a fairly common trap for
Cortex-M0+ hobby boards where only SWD is wired out.

**Ask**

Any of these would have saved us real debugging time — in increasing order of
ambition:

1. When attach fails with the default protocol and the probe supports both,
   print a hint: "target did not respond on JTAG; try `--protocol swd`";
2. Retry automatically with the other protocol before giving up (at least for
   `info`/`list`-style discovery commands);
3. Document the default-protocol behaviour more prominently in the
   getting-started docs.

Happy to test any of the above on hardware.
