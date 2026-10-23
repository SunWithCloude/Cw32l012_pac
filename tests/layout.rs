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
    assert_eq!(core::mem::size_of::<Crc>(), 0x10);
    assert_eq!(core::mem::size_of::<Eau>(), 0x14);
    assert_eq!(core::mem::size_of::<Cordic>(), 0x10);
    assert_eq!(core::mem::size_of::<Ramctrl>(), 0x10);
    assert_eq!(core::mem::size_of::<Lvd>(), 0x0C);
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
fn crc_and_eau_offsets_match_headers() {
    // CRC 的 0x04 是保留字；DR 必须落在 0x08，否则整块寄存器错位。
    let c: Crc = unsafe { core::mem::zeroed() };
    let base = &c as *const Crc as usize;
    assert_eq!(core::ptr::addr_of!(c.dr) as usize - base, 0x08);
    assert_eq!(core::ptr::addr_of!(c.result) as usize - base, 0x0C);

    let e: Eau = unsafe { core::mem::zeroed() };
    let base = &e as *const Eau as usize;
    assert_eq!(core::ptr::addr_of!(e.quotient) as usize - base, 0x0C);
    assert_eq!(core::ptr::addr_of!(e.remainder) as usize - base, 0x10);
}

#[test]
fn interrupt_all_is_dense_and_ordered() {
    // Interrupt::ALL 的顺序即 IRQ 号；向量表、NVIC、typelevel 都以它为基准。
    for (i, irq) in Interrupt::ALL.iter().enumerate() {
        assert_eq!(*irq as usize, i, "Interrupt::ALL[{i}] 的 IRQ 号不对");
        assert_eq!(irq.number() as usize, i);
        assert_eq!(Interrupt::try_from(i as u8), Ok(*irq));
    }
    assert_eq!(Interrupt::try_from(255), Err(255));
}

#[test]
fn typelevel_irq_numbers_match_enum() {
    use cw32l012_pac::interrupt as tl;

    macro_rules! check {
        ($($ty:ident = $variant:ident),* $(,)?) => {
            $(
                assert_eq!(
                    <tl::$ty as tl::Interrupt>::NUMBER,
                    Interrupt::$variant as u16,
                    concat!("typelevel ", stringify!($ty), " 的 IRQ 号与枚举不一致")
                );
            )*
        };
    }

    check! {
        WDT = Wdt, LVD = Lvd, RTC = Rtc, FLASHRAM = FlashRam, RCC = Rcc,
        GPIOA = GpioA, GPIOB = GpioB, GPIOC = GpioC, GPIOF = GpioF,
        DMA12 = Dma12, DMA34 = Dma34, CORDIC = Cordic, ADC1 = Adc1, ATIM = Atim,
        VC13 = Vc13, VC24 = Vc24, GTIM1 = Gtim1, GTIM2 = Gtim2, GTIM34 = Gtim34,
        LPTIM = Lptim, BTIM1 = Btim1, BTIM2 = Btim2, BTIM3_HALLTIM = Btim3Halltim,
        I2C1 = I2c1, I2C2 = I2c2, SPI1 = Spi1, SPI23 = Spi23,
        UART1 = Uart1, UART2 = Uart2, UART3 = Uart3, ADC2_DAC = Adc2Dac,
        FAULT = Fault,
    }
}

#[test]
fn build_script_irq_table_is_ordered() {
    // build.rs 里的 IRQS 表是第三份中断清单（另两份：Interrupt 枚举、
    // interrupt 模块）。数组顺序就是向量槽位，编号必须与下标一致，
    // 否则 device.x 的 PROVIDE 会把中断绑到错误的槽。
    let src = include_str!("../build.rs");
    let mut nums = Vec::new();
    for chunk in src.split("(\"").skip(1) {
        let after = chunk.split_once("\",").map_or("", |(_, rest)| rest);
        let digits: String = after
            .chars()
            .skip_while(|c| !c.is_ascii_digit())
            .take_while(char::is_ascii_digit)
            .collect();
        if let Ok(n) = digits.parse::<u16>() {
            nums.push(n);
        }
    }
    assert_eq!(nums.len(), 32, "build.rs 的 IRQS 表应有 32 项");
    for (i, n) in nums.iter().enumerate() {
        assert_eq!(*n as usize, i, "build.rs IRQS 第 {i} 项的编号与槽位不一致");
    }
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
    assert_eq!(CRC_BASE, 0x4002_3000);
    assert_eq!(EAU_BASE, 0x4002_3200);
    assert_eq!(CORDIC_BASE, 0x4002_3400);
    assert_eq!(RAM_BASE, 0x4002_2400);
    assert_eq!(LVD_BASE, 0x4000_00A4);
}

#[test]
fn build_script_irq_names_match_symbols() {
    // 中断清单在三处维护：Interrupt 枚举、interrupt 模块（含 SYMBOLS）、
    // build.rs 的 IRQS 表（device.x 的 PROVIDE 名单）。编号一致性已有
    // 专项测试；本测试校验**符号名与顺序**逐项一致——build.rs 里若把
    // "UART1" 手滑改成别的，会在应用链接期才炸出难定位的 undefined
    // reference，这里提前到 CI。
    let src = include_str!("../build.rs");
    let mut pairs: Vec<(&str, u16)> = Vec::new();
    for chunk in src.split("(\"").skip(1) {
        if let Some((name, rest)) = chunk.split_once("\",") {
            let digits: String = rest
                .chars()
                .skip_while(|c| !c.is_ascii_digit())
                .take_while(char::is_ascii_digit)
                .collect();
            if let Ok(n) = digits.parse::<u16>() {
                pairs.push((name, n));
            }
        }
    }
    let symbols = cw32l012_pac::interrupt::SYMBOLS;
    assert_eq!(
        pairs.len(),
        symbols.len(),
        "build.rs IRQS 与 SYMBOLS 项数不一致"
    );
    for (i, (b, sym)) in pairs.iter().zip(symbols.iter()).enumerate() {
        assert_eq!(
            b.0, sym.0,
            "第 {i} 项符号名不一致：build.rs={:?} symbols={:?}",
            b.0, sym.0
        );
        assert_eq!(b.1, sym.1, "第 {i} 项 IRQ 号不一致");
    }
}

#[test]
fn symbols_are_dense_and_match_enum_names() {
    // SYMBOLS 编号 0..31 连续（向量槽位即数组下标），且首尾抽查。
    let symbols = cw32l012_pac::interrupt::SYMBOLS;
    assert_eq!(symbols.len(), 32);
    for (i, &(_, n)) in symbols.iter().enumerate() {
        assert_eq!(n as usize, i, "SYMBOLS 第 {i} 项编号不连续");
    }
    assert_eq!(symbols[0].0, "WDT");
    assert_eq!(symbols[27].0, "UART1");
    assert_eq!(symbols[31].0, "FAULT");
}
