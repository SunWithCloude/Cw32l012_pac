// rt feature 构建脚本：生成 device.x（32 个外设中断的 PROVIDE 弱别名）并把它
// 所在目录加进链接搜索路径，供 cortex-m-rt 的 link.x 用 `INCLUDE device.x`
// 找到。
//
// 注意：`-Tlink.x` 必须由**应用**提供（`.cargo/config.toml` 的 rustflags，
// 或应用自己的 build.rs），一个库 crate 的 link-arg 不会传播到最终固件。
// 应用侧还需要 memory.x —— 里面写 FLASH / RAM 的 ORIGIN 与 LENGTH。
//
// 参考 cortex-m-rt 文档 "Incorporating device specific interrupts" 一节
// （svd2rust PAC 的标准做法）。
use std::env;
use std::fs;
use std::path::PathBuf;

/// (向量符号名, IRQ 号) —— 符号名必须与 Interrupt 枚举的向量名一致，
/// 数组下标即向量槽位（tests/layout.rs 会校验两者对齐）。
const IRQS: [(&str, u16); 32] = [
    ("WDT", 0),
    ("LVD", 1),
    ("RTC", 2),
    ("FLASHRAM", 3),
    ("RCC", 4),
    ("GPIOA", 5),
    ("GPIOB", 6),
    ("GPIOC", 7),
    ("GPIOF", 8),
    ("DMA12", 9),
    ("DMA34", 10),
    ("CORDIC", 11),
    ("ADC1", 12),
    ("ATIM", 13),
    ("VC13", 14),
    ("VC24", 15),
    ("GTIM1", 16),
    ("GTIM2", 17),
    ("GTIM34", 18),
    ("LPTIM", 19),
    ("BTIM1", 20),
    ("BTIM2", 21),
    ("BTIM3_HALLTIM", 22),
    ("I2C1", 23),
    ("I2C2", 24),
    ("SPI1", 25),
    ("SPI23", 26),
    ("UART1", 27),
    ("UART2", 28),
    ("UART3", 29),
    ("ADC2_DAC", 30),
    ("FAULT", 31),
];

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let target = env::var("TARGET").unwrap();
    let rt_enabled = env::var_os("CARGO_FEATURE_RT").is_some();
    if !rt_enabled || !target.starts_with("thumb") {
        return;
    }

    // 表内编号必须等于数组下标，否则下面的 PROVIDE 会把中断绑到错误的槽位。
    for (slot, (name, num)) in IRQS.iter().enumerate() {
        assert_eq!(
            *num as usize, slot,
            "build.rs 的 IRQS 表顺序错位：{name} 标称 IRQ{num}，实际排在槽位 {slot}"
        );
    }

    let mut device_x =
        String::from("/* 由 build.rs 生成：未绑定的外设中断弱别名到 DefaultHandler */\n");
    for (name, _) in IRQS {
        device_x.push_str(&format!("PROVIDE({name} = DefaultHandler);\n"));
    }

    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out.join("device.x"), device_x).unwrap();

    // link-search 会传播到最终固件的链接命令行，link.x 的 INCLUDE device.x 靠它。
    println!("cargo:rustc-link-search=native={}", out.display());
}
