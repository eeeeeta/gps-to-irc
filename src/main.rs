#![no_main]
#![no_std]
#![feature(alloc)]
#![feature(lang_items)]

extern crate cortex_m;
#[macro_use(block)]
extern crate nb;
extern crate cortex_m_rt as rt;
extern crate panic_semihosting;
extern crate stm32f103xx_hal as hal;
extern crate nmea_slimline as nmea;
extern crate cortex_m_semihosting;
extern crate alloc_cortex_m;
#[macro_use]
extern crate alloc;

use alloc::prelude::*;
use hal::prelude::*;
use hal::serial::Serial;
use rt::{entry, exception, ExceptionFrame};
use alloc_cortex_m::CortexMHeap;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[entry]
fn main() -> ! {
    let dp = hal::stm32f103xx::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let start = rt::heap_start() as usize;
    let size = 19 * 1024;
    unsafe { ALLOCATOR.init(start, size) }

    let mut led_read = gpioa.pa6.into_push_pull_output(&mut gpioa.crl);
    let mut led_err = gpioa.pa5.into_push_pull_output(&mut gpioa.crl);
    let tx = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
    let rx = gpioa.pa3;

    let serial1 = Serial::usart2(
        dp.USART2,
        (tx, rx),
        &mut afio.mapr,
        9_600.bps(),
        clocks,
        &mut rcc.apb1
    );
    let (mut tx, mut rx) = serial1.split();
    let tx3 = gpiob.pb10.into_alternate_push_pull(&mut gpiob.crh);
    let rx3 = gpiob.pb11;
    let serial2 = Serial::usart3(
        dp.USART3,
        (tx3, rx3),
        &mut afio.mapr,
        9_600.bps(),
        clocks,
        &mut rcc.apb1,
    );
    let (mut tx3, mut rx3) = serial2.split();
    led_read.set_high();
    for b in b"Relaying Test\n" {
        block!(tx3.write(*b)).unwrap();
    }
    led_read.set_low();
    let mut nmea_buf = Vec::with_capacity(102);
    loop {
        match rx.read() {
            Ok(b) => {
                if nmea_buf.len() >= 102 {
                    nmea_buf.truncate(0);
                }
                nmea_buf.push(b);
                if b == b'\n' {
                    led_read.set_high();
                    led_err.set_high();
                    let res = format!("{:?}\r\n", nmea::parse(&nmea_buf));
                    for b in res.as_bytes() {
                        block!(tx3.write(*b)).unwrap();
                    }
                    led_read.set_low();
                    led_err.set_low();
                    nmea_buf.truncate(0);
                }
                led_read.set_high();
                led_err.set_low();
            },
            Err(nb::Error::WouldBlock) => {
                led_read.set_low();
            },
            Err(nb::Error::Other(e)) => {
                led_err.set_high();
            }
        }
    }
}

#[lang = "oom"]
#[no_mangle]
pub fn rust_oom(_: alloc::alloc::Layout) -> ! {
    use core::fmt::Write;

    match cortex_m_semihosting::hio::hstdout() {
        Ok(mut fd) => {
            let _ = write!(fd, "Out of memory!");
        },
        Err(_) => {
            // welp.
        }
    }
    loop {}
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
