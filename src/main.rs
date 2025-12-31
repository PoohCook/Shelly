#![no_main]
#![no_std]

// Halt on panic
use panic_halt as _; // panic handler

use cortex_m_rt::entry;
use stm32f4xx_hal as hal;

use crate::hal::pac;
use crate::hal::pac::TIM2;
use crate::hal::prelude::*;
use crate::hal::timer::Counter;

static mut EP_MEMORY: [u32; 1024] = [0; 1024];
use ws2812_spi as ws2812;

use rtt_target::rprintln;
use rtt_target::rtt_init_print;

mod test_points;
use test_points::{*};

mod pallet;
use pallet::Colors;

mod light_ports;
use light_ports::*;


#[entry]
fn main() -> ! {
    rtt_init_print!();

    19200.bps();

    // Acquire the device peripherals
    let dp = pac::Peripherals::take().unwrap();

    // Configure the RCC (Reset and Clock Control) peripheral to enable GPIO
    let rcc = dp.RCC.constrain();
    let clocks: hal::rcc::Clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

    let mut sys_timer:Counter<TIM2, 1000> = dp.TIM2.counter_ms(&clocks);
    sys_timer.start(u32::MAX.millis()).unwrap();

    let gpioa = dp.GPIOA.split();
    // let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();
    // let gpiod = dp.GPIOD.split();
    // let gpioe = dp.GPIOE.split();

    // Setup test point support
    let mut test_point = TestPoints::new(
        gpioc.pc0, gpioc.pc1, gpioc.pc2, gpioc.pc3, gpioc.pc4, gpioc.pc5, gpioc.pc6, gpioc.pc7,
    );
    test_point.reset_all();

    //  Initialize Ws2812 LED support
    let mut buffer = [0u8; (LED_NUM * 12) + 30];
    let mut lights = LightPorts::new(gpioa.pa5, gpioa.pa7, dp.SPI1, &mut buffer, &clocks, &sys_timer);

    rprintln!("USB Built");
    let mut count: u32 = 0;

    lights.set_blade(0, Colors::Red.as_rgb(), false);
    lights.set_blade(1, Colors::Yellow.as_rgb(), false);
    lights.set_blade(2, Colors::Green.as_rgb(), false);
    lights.set_blade(3, Colors::Cyan.as_rgb(), false);
    lights.set_blade(4, Colors::Blue.as_rgb(), false);
    lights.set_blade(5, Colors::Magenta.as_rgb(), false);
    lights.set_blade(6, Colors::Red.as_rgb(), false);
    lights.set_blade(7, Colors::Yellow.as_rgb(), false);
    lights.set_blade(8, Colors::Green.as_rgb(), false);
    lights.set_blade(9, Colors::Cyan.as_rgb(), false);
    lights.set_blade(10, Colors::Blue.as_rgb(), false);
    lights.set_blade(11, Colors::Magenta.as_rgb(), false);
    lights.set_blade(12, Colors::Red.as_rgb(), false);
    lights.set_blade(13, Colors::Yellow.as_rgb(), false);
    lights.set_blade(14, Colors::Green.as_rgb(), false);
    lights.set_blade(15, Colors::Cyan.as_rgb(), false);
    lights.set_blade(16, Colors::Blue.as_rgb(), false);
    lights.set_blade(17, Colors::Magenta.as_rgb(), false);
    lights.set_blade(18, Colors::Red.as_rgb(), false);
    lights.set_blade(19, Colors::Yellow.as_rgb(), false);
    lights.set_blade(20, Colors::Green.as_rgb(), false);
    lights.set_blade(21, Colors::Cyan.as_rgb(), false);
    lights.set_blade(22, Colors::Blue.as_rgb(), false);
    lights.set_blade(23, Colors::Magenta.as_rgb(), false);
    lights.set_blade(24, Colors::Red.as_rgb(), false);
    lights.set_blade(25, Colors::Yellow.as_rgb(), false);
    lights.set_blade(26, Colors::Green.as_rgb(), false);
    lights.set_blade(27, Colors::Cyan.as_rgb(), false);
    lights.set_blade(28, Colors::Blue.as_rgb(), false);
    lights.set_blade(29, Colors::Magenta.as_rgb(), false);
    lights.set_blade(30, Colors::Red.as_rgb(), false);
    lights.set_blade(31, Colors::Red.as_rgb(), false);

    loop {

        // refresh the ws2812 leds to facilitate blinking behavour
        lights.refresh( true);

        // delay 1 msec to reduce overhead
        // this is a bit mickey mouse but it hunts for now
        let timeout: fugit::Instant<u32, 1, 1000> = sys_timer.now() + 1.millis();
        while sys_timer.now() < timeout { }

        count += 1;
        if count > 1000{
            count = 0;
            test_point.tp1.toggle();
        }

    }}
