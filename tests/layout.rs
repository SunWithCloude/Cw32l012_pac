//! 布局与中断表回归测试（宿主机运行，不触碰真实地址）。

use cw32l012_pac::*;

#[test]
fn register_block_sizes_match_official_headers() {
    // 与 lib.rs 内编译期断言同一组数据；此处在测试侧再独立复核一遍，
    // 万一断言被误删，CI 仍能拦住布局漂移。
    assert_eq!(core::mem::size_of::<Gpio>(), 0x64);
    assert_eq!(core::mem::size_of::<Uart>(), 0x40);
    assert_eq!(core::mem::size_of::<Spi>(), 0x20);
    assert_eq!(core::mem::size_of::<I2c>(), 0x74);
    assert_eq!(core::mem::size_of::<Btim>(), 0x30);
    assert_eq!(core::mem::size_of::<Adc>(), 0x7C);
    assert_eq!(core::mem::size_of::<Bgr>(), 4);
    assert_eq!(core::mem::size_of::<Lptim>(), 0x24);
    assert_eq!(core::mem::size_of::<Halltim>(), 0x24);
    assert_eq!(core::mem::size_of::<Vc>(), 0x14);
    assert_eq!(core::mem::size_of::<Opa>(), 0x08);
    assert_eq!(core::mem::size_of::<Atim>(), 0x74);
    assert_eq!(core::mem::size_of::<Wwdt>(), 0x0C);
    assert_eq!(core::mem::size_of::<Dac>(), 0x38);
    assert_eq!(core::mem::size_of::<Iwdt>(), 0x28);
    assert_eq!(core::mem::size_of::<Gtim>(), 0x68);
    assert_eq!(core::mem::size_of::<Sysctrl>(), 0x74);
    assert_eq!(core::mem::size_of::<Flashctrl>(), 0x70);
    assert_eq!(core::mem::size_of::<DmaChannel>(), 0x20);
    assert_eq!(core::mem::size_of::<Dma>(), 0xA0);
    assert_eq!(core::mem::size_of::<Rtc>(), 0x48);
}

#[test]
fn gpio_pdr_sits_at_0x0c() {
    // 官方 SVD 缺失的 GPIOx_PDR：验证它确实在 0x0C 而非被 reserved 吞掉。
    let g: Gpio = unsafe { core::mem::zeroed() };
    let base = &g as *const Gpio as usize;
    let pdr = core::ptr::addr_of!(g.pdr) as usize;
    assert_eq!(pdr - base, 0x0C);
}

#[test]
fn interrupt_table_has_all_32_entries() {
    // 官方 SVD 的 __INTERRUPTS 只有 31 项；这里验证 IRQ31 = FAULT 存在。
    assert_eq!(Interrupt::Fault as u32, 31);
    assert_eq!(Interrupt::Wdt as u32, 0);
    // 抽查几个与手册表 5-1 一致的映射。
    assert_eq!(Interrupt::Uart1 as u32, 27);
    assert_eq!(Interrupt::Adc2Dac as u32, 30);
    assert_eq!(Interrupt::Btim3Halltim as u32, 22);
}

#[test]
fn base_addresses_match_memory_map() {
    assert_eq!(SYSCTRL_BASE, 0x4000_4000);
    assert_eq!(GPIOA_BASE, 0x4800_0000);
    assert_eq!(UART1_BASE, 0x4000_0C00);
    assert_eq!(DMA_BASE, 0x4002_0000);
    assert_eq!(RTC_BASE, 0x4000_4400);
    assert_eq!(FLASH_BASE, 0x0000_0000);
    assert_eq!(SRAM_BASE, 0x2000_0000);
}
