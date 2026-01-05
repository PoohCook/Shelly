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

use ws2812_spi as ws2812;

use rtt_target::rprintln;
use rtt_target::rtt_init_print;

mod test_points;
use test_points::{*};

mod pallet;

mod light_ports;
use light_ports::*;

mod effects;
use effects::*;

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

    // Initialize the effects manager
    let mut effect_manager = EffectManager::new(&sys_timer);

    rprintln!("Effects Started");
    let mut count: u32 = 0;

    loop {
        // Update visual effects
        let updated = effect_manager.update(&mut lights, &sys_timer);

        // refresh the ws2812 leds to facilitate blinking behavour
        lights.refresh(updated);

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
