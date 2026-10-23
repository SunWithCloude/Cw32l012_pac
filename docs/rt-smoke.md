# `rt` feature 冒烟验证

验证目标：`rt` feature 生成的向量表能被 `cortex-m-rt` 正确链接、能启动，
并且"未绑定的中断落到 `DefaultHandler`、用户强符号能覆盖单个中断"。

本文件记录 2026-10-24 在 CW32L012 实板上跑通的那套最小工程，日后改
`build.rs` / 向量表后可原样复现。

## 1. 临时工程

放在仓库外（例如 `%TEMP%\pac_rt_smoke`），避免污染 PAC 本身：

```
pac_rt_smoke/
├── .cargo/config.toml
├── Cargo.toml
├── memory.x
└── src/main.rs
```

`.cargo/config.toml`：

```toml
[target.thumbv6m-none-eabi]
rustflags = ["-C", "link-arg=-Tlink.x"]
```

`Cargo.toml`：

```toml
[package]
name = "pac-rt-smoke"
version = "0.1.0"
edition = "2021"

[dependencies]
cw32l012-pac = { path = "C:/Users/Administrator/Desktop/cw32l012-pac", features = ["rt"] }
cortex-m-rt = "0.7"
```

`memory.x`：

```
MEMORY
{
  FLASH : ORIGIN = 0x00000000, LENGTH = 64K
  RAM   : ORIGIN = 0x20000000, LENGTH = 8K
}
```

`src/main.rs`：

```rust
#![no_std]
#![no_main]

use cortex_m_rt::entry;

/// 覆盖 IRQ27（UART1）：强符号应压过 device.x 里的 PROVIDE 弱别名。
#[no_mangle]
pub extern "C" fn UART1() {}

#[entry]
fn main() -> ! {
    // 必须真正"用到" PAC：未被引用的依赖 rustc 不会交给链接器，
    // link.x 的 EXTERN(__INTERRUPTS) 就无对象可拉。
    let irq = cw32l012_pac::Interrupt::Uart1;

    const MAGIC: *mut u32 = 0x2000_1F00 as *mut u32;
    unsafe { core::ptr::write_volatile(MAGIC, 0xC0DE_0002) };

    loop {
        core::hint::spin_loop();
        let _ = irq.number();
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
```

## 2. 链接期检查（不需要硬件）

用 pyelftools 读 ELF：

```python
# __INTERRUPTS 应落在 0x40，size = 128（32 项 × 4）
# .vector_table 的前 0x40 字节是 MSP + 15 个内核异常向量
```

实测结果（`thumbv6m-none-eabi`，release）：

| 符号 | 地址 | 说明 |
|------|------|------|
| `__INTERRUPTS` | `0x0040` | size 128，正好 32 项 |
| `DefaultHandler` | `0x0111` | 未绑定中断全部指到这里 |
| `UART1` | `0x00fd` | 我们写的强符号，size 6 |

32 项里只有 IRQ27 指向 `UART1`，其余 31 项全部落在 `DefaultHandler`，
说明 `device.x` 的 `PROVIDE` 弱别名与强符号覆盖都按预期工作。

## 3. 启动验证（需要 ST-Link）

```bash
cargo build --release --target thumbv6m-none-eabi
probe-rs download --probe 0483:3748 --protocol swd --chip CW32L012x8 \
    target/thumbv6m-none-eabi/release/pac-rt-smoke
probe-rs reset  --probe 0483:3748 --protocol swd --chip CW32L012x8
probe-rs read b8 0x20001F00 4 --probe 0483:3748 --protocol swd --chip CW32L012x8
```

预期输出 `20001f00: 02 00 de c0`（小端 `0xC0DE_0002`），即 `Reset` →
`.data`/`.bss` 初始化 → `main` 一路正常。

## 4. 已知坑

- **必须引用 PAC**，哪怕只是 `let _ = Interrupt::Uart1;`。否则链接器报
  `ERROR(cortex-m-rt): The interrupt vectors are missing.`——不是向量表写错了，
  而是整个 rlib 没进链接命令行。
- `-Tlink.x` 要在应用侧加。库 crate 的 `cargo:rustc-link-arg` 不会传播到
  最终固件；PAC 的 `build.rs` 只负责把 `device.x` 放进链接搜索路径。
- 别同时开 `cw32l012-hal` 的 `runtime`：两套向量表（`UART1` 与 `UART1_irq`）
  会重复定义。
