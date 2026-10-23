# cw32l012-pac

[![CI](https://github.com/SunWithCloude/Cw32l012_pac/actions/workflows/ci.yml/badge.svg)](https://github.com/SunWithCloude/Cw32l012_pac/actions/workflows/ci.yml)
[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#许可)

[English](README.md) | **简体中文**

武汉芯源半导体（CMSemicon）**CW32L012**（Cortex-M0+，64KB Flash / 8KB SRAM）
的独立寄存器访问层（PAC）。**零外部依赖、完全自包含**，只使用 `core` —— 不经
`svd2rust`、不用厂商 SVD、不引 C 头文件。

寄存器数据转录自《CW32L012 用户手册》V1.4，与
[cw32l012-hal](https://github.com/SunWithCloude/Rust_Cw32L012) v0.9.1 的 `pac`
模块逐字节一致（同一来源，且已在实板验证）。

## 为什么不用 crates.io 上的 `cw32l012_pac` 0.1.1？

官方 crate（`svd2rust` v0.37.1 生成，2026-06-25 发布）所依据的 SVD 有三处
错误/缺漏，本 crate 依照 UM V1.4 逐项修正：

| 位置 | 官方 PAC | 本 crate |
|------|----------|----------|
| `GPIOx_PDR @ 0x0C` | 整个 `0x08–0x0F` 被标记 reserved | 已建模（UM V1.4 §9.6.3）。注意芯片上**仅 PF3 有实际下拉**，复位值 `0x0000_0008` |
| `IRQ31 = FAULT` | `__INTERRUPTS` 数组仅 31 项 | 完整 32 项 `Interrupt` 枚举 |
| `SYSCTRL.PLL @ 0x28` | CW32F 系列 SVD 残留（L012 时钟树无 PLL） | 视为保留（`_pad0` 占位） |

## 提供内容

- 存储器映射与全部外设基地址常量（手册表 2-1）
- 26 个类型化寄存器块：`Sysctrl` / `Gpio` / `Flashctrl` / `Uart` / `Spi` /
  `I2c` / `Btim` / `Adc` / `Dac` / `Iwdt` / `Gtim` / `Bgr` / `Lptim` /
  `Halltim` / `Vc` / `VcRef` / `Opa` / `Atim` / `Wwdt` / `Rtc` / `Dma` /
  `Crc` / `Eau` / `Cordic` / `Ramctrl` / `Lvd`
- `Rw` / `Ro` / `Wo` 寄存器原语（volatile 读 / 写 / 读改写）
- 各外设位段常量（`sysctrl`、`gpio`、`uart` 等共 26 个模块）
- `Interrupt` 中断号枚举（IRQ0..31），带 `ALL` 遍历表、`TryFrom<u8>` 与 NVIC
  操作（`enable` / `disable` / `pend` / `unpend` / `is_enabled` / `is_pending`）
- `interrupt` 模块：每个中断一个零尺寸标记类型，供 embassy 风格
  `bind_interrupts!` 做编译期绑定
- `Peripherals` 外设单例（`take()` / `steal()`）
- 编译期布局断言（与官方头文件 `XXX_TypeDef` 尺寸逐项对照），
  另有宿主机回归测试 `tests/layout.rs`

## feature

| feature | 作用 |
|---------|------|
| （默认） | 纯寄存器层，零依赖 |
| `rt` | `cortex-m-rt` device 模式：生成 `__INTERRUPTS` 向量表 + `device.x`（`PROVIDE(<irq> = DefaultHandler)` 弱别名） |

启用 `rt` 后，`memory.x` 与 `-Tlink.x` 仍由应用提供（一般写在
`.cargo/config.toml`）——库 crate 的链接参数不会传播到最终固件。

## 使用示例

```rust,ignore
use cw32l012_pac::{sysctrl, Peripherals};

let p = unsafe { Peripherals::steal() };

// 打开 GPIOA 时钟（AHBEN 写入需带 KEY）
p.sysctrl.ahben.write(sysctrl::KEY | sysctrl::AHBEN_GPIOA);

// 原子置位 PA5
p.gpioa.bsrr.write(1 << 5);
```

作为依赖引入（与本仓库并列放一份 HAL 工程时）：

```toml
[dependencies]
cw32l012-pac = { path = "../cw32l012-pac" }
```

## 中断运行时

本 crate 提供两条互斥的路子，**不要把两者同时链接进一个固件**：

1. **`rt` feature** —— `cortex-m-rt` device 模式。向量符号是 `UART1`、
   `ADC1` 这样的原名（`__INTERRUPTS` 顺序与手册表 5-1 一致），这是 embassy /
   标准嵌入式 Rust 的走法；
2. **`cw32l012-hal` 的 `runtime` feature** —— 自包含向量表，符号是
   `UART1_irq` 这类带后缀的名字，另带 `interrupt!` 宏。

两条都不用？那就自备启动文件与向量表。

## 验证状态

- `thumbv6m-none-eabi`：debug + release + `clippy -D warnings`（默认与
  `rt` 两套 feature）+ rustdoc 全部零警告；`cargo fmt --check` 干净。
- 宿主机测试（`cargo test`，8 项）断言寄存器块尺寸与偏移、基地址、
  `GPIOx_PDR @ 0x0C`、全部 32 个中断号，以及三份中断清单
  （枚举 / typelevel 标记 / `build.rs`）互相对齐。
- `#![forbid(unsafe_op_in_unsafe_fn)]`：`unsafe fn` 内的每次解引用都显式写出。
- GPIO 输出路径已在 CW32L012 实板验证（寄存器回读符合手册）。
- 寄存器数据与 `cw32l012-hal` v0.9.1 一致（同一转录来源）。

版本历史见 [CHANGELOG.md](CHANGELOG.md)。

## 发布注意

crates.io 上的 `cw32l012_pac` 名字已被 `svd2rust` 版本占用。若要发布本
crate，需要换名（例如 `cw32l012-pac-um`）；本地 path 依赖不受影响。

## 许可

MIT OR Apache-2.0，与 `cw32l012-hal` 相同。
