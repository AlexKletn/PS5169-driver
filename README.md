# PS5169-driver

[Русская версия](README.ru.md)

[Contributing](CONTRIBUTING.md) · [GPL-2.0-only](LICENSE)

The `ps5169` Rust crate handles chip identification, reset, and USB 3 and DisplayPort routing. It uses `no_std` and receives its register I/O and board settings from the caller. The Linux adapter registers a Type-C orientation switch and retimer for each bound I²C device.

## Build the Linux module

Use a configured Rust-enabled Linux kernel tree with Type-C and I²C support. The kernel must expose the I²C Rust driver, I²C SMBus helpers, and Type-C mux and retimer APIs used by the adapter.

```sh
make -C "$KERNEL_BUILD" M="$PWD/kernel" MO="$PWD/target/kernel" \
  PS5169_PROFILE="$PWD/profiles/tb371fc.toml" modules
```

## Board configuration

Use `compatible = "parade,ps5169"` with the standard Type-C graph and switch properties. Select a TOML profile with `PS5169_PROFILE` when building with Cargo or Kbuild. The supplied [TB371FC profile](profiles/tb371fc.toml) defines ordered register/value pairs for initialization and transitions in USB, DisplayPort, and mixed mode. Initialization must include the USB RX equalization register `0x51`, which the driver checks together with the route after resume. Profile generation requires Python 3.11 or later.

The optional `parade,defer-id-timeout` makes a timed-out chip ID read during probe return `EPROBE_DEFER`. The TB371FC profile enables this workaround while the missing startup dependency is investigated.

On a deferred probe, the failing ID register and elapsed time appear in `/sys/kernel/debug/devices_deferred`. Successful initialization logs its total time. The reset wait has a kernel log showing the requested and elapsed durations.
