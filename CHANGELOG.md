# Changelog

本项目版本号遵循 [SemVer](https://semver.org/lang/zh-CN/)；条目格式参考
[Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)。

## [0.2.0] - 2026-10-24

### 新增

- 补齐 5 个此前只有基地址、没有寄存器块的外设：`Crc`、`Eau`、`Cordic`、
  `Ramctrl`、`Lvd`（寄存器块共 26 个），各自带位段常量
  （`crc` / `eau` / `cordic` / `ram` / `lvd` 模块）与编译期尺寸断言。
- 基地址常量：`LVD_BASE`；并为官方头文件的 `RAM_BASE` 补了别名。
- `Peripherals` 增加 `crc` / `eau` / `cordic` / `ramctrl` / `lvd` 字段。
- `Interrupt::ALL`（32 项、按 IRQ 号排列）、`number()`、`is_enabled()`、
  `is_pending()`、`TryFrom<u8>`。
- `rt` feature：`cortex-m-rt` device 模式的 `__INTERRUPTS` 向量表，配合
  `build.rs` 生成的 `device.x`（32 条 `PROVIDE(<irq> = DefaultHandler)`）。
- `interrupt` 模块：每个中断一个零尺寸标记类型（`interrupt::UART1` 等），
  供 embassy 风格 `bind_interrupts!` 做编译期绑定。
- `CHANGELOG.md`；CI 增加 `rt` 组合、宿主机 `--all-targets` clippy 与文档任务。

### 变更

- `build.rs` 在生成 `device.x` 前校验 IRQS 表：编号与数组下标不一致直接
  编译失败（表顺序即向量槽位）。
- 删掉 `build.rs` 里无效的 `cargo:rustc-link-arg=-Tlink.x`：库 crate 的链接
  参数不会传播到最终固件，`-Tlink.x` 应由应用提供（已在注释与 README 写明）。
- 顶层的基地址常量逐个补文档（此前成组常量共用一条注释）。
- 源码统一 LF 换行。

### 修复

- `unsafe trait interrupt::Interrupt` 缺少 `# Safety` 段，`clippy -D warnings`
  会直接失败（0.1.0 之后的开发分支引入）。
- `steal()` 等 `unsafe fn` 内的解引用改为显式 `unsafe {}` 块，并打开
  `#![forbid(unsafe_op_in_unsafe_fn)]`，为迁移 Rust 2024 做准备。

## [0.1.0] - 2026-10-24

### 新增

- 首个版本：从 `cw32l012-hal` v0.9.1 的 `pac` 模块抽出，零依赖、只用 `core`。
- 存储器映射与全部外设基地址；21 个类型化寄存器块；`Rw` / `Ro` / `Wo`
  寄存器原语；位段常量；`Interrupt` 枚举（完整 32 项）与 NVIC 操作；
  `Peripherals::take()` / `steal()`。
- 相对 crates.io 上 `svd2rust` 生成的 `cw32l012_pac` 0.1.1，修正三处 SVD 缺陷：
  `GPIOx_PDR @ 0x0C`、缺失的 `IRQ31 = FAULT`、CW32F 残留的 `SYSCTRL.PLL @ 0x28`。
