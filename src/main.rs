#![no_main]
#![no_std]

extern crate cortex_m;
#[macro_use(block)]
extern crate nb;
extern crate cortex_m_rt as rt;
extern crate panic_semihosting;
extern crate stm32f103xx_hal as hal;

use hal::prelude::*;
use hal::serial::Serial;
use rt::{entry, exception, ExceptionFrame};

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

    let tx1 = gpiob.pb6.into_alternate_push_pull(&mut gpiob.crl);
    let rx1 = gpiob.pb7;

    let serial1 = Serial::usart1(
        dp.USART1,
        (tx1, rx1),
        &mut afio.mapr,
        9_600.bps(),
        clocks,
        &mut rcc.apb2
    );
    let (mut tx1, mut rx1) = serial1.split();

    let tx2 = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
    let rx2 = gpioa.pa3;

    let serial2 = Serial::usart2(
        dp.USART2,
        (tx2, rx2),
        &mut afio.mapr,
        9_600.bps(),
        clocks,
        &mut rcc.apb1
    );
    let (mut tx2, mut rx2) = serial2.split();

    loop {
        if let Ok(b) = rx1.read() {
            block!(tx2.write(b)).ok();
        }
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
