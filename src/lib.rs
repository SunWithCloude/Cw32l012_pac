//! # cw32l012-pac
//!
//! 武汉芯源半导体（CMSemicon）**CW32L012** Cortex-M0+ 微控制器的独立
//! 寄存器访问层（PAC）。
//!
//! 从 [`cw32l012-hal`](https://github.com/SunWithCloude/Rust_Cw32L012) v0.9.1
//! 的 `pac` 模块抽取而成，**零外部依赖、完全自包含**，提供：
//!
//! - 存储器映射与全部外设基地址常量（手册表 2-1）
//! - 类型化寄存器块（26 个）：`Sysctrl` / `Gpio` / `Flashctrl` / `Uart` / `Spi` /
//!   `I2c` / `Btim` / `Adc` / `Dac` / `Iwdt` / `Gtim` / `Bgr` / `Lptim` /
//!   `Halltim` / `Vc` / `VcRef` / `Opa` / `Atim` / `Wwdt` / `Rtc` / `Dma` /
//!   `Crc` / `Eau` / `Cordic` / `Ramctrl` / `Lvd`
//! - `Rw` / `Ro` / `Wo` 寄存器原语（volatile 读 / 写 / 读改写）
//! - 各外设位段常量（[`sysctrl`]、[`gpio`]、[`uart`] 等模块）
//! - [`Interrupt`] 中断号枚举（IRQ0..31）与 NVIC 操作，含
//!   [`ALL`](Interrupt::ALL) 遍历表与 `TryFrom<u8>`
//! - `interrupt` 模块：typelevel 中断标记（embassy `bind_interrupts!` 用）
//! - [`Peripherals`] 外设单例（[`take`](Peripherals::take) / [`steal`](Peripherals::steal)）
//! - 编译期布局静态断言（与官方头文件 `XXX_TypeDef` 尺寸逐项对照）
//!
//! ## feature
//!
//! | feature | 作用 |
//! |---------|------|
//! | （默认） | 纯寄存器层，零依赖 |
//! | `rt` | 用 `cortex-m-rt` 的 device 模式生成 `__INTERRUPTS` 向量表 + `device.x` 弱别名；应用侧需自备 `memory.x` 与 `-Tlink.x` |
//!
//! ## 与 crates.io 上 `cw32l012_pac` 0.1.1 的差异
//!
//! 官方 crate（svd2rust v0.37.1 生成，2026-06-25 发布）所依据的 SVD 存在
//! 三处错误/缺漏，本 crate 依照《CW32L012 用户手册》V1.4 逐项修正：
//!
//! | 位置 | 官方 PAC | 本 crate |
//! |------|----------|----------|
//! | `GPIOx_PDR @ 0x0C` | 整个 `0x08–0x0F` 被标记 reserved | 已建模（UM V1.4 §9.6.3；仅 PF3 有实际下拉，复位值 `0x0000_0008`） |
//! | `IRQ31 = FAULT` | `__INTERRUPTS` 数组仅 31 项 | 完整 32 项 [`Interrupt`] 枚举 |
//! | `SYSCTRL.PLL @ 0x28` | CW32F 系列 SVD 残留（L012 无 PLL） | 视为保留（`_pad0` 占位） |
//!
//! ## 数据来源与验证状态
//!
//! 寄存器偏移、位段与复位值转录自《CW32L012 用户手册》V1.4（武汉芯源
//! 半导体，2026-06），并已与官方 PAC / 头文件逐项对照：
//!
//! - 通过 `thumbv6m-none-eabi` 交叉编译验证（零警告）；
//! - GPIO 输出路径已于 2026-10-17 在 CW32L012 实板验证（PC13 闪烁）；
//! - 量产前请对照手册复核关键寄存器位。
//!
//! 本层仅提供类型化寄存器访问；位段语义请配合 HAL 层或手册使用。

// 寄存器文档大量使用 `CR1[7:0]`、`SQRCH0[3:0]` 这类位域写法，rustdoc 会把
// 方括号当成 intra-doc 链接去解析，产生大量"链接无法解析"的噪声警告。
// 文档以手册位域表为主，统一关掉这类警告。
#![no_std]
#![allow(rustdoc::broken_intra_doc_links)]
// unsafe fn 内的裸指针解引用也要求显式 unsafe 块（Rust 2024 起为默认行为），
// 提前打开以免将来迁移 edition 时一次性爆出几十处。
#![forbid(unsafe_op_in_unsafe_fn)]

// ================= 存储器映射常量（表 2-1） =================

/// 主 FLASH 基地址（64KB）
pub const FLASH_BASE: usize = 0x0000_0000;
/// 主 FLASH 容量
pub const FLASH_SIZE: usize = 64 * 1024;
/// SRAM 基地址（8KB，带奇偶校验）
pub const SRAM_BASE: usize = 0x2000_0000;
/// SRAM 容量
pub const SRAM_SIZE: usize = 8 * 1024;
/// 启动程序存储器 BootLoader（2KB）
pub const BOOT_BASE: usize = 0x0010_0000;
/// OTP 存储器（22B）
pub const OTP_BASE: usize = 0x0010_0760;

// ================= 外设基地址（表 2-1） =================

/// 系统控制（复位与时钟）
pub const SYSCTRL_BASE: usize = 0x4000_4000;
/// UART1
pub const UART1_BASE: usize = 0x4000_0C00;
/// UART2
pub const UART2_BASE: usize = 0x4000_1000;
/// UART3
pub const UART3_BASE: usize = 0x4000_2000;
/// GPIO 端口 A
pub const GPIOA_BASE: usize = 0x4800_0000;
/// GPIO 端口 B
pub const GPIOB_BASE: usize = 0x4800_0100;
/// GPIO 端口 C
pub const GPIOC_BASE: usize = 0x4800_0200;
/// GPIO 端口 F
pub const GPIOF_BASE: usize = 0x4800_0300;

// ---- 以下外设暂无 HAL 封装，本 crate 只提供寄存器块与位段 ----

/// ADC12 / VC1234 / LVD / OPA12 / DAC（AHB 桥）
pub const ADC12_BASE: usize = 0x4000_0000;
/// BGR 基准/温度传感器（与 ADC1 同页，BGR_CR@+0xFC）
pub const BGR_BASE: usize = 0x4000_0000;
/// ADC1（实例 1）
pub const ADC1_BASE: usize = 0x4000_0000;
/// ADC2（实例 2，实例步进 0x100）
pub const ADC2_BASE: usize = 0x4000_0100;
/// DAC（数字模拟转换器，双通道）
pub const DAC_BASE: usize = 0x4000_00C0;
/// VC1/VC2 电阻分压共享寄存器（VC12_REF，与 VC1 基地址重合）
pub const VC12REF_BASE: usize = 0x4000_0080;
/// VC3/VC4 电阻分压共享寄存器（VC34_REF，与 VC3 基地址重合）
pub const VC34REF_BASE: usize = 0x4000_0180;
/// VC1 比较器
pub const VC1_BASE: usize = 0x4000_0080;
/// VC2 比较器
pub const VC2_BASE: usize = 0x4000_0090;
/// VC3 比较器
pub const VC3_BASE: usize = 0x4000_0180;
/// VC4 比较器（VC1..VC4 实例步进 0x10，§27.6）
pub const VC4_BASE: usize = 0x4000_0190;
/// OPA1 运算放大器
pub const OPA1_BASE: usize = 0x4000_00B0;
/// OPA2 运算放大器（§29.5，实例步进 0x08）
pub const OPA2_BASE: usize = 0x4000_00B8;
/// SPI2
pub const SPI2_BASE: usize = 0x4000_0400;
/// SPI1
pub const SPI1_BASE: usize = 0x4000_0800;
/// ATIM（高级定时器）
pub const ATIM_BASE: usize = 0x4000_1400;
/// GTIM1（通用定时器）
pub const GTIM1_BASE: usize = 0x4000_1800;
/// GTIM2
pub const GTIM2_BASE: usize = 0x4000_1C00;
/// GTIM3
pub const GTIM3_BASE: usize = 0x4000_2400;
/// GTIM4
pub const GTIM4_BASE: usize = 0x4000_2800;
/// SPI3
pub const SPI3_BASE: usize = 0x4000_3400;
/// RTC
pub const RTC_BASE: usize = 0x4000_4400;
/// BTIM1（基本定时器）
pub const BTIM1_BASE: usize = 0x4000_4800;
/// BTIM2
pub const BTIM2_BASE: usize = 0x4000_4840;
/// BTIM3（BTIM1..3 实例步进 0x40）
pub const BTIM3_BASE: usize = 0x4000_4880;
/// 兼容别名：BTIM1 基地址
pub const BTIM123_BASE: usize = BTIM1_BASE;
/// IWDT（独立看门狗）
pub const IWDT_BASE: usize = 0x4000_5000;
/// WWDT（窗口看门狗）
pub const WWDT_BASE: usize = 0x4000_5400;
/// I2C1
pub const I2C1_BASE: usize = 0x4000_5800;
/// I2C2
pub const I2C2_BASE: usize = 0x4000_5C00;
/// LPTIM（低功耗定时器）
pub const LPTIM_BASE: usize = 0x4000_6000;
/// HALLTIM（霍尔定时器）
pub const HALLTIM_BASE: usize = 0x4000_6400;
/// DMA
pub const DMA_BASE: usize = 0x4002_0000;
/// FLASH 控制
pub const FLASHCTRL_BASE: usize = 0x4002_2000;
/// RAM 控制
pub const RAMCTRL_BASE: usize = 0x4002_2400;
/// 兼容别名：与官方头文件的 `RAM_BASE` 一致
pub const RAM_BASE: usize = RAMCTRL_BASE;
/// CRC
pub const CRC_BASE: usize = 0x4002_3000;
/// EAU（扩展算术单元）
pub const EAU_BASE: usize = 0x4002_3200;
/// CORDIC
pub const CORDIC_BASE: usize = 0x4002_3400;
/// LVD（低压检测，UM 第 10 章）
pub const LVD_BASE: usize = 0x4000_00A4;

// ================= 通用寄存器原语 =================

/// 可读可写寄存器
#[repr(transparent)]
pub struct Rw<T: Copy>(T);

impl<T: Copy> Rw<T> {
    /// volatile 读取
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { core::ptr::read_volatile(core::ptr::addr_of!(self.0)) }
    }

    /// volatile 写入
    #[inline(always)]
    pub fn write(&self, value: T) {
        unsafe { core::ptr::write_volatile(core::ptr::addr_of!(self.0) as *mut T, value) }
    }

    /// 读-改-写
    #[inline(always)]
    pub fn modify<F: FnOnce(T) -> T>(&self, f: F) {
        let v = self.read();
        self.write(f(v));
    }
}

/// 只读寄存器
#[repr(transparent)]
pub struct Ro<T: Copy>(T);

impl<T: Copy> Ro<T> {
    /// volatile 读取
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { core::ptr::read_volatile(core::ptr::addr_of!(self.0)) }
    }
}

/// 只写寄存器（读操作无意义）
#[repr(transparent)]
pub struct Wo<T: Copy>(T);

impl<T: Copy> Wo<T> {
    /// volatile 写入
    #[inline(always)]
    pub fn write(&self, value: T) {
        unsafe { core::ptr::write_volatile(core::ptr::addr_of!(self.0) as *mut T, value) }
    }
}

// ================= SYSCTRL（表 4-9） =================

/// 系统控制寄存器组（RCC）
#[allow(dead_code)]
#[repr(C)]
pub struct Sysctrl {
    /// 0x00 系统控制 0：SysClk 源切换 / HCLK、PCLK 分频（KEY 保护）
    pub cr0: Rw<u32>,
    /// 0x04 系统控制 1（KEY 保护）
    pub cr1: Rw<u32>,
    /// 0x08 系统控制 2（KEY 保护）
    pub cr2: Rw<u32>,
    /// 0x0C 系统中断使能
    pub ier: Rw<u32>,
    /// 0x10 系统中断标志
    pub isr: Ro<u32>,
    /// 0x14 系统中断标志清除
    pub icr: Wo<u32>,
    /// 0x18 内部高频时钟控制（HSI）
    pub hsi: Rw<u32>,
    /// 0x1C 外部高频晶体控制（HSE）
    pub hse: Rw<u32>,
    /// 0x20 内部低频时钟控制（LSI）
    pub lsi: Rw<u32>,
    /// 0x24 外部低频晶体控制（LSE）
    pub lse: Rw<u32>,
    _pad0: [u32; 1], // 0x28 PLL 寄存器未建模，占位于 _pad0
    /// 0x2C 调试状态定时器控制
    pub debug: Rw<u32>,
    /// 0x30 AHB 外设时钟使能（KEY 保护）
    pub ahben: Rw<u32>,
    /// 0x34 APB 外设时钟使能 2（KEY 保护）
    pub apben2: Rw<u32>,
    /// 0x38 APB 外设时钟使能 1（KEY 保护）
    pub apben1: Rw<u32>,
    _pad1: [u32; 1], // 0x3C
    /// 0x40 AHB 外设复位（KEY 保护）
    pub ahbrst: Rw<u32>,
    /// 0x44 APB 外设复位 2（KEY 保护）
    pub apbrst2: Rw<u32>,
    /// 0x48 APB 外设复位 1（KEY 保护）
    pub apbrst1: Rw<u32>,
    /// 0x4C 系统复位标志
    pub resetflag: Rw<u32>,
    _pad2: [u32; 8], // 0x50..0x6F
    /// 0x70 系统时钟输出控制
    pub mco: Rw<u32>,
}

// ================= GPIO（表 9-4） =================

/// GPIO 端口寄存器组（x = A/B/C/F）
#[allow(dead_code)]
#[repr(C)]
pub struct Gpio {
    /// 0x00 输入输出方向（注意：1=输入，0=输出）
    pub dir: Rw<u32>,
    /// 0x04 输出模式（1=开漏，0=推挽）
    pub opendrain: Rw<u32>,
    /// 0x08 保留（官方头文件 RESERVED）
    _pad08: [u32; 1],
    /// 0x0C 下拉电阻使能（0=禁止，1=使能；UM V1.4 §9.6.3）
    ///
    /// ⚠ 手册注明：芯片上**仅 PF3 引脚具有实际下拉电阻**——本寄存器复位值
    /// 0x0000_0008，即 PF3 复位默认使能下拉。其余引脚写 1 无下拉效果。
    /// 当 PDR.PINy 与 PUR.PINy 同时置 1 时，实际生效的是上拉。
    pub pdr: Rw<u32>,
    /// 0x10 上拉电阻使能
    pub pur: Rw<u32>,
    /// 0x14 复用功能选择高段（pin8..15，每脚 4bit）
    pub afrh: Rw<u32>,
    /// 0x18 复用功能选择低段（pin0..7，每脚 4bit）
    pub afrl: Rw<u32>,
    /// 0x1C 模拟数字配置（1=模拟）
    pub analog: Rw<u32>,
    _pad1: [u32; 1], // 0x20
    /// 0x24 上升沿中断使能
    pub riseie: Rw<u32>,
    /// 0x28 下降沿中断使能
    pub fallie: Rw<u32>,
    _pad2: [u32; 2], // 0x2C..0x33
    /// 0x34 中断标志
    pub isr: Ro<u32>,
    /// 0x38 中断标志清除
    pub icr: Wo<u32>,
    _pad3: [u32; 1], // 0x3C
    /// 0x40 中断数字滤波器配置
    pub filter: Rw<u32>,
    _pad4: [u32; 3], // 0x44..0x4F
    /// 0x50 输入数据
    pub idr: Ro<u32>,
    /// 0x54 输出数据
    pub odr: Rw<u32>,
    /// 0x58 位清零（写 1 清零对应 ODR 位）
    pub brr: Wo<u32>,
    /// 0x5C 位置位清零（低 16 位置 1，高 16 位清零；置位优先）
    pub bsrr: Wo<u32>,
    /// 0x60 位翻转（写 1 翻转）
    pub tog: Wo<u32>,
}

// ================= FLASHCTRL（表 7-5） =================

/// FLASH 控制寄存器组（KEY 保护；基地址 0x4002_2000）
#[allow(dead_code)]
#[repr(C)]
pub struct Flashctrl {
    /// 0x00 控制寄存器 1：工作模式 MODE[4:2]（0x00=读，页擦除/编程见手册 §7.5）
    pub cr1: Rw<u32>,
    /// 0x04 控制寄存器 2：WAIT[2:0] / FETCH / CACHE（读等待周期配置）
    pub cr2: Rw<u32>,
    /// 0x08 擦写锁定寄存器（LOCK0..15，每 bit 控制 8 页；1=开放）
    pub pagelock: Rw<u32>,
    _pad0: [u32; 5], // 0x0C..0x1F
    /// 0x20 中断使能寄存器
    pub ier: Rw<u32>,
    /// 0x24 中断标志寄存器
    pub isr: Ro<u32>,
    /// 0x28 中断标志清除寄存器
    pub icr: Wo<u32>,
    _pad1: [u32; 17], // 0x2C..0x6F
                      // 0x70 FLASH_SDKCFR（安全运行库区域寄存器）未建模
}

// ================= UART（表 21-13） =================

/// UART 寄存器组（x = 1/2/3）
#[allow(dead_code)]
#[repr(C)]
pub struct Uart {
    /// 0x00 控制寄存器 1
    pub cr1: Rw<u32>,
    /// 0x04 控制寄存器 2
    pub cr2: Rw<u32>,
    /// 0x08 中断使能寄存器
    pub ier: Rw<u32>,
    /// 0x0C 波特率计数器整数部分（BRRI[15:0]）
    pub brri: Rw<u32>,
    /// 0x10 波特率计数器小数部分（BRRF[3:0]）
    pub brrf: Rw<u32>,
    /// 0x14 定时器重载值寄存器
    pub timarr: Rw<u32>,
    /// 0x18 定时器计数值寄存器
    pub timcnt: Rw<u32>,
    /// 0x1C 中断标志寄存器
    pub isr: Ro<u32>,
    /// 0x20 中断标志清除寄存器（R1W0：写 0 清除，写 1 无动作）
    pub icr: Wo<u32>,
    /// 0x24 接收数据寄存器（RDR[8:0]）
    pub rdr: Ro<u32>,
    /// 0x28 发送数据寄存器（TDR[8:0]，含 IDLE[9] / BREAK[10]）
    pub tdr: Wo<u32>,
    _pad0: [u32; 1], // 0x2C（官方 RESERVED）
    /// 0x30 接收地址匹配寄存器（ADDR[7:0]）
    pub addr: Rw<u32>,
    /// 0x34 接收地址掩码寄存器（MASK[7:0]）
    pub mask: Rw<u32>,
    /// 0x38 控制寄存器 3
    pub cr3: Rw<u32>,
    /// 0x3C 接收地址匹配数据寄存器（RXMATCH[8:0]）
    pub rxmatch: Ro<u32>,
}

/// SPI 寄存器组（§22.7，x = 1/2/3）
#[allow(dead_code)]
#[repr(C)]
pub struct Spi {
    /// 0x00 控制寄存器 1（仅在 CR2.EN=0 时可写）
    pub cr1: Rw<u32>,
    /// 0x04 控制寄存器 2（EN[0]）
    pub cr2: Rw<u32>,
    /// 0x08 控制寄存器 3（HDOE[0]，单线半双工方向）
    pub cr3: Rw<u32>,
    /// 0x0C 从机选择寄存器（SSI[0]，主机 SSM=1 时输出到 SPI_CS）
    pub ssi: Rw<u32>,
    /// 0x10 中断使能寄存器
    pub ier: Rw<u32>,
    /// 0x14 中断标志寄存器（BUSY/TXE/RXNE…）
    pub isr: Ro<u32>,
    /// 0x18 中断标志清除寄存器（R1W0：写 0 清除，写 1 无动作）
    pub icr: Wo<u32>,
    /// 0x1C 数据寄存器（DR[15:0]，写发送读接收）
    pub dr: Rw<u32>,
}

/// I2C 主机侧寄存器组（§23.8，x = 1/2；从机区 0x100+ 未建模）
#[allow(dead_code)]
#[repr(C)]
pub struct I2c {
    /// 0x00..0x07 保留
    _pad0: [u32; 2],
    /// 0x08 总线输入信号选择寄存器
    pub insel: Rw<u32>,
    /// 0x0C 主机中断标志清除寄存器（R1W0：写 0 清除，写 1 无动作）
    pub micr: Wo<u32>,
    /// 0x10 主机控制寄存器 0（MEN/FIFO 复位/时钟源）
    pub mcr0: Rw<u32>,
    /// 0x14 主机中断标志寄存器（TXE/RXNE/NACK/ARBI/BUSY…）
    pub misr: Ro<u32>,
    /// 0x18 主机中断使能寄存器
    pub mier: Rw<u32>,
    /// 0x1C 主机 DMA 使能寄存器
    pub mder: Rw<u32>,
    /// 0x20 主机控制寄存器 1（RDMO/CIRFIFO）
    pub mcr1: Rw<u32>,
    /// 0x24 主机控制寄存器 2（PRESCALE/MATCFG/PINCFG/IGNACK/AUTOSTOP）
    pub mcr2: Rw<u32>,
    /// 0x28 主机控制寄存器 3（FLTSCL/FLTSDA/BUSIDLE）
    pub mcr3: Rw<u32>,
    /// 0x2C 主机控制寄存器 4（PINLOW 超时）
    pub mcr4: Rw<u32>,
    /// 0x30..0x3F 保留
    _pad1: [u32; 4],
    /// 0x40 主机数据匹配寄存器（MATCH0/MATCH1）
    pub mmatch: Rw<u32>,
    /// 0x44 保留
    _pad2: u32,
    /// 0x48 主机时序配置寄存器（CLKHI/CLKLO/SETHOLD/DATAVD；MEN=1 后禁改）
    pub mccr: Rw<u32>,
    /// 0x4C..0x57 保留
    _pad3: [u32; 3],
    /// 0x58 主机 FIFO 控制寄存器
    pub mfifocr: Rw<u32>,
    /// 0x5C 主机 FIFO 状态寄存器
    pub mfifosr: Ro<u32>,
    /// 0x60 主机发送数据寄存器（CMD[10:8]+DATA[7:0]，全 WO）
    pub mtdr: Wo<u32>,
    /// 0x64..0x6F 保留
    _pad4: [u32; 3],
    /// 0x70 主机接收数据寄存器（DATA[7:0]+EMPTY[14]）
    pub mrdr: Ro<u32>,
}

/// BTIM 基础定时器寄存器组（§14.10，x = 1/2/3）
#[allow(dead_code)]
#[repr(C)]
pub struct Btim {
    /// 0x00 控制寄存器 1（EN/UDIS/URS/ONESHOT/TOGEN）
    pub cr1: Rw<u32>,
    /// 0x04 控制寄存器 2（MMS 主模式输出选择）
    pub cr2: Rw<u32>,
    /// 0x08 从模式控制寄存器（SMS=0 为内部计数）
    pub smcr: Rw<u32>,
    /// 0x0C DMA/中断使能寄存器（UIE/TIE/UDE/TDE）
    pub dier: Rw<u32>,
    /// 0x10 中断标志寄存器（UIF/TIF）
    pub isr: Ro<u32>,
    /// 0x14 事件生成寄存器（UG/TG，写 1 触发）
    pub egr: Wo<u32>,
    /// 0x18 中断标志清除寄存器（R1W0：写 0 清除）
    pub icr: Wo<u32>,
    /// 0x1C..0x23 保留
    _pad0: [u32; 2],
    /// 0x24 计数寄存器（CNT[15:0]）
    pub cnt: Ro<u32>,
    /// 0x28 预分频寄存器（PSC[15:0]：CK_CNT = PCLK/(PSC+1)）
    pub psc: Rw<u32>,
    /// 0x2C 自动重载寄存器（ARR[15:0]，复位 0xFFFF）
    pub arr: Rw<u32>,
}

/// ADC 寄存器组（§25.12，x = 1/2）
#[allow(dead_code)]
#[repr(C)]
pub struct Adc {
    /// 0x00 控制寄存器（EN/CONT/CLK/ENS/SLAVE；bit8 复位为 1 须保留）
    pub cr: Rw<u32>,
    /// 0x04 启动寄存器（写 1 启动，写 0 停止并复位序列）
    pub start: Rw<u32>,
    /// 0x08 模拟看门狗阈值寄存器
    pub awdtr: Rw<u32>,
    /// 0x0C 模拟看门狗配置寄存器
    pub awdcr: Rw<u32>,
    /// 0x10 外部触发启动寄存器
    pub trigger: Rw<u32>,
    /// 0x14 保留
    _pad0: u32,
    /// 0x18 采样周期配置（SQRCH0..7，各 4bit，表 25-2）
    pub sample: Rw<u32>,
    /// 0x1C 保留（SAMPLE 与 SQRCFR 间空洞）
    _pad_s: u32,
    /// 0x20 序列转换通道配置（SQRCH0..7，各 4bit，表 25-4）
    pub sqrcfr: Rw<u32>,
    /// 0x24..0x2F 保留
    _pad1: [u32; 3],
    /// 0x30 序列转换结果 0（RESULT[11:0]）
    pub result0: Ro<u32>,
    /// 0x34 序列转换结果 1
    pub result1: Ro<u32>,
    /// 0x38 序列转换结果 2
    pub result2: Ro<u32>,
    /// 0x3C 序列转换结果 3
    pub result3: Ro<u32>,
    /// 0x40 序列转换结果 4
    pub result4: Ro<u32>,
    /// 0x44 序列转换结果 5
    pub result5: Ro<u32>,
    /// 0x48 序列转换结果 6
    pub result6: Ro<u32>,
    /// 0x4C 序列转换结果 7
    pub result7: Ro<u32>,
    /// 0x50..0x6F 保留
    _pad2: [u32; 8],
    /// 0x70 中断使能寄存器
    pub ier: Rw<u32>,
    /// 0x74 中断标志清除寄存器（R1W0：写 0 清除）
    pub icr: Wo<u32>,
    /// 0x78 中断标志寄存器（EOC/EOS/AWDL/AWDH）
    pub isr: Ro<u32>,
}

/// DAC 寄存器组（§26.10，DAC_BASE=0x4000_00C0）
#[allow(dead_code)]
#[repr(C)]
pub struct Dac {
    /// 0x00 控制寄存器 0（EN1/EN2/TEN/TSEL/WAVE/MAMP/DMA）
    pub cr0: Rw<u32>,
    /// 0x04 软件触发寄存器（SWTRIG1/2，写 1 触发自动清零）
    pub swtrgr: Wo<u32>,
    /// 0x08 通道 1 12 位右对齐数据保持
    pub dhr12r1: Rw<u32>,
    /// 0x0C 通道 1 12 位左对齐数据保持
    pub dhr12l1: Rw<u32>,
    /// 0x10 通道 1 8 位右对齐数据保持
    pub dhr8r1: Rw<u32>,
    /// 0x14 通道 2 12 位右对齐数据保持
    pub dhr12r2: Rw<u32>,
    /// 0x18 通道 2 12 位左对齐数据保持
    pub dhr12l2: Rw<u32>,
    /// 0x1C 通道 2 8 位右对齐数据保持
    pub dhr8r2: Rw<u32>,
    /// 0x20 双通道 12 位右对齐（低 16 位 CH1、高 16 位 CH2）
    pub dhr12rd: Rw<u32>,
    /// 0x24 双通道 12 位左对齐
    pub dhr12ld: Rw<u32>,
    /// 0x28 双通道 8 位右对齐
    pub dhr8rd: Rw<u32>,
    /// 0x2C 通道 1 数据输出（DACC1DOR[11:0]，RO）
    pub dor1: Ro<u32>,
    /// 0x30 通道 2 数据输出（DACC2DOR[11:0]，RO）
    pub dor2: Ro<u32>,
    /// 0x34 控制寄存器 1（DAC_OUT1[0]/DAC_OUT2[1] 引脚输出使能）
    pub cr1: Rw<u32>,
}

/// IWDT 独立看门狗寄存器组（§19.6，IWDT_BASE=0x4000_5000）
#[allow(dead_code)]
#[repr(C)]
pub struct Iwdt {
    /// 0x00 键值寄存器（WO：0xCCCC 启动 / 0xAAAA 喂狗 / 0x5555 解锁 /
    /// 0x5A5A→0xA5A5 停止；其它值=重新上锁）
    pub kr: Wo<u32>,
    /// 0x04 控制寄存器（PRS[2:0]/ACTION/IE/PAUSE；写需 KR=0x5555 解锁）
    pub cr: Rw<u32>,
    /// 0x08 重载寄存器（ARR[11:0]，复位 0xFFF；写需解锁）
    pub arr: Rw<u32>,
    /// 0x0C 状态寄存器（OV[3] RW0：写 0 清除溢出标志）
    pub sr: Rw<u32>,
    /// 0x10 窗口寄存器（WINR[11:0]，写需解锁）
    pub winr: Rw<u32>,
    /// 0x14..0x23 保留
    _pad0: [u32; 4],
    /// 0x24 计数值寄存器（CNT[11:0]，RO，复位 0xFFF）
    pub cnt: Ro<u32>,
}

/// GTIM 通用定时器寄存器组（§16.10，x = 1/2/3/4）
#[allow(dead_code)]
#[repr(C)]
pub struct Gtim {
    /// 0x00 控制寄存器 1（EN/UDIS/URS/OPM/DIR/CMS/ARPE/CKD）
    pub cr1: Rw<u32>,
    /// 0x04 控制寄存器 2（MMS 主模式输出）
    pub cr2: Rw<u32>,
    /// 0x08 从模式控制寄存器（SMS/TS）
    pub smcr: Rw<u32>,
    /// 0x0C DMA/中断使能寄存器
    pub dier: Rw<u32>,
    /// 0x10 中断标志寄存器（UIF/CC1IF..CC4IF）
    pub isr: Ro<u32>,
    /// 0x14 事件生成寄存器（UG/CC1G..CC4G，写 1 触发）
    pub egr: Wo<u32>,
    /// 0x18 捕获/比较模式寄存器 1（CAP/CMP 同址：OC1M[6:4]/OC1PE@3）
    pub ccmr1: Rw<u32>,
    /// 0x1C 捕获/比较模式寄存器 2（OC2M[6:4]/OC2PE@3）
    pub ccmr2: Rw<u32>,
    /// 0x20 捕获/比较使能寄存器（CC1E@0/CC1P@1…CC4E@12/CC4P@13）
    pub ccer: Rw<u32>,
    /// 0x24 计数寄存器（CNT[15:0]）
    pub cnt: Ro<u32>,
    /// 0x28 预分频寄存器（PSC[15:0]）
    pub psc: Rw<u32>,
    /// 0x2C 自动重载寄存器（ARR[15:0]）
    pub arr: Rw<u32>,
    /// 0x30 保留
    _pad0: u32,
    /// 0x34 捕获/比较寄存器 1
    pub ccr1: Rw<u32>,
    /// 0x38 捕获/比较寄存器 2
    pub ccr2: Rw<u32>,
    /// 0x3C 捕获/比较寄存器 3
    pub ccr3: Rw<u32>,
    /// 0x40 捕获/比较寄存器 4
    pub ccr4: Rw<u32>,
    /// 0x44..0x57 保留
    _pad1: [u32; 5],
    /// 0x58 编码器控制寄存器
    pub ecr: Rw<u32>,
    /// 0x5C TI 输入选择寄存器
    pub tisel: Rw<u32>,
    /// 0x60 复用功能选项寄存器 1
    pub af1: Rw<u32>,
    /// 0x64 复用功能选项寄存器 2
    pub af2: Rw<u32>,
}

/// BGR 模块控制寄存器（§25.12.19；与 ADC1 同页，BGR_BASE=0x4000_0000）
#[allow(dead_code)]
#[repr(C)]
pub struct Bgr {
    /// 0xFC 控制寄存器（BGREN[0]/TSEN[1]）
    pub cr: Rw<u32>,
}

/// LPTIM 低功耗定时器（0x4000_6000，§15；LSI/LSE 域，深度休眠可运行）
#[repr(C)]
pub struct Lptim {
    /// 0x00 中断和状态寄存器（CMPM/ARRM/EXTTRIG/CMPOK/ARROK/UP/DOWN）
    pub isr: Ro<u32>,
    /// 0x04 中断清除寄存器（R1W0：对应位写 0 清除；复位 0x7F）
    pub icr: Wo<u32>,
    /// 0x08 中断使能寄存器（**仅 EN=0 时可写**）
    pub ier: Rw<u32>,
    /// 0x0C 配置寄存器（ICLKSRC/PRS/TRIGEN/COUNTMD 等；仅 EN=0 时可写）
    pub cfgr: Rw<u32>,
    /// 0x10 控制寄存器 0（EN/SNGSTART/CNTSTART/ARST/SRST；复位 0x18）
    pub cr0: Rw<u32>,
    /// 0x14 比较寄存器（15:0）
    pub cmp: Rw<u32>,
    /// 0x18 自动重载寄存器（15:0，复位 0x1）
    pub arr: Rw<u32>,
    /// 0x1C 计数器寄存器（RO；需连续双读一致）
    pub cnt: Ro<u32>,
    /// 0x20 控制寄存器 1（CH1/CH2 输入源：引脚或 VC1-4 输出）
    pub cr1: Rw<u32>,
}

/// HALLTIM 寄存器组（HALLTIM_BASE=0x4000_6400，表 18-4）
///
/// 24bit 递增计数器 + 3 路捕获 + 1 路 PWM 输出。计数时钟 = PCLK/DIV，
/// 捕获沿到来时硬件把 CNT 存入 WIDTH 并清零 CNT。
pub struct Halltim {
    /// 0x00 控制寄存器（EN/FLT1EN/DIV/MMS/FLT2LEN/SOFTCAP，复位 0x0000_0004）
    pub cr: Rw<u32>,
    /// 0x04 DMA/中断使能（CAPIE/OVIE/MATCHIE + CAPDMA/OVDMA/MATCHDMA/SOFTDMA）
    pub dier: Rw<u32>,
    /// 0x08 24bit 计数值（RW0：写 0 清零）
    pub cnt: Rw<u32>,
    /// 0x0C 自动重装载值（24bit，复位 0x00FF_FFFF）
    pub arr: Rw<u32>,
    /// 0x10 HALL 脉冲宽度捕获值（24bit，RO）
    pub width: Ro<u32>,
    /// 0x14 比较寄存器（24bit，PWM 占空比）
    pub ccr: Rw<u32>,
    /// 0x18 中断标志（CAPF/OVF/MATCHF，RO）
    pub isr: Ro<u32>,
    /// 0x1C 中断标志清除（R1W0：对应位写 0 清除，复位 0x0000_FFFF）
    pub icr: Wo<u32>,
    /// 0x20 状态寄存器（CHxF 滤波后电平 / CHxS 原始电平）
    pub state: Ro<u32>,
}

/// VCx 模拟电压比较器寄存器组（VCx_BASE + 0x04..0x10，表 27-5）
///
/// 注意：`+0x00` 位置不是本实例的寄存器——VC1/VC3 的该地址是共享分压
/// 寄存器 [`VcRef`]，VC2/VC4 的该地址是**前一个实例的 SR**，故建模为保留。
pub struct Vc {
    _ref_or_prev_sr: u32, // 0x00
    /// 0x04 控制寄存器 0（INP/INN/WINDOW/POL/IE/HYS/RESP/EN）
    pub cr0: Rw<u32>,
    /// 0x08 控制寄存器 1（BLANKLVL/BLANKTIME/HIGHIE/RISEIE/FALLIE/FLTCLK/FLTTIME）
    pub cr1: Rw<u32>,
    /// 0x0C 控制寄存器 2（BLANK 窗口触发源使能，每位一个触发源）
    pub cr2: Rw<u32>,
    /// 0x10 状态寄存器（FLTV[1] RO + INTF[0] RW0）
    pub sr: Rw<u32>,
}

/// VC 电阻分压参考源（VC12_REF @0x4000_0080 / VC34_REF @0x4000_0180）
///
/// 输出电压 = VIN × (1 + DIV) / 8，VIN 可选 VDDA 或 Vcore(≈1.6V)。
/// VC1/VC2 共用一份，VC3/VC4 共用另一份。
pub struct VcRef {
    /// 0x00 分压控制（VIN[5] / EN[4] / DIV[3:0]）
    pub cr: Rw<u32>,
}

/// OPAx 运算放大器寄存器组（OPA1_BASE=0x4000_00B0 / OPA2_BASE=0x4000_00B8）
pub struct Opa {
    /// 0x00 控制寄存器（BIAS/AMP/INNxEN/INPxEN/MODE/EN，复位 0x0000_E000）
    pub cr: Rw<u32>,
    /// 0x04 校准寄存器（触发源选择 + CLKDIV/AZRUN/CALPERIOD/SOFTTRIG/START/CALMODE/CALEN）
    pub cal: Rw<u32>,
}

// ================= ATIM（高级定时器，表 17-x，§17.10） =================

/// ATIM 寄存器组（单实例，ATIM_BASE=0x4000_1400）
///
/// 注意：CCMR1CAP/CCMR1CMP 等捕获与比较寄存器为**同址别名**（0x18/0x1C/0x50），
/// 各建模为一个字段；ICR 位于 0x70（在 AF2@0x64 与 TISEL2@0x6C 之后）。
#[allow(dead_code)]
#[repr(C)]
pub struct Atim {
    /// 0x00 控制寄存器 1（CEN/UDIS/URS/OPM/DIR/CMS/ARPE/CKD）
    pub cr1: Rw<u32>,
    /// 0x04 控制寄存器 2（CCPC/CCUS/CCDS/MMS[2:0]/TI1S/OISx/OISxN）
    pub cr2: Rw<u32>,
    /// 0x08 从模式控制寄存器（MSM/TS[3:0]/ETF/ETPS/ETP/ECE）
    pub smcr: Rw<u32>,
    /// 0x0C DMA/中断使能寄存器（UIE 及各 CCxIE/TIE/BIE/COMIE）
    pub dier: Rw<u32>,
    /// 0x10 中断标志寄存器（UIF 及各 CCxIF/COMIF/TRG/BIF/B2IF）
    pub isr: Ro<u32>,
    /// 0x14 事件生成寄存器（UG/CCxG/COMG/TG/BG/B2G/CC5G/CC6G）
    pub egr: Wo<u32>,
    /// 0x18 比较模式寄存器 1（OC1M[6:4]+OC1MH[16]/OC1PE、OC2 同布局高位）
    pub ccmr1: Rw<u32>,
    /// 0x1C 比较模式寄存器 2（OC3/OC4，布局同 CCMR1）
    pub ccmr2: Rw<u32>,
    /// 0x20 捕获/比较使能寄存器（CCyE/CCyP/CCyNE/CCyNP，每通道 4bit；CC5/6 无互补）
    pub ccer: Rw<u32>,
    /// 0x24 计数寄存器 CNT[15:0]
    pub cnt: Ro<u32>,
    /// 0x28 预分频寄存器 PSC[15:0]
    pub psc: Rw<u32>,
    /// 0x2C 自动重载寄存器 ARR[15:0]（复位 0xFFFF）
    pub arr: Rw<u32>,
    /// 0x30 重复计数寄存器 REP[15:0]
    pub rcr: Rw<u32>,
    /// 0x34 捕获/比较寄存器 1
    pub ccr1: Rw<u32>,
    /// 0x38 捕获/比较寄存器 2
    pub ccr2: Rw<u32>,
    /// 0x3C 捕获/比较寄存器 3
    pub ccr3: Rw<u32>,
    /// 0x40 捕获/比较寄存器 4
    pub ccr4: Rw<u32>,
    /// 0x44 刹车和死区寄存器（DTG[7:0]/LOCK/OSSI/OSSR/BKE/BKP/AOE/MOE/BKF/BK2F/BK2E/BK2P）
    pub bdtr: Rw<u32>,
    /// 0x48 捕获/比较寄存器 5
    pub ccr5: Rw<u32>,
    /// 0x4C 捕获/比较寄存器 6
    pub ccr6: Rw<u32>,
    /// 0x50 比较模式寄存器 3（OC5/OC6，布局同 CCMR1）
    pub ccmr3: Rw<u32>,
    /// 0x54 死区时间寄存器 2（DTPE/DATE/DTGF）
    pub dtr2: Rw<u32>,
    /// 0x58 编码控制寄存器
    pub ecr: Rw<u32>,
    /// 0x5C TI 输入选择寄存器 1
    pub tisel1: Rw<u32>,
    /// 0x60 复用功能选项寄存器 1（ETRSEL，复位 0x1）
    pub af1: Rw<u32>,
    /// 0x64 复用功能选项寄存器 2（复位 0x1）
    pub af2: Rw<u32>,
    _pad0: [u32; 1], // 0x68 保留
    /// 0x6C TI 输入选择寄存器 2
    pub tisel2: Rw<u32>,
    /// 0x70 中断标志清除寄存器（R1W0：写 0 清除；复位 0x00FF_3FFF）
    pub icr: Wo<u32>,
}

// ================= WWDT（窗口看门狗，表 20-1） =================

/// WWDT 寄存器组（单实例，WWDT_BASE=0x4000_5400）
#[allow(dead_code)]
#[repr(C)]
pub struct Wwdt {
    /// 0x00 控制寄存器 0（EN[7] RW1 不可关、WCNT[6:0] 读=当前值/写=刷新喂狗）
    pub cr0: Rw<u32>,
    /// 0x04 控制寄存器 1（IE[10] 置 1 不可清、PRS[9:7]、WINR[6:0]）
    pub cr1: Rw<u32>,
    /// 0x08 状态寄存器（POV[0]，写 0 清除）
    pub sr: Rw<u32>,
}

// ================= 位定义 =================

/// SYSCTRL 位定义（§4.7）
pub mod sysctrl {
    /// 写保护键：CR0 / CR1 / CR2 / AHBEN / APBEN1 / APBEN2 / AHBRST / APBRST1 / APBRST2
    /// 高 16 位必须为 0x5A5A，否则写入无效
    pub const KEY: u32 = 0x5A5A << 16;

    // ---- CR0 (0x00) ----
    /// SysClk 时钟源（CR0[2:0]）：HSI
    pub const SYSCLK_HSI: u32 = 0b000;
    /// SysClk 时钟源（CR0[2:0]）：HSE
    pub const SYSCLK_HSE: u32 = 0b001;
    /// SysClk 时钟源（CR0[2:0]）：LSI
    pub const SYSCLK_LSI: u32 = 0b011;
    /// SysClk 时钟源（CR0[2:0]）：LSE
    pub const SYSCLK_LSE: u32 = 0b100;
    /// HCLK 分频位（CR0[7:5]），分频系数 = 2^HCLKPRS
    pub const HCLKPRS_POS: u32 = 5;
    /// HCLK 分频位掩码
    pub const HCLKPRS_MASK: u32 = 0x7 << HCLKPRS_POS;
    /// PCLK 分频位（CR0[4:3]），分频系数 = 2^PCLKPRS
    pub const PCLKPRS_POS: u32 = 3;
    /// PCLK 分频位掩码
    pub const PCLKPRS_MASK: u32 = 0x3 << PCLKPRS_POS;

    // ---- CR2 (0x08)：复位值 0x0000_0C00 ----
    /// FLASH 取指周期配置（CR2[6:4]，与 FLASH_CR2.WAIT[2:0] 功能相同）
    pub const CR2_FLASHWAIT_POS: u32 = 4;
    /// FLASH 取指周期配置掩码
    pub const CR2_FLASHWAIT_MASK: u32 = 0x7 << CR2_FLASHWAIT_POS;

    // ---- CR1 (0x04)：KEY 保护 ----
    /// LSI 振荡器使能（CR1[3]，§4.5.5 步骤 2）——**复位默认关闭**
    pub const CR1_LSIEN: u32 = 1 << 3;

    // ---- LSI (0x20)：复位值 0x---- ----
    /// LSI 稳定标志（LSI[15]，RO）：1 = 输出稳定，可用于 LPTIM/IWDT 等
    pub const LSI_STABLE: u32 = 1 << 15;
    /// LSI 频率微调（LSI[8:0]）：每 +1 ≈ +0.4%
    pub const LSI_TRIM_MASK: u32 = 0x1FF;
    /// LSI 稳定检测等待周期（LSI[11:10]）：00=6 / 01=18 / 10=34 / 11=66 个周期
    pub const LSI_WAITCYCLE_POS: u32 = 10;
    /// LSI 标称频率（Hz）：出厂 TRIM 校准后精度约 ±1%
    pub const LSI_HZ: u32 = 32_800;

    // ---- LSE (0x24)：复位值 0x0000_0A22，仅 POR 可复位 ----
    /// LSE 稳定标志（LSE[18]，RO）
    pub const LSE_STABLE: u32 = 1 << 18;
    /// LSE 引脚锁定为模拟（LSE[17]，需 LSELOCK=1）
    pub const LSE_PINLOCK: u32 = 1 << 17;
    /// 驱动能力（LSE[3:0]）
    pub const LSE_DRIVER_MASK: u32 = 0xF;

    // ---- CR1 LSE 控制位 ----
    /// LSE 使能（CR1[4]，KEY 保护）
    pub const CR1_LSEEN: u32 = 1 << 4;
    /// LSE 启用后不可关闭（CR1[5]）
    pub const CR1_LSELOCK: u32 = 1 << 5;
    /// LSE 时钟监测（CR1[6]）
    pub const CR1_LSECCS: u32 = 1 << 6;

    // ---- AHBEN (0x30) ----
    /// AHBEN.DMA（bit0）：DMA 配置时钟及工作时钟
    pub const AHBEN_DMA: u32 = 1 << 0;
    /// AHBEN.FLASH（bit1）：FLASH 模块配置时钟
    pub const AHBEN_FLASH: u32 = 1 << 1;
    /// AHBEN.CRC（bit2）：CRC 配置时钟及工作时钟
    pub const AHBEN_CRC: u32 = 1 << 2;
    /// AHBEN.EAU（bit3）：EAU 配置时钟及工作时钟
    pub const AHBEN_EAU: u32 = 1 << 3;
    /// AHBEN.GPIOA（bit4）
    pub const AHBEN_GPIOA: u32 = 1 << 4;
    /// AHBEN.GPIOB（bit5）
    pub const AHBEN_GPIOB: u32 = 1 << 5;
    /// AHBEN.GPIOC（bit6）
    pub const AHBEN_GPIOC: u32 = 1 << 6;
    /// AHBEN.GPIOF（bit7）
    pub const AHBEN_GPIOF: u32 = 1 << 7;
    /// AHBEN.CORDIC（bit8）
    pub const AHBEN_CORDIC: u32 = 1 << 8;

    // ---- HSI (0x18) ----
    /// HSIOSC 时钟稳定标志（只读）
    pub const HSI_STABLE: u32 = 1 << 15;
    /// HSI 分频编码位（HSI.DIV[14:11]）
    pub const HSI_DIV_POS: u32 = 11;
    /// HSI 分频编码掩码
    pub const HSI_DIV_MASK: u32 = 0xF << HSI_DIV_POS;
    /// HSI 分频编码 → 分频数（手册 §4.7.4 查找表：
    /// 0000=/32、0001~/1 … 1010=/10、1011=/12、1100=/16、1101=/20、
    /// 1110=/24（复位默认）、1111=/28；未定义编码返回 0）
    pub const fn hsi_div_code_to_divisor(code: u32) -> u32 {
        match code {
            0 => 32,
            1..=10 => code,
            11 => 12,
            12 => 16,
            13 => 20,
            14 => 24,
            15 => 28,
            _ => 0,
        }
    }

    /// HSI.TRIM[10:0] 振荡器校准位
    pub const HSI_TRIM_MASK: u32 = 0x7FF;

    // ---- APBEN1 (0x38) ----
    /// SPI1 时钟使能（bit2）
    pub const APBEN1_SPI1: u32 = 1 << 2;
    /// ADC 时钟使能（bit0）
    pub const APBEN1_ADC: u32 = 1 << 0;
    /// VC 比较器配置时钟使能（bit1）
    pub const APBEN1_VC: u32 = 1 << 1;
    /// UART1 时钟使能（bit3）
    pub const APBEN1_UART1: u32 = 1 << 3;
    /// UART2 时钟使能（bit4）
    pub const APBEN1_UART2: u32 = 1 << 4;
    /// ATIM 时钟使能（bit5）
    pub const APBEN1_ATIM: u32 = 1 << 5;
    /// GTIM1 时钟使能（bit6）
    pub const APBEN1_GTIM1: u32 = 1 << 6;
    /// GTIM2 时钟使能（bit7）
    pub const APBEN1_GTIM2: u32 = 1 << 7;
    /// UART3 时钟使能（bit8）
    pub const APBEN1_UART3: u32 = 1 << 8;
    /// GTIM3 时钟使能（bit11）
    pub const APBEN1_GTIM3: u32 = 1 << 11;
    /// GTIM4 时钟使能（bit12）
    pub const APBEN1_GTIM4: u32 = 1 << 12;
    /// SPI2 时钟使能（bit13）
    pub const APBEN1_SPI2: u32 = 1 << 13;
    /// SPI3 时钟使能（bit14）
    pub const APBEN1_SPI3: u32 = 1 << 14;

    // ---- APBEN2 (0x34) ----
    /// RTC 时钟使能（bit1）
    pub const APBEN2_RTC: u32 = 1 << 1;
    /// BTIM1-3 时钟使能（bit2）
    pub const APBEN2_BTIM: u32 = 1 << 2;
    /// IWDT 时钟使能（bit4）
    pub const APBEN2_IWDT: u32 = 1 << 4;
    /// WWDT 时钟使能（bit5）
    pub const APBEN2_WWDT: u32 = 1 << 5;
    /// I2C1 时钟使能（bit6）
    pub const APBEN2_I2C1: u32 = 1 << 6;
    /// LPTIM 时钟使能（bit7）
    pub const APBEN2_LPTIM: u32 = 1 << 7;
    /// OPA 运放配置时钟使能（bit9，§4.7.13）
    pub const APBEN2_OPA: u32 = 1 << 9;
    /// DAC 时钟使能（bit10）
    pub const APBEN2_DAC: u32 = 1 << 10;
    /// I2C2 时钟使能（bit11）
    pub const APBEN2_I2C2: u32 = 1 << 11;
    /// HALLTIM 时钟使能（bit12）
    pub const APBEN2_HALLTIM: u32 = 1 << 12;
}

/// GPIO 位定义（§9.6）
pub mod gpio {
    /// DIR 位取值：输入（注意与部分 MCU 相反）
    pub const DIR_INPUT: u32 = 1;
    /// DIR 位取值：输出
    pub const DIR_OUTPUT: u32 = 0;
}

/// UART 位定义（§21.9）
pub mod uart {
    // ---- CR1 (0x00) ----
    /// CR1.SYNC[14]：同步模式
    pub const CR1_SYNC: u32 = 1 << 14;
    /// CR1.SOURCE[13:12]=00：UCLK 来自 PCLK（复位默认）
    pub const CR1_SOURCE_PCLK: u32 = 0 << 12;
    /// CR1.SIGNAL[11]：单线半双工
    pub const CR1_SIGNAL: u32 = 1 << 11;
    /// CR1.MSBF[7]：高位先行
    pub const CR1_MSBF: u32 = 1 << 7;
    /// CR1.CHLEN[6]=1：9 位字长（0 为 8 位）
    pub const CR1_CHLEN_9: u32 = 1 << 6;
    /// CR1.STOP[5:4]=01：1.5 停止位
    pub const CR1_STOP_1_5: u32 = 1 << 4;
    /// CR1.STOP[5:4]=10：2 停止位
    pub const CR1_STOP_2: u32 = 2 << 4;
    /// CR1.PARITYEN[3]：奇偶校验使能
    pub const CR1_PARITYEN: u32 = 1 << 3;
    /// CR1.PARITY[2]=1：奇校验（0 为偶校验）
    pub const CR1_PARITY_ODD: u32 = 1 << 2;
    /// CR1.RXEN[1]：接收使能
    pub const CR1_RXEN: u32 = 1 << 1;
    /// CR1.TXEN[0]：发送使能
    pub const CR1_TXEN: u32 = 1 << 0;

    // ---- ISR (0x1C) / IER (0x08) 共用位 ----
    /// 接收数据匹配
    pub const ISR_RXMATCH: u32 = 1 << 12;
    /// 溢出错误
    pub const ISR_ORE: u32 = 1 << 11;
    /// 噪声错误
    pub const ISR_NE: u32 = 1 << 10;
    /// 奇偶校验错误
    pub const ISR_PE: u32 = 1 << 9;
    /// 帧结构错误
    pub const ISR_FE: u32 = 1 << 8;
    /// CTS 电平变化
    pub const ISR_CTS: u32 = 1 << 7;
    /// 定时器溢出
    pub const ISR_TIMOV: u32 = 1 << 6;
    /// 自动波特率检测完成
    pub const ISR_BAUD: u32 = 1 << 5;
    /// 间隔段接收完成
    pub const ISR_RXBRK: u32 = 1 << 4;
    /// 空闲字符接收完成
    pub const ISR_RXIDLE: u32 = 1 << 3;
    /// 接收完成
    pub const ISR_RC: u32 = 1 << 2;
    /// 发送完成
    pub const ISR_TC: u32 = 1 << 1;
    /// 发送缓冲空
    pub const ISR_TXBE: u32 = 1 << 0;

    // ---- ICR (0x20)：R1W0 语义（写 0 清除，写 1 无动作）----
    /// ICR 复位值：全部位为 1（无动作状态）
    pub const ICR_NONE: u32 = 0x1FFF;
    /// 写 0 清除 ORE（溢出错误）标志
    pub const ICR_CLEAR_ORE: u32 = 0x1FFF & !ISR_ORE;
    /// 写 0 清除 NE（噪声错误）标志
    pub const ICR_CLEAR_NE: u32 = 0x1FFF & !ISR_NE;
    /// 写 0 清除 PE（校验错误）标志
    pub const ICR_CLEAR_PE: u32 = 0x1FFF & !ISR_PE;
    /// 写 0 清除 FE（帧错误）标志
    pub const ICR_CLEAR_FE: u32 = 0x1FFF & !ISR_FE;
    /// 写 0 清除全部接收错误标志（ORE/NE/PE/FE）
    pub const ICR_CLEAR_RX_ERRORS: u32 = 0x1FFF & !(ISR_ORE | ISR_NE | ISR_PE | ISR_FE);
}

/// SPI 位定义（§22.7，UM V1.4）
pub mod spi {
    // ---- CR1 (0x00)：复位值 0x0000_0700（BR=7 → PCLK/16）----
    /// 波特率 N[30:24]：SCK = PCLK / (2×(N+1))，N=0..=127
    pub const CR1_BR_POS: u32 = 24;
    pub const CR1_BR_MASK: u32 = 0x7F << CR1_BR_POS;
    /// 主机延后采样（CR1[20]，约 20ns）
    pub const CR1_SMP: u32 = 1 << 20;
    /// 从机 MISO 输出配置（CR1[19]）
    pub const CR1_MISOHD: u32 = 1 << 19;
    /// 输入滤波使能（CR1[18]，仅 fSCK ≤ fPCLK/16 时使用）
    pub const CR1_FLTEN: u32 = 1 << 18;
    /// 通信模式 MODE[17:16]：00 全双工 / 01 单工单发 / 10 单工单收 / 11 单线半双工
    pub const CR1_MODE_POS: u32 = 16;
    pub const CR1_MODE_MASK: u32 = 0x3 << CR1_MODE_POS;
    pub const CR1_MODE_FULL_DUPLEX: u32 = 0;
    pub const CR1_MODE_TX_ONLY: u32 = 1 << CR1_MODE_POS;
    pub const CR1_MODE_RX_ONLY: u32 = 2 << CR1_MODE_POS;
    pub const CR1_MODE_HALF_DUPLEX: u32 = 3 << CR1_MODE_POS;
    /// 帧间隔 GAP[15:12]：0=无间隔，N=T SCK×N
    pub const CR1_GAP_MASK: u32 = 0xF << 12;
    /// 帧宽度 WIDTH[11:8]：宽度 = WIDTH+1（4..16 bit；写 0~2 自动按 8bit）
    pub const CR1_WIDTH_POS: u32 = 8;
    pub const CR1_WIDTH_MASK: u32 = 0xF << CR1_WIDTH_POS;
    /// LSB 先行（CR1[7]；0=MSB 先行）
    pub const CR1_LSBF: u32 = 1 << 7;
    /// 时钟极性 CPOL（CR1[3]：0 待机低 / 1 待机高）
    pub const CR1_CPOL: u32 = 1 << 3;
    /// 时钟相位 CPHA（CR1[2]：0 前沿采样 / 1 后沿采样）
    pub const CR1_CPHA: u32 = 1 << 2;
    /// 从机选择配置 SSM（CR1[1]，主机置 1：CS 引脚输出 SSI 电平）
    pub const CR1_SSM: u32 = 1 << 1;
    /// 主机模式 MSTR（CR1[0]）
    pub const CR1_MSTR: u32 = 1 << 0;

    // ---- CR2 (0x04)：复位值 0x0000_0000 ----
    /// DMA 发送使能（CR2[2]）
    pub const CR2_DMATX: u32 = 1 << 2;
    /// DMA 接收使能（CR2[1]）
    pub const CR2_DMARX: u32 = 1 << 1;
    /// 模块使能（CR2[0]）——EN=1 期间 CR1 不可写
    pub const CR2_EN: u32 = 1 << 0;

    // ---- CR3 (0x08) ----
    /// 单线半双工发送方向（CR3[0]：1 仅发送 / 0 仅接收）
    pub const CR3_HDOE: u32 = 1 << 0;

    // ---- SSI (0x0C)：复位值 0x0000_0001 ----
    /// 从机选择输出电平（SSI[0]，主机 SSM=1 时 0=CS 输出低 / 1=输出高）
    pub const SSI_HIGH: u32 = 1;

    // ---- ISR (0x14)：复位值 0x0000_0001 ----
    /// 从机选择信号电平状态（ISR[9]）
    pub const ISR_SSLVL: u32 = 1 << 9;
    /// 总线忙（ISR[8]：移位进行中；发送完成 = TXE=1 且 BUSY=0）
    pub const ISR_BUSY: u32 = 1 << 8;
    /// 模式错误（ISR[7]：主机 CS 输入被拉低）
    pub const ISR_MODF: u32 = 1 << 7;
    /// 从机选择错误（ISR[6]）
    pub const ISR_SSERR: u32 = 1 << 6;
    /// 接收上溢（ISR[5]）
    pub const ISR_OV: u32 = 1 << 5;
    /// 发送下溢（ISR[4]，从机模式）
    pub const ISR_UD: u32 = 1 << 4;
    /// 从机选择上升沿（ISR[3]）
    pub const ISR_SSR: u32 = 1 << 3;
    /// 从机选择下降沿（ISR[2]）
    pub const ISR_SSF: u32 = 1 << 2;
    /// 接收缓冲非空（ISR[1]：读 DR 清除）
    pub const ISR_RXNE: u32 = 1 << 1;
    /// 发送缓冲空（ISR[0]）
    pub const ISR_TXE: u32 = 1 << 0;

    // ---- ICR (0x18)：R1W0（写 0 清除目标位，写 1 无动作）----
    /// 写入值：除目标位为 0 外其余全 1。例：FLUSH=…1110（仅 bit0 为 0）
    pub const ICR_CLEAR_MODF: u32 = !(1 << 7);
    pub const ICR_CLEAR_OV: u32 = !(1 << 5);
    /// 清空发送缓冲与移位寄存器（ICR[0] 写 0）
    pub const ICR_FLUSH: u32 = !1;
}

/// I2C 主机位定义（§23.8，UM V1.4）
pub mod i2c {
    // ---- MCR0 (0x10)：复位值 0x0000_0000 ----
    /// 主机使能（MCR0[0]；置 1 后 MCCR 禁改）
    pub const MCR0_MEN: u32 = 1 << 0;
    /// 主机软复位（MCR0[1]）
    pub const MCR0_RESET: u32 = 1 << 1;
    /// DeepSleep 保持运行（MCR0[2]）
    pub const MCR0_SLEEPEN: u32 = 1 << 2;
    /// 复位发送 FIFO（MCR0[8]，写 1 生效）
    pub const MCR0_TXFIFORST: u32 = 1 << 8;
    /// 复位接收 FIFO（MCR0[9]，写 1 生效）
    pub const MCR0_RXFIFORST: u32 = 1 << 9;
    /// 功能时钟源（MCR0[7:6]）：00=PCLK（默认）
    pub const MCR0_CLKSRC_PCLK: u32 = 0;

    // ---- MCR2 (0x24)：复位值 0x0000_0000 ----
    /// TCLKD 分频（MCR2[2:0]）：TCLKD = TCLK / 2^PRESCALE
    pub const MCR2_PRESCALE_POS: u32 = 0;
    pub const MCR2_PRESCALE_MASK: u32 = 0x7;
    /// 自动 STOP（MCR2[8]：发送 FIFO 空后自动产生 STOP）
    pub const MCR2_AUTOSTOP: u32 = 1 << 8;
    /// 忽略 NACK（MCR2[9]）
    pub const MCR2_IGNACK: u32 = 1 << 9;
    /// SCL/SDA 引脚配置（MCR2[26:24]）：000=开漏输入输出
    pub const MCR2_PINCFG_OD_IO: u32 = 0;

    // ---- MCR3 (0x28) ----
    /// SCL 数字滤波（MCR3[19:16]，单位 TCLKD）
    pub const MCR3_FLTSCL_POS: u32 = 16;
    /// SDA 数字滤波（MCR3[27:24]，单位 TCLKD）
    pub const MCR3_FLTSDA_POS: u32 = 24;

    // ---- MISR (0x14)：复位值 0x0000_0001 ----
    /// 发送 FIFO 空（MISR[0]，可写下一字节/命令）
    pub const MISR_TXE: u32 = 1 << 0;
    /// 接收 FIFO 非空（MISR[1]，可读 MRDR）
    pub const MISR_RXNE: u32 = 1 << 1;
    /// 数据包完成（MISR[8]）
    pub const MISR_PACKET: u32 = 1 << 8;
    /// STOP 信号产生（MISR[9]）
    pub const MISR_STOP: u32 = 1 << 9;
    /// NACK（MISR[10]：地址或数据被拒答）
    pub const MISR_NACK: u32 = 1 << 10;
    /// 多主机仲裁丢失（MISR[11]）
    pub const MISR_ARBI: u32 = 1 << 11;
    /// FIFO 出错（MISR[12]）
    pub const MISR_FIFO: u32 = 1 << 12;
    /// 引脚低电平超时（MISR[13]）
    pub const MISR_PINLOW: u32 = 1 << 13;
    /// 接收数据匹配（MISR[14]）
    pub const MISR_MATCH: u32 = 1 << 14;
    /// 主机忙（MISR[24]）
    pub const MISR_MSTBUSY: u32 = 1 << 24;
    /// 总线忙（MISR[25]，任一主机持有总线）
    pub const MISR_BUSBUSY: u32 = 1 << 25;

    // ---- MTDR (0x60)：CMD[10:8] + DATA[7:0]，全 WO ----
    /// CMD=000：发送一个字节（期待 ACK）
    pub const CMD_TX: u32 = 0;
    /// CMD=001：接收 DATA+1 个字节存入 MRDR
    pub const CMD_RXCOUNT: u32 = 1;
    /// CMD=010：产生 STOP
    pub const CMD_STOP: u32 = 2;
    /// CMD=100：(Repeated)START + 发送一个字节（期待 ACK）
    pub const CMD_START_ACK: u32 = 4;
    /// CMD=101：(Repeated)START + 发送一个字节（期待 NACK）
    pub const CMD_START_NACK: u32 = 5;
    pub const MTDR_CMD_POS: u32 = 8;

    // ---- MRDR (0x70)：复位值 0x0000_4000 ----
    /// 接收 FIFO 空（MRDR[14]）
    pub const MRDR_EMPTY: u32 = 1 << 14;
    pub const MRDR_DATA_MASK: u32 = 0xFF;

    // ---- MCCR (0x48)：复位值 0x0000_0000；MEN=1 后禁改 ----
    pub const MCCR_DATAVD_POS: u32 = 24;
    pub const MCCR_SETHOLD_POS: u32 = 16;
    pub const MCCR_CLKHI_POS: u32 = 8;
    pub const MCCR_CLKLO_POS: u32 = 0;
}

/// BTIM 基础定时器位定义（§14.10，UM V1.4）
pub mod btim {
    // ---- CR1 (0x00)：复位值 0x0000_0000 ----
    /// 计数器使能（CR1[0]）
    pub const CR1_EN: u32 = 1 << 0;
    /// 更新事件禁止（CR1[1]：1=不产生更新事件/中断）
    pub const CR1_UDIS: u32 = 1 << 1;
    /// 更新请求源（CR1[2]：1=仅计数器溢出产生更新）
    pub const CR1_URS: u32 = 1 << 2;
    /// 单次模式（CR1[3]：发生更新事件后自动停止）
    pub const CR1_ONESHOT: u32 = 1 << 3;
    /// UIF 标志映射到 ARR[15]（CR1[11]）
    pub const CR1_UIFREMAP: u32 = 1 << 11;
    /// TOG 引脚输出使能（CR1[15]）
    pub const CR1_TOGEN: u32 = 1 << 15;

    // ---- DIER (0x0C) ----
    /// 更新中断使能（DIER[0]）
    pub const DIER_UIE: u32 = 1 << 0;
    /// 触发中断使能（DIER[6]）
    pub const DIER_TIE: u32 = 1 << 6;
    /// 更新 DMA 使能（DIER[8]）
    pub const DIER_UDE: u32 = 1 << 8;

    // ---- ISR (0x10) ----
    /// 更新中断标志（ISR[0]）
    pub const ISR_UIF: u32 = 1 << 0;
    /// 触发中断标志（ISR[6]）
    pub const ISR_TIF: u32 = 1 << 6;

    // ---- EGR (0x14)：写 1 触发，硬件自动清零 ----
    /// 软件产生更新事件（重新装载 PSC/ARR、清零计数器）
    pub const EGR_UG: u32 = 1 << 0;

    // ---- ICR (0x18)：R1W0（写 0 清除），复位 0x41 ----
    /// 清除更新标志的写入值（仅 bit0 为 0）
    pub const ICR_CLEAR_UIF: u32 = !1;
}

/// ADC 位定义（§25.12，UM V1.4）
pub mod adc {
    // ---- CR (0x00)：复位值 0x0000_0100（bit8 保留位为 1，写入时须保留）----
    /// 复位基值（含 bit8 保留位）
    pub const CR_RESET: u32 = 0x100;
    /// ADC 使能（CR[0]）
    pub const CR_EN: u32 = 1 << 0;
    /// 持续转换模式（CR[1]；0=单次）
    pub const CR_CONT: u32 = 1 << 1;
    /// ADCCLK 分频（CR[3:2]）：00=PCLK/1、01=/2、10=/4、11=/8
    pub const CR_CLK_POS: u32 = 2;
    pub const CR_CLK_MASK: u32 = 0x3 << CR_CLK_POS;
    /// 序列通道数量（CR[6:4]）：ENS = N-1（转换 SQRCH0..SQRCH{ENS}）
    pub const CR_ENS_POS: u32 = 4;
    pub const CR_ENS_MASK: u32 = 0x7 << CR_ENS_POS;
    /// 从机同步模式（CR[7]）
    pub const CR_SLAVE: u32 = 1 << 7;

    // ---- START (0x04) ----
    /// 启动转换（写 1）；写 0 停止并复位序列
    pub const START: u32 = 1 << 0;

    // ---- SQRCFR (0x20) / SAMPLE (0x18)：SQRCHy 各 4bit（SQRCH0 在 bit0）----
    pub const SQRCH0_POS: u32 = 0; // SQRCHy 位置 = y*4
    pub const SQRCH_MASK: u32 = 0xF;

    // ---- IER (0x70)：与 ISR 同位序 ----
    /// 转换完成中断使能（IER[0]）
    pub const IER_EOC: u32 = 1 << 0;
    /// 序列完成中断使能（IER[1]）
    pub const IER_EOS: u32 = 1 << 1;
    /// EOC 触发 DMA（IER[5]）
    pub const IER_DMAEOC: u32 = 1 << 5;
    /// EOS 触发 DMA（IER[6]）
    pub const IER_DMAEOS: u32 = 1 << 6;

    // ---- ISR (0x78) / ICR (0x74) ----
    /// 单通道转换完成（ISR[0]）
    pub const ISR_EOC: u32 = 1 << 0;
    /// 序列转换完成（ISR[1]）
    pub const ISR_EOS: u32 = 1 << 1;
    /// 看门狗低阈值（ISR[3]）/高阈值（ISR[4]）
    pub const ISR_AWDL: u32 = 1 << 3;
    pub const ISR_AWDH: u32 = 1 << 4;
    /// ICR 写入值：清除 EOC+EOS（R1W0，其余位写 1 无动作）
    pub const ICR_CLEAR_EOC_EOS: u32 = !(ISR_EOC | ISR_EOS);

    // ---- 通道号（表 25-4，ADC1 视角）----
    /// 内置温度传感器（仅 ADC1；需 BGR_CR.TSEN）
    pub const CH_TS: u32 = 0xE;
    /// 1.2V 内核基准（仅 ADC1；需 BGR_CR.BGREN）
    pub const CH_BGR: u32 = 0xF;
}

/// BGR 位定义（§25.12.19）
pub mod bgr {
    /// BGR 基准使能（BGR_CR[0]；IN15 通道前置条件）
    pub const CR_BGREN: u32 = 1 << 0;
    /// 温度传感器使能（BGR_CR[1]；使能后约 30µs 才能转换 TS 通道）
    pub const CR_TSEN: u32 = 1 << 1;
}

/// LPTIM 位定义（§15）
pub mod lptim {
    // ---- ISR (0x00) / IER (0x08) / ICR (0x04)，位序共用 ----
    /// 计数值与 CMP 相等
    pub const CMPM: u32 = 1 << 0;
    /// 计数值与 ARR 相等（周期中断）
    pub const ARRM: u32 = 1 << 1;
    /// 有效触发沿
    pub const EXTTRIG: u32 = 1 << 2;
    /// CMP 写入已生效
    pub const CMPOK: u32 = 1 << 3;
    /// ARR 写入已生效
    pub const ARROK: u32 = 1 << 4;
    /// 上溢（编码模式）
    pub const UP: u32 = 1 << 5;
    /// 下溢（编码模式）
    pub const DOWN: u32 = 1 << 6;
    /// 全部标志位掩码
    pub const FLAGS_MASK: u32 = 0x7F;

    // ---- CFGR (0x0C) ----
    /// 计数时钟源：00/01=PCLK、10=LSE、11=LSI
    pub const CFGR_ICLKSRC_POS: u32 = 25;
    pub const CFGR_ICLKSRC_LSE: u32 = 2 << CFGR_ICLKSRC_POS;
    pub const CFGR_ICLKSRC_LSI: u32 = 3 << CFGR_ICLKSRC_POS;
    /// 0=定时模式（ICLK 分频驱动）、1=计数模式（CH1 边沿驱动）
    pub const CFGR_COUNTMD: u32 = 1 << 23;
    /// 0=寄存器立即更新、1=周期结束时更新
    pub const CFGR_PRELOAD: u32 = 1 << 22;
    /// PWM 输出极性（0=正向 CNT≥CMP 高）
    pub const CFGR_WAVPOL: u32 = 1 << 21;
    /// 波形模式（0=单脉冲随启动方式、1=单次置位）
    pub const CFGR_WAVE: u32 = 1 << 20;
    /// 触发事件是否清零重计（0=忽略、1=重启）
    pub const CFGR_TIMOUT: u32 = 1 << 19;
    /// 00=软件启动、01/10/11=硬件触发沿
    pub const CFGR_TRIGEN_POS: u32 = 17;
    /// 触发源：0000=ETR 引脚、0110..1001=VC1..4 输出、0001..0101=RTC 事件
    pub const CFGR_TRIGSEL_POS: u32 = 13;
    /// 预分频 ICLK/2^n（n=0..7 → DIV1..DIV128）
    pub const CFGR_PRS_POS: u32 = 9;

    // ---- CR0 (0x10)，复位值 0x18 ----
    /// 内核使能
    pub const CR0_EN: u32 = 1 << 0;
    /// 单次启动（一个计数周期）
    pub const CR0_SNGSTART: u32 = 1 << 1;
    /// 连续启动
    pub const CR0_CNTSTART: u32 = 1 << 2;
    /// 同步复位计数（W0：写 0 复位，完成自动回 1）
    pub const CR0_SRST: u32 = 1 << 3;
    /// 异步复位计数（W0：读 1 后可写 0，完成自动回 1）
    pub const CR0_ARST: u32 = 1 << 4;

    // ---- CR1 (0x20) ----
    /// CH1 输入源：000=CH1 引脚、001..100=VC1..4_OUT
    pub const CR1_CH1SRC_POS: u32 = 0;
    /// CH2 输入源（同上）
    pub const CR1_CH2SRC_POS: u32 = 3;
}

/// HALLTIM 位定义（§18.8，UM V1.4）
pub mod halltim {
    // ---- CR (0x00)：复位 0x0000_0004 ----
    /// 模块使能（CR[0]）
    pub const CR_EN: u32 = 1 << 0;
    /// 第一级滤波器（5/7 低通）使能（CR[1]）
    pub const CR_FLT1EN: u32 = 1 << 1;
    /// 计数时钟分频（CR[3:2]）：00=PCLK、01=/2、10=/4、11=/8
    pub const CR_DIV_POS: u32 = 2;
    pub const CR_DIV_MASK: u32 = 0x3 << CR_DIV_POS;
    /// 触发输出选择（CR[6:4]）
    pub const CR_MMS_POS: u32 = 4;
    pub const CR_MMS_MASK: u32 = 0x7 << CR_MMS_POS;
    /// 第二级数字滤波宽度（CR[22:8]）：滤除 < FLT2LEN×T_HALLCLK 的脉冲
    pub const CR_FLT2LEN_POS: u32 = 8;
    pub const CR_FLT2LEN_MASK: u32 = 0x7FFF << CR_FLT2LEN_POS;
    /// 软件捕获触发（CR[23]，R0W1：写 1 生成一次捕获事件）
    pub const CR_SOFTCAP: u32 = 1 << 23;

    // ---- DIER (0x04) ----
    pub const DIER_CAPIE: u32 = 1 << 0;
    pub const DIER_OVIE: u32 = 1 << 1;
    pub const DIER_MATCHIE: u32 = 1 << 2;
    pub const DIER_CAPDMA: u32 = 1 << 4;
    pub const DIER_OVDMA: u32 = 1 << 5;
    pub const DIER_MATCHDMA: u32 = 1 << 6;
    pub const DIER_SOFTDMA: u32 = 1 << 7;

    // ---- ISR (0x18) / ICR (0x1C)：位序共用 ----
    /// 输入捕获标志（ISR[0] / ICR[0] R1W0）
    pub const CAPF: u32 = 1 << 0;
    /// 计数器溢出标志（ISR[1] / ICR[1] R1W0）
    pub const OVF: u32 = 1 << 1;
    /// 比较匹配标志（ISR[2] / ICR[2] R1W0）
    pub const MATCHF: u32 = 1 << 2;
    /// 全部标志位掩码
    pub const FLAGS_MASK: u32 = 0x7;

    // ---- STATE (0x20) ----
    /// CH1 滤波后电平 / CH2 / CH3
    pub const STATE_CH1F: u32 = 1 << 0;
    pub const STATE_CH2F: u32 = 1 << 1;
    pub const STATE_CH3F: u32 = 1 << 2;
    /// CH1 原始电平 / CH2 / CH3
    pub const STATE_CH1S: u32 = 1 << 4;
    pub const STATE_CH2S: u32 = 1 << 5;
    pub const STATE_CH3S: u32 = 1 << 6;

    // ---- CNT/ARR/WIDTH/CCR：24bit ----
    pub const DATA_MASK: u32 = 0x00FF_FFFF;
}

/// VC 电压比较器位定义（§27.7，UM V1.4）
pub mod vc {
    // ---- VCxy_REF (0x00) ----
    /// 分压器输出配置（REF[3:0]）：Vout = Vin × (1+DIV)/8
    pub const REF_DIV_MASK: u32 = 0xF;
    /// 分压器使能（REF[4]）
    pub const REF_EN: u32 = 1 << 4;
    /// 分压器输入选择（REF[5]）：0=VDDA、1=Vcore(≈1.6V)
    pub const REF_VIN_VCORE: u32 = 1 << 5;

    // ---- CR0 (0x04) ----
    /// 比较器使能（CR0[0]）
    pub const CR0_EN: u32 = 1 << 0;
    /// 高速响应（CR0[1]；1=高速，硬件自动开启 BGR）
    pub const CR0_RESP: u32 = 1 << 1;
    /// 迟滞使能（CR0[3]；约 20mV）
    pub const CR0_HYS: u32 = 1 << 3;
    /// 中断使能（CR0[5]）
    pub const CR0_IE: u32 = 1 << 5;
    /// 输出极性（CR0[6]）：0=正端>负端输出高、1=反相
    pub const CR0_POL: u32 = 1 << 6;
    /// 窗口比较模式（CR0[7]）
    pub const CR0_WINDOW: u32 = 1 << 7;
    /// 负端输入选择（CR0[9:8]）
    pub const CR0_INN_POS: u32 = 8;
    pub const CR0_INN_MASK: u32 = 0x3 << CR0_INN_POS;
    /// 正端输入选择（CR0[13:12]）
    pub const CR0_INP_POS: u32 = 12;
    pub const CR0_INP_MASK: u32 = 0x3 << CR0_INP_POS;

    // ---- CR1 (0x08)：复位 0x0000_0010 ----
    /// 滤波时间（CR1[3:0]）
    pub const CR1_FLTTIME_MASK: u32 = 0xF;
    /// 滤波时钟（CR1[4]）：0=LSI、1=PCLK（默认 PCLK，DeepSleep 省电）
    pub const CR1_FLTCLK_PCLK: u32 = 1 << 4;
    /// 下降沿中断（CR1[5]）
    pub const CR1_FALLIE: u32 = 1 << 5;
    /// 上升沿中断（CR1[6]）
    pub const CR1_RISEIE: u32 = 1 << 6;
    /// 高电平中断（CR1[7]）
    pub const CR1_HIGHIE: u32 = 1 << 7;
    /// BLANK 窗口时长（CR1[10:8]）
    pub const CR1_BLANKTIME_POS: u32 = 8;
    pub const CR1_BLANKTIME_MASK: u32 = 0x7 << CR1_BLANKTIME_POS;
    /// BLANK 期间输出电平（CR1[11]）
    pub const CR1_BLANKLVL: u32 = 1 << 11;

    // ---- CR2 (0x0C)：BLANK 触发源（每位一个，0=禁用） ----
    /// ATIM TRGO / ATIM TRGO2
    pub const CR2_ATIMTRGO: u32 = 1 << 0;
    pub const CR2_ATIMTRGO2: u32 = 1 << 1;
    /// BTIM1/2/3 TRGO
    pub const CR2_BTIM1TRGO: u32 = 1 << 28;
    pub const CR2_BTIM2TRGO: u32 = 1 << 29;
    pub const CR2_BTIM3TRGO: u32 = 1 << 30;
    /// HALLTIM TRGO
    pub const CR2_HALLTIMTRGO: u32 = 1 << 31;

    // ---- SR (0x10) ----
    /// 中断标志（SR[0]，RW0：写 0 清除）
    pub const SR_INTF: u32 = 1 << 0;
    /// 滤波后输出电平（SR[1]，RO）
    pub const SR_FLTV: u32 = 1 << 1;
}

/// OPA 运算放大器位定义（§29.6，UM V1.4）
pub mod opa {
    // ---- CR (0x00)：复位 0x0000_E000 ----
    /// 使能（CR[0]）
    pub const CR_EN: u32 = 1 << 0;
    /// 工作模式（CR[2:1]）：0x=外部放大、10=内部 PGA、11=跟随器
    pub const CR_MODE_POS: u32 = 1;
    pub const CR_MODE_MASK: u32 = 0x3 << CR_MODE_POS;
    pub const MODE_EXTERNAL: u32 = 0x0 << CR_MODE_POS;
    pub const MODE_PGA: u32 = 0x2 << CR_MODE_POS;
    pub const MODE_FOLLOWER: u32 = 0x3 << CR_MODE_POS;
    /// 正端输入导通（CR[4..7]：INP1..INP4，INP4 = DAC 内部）
    pub const CR_INP1EN: u32 = 1 << 4;
    pub const CR_INP2EN: u32 = 1 << 5;
    pub const CR_INP3EN: u32 = 1 << 6;
    pub const CR_INP4EN: u32 = 1 << 7;
    /// 负端输入导通（CR[8..9]：INN1/INN2）
    pub const CR_INN1EN: u32 = 1 << 8;
    pub const CR_INN2EN: u32 = 1 << 9;
    /// PGA 放大倍数（CR[12:10]）：00=64/32(×2) … 100=64/2(×32)
    pub const CR_AMP_POS: u32 = 10;
    pub const CR_AMP_MASK: u32 = 0x7 << CR_AMP_POS;
    /// 偏置电流（CR[15:13]）：code+1 µA，111=8µA（复位默认）
    pub const CR_BIAS_POS: u32 = 13;
    pub const CR_BIAS_MASK: u32 = 0x7 << CR_BIAS_POS;

    // ---- CAL (0x04) ----
    /// 校准功能使能（CAL[0]）
    pub const CAL_EN: u32 = 1 << 0;
    /// 校准方式（CAL[1]）：0=触发校准、1=电平校准
    pub const CAL_MODE_LEVEL: u32 = 1 << 1;
    /// 电平校准启动（CAL[2]）：1=启动、0=停止
    pub const CAL_START: u32 = 1 << 2;
    /// 软件触发校准（CAL[3]，W1：写 1 触发一次）
    pub const CAL_SOFTTRIG: u32 = 1 << 3;
    /// 校准时长（CAL[7:4]）：8 << code 个 OPACLK 周期（0000=8 … 1111=262144）
    pub const CAL_PERIOD_POS: u32 = 4;
    pub const CAL_PERIOD_MASK: u32 = 0xF << CAL_PERIOD_POS;
    /// 校准进行中（CAL[10]，RO）
    pub const CAL_AZRUN: u32 = 1 << 10;
    /// 校准时钟分频（CAL[13:11]）
    pub const CAL_CLKDIV_POS: u32 = 11;
    pub const CAL_CLKDIV_MASK: u32 = 0x7 << CAL_CLKDIV_POS;
    /// ADC1 / ADC2 启动信号触发校准
    pub const CAL_ADC2START: u32 = 1 << 15;
    pub const CAL_ADC1START: u32 = 1 << 14;
}

/// 出厂校准值地址（§25.8/25.9 表 25-5 及正文）
pub mod calib {
    /// 温度传感器 T0 初始校准温度（8bit，单位 0.5℃，@0x0010_07CD）
    pub const TS_T0_ADDR: usize = 0x0010_07CD;
    /// 温度传感器 Trim 校准值（16bit，VDDA 参考，@0x0010_07CE）
    pub const TS_TRIM_ADDR: usize = 0x0010_07CE;
    /// VDDA 工厂校准值（16bit，mV，@0x0010_07D2；VDD = cal×4095/RESULT0）
    pub const VDDA_ADDR: usize = 0x0010_07D2;
    /// LSI 32.8kHz TRIM 工厂校准值（16bit，@0x0010_07C2；写入 SYSCTRL_LSI.TRIM）
    pub const LSI_TRIM_ADDR: usize = 0x0010_07C2;
}

/// DAC 位定义（§26.10，UM V1.4）
pub mod dac {
    // ---- CR0 (0x00) ----
    /// 通道 1 使能（CR0[0]）
    pub const CR0_EN1: u32 = 1 << 0;
    /// 通道 1 触发使能（CR0[1]；0=写 DHR 后 1 个 APCLK 自动转入 DOR）
    pub const CR0_TEN1: u32 = 1 << 1;
    /// 通道 2 使能（CR0[16]）
    pub const CR0_EN2: u32 = 1 << 16;
    /// 通道 2 触发使能（CR0[17]）
    pub const CR0_TEN2: u32 = 1 << 17;

    // ---- SWTRGR (0x04) ----
    /// 软件触发通道 1（写 1，硬件自动清零）
    pub const SWTRIG1: u32 = 1 << 0;
    /// 软件触发通道 2
    pub const SWTRIG2: u32 = 1 << 1;

    // ---- CR1 (0x34) ----
    /// 通道 1 输出到引脚（内部互联不受影响）
    pub const CR1_OUT1: u32 = 1 << 0;
    /// 通道 2 输出到引脚
    pub const CR1_OUT2: u32 = 1 << 1;

    /// 12 位数据掩码
    pub const DHR12_MASK: u32 = 0xFFF;
}

/// IWDT 位定义（§19.6，UM V1.4）
pub mod iwdt {
    // ---- KR (0x00) 键值 ----
    /// 启动计数器
    pub const KR_START: u32 = 0xCCCC;
    /// 重载计数值（喂狗）
    pub const KR_FEED: u32 = 0xAAAA;
    /// 解锁 CR/ARR/WINR 写保护
    pub const KR_UNLOCK: u32 = 0x5555;
    /// 停止看门狗第一键（随后写 0xA5A5）
    pub const KR_STOP1: u32 = 0x5A5A;
    /// 停止看门狗第二键
    pub const KR_STOP2: u32 = 0xA5A5;

    // ---- CR (0x04)：复位值 0x0000_0000 ----
    /// 计数时钟分频（CR[2:0]）：000=/4 … 111=/512（RC10K 基准）
    pub const CR_PRS_POS: u32 = 0;
    pub const CR_PRS_MASK: u32 = 0x7;
    /// 溢出动作（CR[3]）：0=复位（默认）/ 1=中断
    pub const CR_ACTION_RESET: u32 = 0;
    /// 溢出动作 = 中断（CR[3] 置 1，配合 CR_IE 使用）
    pub const CR_ACTION_IRQ: u32 = 1 << 3;
    /// 看门狗中断使能（CR[4]）
    pub const CR_IE: u32 = 1 << 4;
    /// DeepSleep 自动暂停（CR[5]）
    pub const CR_PAUSE: u32 = 1 << 5;

    // ---- SR (0x0C) ----
    /// ARR 更新完成标志（SR[0]？以手册 §19.6.6 为准：CRF）
    pub const SR_CRF: u32 = 1 << 0;

    /// RC10K 标称频率（Hz，±负载漂移；超时计算基准）
    pub const RC10K_HZ: u32 = 10_000;
    /// ARR 最大值（12bit）
    pub const ARR_MAX: u32 = 0xFFF;
}

/// GTIM 位定义（§16.10，UM V1.4；STM32 风格）
pub mod gtim {
    // ---- CR1 (0x00) ----
    /// 计数器使能（CR1[0]）
    pub const CR1_EN: u32 = 1 << 0;
    /// 更新事件禁止（CR1[1]）
    pub const CR1_UDIS: u32 = 1 << 1;
    /// 单次模式（CR1[3]）
    pub const CR1_OPM: u32 = 1 << 3;
    /// ARR 预装载使能（CR1[7]）
    pub const CR1_ARPE: u32 = 1 << 7;

    // ---- EGR (0x14) ----
    /// 软件产生更新事件（重装影子寄存器）
    pub const EGR_UG: u32 = 1 << 0;

    // ---- CCMR1 (0x18) 输出比较模式（通道 1/2，STM32 风格）----
    /// 输出比较 1 模式（CCMR1[6:4]）
    pub const OC1M_POS: u32 = 4;
    pub const OC1M_MASK: u32 = 0x7 << OC1M_POS;
    /// 输出比较 1 模式 PWM1（CNT<CCR 时有效电平）
    pub const OC1M_PWM1: u32 = 6 << OC1M_POS;
    /// 输出比较 1 模式 PWM2
    pub const OC1M_PWM2: u32 = 7 << OC1M_POS;
    /// 输出比较 1 预装载使能（CCMR1[3]）
    pub const OC1PE: u32 = 1 << 3;
    /// 通道 1 配置为输出（CCMR1[1:0]=00）
    pub const CC1S_OUTPUT: u32 = 0;
    /// 输出比较 2 模式 PWM1（CCMR1[14:12]）
    pub const OC2M_POS: u32 = 12;
    pub const OC2M_MASK: u32 = 0x7 << OC2M_POS;
    pub const OC2M_PWM1: u32 = 6 << OC2M_POS;
    /// 输出比较 2 预装载使能（CCMR1[11]）
    pub const OC2PE: u32 = 1 << 11;
    /// 通道 2 配置为输出（CCMR1[9:8]=00）
    pub const CC2S_OUTPUT: u32 = 0;
    /// 输出比较 3 模式 PWM1（CCMR2[6:4]）
    pub const OC3M_POS: u32 = 4;
    pub const OC3M_MASK: u32 = 0x7 << OC3M_POS;
    pub const OC3M_PWM1: u32 = 6 << OC3M_POS;
    /// 输出比较 3 预装载使能（CCMR2[3]）
    pub const OC3PE: u32 = 1 << 3;
    /// 输出比较 4 模式 PWM1（CCMR2[14:12]）
    pub const OC4M_POS: u32 = 12;
    pub const OC4M_MASK: u32 = 0x7 << OC4M_POS;
    pub const OC4M_PWM1: u32 = 6 << OC4M_POS;
    /// 输出比较 4 预装载使能（CCMR2[11]）
    pub const OC4PE: u32 = 1 << 11;

    // ---- CCER (0x20) ----
    /// 捕获/比较 1 输出使能（CCER[0]）
    pub const CC1E: u32 = 1 << 0;
    /// 捕获/比较 1 输出极性（CCER[1]：0=高有效）
    pub const CC1P: u32 = 1 << 1;
    pub const CC2E: u32 = 1 << 4;
    pub const CC2P: u32 = 1 << 5;
    pub const CC3E: u32 = 1 << 8;
    pub const CC3P: u32 = 1 << 9;
    pub const CC4E: u32 = 1 << 12;
    pub const CC4P: u32 = 1 << 13;

    // ---- ISR (0x10) ----
    /// 更新中断标志（ISR[0]）
    pub const ISR_UIF: u32 = 1 << 0;
}

/// ATIM 位定义（§17.10）
pub mod atim {
    // ---- CR1 (0x00)：复位 0x0000_0000 ----
    pub const CR1_CEN: u32 = 1 << 0;
    pub const CR1_UDIS: u32 = 1 << 1;
    pub const CR1_URS: u32 = 1 << 2;
    pub const CR1_OPM: u32 = 1 << 3;
    /// ARR 预装载使能（CR1[7]）
    pub const CR1_ARPE: u32 = 1 << 7;
    /// 时钟分频 CKD[9:8]：f_DTS = f_PCLK / (1|2|4)
    pub const CR1_CKD_POS: u32 = 8;
    pub const CR1_CKD_MASK: u32 = 0x3 << CR1_CKD_POS;

    // ---- EGR (0x14) ----
    /// 更新事件生成（EGR[0]，硬件自动清零）
    pub const EGR_UG: u32 = 1 << 0;

    // ---- CCMR 比较模式（0x18/0x1C/0x50；OCy 低通道在低位半区、y+1 在高位半区）----
    pub const OC1M_POS: u32 = 4;
    pub const OC1M_MASK: u32 = 0x7 << OC1M_POS;
    /// 模式 1 的第 4 位（OC1MH，PWM1=0b0110 → M=110 且 MH=0）
    pub const OC1MH: u32 = 1 << 16;
    pub const OC1M_PWM1: u32 = 6 << OC1M_POS;
    pub const OC1PE: u32 = 1 << 3;
    pub const OC2M_POS: u32 = 12;
    pub const OC2M_MASK: u32 = 0x7 << OC2M_POS;
    pub const OC2MH: u32 = 1 << 24;
    pub const OC2M_PWM1: u32 = 6 << OC2M_POS;
    pub const OC2PE: u32 = 1 << 11;
    // CCMR2（OC3/OC4）与 CCMR3（OC5/OC6）布局完全同 CCMR1

    // ---- CCER (0x20) ----
    pub const CC1E: u32 = 1 << 0;
    pub const CC1P: u32 = 1 << 1;
    /// 通道 1 互补输出（CH1N）使能（CCER[2]）
    pub const CC1NE: u32 = 1 << 2;
    /// 通道 1 互补输出极性（CCER[3]：0=高有效）
    pub const CC1NP: u32 = 1 << 3;
    pub const CC2E: u32 = 1 << 4;
    pub const CC2P: u32 = 1 << 5;
    pub const CC2NE: u32 = 1 << 6;
    pub const CC2NP: u32 = 1 << 7;
    pub const CC3E: u32 = 1 << 8;
    pub const CC3P: u32 = 1 << 9;
    pub const CC3NE: u32 = 1 << 10;
    pub const CC3NP: u32 = 1 << 11;
    pub const CC4E: u32 = 1 << 12;
    pub const CC4P: u32 = 1 << 13;
    pub const CC4NE: u32 = 1 << 14;
    pub const CC4NP: u32 = 1 << 15;
    pub const CC5E: u32 = 1 << 16;
    pub const CC5P: u32 = 1 << 17;
    pub const CC6E: u32 = 1 << 20;
    pub const CC6P: u32 = 1 << 21;

    // ---- BDTR (0x44)：复位 0x0000_0000 ----
    /// 死区时间发生器 DTG[7:0]（编码见 [`encode_dtg`] 语义）
    pub const BDTR_DTG_MASK: u32 = 0xFF;
    /// 主输出使能（BDTR[15]）：ATIM 的 PWM 输出总开关，必须置 1
    pub const BDTR_MOE: u32 = 1 << 15;
    /// 自动输出使能（BDTR[14]）：刹车恢复后下一更新事件自动置 MOE
    pub const BDTR_AOE: u32 = 1 << 14;
    /// 刹车 BRK 极性（BDTR[13]：0=低电平有效）
    pub const BDTR_BKP: u32 = 1 << 13;
    /// 刹车使能（BDTR[12]）
    pub const BDTR_BKE: u32 = 1 << 12;
    /// 运行态"关闭时输出"（BDTR[11]）
    pub const BDTR_OSSR: u32 = 1 << 11;
    /// 刹车/空闲态"关闭时输出"（BDTR[10]）
    pub const BDTR_OSSI: u32 = 1 << 10;
    /// 写保护级别 LOCK[9:8]（00=无保护）
    pub const BDTR_LOCK_POS: u32 = 8;
    /// 刹车 BRK 滤波器 BKF[19:16]
    pub const BDTR_BKF_POS: u32 = 16;
    /// 刹车 2 使能（BDTR[24]）
    pub const BDTR_BK2E: u32 = 1 << 24;
    /// 刹车 2 极性（BDTR[25]：0=低电平有效）
    pub const BDTR_BK2P: u32 = 1 << 25;

    // ---- ISR (0x10) / ICR (0x70)：R1W0，写 0 清除 ----
    pub const ISR_UIF: u32 = 1 << 0;
    /// 清除更新标志的写入值（仅 bit0 为 0）
    pub const ICR_CLEAR_UIF: u32 = !1;
}

/// WWDT 位定义（§20.6）
pub mod wwdt {
    // ---- CR0 (0x00)：复位 0x0000_007F ----
    /// WWDT 使能（CR0[7]，RW1：置 1 后不可关闭，仅复位可停）
    pub const CR0_EN: u32 = 1 << 7;
    /// 计数值域 WCNT[6:0]（读=当前值、写=刷新/喂狗；写 ≤0x3F 立即复位）
    pub const CR0_WCNT_MASK: u32 = 0x7F;

    // ---- CR1 (0x04)：复位 0x0000_007F ----
    /// 预溢出中断使能（CR1[10]：置 1 后不可清零）
    pub const CR1_IE: u32 = 1 << 10;
    /// 预分频 PRS[9:7]：计数时钟 = PCLK / 4096 / 2^PRS
    pub const CR1_PRS_POS: u32 = 7;
    pub const CR1_PRS_MASK: u32 = 0x7 << CR1_PRS_POS;
    /// 窗口值域 WINR[6:0]（必须 < 计数器初值；喂狗下限）
    pub const CR1_WINR_MASK: u32 = 0x7F;

    // ---- SR (0x08) ----
    /// 预溢出标志（SR[0]，写 0 清除；计到 0x40 时置位）
    pub const SR_POV: u32 = 1 << 0;
}

/// FLASH 控制器位定义（§7.10）
pub mod flash {
    /// 写保护键：CR1 / CR2 / PAGELOCK 高 16 位必须为 0x5A5A
    pub const KEY: u32 = 0x5A5A << 16;

    // ---- CR2 (0x04)：复位值 0x0000_0000 ----
    /// 取指周期等待 HCLK 周期个数（CR2[2:0]）：
    /// 000=≤24MHz、001=≤48MHz、010=≤72MHz、011=≤96MHz（表 7-1）
    pub const CR2_WAIT_MASK: u32 = 0x7;
    /// 预取指使能（CR2[3]；WAIT≠0 时建议使能）
    pub const CR2_FETCH: u32 = 1 << 3;
    /// 缓存使能（CR2[4]；WAIT≠0 时建议使能）
    pub const CR2_CACHE: u32 = 1 << 4;
    /// 写 1 再写 0 清除 CACHE 缓存（CR2[5]）
    pub const CR2_CACHEINVALID: u32 = 1 << 5;

    // ---- CR1 (0x00)：KEY 保护，MODE 在 [1:0] ----
    /// 操作模式（CR1[1:0]）：0=读、1=写（编程）、2=页擦除、3=片擦除
    pub const CR1_MODE_MASK: u32 = 0x3;
    pub const MODE_READ: u32 = 0;
    pub const MODE_PROGRAM: u32 = 1;
    pub const MODE_PAGE_ERASE: u32 = 2;
    /// 读保护等级（CR1[6:5]，RO）
    pub const CR1_SECURITY_POS: u32 = 5;
    pub const CR1_SECURITY_MASK: u32 = 0x3 << CR1_SECURITY_POS;

    // ---- ISR (0x24) ----
    /// 擦写地址落在 PC 所在页（ISR[0]）
    pub const ISR_PC: u32 = 1 << 0;
    /// 擦写地址落在被锁定的页（ISR[1]）
    pub const ISR_PAGELOCK: u32 = 1 << 1;
    /// 擦写区域包含安全运行库（ISR[2]）
    pub const ISR_SDKERR: u32 = 1 << 2;
    /// 擦写时 CACHE/FETCH 未关闭（ISR[3]）
    pub const ISR_CACHEON: u32 = 1 << 3;
    /// 目标地址不全是 0xFF（ISR[4]）
    pub const ISR_PROG: u32 = 1 << 4;
    /// 擦写忙（ISR[5]）
    pub const ISR_BUSY: u32 = 1 << 5;
    /// 全部错误标志（用于 ICR 写 !mask 清除）
    pub const ISR_ERRORS: u32 = ISR_PC | ISR_PAGELOCK | ISR_SDKERR | ISR_CACHEON | ISR_PROG;
}

/// RTC 寄存器组（RTC_BASE = 0x4000_4400，表 13-6）
///
/// 注意：除 `key` / `icr` 外，上电后全部**锁定**，需先向 KEY 写 `0xCA`、`0x53`。
#[allow(dead_code)]
#[repr(C)]
pub struct Rtc {
    /// 0x00 键值寄存器（WO）：0xCA→0x53 解锁，0xCA→任意非 0x53 锁定
    pub key: Wo<u32>,
    /// 0x04 控制 0：START[7] / RTC1HZ[6:5] / H24[3] / INTERVAL[2:0]
    pub cr0: Rw<u32>,
    /// 0x08 控制 1：SOURCE[10:8] / WAIT[2]（RO）
    pub cr1: Rw<u32>,
    /// 0x0C 控制 2：ALARMBEN[10]/ALARMAEN[9]/AWTEN[7]/TAMPEN[6]/RTCOUT[5:4]/...
    pub cr2: Rw<u32>,
    /// 0x10 时钟误差补偿（EN[15]/SIGN[14]/PERIOD[13:12]/COMP[11:0]）
    pub compcfr1: Rw<u32>,
    /// 0x14 日期（WEEK[26:24]/YEAR[23:16]/MONTH[12:8]/DAY[5:0]，BCD）
    pub date: Rw<u32>,
    /// 0x18 时间（HOUR[21:16]/MINUTE[14:8]/SECOND[6:0]，BCD）
    pub time: Rw<u32>,
    /// 0x1C 闹钟 A
    pub alarma: Rw<u32>,
    /// 0x20 闹钟 B
    pub alarmb: Rw<u32>,
    /// 0x24 时间戳日期（RO）
    pub tampdate: Ro<u32>,
    /// 0x28 时间戳时间（RO）
    pub tamptime: Ro<u32>,
    /// 0x2C 唤醒定时器重载值
    pub awtarr: Rw<u32>,
    /// 0x30 中断使能
    pub ier: Rw<u32>,
    /// 0x34 中断标志（RO）
    pub isr: Ro<u32>,
    /// 0x38 中断标志清除（**R1W0：写 0 清除**，复位 0x7F）
    pub icr: Rw<u32>,
    /// 0x3C 唤醒定时器当前计数值（RO）
    pub awtcnt: Ro<u32>,
    /// 0x40 预分频（PSC1[27:20] / PSC2[19:0]，复位 0x0000_3FFF）
    pub psc: Rw<u32>,
    /// 0x44 亚秒计数值（RO）
    pub sscnt: Ro<u32>,
}

/// RTC 位定义（§13.5 + 表 13-5）
pub mod rtc {
    // ---- KEY ----
    /// 解锁第一字节
    pub const KEY_UNLOCK1: u32 = 0xCA;
    /// 解锁第二字节
    pub const KEY_UNLOCK2: u32 = 0x53;

    // ---- CR0 (0x04) ----
    /// 计时器启动（CR0[7]）
    pub const CR0_START: u32 = 1 << 7;
    /// RTC_1Hz 输出使能（CR0[6:5]，写 01）
    pub const CR0_RTC1HZ: u32 = 1 << 5;
    /// 24 小时制（CR0[3]）：0=12 小时制
    pub const CR0_H24: u32 = 1 << 3;
    /// 周期中断间隔（CR0[2:0]，表 13-5）：0=关、2=1s、3=1min、4=1h、5=1d
    pub const CR0_INTERVAL_MASK: u32 = 0x7;

    // ---- CR1 (0x08) ----
    /// RTCCLK 来源（CR1[10:8]）
    pub const CR1_SOURCE_POS: u32 = 8;
    pub const CR1_SOURCE_MASK: u32 = 0x7 << CR1_SOURCE_POS;
    /// 来源编码
    pub const SRC_LSE: u32 = 0;
    pub const SRC_HSE: u32 = 1;
    pub const SRC_LSI: u32 = 2;
    pub const SRC_HSI: u32 = 3;
    /// 计时寄存器装载中（CR1[2]，RO）：1 = 不可读写
    pub const CR1_WAIT: u32 = 1 << 2;

    // ---- CR2 (0x0C) ----
    pub const CR2_ALARMBEN: u32 = 1 << 10;
    pub const CR2_ALARMAEN: u32 = 1 << 9;
    pub const CR2_AWTEN: u32 = 1 << 7;
    pub const CR2_TAMPEN: u32 = 1 << 6;
    pub const CR2_RTCOUT_POS: u32 = 4;
    pub const CR2_TAMPEDGE: u32 = 1 << 3;
    pub const CR2_AWTSRC: u32 = 1 << 2;
    pub const CR2_AWTPRS_MASK: u32 = 0x3;

    // ---- DATE (0x14) ----
    pub const DATE_WEEK_POS: u32 = 24;
    pub const DATE_WEEK_MASK: u32 = 0x7 << DATE_WEEK_POS;
    pub const DATE_YEAR_POS: u32 = 16;
    pub const DATE_YEAR_MASK: u32 = 0xFF << DATE_YEAR_POS;
    pub const DATE_MONTH_POS: u32 = 8;
    pub const DATE_MONTH_MASK: u32 = 0x1F << DATE_MONTH_POS;
    pub const DATE_DAY_MASK: u32 = 0x3F;

    // ---- TIME (0x18) ----
    pub const TIME_HOUR_POS: u32 = 16;
    pub const TIME_HOUR_MASK: u32 = 0x3F << TIME_HOUR_POS;
    pub const TIME_MINUTE_POS: u32 = 8;
    pub const TIME_MINUTE_MASK: u32 = 0x7F << TIME_MINUTE_POS;
    pub const TIME_SECOND_MASK: u32 = 0x7F;

    // ---- ALARMA / ALARMB (0x1C / 0x20) ----
    pub const ALARM_WEEKMASK_POS: u32 = 24;
    /// 1 = 该闹钟与小时无关（屏蔽）
    pub const ALARM_HOUREN: u32 = 1 << 23;
    pub const ALARM_HOUR_POS: u32 = 16;
    /// 1 = 该闹钟与分钟无关（屏蔽）
    pub const ALARM_MINUTEEN: u32 = 1 << 15;
    pub const ALARM_MINUTE_POS: u32 = 8;
    /// 1 = 该闹钟与秒钟无关（屏蔽）
    pub const ALARM_SECONDEN: u32 = 1 << 7;
    pub const ALARM_SECOND_MASK: u32 = 0x7F;

    // ---- IER (0x30) / ISR (0x34) / ICR (0x38) ----
    pub const IRQ_ALARMA: u32 = 1 << 0;
    pub const IRQ_ALARMB: u32 = 1 << 1;
    pub const IRQ_AWTIMER: u32 = 1 << 2;
    pub const IRQ_TAMP: u32 = 1 << 3;
    pub const IRQ_TAMPOV: u32 = 1 << 4;
    pub const IRQ_INTERVAL: u32 = 1 << 6;
    /// 全部已定义的标志位（bit0..4 + bit6，bit5/7 为保留位）
    pub const ICR_ALL: u32 = 0x5F;
    /// ICR 写入值：清除全部标志（R1W0：写 0 清除，保留位写 1）
    pub const ICR_CLEAR_ALL: u32 = !ICR_ALL;

    // ---- PSC (0x40) ----
    pub const PSC_PSC1_POS: u32 = 20;
    pub const PSC_PSC2_MASK: u32 = 0x000F_FFFF;
    /// LSE/LSI 32768Hz → TICKCLK 2Hz 的默认 PSC 值（PSC1=0、PSC2=0x3FFF）
    pub const PSC_32768_TO_2HZ: u32 = 0x3FFF;

    // ---- SSCNT (0x44) ----
    pub const SSCNT_HALF: u32 = 1 << 20;
    pub const SSCNT_LOW_MASK: u32 = 0x000F_FFFF;
}

/// CRC 位定义（§30，UM V1.4）
///
/// MODE 取值与厂商 `cw32l012_crc.h` 的 `CRC16_*` 常量一致。
pub mod crc {
    // ---- CR (0x00) ----
    /// 多项式选择（CR[3:0]）
    pub const MODE_POS: u32 = 0;
    pub const MODE_MASK: u32 = 0xF;

    /// 0：CRC16-IBM（x16+x15+x2+1，反射）
    pub const MODE_IBM: u32 = 0x0;
    /// 1：CRC16-MAXIM
    pub const MODE_MAXIM: u32 = 0x1;
    /// 2：CRC16-USB
    pub const MODE_USB: u32 = 0x2;
    /// 3：CRC16-MODBUS
    pub const MODE_MODBUS: u32 = 0x3;
    /// 4：CRC16-CCITT
    pub const MODE_CCITT: u32 = 0x4;
    /// 5：CRC16-CCITT-FALSE
    pub const MODE_CCITTFALSE: u32 = 0x5;
    /// 6：CRC16-X25
    pub const MODE_X25: u32 = 0x6;
    /// 7：CRC16-XMODEM
    pub const MODE_XMODEM: u32 = 0x7;

    // ---- DR (0x08) / RESULT (0x0C) ----
    /// 数据寄存器有效位（DR[7:0]）：每次写 1 字节
    pub const DR_MASK: u32 = 0xFF;
    /// 结果有效位（RESULT[15:0]）
    pub const RESULT_MASK: u32 = 0xFFFF;
}

/// EAU 位定义（扩展算术单元，UM V1.4）
pub mod eau {
    // ---- CSR (0x00) ----
    /// 运算模式（CSR[1:0]）
    pub const MODE_POS: u32 = 0;
    pub const MODE_MASK: u32 = 0x3;
    /// 0：无符号除法
    pub const MODE_UNSIGNED_DIV: u32 = 0x0;
    /// 1：有符号除法
    pub const MODE_SIGNED_DIV: u32 = 0x1;
    /// 2：开方
    pub const MODE_SQRT: u32 = 0x2;

    /// 有符号除法溢出（CSR[2]，只读）
    pub const OVR: u32 = 1 << 2;
    /// 除数为零（CSR[3]，只读）
    pub const ZERO: u32 = 1 << 3;
    /// 运算进行中（CSR[4]，只读）；启动后轮询此位直到清零
    pub const BUSY: u32 = 1 << 4;
}

/// CORDIC 位定义（UM V1.4）
///
/// 功能编码与厂商 `cw32l012_cordic.h` 的 `cordic_func_t` 一致。
pub mod cordic {
    // ---- CSR (0x00) ----
    /// 功能选择（CSR[3:0]）
    pub const FUNC_POS: u32 = 0;
    pub const FUNC_MASK: u32 = 0xF;
    /// 0：余弦 cos
    pub const FUNC_COS: u32 = 0x0;
    /// 1：正弦 sin
    pub const FUNC_SIN: u32 = 0x1;
    /// 2：相位角 atan2(y, x)
    pub const FUNC_ATAN2: u32 = 0x2;
    /// 3：模 hypot(x, y)
    pub const FUNC_HYPOT: u32 = 0x3;
    /// 4：反正切 atan
    pub const FUNC_ATAN: u32 = 0x4;
    /// 5：双曲余弦 cosh
    pub const FUNC_COSH: u32 = 0x5;
    /// 6：双曲正弦 sinh
    pub const FUNC_SINH: u32 = 0x6;
    /// 7：双曲反正切 atanh
    pub const FUNC_ATANH: u32 = 0x7;
    /// 8：自然对数 ln
    pub const FUNC_LN: u32 = 0x8;
    /// 9：平方根 sqrt
    pub const FUNC_SQRT: u32 = 0x9;

    /// 输入缩放（CSR[6:4]）：预移位次数，用于把数据压进 Q1.15/Q1.31 范围
    pub const SCALE_POS: u32 = 4;
    pub const SCALE_MASK: u32 = 0x7 << SCALE_POS;
    /// 数据格式（CSR[7]）：0 = Q1.15，1 = Q1.31
    pub const FORMAT_Q1_15: u32 = 0 << 7;
    pub const FORMAT_Q1_31: u32 = 1 << 7;

    /// 迭代次数（CSR[11:8]）：编码 n → 2×(n+1) 次（0000 = 2 次，1111 = 32 次）
    pub const ITER_POS: u32 = 8;
    pub const ITER_MASK: u32 = 0xF << ITER_POS;
    /// 迭代次数编码换算：`ITER_CODE(steps)`，steps 取偶数 2..=32
    pub const fn iter_code(steps: u32) -> u32 {
        (steps / 2 - 1) & 0xF
    }

    /// 中断使能（CSR[13]）
    pub const IE: u32 = 1 << 13;
    /// EOC 触发 DMA（CSR[14]）
    pub const DMAEOC: u32 = 1 << 14;
    /// BUSY 下降沿触发 DMA（CSR[15]）
    pub const DMAIDLE: u32 = 1 << 15;
    /// 运算完成（CSR[16]，只读，写 1 清除）
    pub const EOC: u32 = 1 << 16;
    /// 运算进行中（CSR[17]，只读）
    pub const BUSY: u32 = 1 << 17;
}

/// RAMCTRL 位定义（SRAM 奇偶校验错误，UM V1.4）
pub mod ram {
    /// 奇偶校验错误中断使能（IER[0]）
    pub const IER_PARITY: u32 = 1 << 0;
    /// 奇偶校验错误标志（ISR[0]，只读）
    pub const ISR_PARITY: u32 = 1 << 0;
    /// 写 1 清除错误标志（ICR[0]）
    pub const ICR_PARITY: u32 = 1 << 0;
}

/// LVD 位定义（低压检测，UM V1.4）
///
/// 阈值档位与滤波时间的编码见 UM 第 10 章。
pub mod lvd {
    // ---- CR0 (0x00) ----
    /// 使能低压检测
    pub const CR0_EN: u32 = 1 << 0;
    /// 检测到低压后的动作选择
    pub const CR0_ACTION: u32 = 1 << 1;
    /// 监测源选择
    pub const CR0_SOURCE: u32 = 1 << 2;
    /// 阈值档位（CR0[6:4]）
    pub const CR0_VTH_POS: u32 = 4;
    pub const CR0_VTH_MASK: u32 = 0x7 << CR0_VTH_POS;
    /// 滤波时钟选择（CR0[8]）
    pub const CR0_FLTCLK: u32 = 1 << 8;

    // ---- CR1 (0x04) ----
    /// 中断使能
    pub const CR1_IE: u32 = 1 << 0;
    /// 电平触发
    pub const CR1_LEVEL: u32 = 1 << 1;
    /// 上升沿触发
    pub const CR1_RISE: u32 = 1 << 2;
    /// 下降沿触发
    pub const CR1_FALL: u32 = 1 << 3;
    /// 滤波时间（CR1[7:4]）
    pub const CR1_FLTTIME_POS: u32 = 4;
    pub const CR1_FLTTIME_MASK: u32 = 0xF << CR1_FLTTIME_POS;

    // ---- SR (0x08) ----
    /// 中断标志（只读）
    pub const SR_INTF: u32 = 1 << 0;
    /// 当前电压低于阈值（只读）
    pub const SR_FLTV: u32 = 1 << 1;
}

// ================= DMA（§8.8） =================

/// DMA 通道寄存器组（每通道 0x20 字节，y = 1..4；通道 1 起始偏移 0x20）
#[allow(dead_code)]
#[repr(C)]
pub struct DmaChannel {
    /// +0x00 控制及状态（EN/TCIE/TEIE/TRANS/SRCINC/DSTINC/SIZE/STATUS/RESTART/TC/TE/重载）
    pub csr: Rw<u32>,
    /// +0x04 传输数量（CNT[15:0]、REPEAT[19:16]）
    pub cnt: Rw<u32>,
    /// +0x08 传输源地址
    pub src: Rw<u32>,
    /// +0x0C 传输目的地址
    pub dst: Rw<u32>,
    /// +0x10 触发源控制（TYPE / SOFTSRC / HARDSRC[7:2]）
    pub trig: Rw<u32>,
    /// +0x14 当前剩余传输数量（RO）
    pub ccnt: Ro<u32>,
    /// +0x18 当前传输源地址（RO）
    pub csrc: Ro<u32>,
    /// +0x1C 当前传输目的地址（RO）
    pub cdst: Ro<u32>,
}

/// DMA 控制器寄存器组（DMA_BASE = 0x4002_0000，表 8-4）
///
/// 通道 y 的寄存器 = 基址 + 0x20×y，即 `ch[y-1]`。
#[allow(dead_code)]
#[repr(C)]
pub struct Dma {
    /// 0x00 中断标志（TCy = bit[2(y-1)]、TEy = bit[2(y-1)+1]，RO）
    pub isr: Ro<u32>,
    /// 0x04 中断标志清除（同 ISR 位序；**R1W0：写 0 清除**，复位值 0xFFFF_FFFF）
    pub icr: Rw<u32>,
    /// 0x08..0x1F 保留
    _reserved: [u32; 6],
    /// 0x20..0x9F 通道 1..4
    pub ch: [DmaChannel; 4],
}

/// DMA 位定义（§8.8）
pub mod dma {
    // ---- CSR（+0x00）----
    /// 通道使能（CSR[0]）
    pub const CSR_EN: u32 = 1 << 0;
    /// 传输完成中断使能（CSR[1]）
    pub const CSR_TCIE: u32 = 1 << 1;
    /// 传输出错中断使能（CSR[2]）
    pub const CSR_TEIE: u32 = 1 << 2;
    /// 块传输模式（CSR[3]）：1=BLOCK（可被打断）、0=BULK（一口气搬完）
    pub const CSR_TRANS: u32 = 1 << 3;
    /// 源地址自增（CSR[4]）
    pub const CSR_SRCINC: u32 = 1 << 4;
    /// 目的地址自增（CSR[5]）
    pub const CSR_DSTINC: u32 = 1 << 5;
    /// 数据位宽（CSR[7:6]）：00=8bit/01=16bit/10=32bit（自增量随之 1/2/4）
    pub const CSR_SIZE_POS: u32 = 6;
    pub const CSR_SIZE_MASK: u32 = 0x3 << CSR_SIZE_POS;
    /// 通道状态（CSR[10:8]）：101=传输完成，其余见手册表 8-2
    pub const CSR_STATUS_POS: u32 = 8;
    pub const CSR_STATUS_MASK: u32 = 0x7 << CSR_STATUS_POS;
    /// 状态编码：传输完成
    pub const STATUS_DONE: u32 = 0b101;
    /// 自动重传（CSR[11]）：完成后按初始配置再启动
    pub const CSR_RESTART: u32 = 1 << 11;
    /// 传输完成标志（CSR[12]，RO；清除走 ICR）
    pub const CSR_TC: u32 = 1 << 12;
    /// 传输出错标志（CSR[13]，RO）
    pub const CSR_TE: u32 = 1 << 13;
    /// 每个数据块完成时源地址重载（CSR[14]；配合 RESTART 做环形缓冲）
    pub const CSR_SRCLOAD: u32 = 1 << 14;
    /// 每个数据块完成时目的地址重载（CSR[15]）
    pub const CSR_DSTLOAD: u32 = 1 << 15;

    // ---- CNT（+0x04）----
    /// 数据块数量（CNT[15:0]，1..=65535）
    pub const CNT_CNT_MASK: u32 = 0xFFFF;
    /// 数据块大小（REPEAT[19:16]，1..=15；每次传输后硬件自动清 0，下次必须重写）
    pub const CNT_REPEAT_POS: u32 = 16;
    pub const CNT_REPEAT_MASK: u32 = 0xF << CNT_REPEAT_POS;

    // ---- TRIG（+0x10）----
    /// 硬件触发（TRIG[0]）：0=软件触发、1=硬件触发
    pub const TRIG_TYPE: u32 = 1 << 0;
    /// 软件触发（TRIG[1]，W1 启动；传输完成后硬件自动清 0）
    pub const TRIG_SOFTSRC: u32 = 1 << 1;
    /// 硬件触发源（TRIG[7:2]，取值见 [`hs`]）
    pub const TRIG_HARDSRC_POS: u32 = 2;
    pub const TRIG_HARDSRC_MASK: u32 = 0x3F << TRIG_HARDSRC_POS;

    /// 硬件触发源编码（表 8-8，HARDSRC 6bit）
    pub mod hs {
        // UART
        pub const UART1_RX: u8 = 0x00;
        pub const UART1_TX: u8 = 0x01;
        pub const UART2_RX: u8 = 0x02;
        pub const UART2_TX: u8 = 0x03;
        pub const UART3_RX: u8 = 0x04;
        pub const UART3_TX: u8 = 0x05;
        // SPI
        pub const SPI1_RX: u8 = 0x06;
        pub const SPI1_TX: u8 = 0x07;
        pub const SPI2_RX: u8 = 0x08;
        pub const SPI2_TX: u8 = 0x09;
        pub const SPI3_RX: u8 = 0x0A;
        pub const SPI3_TX: u8 = 0x0B;
        // ADC
        pub const ADC1_SEQ: u8 = 0x0C;
        pub const ADC1_SINGLE: u8 = 0x0D;
        pub const ADC2_SEQ: u8 = 0x0E;
        pub const ADC2_SINGLE: u8 = 0x0F;
        // DAC
        pub const DAC1: u8 = 0x10;
        pub const DAC2: u8 = 0x11;
        /// HALLTIM
        pub const HALLTIM: u8 = 0x12;
        // BTIM
        pub const BTIM1_UPD: u8 = 0x13;
        pub const BTIM1_TRIG: u8 = 0x14;
        pub const BTIM2_UPD: u8 = 0x15;
        pub const BTIM2_TRIG: u8 = 0x16;
        pub const BTIM3_UPD: u8 = 0x17;
        pub const BTIM3_TRIG: u8 = 0x18;
        // GTIM1
        pub const GTIM1_TRIG: u8 = 0x19;
        pub const GTIM1_UPD: u8 = 0x1A;
        pub const GTIM1_CC1: u8 = 0x1B;
        pub const GTIM1_CC2: u8 = 0x1C;
        pub const GTIM1_CC3: u8 = 0x1D;
        pub const GTIM1_CC4: u8 = 0x1E;
        // GTIM2
        pub const GTIM2_TRIG: u8 = 0x1F;
        pub const GTIM2_UPD: u8 = 0x20;
        pub const GTIM2_CC1: u8 = 0x21;
        pub const GTIM2_CC2: u8 = 0x22;
        pub const GTIM2_CC3: u8 = 0x23;
        pub const GTIM2_CC4: u8 = 0x24;
        // GTIM3
        pub const GTIM3_TRIG: u8 = 0x25;
        pub const GTIM3_UPD: u8 = 0x26;
        pub const GTIM3_CC1: u8 = 0x27;
        pub const GTIM3_CC2: u8 = 0x28;
        pub const GTIM3_CC3: u8 = 0x29;
        pub const GTIM3_CC4: u8 = 0x2A;
        // GTIM4
        pub const GTIM4_TRIG: u8 = 0x2B;
        pub const GTIM4_UPD: u8 = 0x2C;
        pub const GTIM4_CC1: u8 = 0x2D;
        pub const GTIM4_CC2: u8 = 0x2E;
        pub const GTIM4_CC3: u8 = 0x2F;
        pub const GTIM4_CC4: u8 = 0x30;
        // ATIM
        pub const ATIM_UPD: u8 = 0x31;
        pub const ATIM_CC1: u8 = 0x32;
        pub const ATIM_CC2: u8 = 0x33;
        pub const ATIM_CC3: u8 = 0x34;
        pub const ATIM_CC4: u8 = 0x35;
        pub const ATIM_CC5: u8 = 0x36;
        pub const ATIM_CC6: u8 = 0x37;
        pub const ATIM_COM: u8 = 0x38;
        pub const ATIM_TRIG: u8 = 0x39;
        // CORDIC
        pub const CORDIC_IDLE: u8 = 0x3A;
        pub const CORDIC_EOC: u8 = 0x3B;
        // I2C
        pub const I2C1_TX: u8 = 0x3C;
        pub const I2C1_RX: u8 = 0x3D;
        pub const I2C2_TX: u8 = 0x3E;
        pub const I2C2_RX: u8 = 0x3F;
    }
}

// ================= CRC（§30，16 位 CRC 计算单元） =================

/// CRC 计算单元寄存器组
///
/// 8 种 16 位标准多项式（IBM / MAXIM / USB / MODBUS / CCITT / CCITTFALSE /
/// X25 / XMODEM），逐字节写入 [`dr`](Crc::dr)，结果从
/// [`result`](Crc::result) 读出。多项式选择见 [`crc`] 模块。
#[repr(C)]
pub struct Crc {
    /// 0x00 控制寄存器：MODE[3:0] 选择多项式
    pub cr: Rw<u32>,
    _pad04: [u32; 1], // 0x04 保留（官方头文件 RESERVED）
    /// 0x08 数据寄存器：写入 1 字节待计算数据（仅 DR[7:0] 有效）
    pub dr: Rw<u32>,
    /// 0x0C 结果寄存器：RESULT[15:0]，只读
    pub result: Ro<u32>,
}

// ================= EAU（扩展算术单元：除法 / 开方） =================

/// 扩展算术单元寄存器组
///
/// Cortex-M0+ 没有硬件除法指令，除法与开方走这个外设：置 MODE → 写
/// DIVIDEND / DIVISOR → 等 BUSY 清零 → 读 QUOTIENT / REMAINDER。
#[repr(C)]
pub struct Eau {
    /// 0x00 控制与状态：MODE[1:0] 可写，OVR / ZERO / BUSY 只读
    pub csr: Rw<u32>,
    /// 0x04 被除数（开方模式为待开方数）
    pub dividend: Rw<u32>,
    /// 0x08 除数（开方模式忽略）
    pub divisor: Rw<u32>,
    /// 0x0C 商（开方模式为结果），只读
    pub quotient: Ro<u32>,
    /// 0x10 余数，只读
    pub remainder: Ro<u32>,
}

// ================= CORDIC（三角函数 / 双曲 / 对数 / 开方） =================

/// CORDIC 协处理器寄存器组（定点 Q1.15 / Q1.31）
///
/// 写完 CSR（功能 + 缩放 + 格式 + 迭代次数）与 X / Y / Z 后，硬件迭代完成置
/// EOC；结果就地覆盖 X / Y / Z。功能编码见 [`cordic`] 模块。
#[repr(C)]
pub struct Cordic {
    /// 0x00 控制与状态：FUNC[3:0] SCALE[6:4] FORMAT[7] ITER[11:8] …，EOC / BUSY 只读
    pub csr: Rw<u32>,
    /// 0x04 数据 X（输入，结果也可能落在这里）
    pub x: Rw<u32>,
    /// 0x08 数据 Y
    pub y: Rw<u32>,
    /// 0x0C 数据 Z（输入角度 / 输出结果）
    pub z: Rw<u32>,
}

// ================= RAMCTRL（SRAM 奇偶校验错误） =================

/// SRAM 控制寄存器组（奇偶校验错误捕获，IRQ3 与 FLASH 共享）
#[repr(C)]
pub struct Ramctrl {
    /// 0x00 中断使能：PARITY
    pub ier: Rw<u32>,
    /// 0x04 出错地址，只读
    pub addr: Ro<u32>,
    /// 0x08 中断标志：PARITY，只读
    pub isr: Ro<u32>,
    /// 0x0C 中断标志清除：写 1 清 PARITY
    pub icr: Rw<u32>,
}

// ================= LVD（低压检测，IRQ1） =================

/// 低压检测寄存器组
///
/// 阈值档位（VTH）与滤波参数的对应关系见 UM 第 10 章；本层只提供位段。
#[repr(C)]
pub struct Lvd {
    /// 0x00 控制 0：EN / ACTION / SOURCE / VTH[6:4] / FLTCLK
    pub cr0: Rw<u32>,
    /// 0x04 控制 1：IE / LEVEL / RISE / FALL / FLTTIME[7:4]
    pub cr1: Rw<u32>,
    /// 0x08 状态：INTF / FLTV，只读
    pub sr: Ro<u32>,
}

// ================= 布局静态断言 =================
// 关键外设寄存器块必须与官方 SVD/头文件布局完全一致（编译期保证）：
// - Gpio：TOG @ 0x60，总大小 0x64（官方 GPIO_TypeDef Size=100）
// - Uart：CR3 @ 0x38、RXMATCH @ 0x3C，总大小 0x40
// - Sysctrl：MCO @ 0x70，总大小 0x74（官方 SYSCTRL_TypeDef Size=116；PLL@0x28 未建模，占位于 _pad0）
// - Flashctrl：官方 FLASH_TypeDef Size=116（0x74），SDKCFR @ 0x70 未建模 → HAL 总大小 0x70
const _: () = {
    assert!(
        core::mem::size_of::<Gpio>() == 0x64,
        "Gpio 布局错误：官方 GPIO_TypeDef 大小为 0x64"
    );
    assert!(
        core::mem::size_of::<Uart>() == 0x40,
        "Uart 布局错误：CR3@0x38 + RXMATCH@0x3C → 0x40"
    );
    assert!(
        core::mem::size_of::<Spi>() == 0x20,
        "Spi 布局错误：DR@0x1C + 4 → 0x20（表 22-4）"
    );
    assert!(
        core::mem::size_of::<I2c>() == 0x74,
        "I2c 布局错误：MRDR@0x70 + 4 → 0x74（表 23-7）"
    );
    assert!(
        core::mem::size_of::<Btim>() == 0x30,
        "Btim 布局错误：ARR@0x2C + 4 → 0x30（表 14-5）"
    );
    assert!(
        core::mem::size_of::<Adc>() == 0x7C,
        "Adc 布局错误：ISR@0x78 + 4 → 0x7C（表 25-7）"
    );
    assert!(
        core::mem::size_of::<Bgr>() == 4,
        "Bgr 布局错误：单寄存器 BGR_CR@0xFC"
    );
    assert!(
        core::mem::size_of::<Lptim>() == 0x24,
        "Lptim 布局错误：CR1@0x20 + 4 → 0x24（表 15-5）"
    );
    assert!(
        core::mem::size_of::<Halltim>() == 0x24,
        "Halltim 布局错误：STATE@0x20 + 4 → 0x24（表 18-4）"
    );
    assert!(
        core::mem::size_of::<Vc>() == 0x14,
        "Vc 布局错误：SR@0x10 + 4 → 0x14（表 27-5，实例步进 0x10）"
    );
    assert!(
        core::mem::size_of::<Opa>() == 0x08,
        "Opa 布局错误：CAL@0x04 + 4 → 0x08（§29.6）"
    );
    assert!(
        core::mem::size_of::<Atim>() == 0x74,
        "Atim 布局错误：ICR@0x70 + 4 → 0x74（§17.10，ICR 在 AF2 之后）"
    );
    assert!(
        core::mem::size_of::<Wwdt>() == 0x0C,
        "Wwdt 布局错误：SR@0x08 + 4 → 0x0C（表 20-1）"
    );
    assert!(
        core::mem::size_of::<Dac>() == 0x38,
        "Dac 布局错误：CR1@0x34 + 4 → 0x38（表 26-3）"
    );
    assert!(
        core::mem::size_of::<Iwdt>() == 0x28,
        "Iwdt 布局错误：CNT@0x24 + 4 → 0x28（表 19-4）"
    );
    assert!(
        core::mem::size_of::<Gtim>() == 0x68,
        "Gtim 布局错误：AF2@0x64 + 4 → 0x68（表 16-17）"
    );
    assert!(
        core::mem::size_of::<Sysctrl>() == 0x74,
        "Sysctrl 布局错误：官方 SYSCTRL_TypeDef 大小为 0x74（MCO@0x70）"
    );
    assert!(
        core::mem::size_of::<Flashctrl>() == 0x70,
        "Flashctrl 布局错误：官方 FLASH_TypeDef 0x74，SDKCFR@0x70 未建模 → 0x70"
    );
    assert!(
        core::mem::size_of::<DmaChannel>() == 0x20,
        "DmaChannel 布局错误：每通道 0x20 字节（表 8-4 步进 32×y）"
    );
    assert!(
        core::mem::size_of::<Dma>() == 0xA0,
        "Dma 布局错误：通道 4 末寄存器 @0x9C + 4 → 0xA0"
    );
    assert!(
        core::mem::size_of::<Rtc>() == 0x48,
        "Rtc 布局错误：SSCNT@0x44 + 4 → 0x48（表 13-6）"
    );
    assert!(
        core::mem::size_of::<Crc>() == 0x10,
        "Crc 布局错误：RESULT@0x0C + 4 → 0x10"
    );
    assert!(
        core::mem::size_of::<Eau>() == 0x14,
        "Eau 布局错误：REMAINDER@0x10 + 4 → 0x14"
    );
    assert!(
        core::mem::size_of::<Cordic>() == 0x10,
        "Cordic 布局错误：Z@0x0C + 4 → 0x10"
    );
    assert!(
        core::mem::size_of::<Ramctrl>() == 0x10,
        "Ramctrl 布局错误：ICR@0x0C + 4 → 0x10"
    );
    assert!(
        core::mem::size_of::<Lvd>() == 0x0C,
        "Lvd 布局错误：SR@0x08 + 4 → 0x0C"
    );
};

// ================= 中断号与 NVIC（手册表 5-1） =================

/// CW32L012 外部中断号（IRQ0..31）。
///
/// ⚠ 官方 SVD 的中断表缺少 IRQ31（FAULT），本枚举为完整 32 项。
///
/// 向量符号名（如 `UART1_irq`）与 cw32l012-hal `runtime` 的中断向量表
/// 一一对应；不使用该运行时时，请自行提供同名向量符号。
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interrupt {
    /// IRQ0：窗口/独立看门狗（符号 `WDT_irq`）
    Wdt = 0,
    /// IRQ1：低压检测（`LVD_irq`）
    Lvd = 1,
    /// IRQ2：RTC（`RTC_irq`）
    Rtc = 2,
    /// IRQ3：FLASH/RAM（`FLASHRAM_irq`）
    FlashRam = 3,
    /// IRQ4：RCC（`RCC_irq`）
    Rcc = 4,
    /// IRQ5：GPIOA（`GPIOA_irq`）
    GpioA = 5,
    /// IRQ6：GPIOB（`GPIOB_irq`）
    GpioB = 6,
    /// IRQ7：GPIOC（`GPIOC_irq`）
    GpioC = 7,
    /// IRQ8：GPIOF（`GPIOF_irq`）
    GpioF = 8,
    /// IRQ9：DMA 通道 1/2（`DMA12_irq`）
    Dma12 = 9,
    /// IRQ10：DMA 通道 3/4（`DMA34_irq`）
    Dma34 = 10,
    /// IRQ11：CORDIC（`CORDIC_irq`）
    Cordic = 11,
    /// IRQ12：ADC1（`ADC1_irq`）
    Adc1 = 12,
    /// IRQ13：ATIM（`ATIM_irq`）
    Atim = 13,
    /// IRQ14：VC1/3（`VC13_irq`）
    Vc13 = 14,
    /// IRQ15：VC2/4（`VC24_irq`）
    Vc24 = 15,
    /// IRQ16：GTIM1（`GTIM1_irq`）
    Gtim1 = 16,
    /// IRQ17：GTIM2（`GTIM2_irq`）
    Gtim2 = 17,
    /// IRQ18：GTIM3/4（`GTIM34_irq`）
    Gtim34 = 18,
    /// IRQ19：LPTIM（`LPTIM_irq`）
    Lptim = 19,
    /// IRQ20：BTIM1（`BTIM1_irq`）
    Btim1 = 20,
    /// IRQ21：BTIM2（`BTIM2_irq`）
    Btim2 = 21,
    /// IRQ22：BTIM3/HALLTIM（`BTIM3_HALLTIM_irq`）
    Btim3Halltim = 22,
    /// IRQ23：I2C1（`I2C1_irq`）
    I2c1 = 23,
    /// IRQ24：I2C2（`I2C2_irq`）
    I2c2 = 24,
    /// IRQ25：SPI1（`SPI1_irq`）
    Spi1 = 25,
    /// IRQ26：SPI2/3（`SPI23_irq`）
    Spi23 = 26,
    /// IRQ27：UART1（`UART1_irq`）
    Uart1 = 27,
    /// IRQ28：UART2（`UART2_irq`）
    Uart2 = 28,
    /// IRQ29：UART3（`UART3_irq`）
    Uart3 = 29,
    /// IRQ30：ADC2/DAC（`ADC2_DAC_irq`）
    Adc2Dac = 30,
    /// IRQ31：FAULT（`FAULT_irq`）——官方 SVD 缺失此中断
    Fault = 31,
}

impl Interrupt {
    /// 全部 32 个中断，按 IRQ 号排列（`ALL[n] as u8 == n`）。
    ///
    /// 便于批量操作 NVIC，例如复位后清干净所有使能与挂起位：
    ///
    /// ```ignore
    /// for irq in Interrupt::ALL {
    ///     irq.disable();
    ///     irq.unpend();
    /// }
    /// ```
    pub const ALL: [Interrupt; 32] = [
        Self::Wdt,
        Self::Lvd,
        Self::Rtc,
        Self::FlashRam,
        Self::Rcc,
        Self::GpioA,
        Self::GpioB,
        Self::GpioC,
        Self::GpioF,
        Self::Dma12,
        Self::Dma34,
        Self::Cordic,
        Self::Adc1,
        Self::Atim,
        Self::Vc13,
        Self::Vc24,
        Self::Gtim1,
        Self::Gtim2,
        Self::Gtim34,
        Self::Lptim,
        Self::Btim1,
        Self::Btim2,
        Self::Btim3Halltim,
        Self::I2c1,
        Self::I2c2,
        Self::Spi1,
        Self::Spi23,
        Self::Uart1,
        Self::Uart2,
        Self::Uart3,
        Self::Adc2Dac,
        Self::Fault,
    ];

    /// IRQ 编号（0..31）
    #[must_use]
    pub const fn number(self) -> u8 {
        self as u8
    }

    /// 是否已在 NVIC 中使能（读 ISER）
    #[must_use]
    pub fn is_enabled(self) -> bool {
        let irq = self.number() as u32;
        unsafe { core::ptr::read_volatile(0xE000_E100 as *const u32) & (1 << irq) != 0 }
    }

    /// 是否处于挂起状态（读 ISPR）
    #[must_use]
    pub fn is_pending(self) -> bool {
        let irq = self.number() as u32;
        unsafe { core::ptr::read_volatile(0xE000_E200 as *const u32) & (1 << irq) != 0 }
    }

    /// 在 NVIC 中使能该中断（ISER 写 1 置位）
    pub fn enable(self) {
        let irq = self as u32;
        unsafe {
            core::ptr::write_volatile(0xE000_E100 as *mut u32, 1 << irq);
        }
    }

    /// 在 NVIC 中禁用该中断（ICER 写 1 清除）
    pub fn disable(self) {
        let irq = self as u32;
        unsafe {
            core::ptr::write_volatile(0xE000_E180 as *mut u32, 1 << irq);
        }
    }

    /// 将该中断置为挂起（ISPR）
    pub fn pend(self) {
        let irq = self as u32;
        unsafe {
            core::ptr::write_volatile(0xE000_E200 as *mut u32, 1 << irq);
        }
    }

    /// 清除挂起状态（ICPR）
    pub fn unpend(self) {
        let irq = self as u32;
        unsafe {
            core::ptr::write_volatile(0xE000_E280 as *mut u32, 1 << irq);
        }
    }
}

impl TryFrom<u8> for Interrupt {
    /// 越界的 IRQ 编号（原样返回）
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match Self::ALL.get(value as usize) {
            Some(&irq) => Ok(irq),
            None => Err(value),
        }
    }
}

// ================= 外设单例 =================

/// 芯片外设单例。[`Peripherals::take`] 只能成功一次，保证外设所有权唯一。
pub struct Peripherals {
    /// 系统控制（复位与时钟）
    pub sysctrl: &'static Sysctrl,
    /// FLASH 控制器（读等待周期 / 缓存 / 擦写控制）
    pub flashctrl: &'static Flashctrl,
    /// GPIO 端口 A（PA0..PA8，视封装）
    pub gpioa: &'static Gpio,
    /// GPIO 端口 B（PB0..PB7，视封装）
    pub gpiob: &'static Gpio,
    /// GPIO 端口 C（PC13..PC15，视封装）
    pub gpioc: &'static Gpio,
    /// GPIO 端口 F（PF0/PF1/PF3/PF6/PF7，视封装）
    pub gpiof: &'static Gpio,
    /// UART1（PA02=TXD, PA03=RXD，AF1；另有 PA09/PA10 组合）
    pub uart1: &'static Uart,
    /// UART2（PA02=TXD, PA03=RXD，AF2；另有 PA07/PA06、PB00/PB01 组合）
    pub uart2: &'static Uart,
    /// UART3（PA04=TXD/PA05=RXD AF1，或 PA01=TXD/PA00=RXD AF2；另有 PA08/09、PA13/14、PB02/PB05 组合）
    pub uart3: &'static Uart,
    /// SPI1（SCK=PA05/PA10、MOSI=PA07/PA12、MISO=PA06/PA11，均 AF3）
    pub spi1: &'static Spi,
    /// SPI2（PB03/PB04/PB05，视封装可用性）
    pub spi2: &'static Spi,
    /// SPI3（视封装可用性）
    pub spi3: &'static Spi,
    /// I2C1（SCL=PA04/PA09、SDA=PA05/PA10，均 AF2，需开漏）
    pub i2c1: &'static I2c,
    /// I2C2（视封装可用性）
    pub i2c2: &'static I2c,
    /// BTIM1 基础定时器
    pub btim1: &'static Btim,
    /// BTIM2 基础定时器
    pub btim2: &'static Btim,
    /// BTIM3 基础定时器
    pub btim3: &'static Btim,
    /// ADC1（内部通道：温度传感器 IN14、1.2V 基准 IN15）
    pub adc1: &'static Adc,
    /// ADC2（仅外部通道，映射与 ADC1 不同，见表 25-4）
    pub adc2: &'static Adc,
    /// DAC 双通道（CH2 内部互联 ADC1_IN12，可无接线回环自测）
    pub dac: &'static Dac,
    /// IWDT 独立看门狗（RC10K 独立时钟域）
    pub iwdt: &'static Iwdt,
    /// LPTIM 低功耗定时器（LSI/LSE 域，DeepSleep 可唤醒，IRQ19）
    pub lptim: &'static Lptim,
    /// HALLTIM 霍尔接口定时器（3 路捕获 + PWM，IRQ22 与 BTIM3 共享）
    pub halltim: &'static Halltim,
    /// VC1 模拟电压比较器（与 VC2 共用 vc12ref 分压源，IRQ14）
    pub vc1: &'static Vc,
    /// VC2 模拟电压比较器（IRQ15）
    pub vc2: &'static Vc,
    /// VC3 模拟电压比较器（与 VC4 共用 vc34ref，IRQ14）
    pub vc3: &'static Vc,
    /// VC4 模拟电压比较器（IRQ15）
    pub vc4: &'static Vc,
    /// VC1/VC2 公共电阻分压参考源
    pub vc12ref: &'static VcRef,
    /// VC3/VC4 公共电阻分压参考源
    pub vc34ref: &'static VcRef,
    /// OPA1 运算放大器（正端 INP4 = DAC_OUT1）
    pub opa1: &'static Opa,
    /// OPA2 运算放大器（正端 INP4 = DAC_OUT2）
    pub opa2: &'static Opa,
    /// WWDT 窗口看门狗（PCLK 时钟域，窗口内喂狗）
    pub wwdt: &'static Wwdt,
    /// ATIM 高级定时器（互补 PWM + 死区 + 刹车）
    pub atim: &'static Atim,
    /// GTIM1 通用定时器（PWM ×4 通道）
    pub gtim1: &'static Gtim,
    /// GTIM2 通用定时器
    pub gtim2: &'static Gtim,
    /// GTIM3 通用定时器
    pub gtim3: &'static Gtim,
    /// GTIM4 通用定时器
    pub gtim4: &'static Gtim,
    /// DMA 控制器（4 通道，AHB 总线，§8）
    pub dma: &'static Dma,
    /// RTC 实时时钟（BCD 日历 + 闹钟 + 唤醒定时器，§13）
    pub rtc: &'static Rtc,
    /// CRC 计算单元（16 位，8 种标准多项式，§30）
    pub crc: &'static Crc,
    /// EAU 扩展算术单元（硬件除法 / 开方）
    pub eau: &'static Eau,
    /// CORDIC 协处理器（三角函数 / 双曲 / 对数 / 开方）
    pub cordic: &'static Cordic,
    /// SRAM 奇偶校验错误控制
    pub ramctrl: &'static Ramctrl,
    /// LVD 低压检测（IRQ1）
    pub lvd: &'static Lvd,
}

impl Peripherals {
    /// 获取外设单例；第二次及以后调用返回 `None`。
    ///
    /// 注：Cortex-M0+ 不支持核心库原子交换指令，此处在单核裸机语义下
    /// 使用 `static mut` 实现一次性标记（请在开启中断前完成首次调用）。
    pub fn take() -> Option<Self> {
        static mut TAKEN: bool = false;
        // SAFETY: 单核裸机环境；约定首次调用发生在使能中断之前。
        unsafe {
            if TAKEN {
                None
            } else {
                TAKEN = true;
                Some(Self::steal())
            }
        }
    }

    /// 无视单例直接取得外设引用（例如中断上下文中再次取用）。
    ///
    /// # Safety
    /// 调用方需自行保证不出现别名所有权冲突（例如重复初始化同一外设）。
    pub unsafe fn steal() -> Self {
        // SAFETY: 调用方保证不产生别名所有权；以下地址全部来自手册表 2-1。
        unsafe {
            Peripherals {
                sysctrl: &*(SYSCTRL_BASE as *const Sysctrl),
                flashctrl: &*(FLASHCTRL_BASE as *const Flashctrl),
                gpioa: &*(GPIOA_BASE as *const Gpio),
                gpiob: &*(GPIOB_BASE as *const Gpio),
                gpioc: &*(GPIOC_BASE as *const Gpio),
                gpiof: &*(GPIOF_BASE as *const Gpio),
                uart1: &*(UART1_BASE as *const Uart),
                uart2: &*(UART2_BASE as *const Uart),
                uart3: &*(UART3_BASE as *const Uart),
                spi1: &*(SPI1_BASE as *const Spi),
                spi2: &*(SPI2_BASE as *const Spi),
                spi3: &*(SPI3_BASE as *const Spi),
                i2c1: &*(I2C1_BASE as *const I2c),
                i2c2: &*(I2C2_BASE as *const I2c),
                btim1: &*(BTIM1_BASE as *const Btim),
                btim2: &*(BTIM2_BASE as *const Btim),
                btim3: &*(BTIM3_BASE as *const Btim),
                adc1: &*(ADC1_BASE as *const Adc),
                adc2: &*(ADC2_BASE as *const Adc),
                dac: &*(DAC_BASE as *const Dac),
                iwdt: &*(IWDT_BASE as *const Iwdt),
                lptim: &*(LPTIM_BASE as *const Lptim),
                halltim: &*(HALLTIM_BASE as *const Halltim),
                vc1: &*(VC1_BASE as *const Vc),
                vc2: &*(VC2_BASE as *const Vc),
                vc3: &*(VC3_BASE as *const Vc),
                vc4: &*(VC4_BASE as *const Vc),
                vc12ref: &*(VC12REF_BASE as *const VcRef),
                vc34ref: &*(VC34REF_BASE as *const VcRef),
                opa1: &*(OPA1_BASE as *const Opa),
                opa2: &*(OPA2_BASE as *const Opa),
                wwdt: &*(WWDT_BASE as *const Wwdt),
                atim: &*(ATIM_BASE as *const Atim),
                gtim1: &*(GTIM1_BASE as *const Gtim),
                gtim2: &*(GTIM2_BASE as *const Gtim),
                gtim3: &*(GTIM3_BASE as *const Gtim),
                gtim4: &*(GTIM4_BASE as *const Gtim),
                dma: &*(DMA_BASE as *const Dma),
                rtc: &*(RTC_BASE as *const Rtc),
                crc: &*(CRC_BASE as *const Crc),
                eau: &*(EAU_BASE as *const Eau),
                cordic: &*(CORDIC_BASE as *const Cordic),
                ramctrl: &*(RAMCTRL_BASE as *const Ramctrl),
                lvd: &*(LVD_BASE as *const Lvd),
            }
        }
    }
}

// ================= typelevel 中断（供 HAL / embassy 绑定） =================

/// typelevel 中断标记类型。
///
/// 每个外设中断一个零尺寸类型（如 [`interrupt::UART1`]），供 embassy 风格的
/// `bind_interrupts!` 宏做编译期"中断-驱动"绑定；`NUMBER` 与 [`Interrupt`]
/// 枚举判别值一致（手册表 5-1）。
pub mod interrupt {
    /// typelevel 中断号契约。
    ///
    /// # Safety
    ///
    /// `NUMBER` 必须是该中断在向量表里的真实 IRQ 编号（手册表 5-1）。编号写错
    /// 会让 `bind_interrupts!` 把中断绑到别的外设上，中断到来时执行错误的处理
    /// 函数，进而破坏外设状态机——这类错误编译器无法检查。本 crate 内的实现
    /// 全部由宏从同一张表生成，请勿手写 `impl`。
    pub unsafe trait Interrupt {
        /// IRQ 编号（0..31）
        const NUMBER: u16;
        /// IRQ 编号（便捷访问）
        fn number() -> u16 {
            Self::NUMBER
        }
    }

    #[allow(non_camel_case_types)]
    macro_rules! typelevel_irqs {
        ($( $(#[$meta:meta])* $name:ident = $num:expr; )*) => {
            $(
                $(#[$meta])*
                #[allow(non_camel_case_types)]
                pub struct $name;
                unsafe impl Interrupt for $name { const NUMBER: u16 = $num; }
            )*
        };
    }

    typelevel_irqs! {
        /// IRQ0：窗口/独立看门狗（向量符号 `WDT`）
        WDT = 0;
        /// IRQ1：低压检测（`LVD`）
        LVD = 1;
        /// IRQ2：RTC（`RTC`）
        RTC = 2;
        /// IRQ3：FLASH/RAM（`FLASHRAM`）
        FLASHRAM = 3;
        /// IRQ4：RCC（`RCC`）
        RCC = 4;
        /// IRQ5：GPIOA（`GPIOA`）
        GPIOA = 5;
        /// IRQ6：GPIOB（`GPIOB`）
        GPIOB = 6;
        /// IRQ7：GPIOC（`GPIOC`）
        GPIOC = 7;
        /// IRQ8：GPIOF（`GPIOF`）
        GPIOF = 8;
        /// IRQ9：DMA 通道 1/2（`DMA12`）
        DMA12 = 9;
        /// IRQ10：DMA 通道 3/4（`DMA34`）
        DMA34 = 10;
        /// IRQ11：CORDIC（`CORDIC`）
        CORDIC = 11;
        /// IRQ12：ADC1（`ADC1`）
        ADC1 = 12;
        /// IRQ13：ATIM（`ATIM`）
        ATIM = 13;
        /// IRQ14：VC1/3（`VC13`）
        VC13 = 14;
        /// IRQ15：VC2/4（`VC24`）
        VC24 = 15;
        /// IRQ16：GTIM1（`GTIM1`）
        GTIM1 = 16;
        /// IRQ17：GTIM2（`GTIM2`）
        GTIM2 = 17;
        /// IRQ18：GTIM3/4（`GTIM34`）
        GTIM34 = 18;
        /// IRQ19：LPTIM（`LPTIM`）
        LPTIM = 19;
        /// IRQ20：BTIM1（`BTIM1`）
        BTIM1 = 20;
        /// IRQ21：BTIM2（`BTIM2`）
        BTIM2 = 21;
        /// IRQ22：BTIM3/HALLTIM（`BTIM3_HALLTIM`）
        BTIM3_HALLTIM = 22;
        /// IRQ23：I2C1（`I2C1`）
        I2C1 = 23;
        /// IRQ24：I2C2（`I2C2`）
        I2C2 = 24;
        /// IRQ25：SPI1（`SPI1`）
        SPI1 = 25;
        /// IRQ26：SPI2/3（`SPI23`）
        SPI23 = 26;
        /// IRQ27：UART1（`UART1`）
        UART1 = 27;
        /// IRQ28：UART2（`UART2`）
        UART2 = 28;
        /// IRQ29：UART3（`UART3`）
        UART3 = 29;
        /// IRQ30：ADC2/DAC（`ADC2_DAC`）
        ADC2_DAC = 30;
        /// IRQ31：FAULT（`FAULT`）——官方 SVD 缺失此中断
        FAULT = 31;
    }
}

// ================= 中断向量表（feature `rt`） =================

/// 外设中断向量表（cortex-m-rt `device` 模式）。
///
/// `__INTERRUPTS` 位于 `.vector_table.interrupts` 段（ORIGIN(FLASH)+0x40，
/// 即 IRQ0 槽位）。每个向量符号由 `device.x` 的 `PROVIDE(sym = DefaultHandler)`
/// 提供默认值；用户（或 embassy-cw32 的 `bind_interrupts!` 宏）用
/// `#[no_mangle] unsafe extern "C" fn UART1()` 强符号覆盖。
#[cfg(all(feature = "rt", target_arch = "arm", target_os = "none"))]
pub mod rt {
    /// 向量表项（Cortex-M ABI：函数指针或 0）
    #[repr(C)]
    pub union Vector {
        /// 中断服务程序
        pub handler: unsafe extern "C" fn(),
        /// 保留槽位（填 0）
        pub reserved: usize,
    }

    extern "C" {
        fn WDT();
        fn LVD();
        fn RTC();
        fn FLASHRAM();
        fn RCC();
        fn GPIOA();
        fn GPIOB();
        fn GPIOC();
        fn GPIOF();
        fn DMA12();
        fn DMA34();
        fn CORDIC();
        fn ADC1();
        fn ATIM();
        fn VC13();
        fn VC24();
        fn GTIM1();
        fn GTIM2();
        fn GTIM34();
        fn LPTIM();
        fn BTIM1();
        fn BTIM2();
        fn BTIM3_HALLTIM();
        fn I2C1();
        fn I2C2();
        fn SPI1();
        fn SPI23();
        fn UART1();
        fn UART2();
        fn UART3();
        fn ADC2_DAC();
        fn FAULT();
    }

    #[link_section = ".vector_table.interrupts"]
    #[no_mangle]
    pub static __INTERRUPTS: [Vector; 32] = [
        Vector { handler: WDT },
        Vector { handler: LVD },
        Vector { handler: RTC },
        Vector { handler: FLASHRAM },
        Vector { handler: RCC },
        Vector { handler: GPIOA },
        Vector { handler: GPIOB },
        Vector { handler: GPIOC },
        Vector { handler: GPIOF },
        Vector { handler: DMA12 },
        Vector { handler: DMA34 },
        Vector { handler: CORDIC },
        Vector { handler: ADC1 },
        Vector { handler: ATIM },
        Vector { handler: VC13 },
        Vector { handler: VC24 },
        Vector { handler: GTIM1 },
        Vector { handler: GTIM2 },
        Vector { handler: GTIM34 },
        Vector { handler: LPTIM },
        Vector { handler: BTIM1 },
        Vector { handler: BTIM2 },
        Vector {
            handler: BTIM3_HALLTIM,
        },
        Vector { handler: I2C1 },
        Vector { handler: I2C2 },
        Vector { handler: SPI1 },
        Vector { handler: SPI23 },
        Vector { handler: UART1 },
        Vector { handler: UART2 },
        Vector { handler: UART3 },
        Vector { handler: ADC2_DAC },
        Vector { handler: FAULT },
    ];
}
