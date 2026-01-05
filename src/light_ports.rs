

use ws2812_spi as ws2812;

use crate::hal::rcc::*;
use crate::hal::pac::*;
use crate::hal::gpio::{NoPin, Pin};
use crate::hal::pac::TIM2;
use crate::hal::prelude::*;
use crate::hal::timer::Counter;
use fugit::Instant;


use crate::hal::spi::Spi;
use crate::ws2812::prerendered::Ws2812;


use smart_leds::{SmartLedsWrite, RGB8};
// use rtt_target::{rprintln, rtt_init_print};

pub const LED_NUM: usize = 32;
const BLINK_MSEC: u32 = 200;

pub struct LightPorts<'a> {
    led_data: [RGB8; LED_NUM],
    blink_mask: [bool; LED_NUM],
    ws: Ws2812<'a, Spi<SPI1>>,
    sys_timer: &'a Counter<TIM2, 1000>,
    blink_on: bool,
    blink_next: Instant<u32, 1, 1000>,
}

impl <'a> LightPorts<'a> {
    pub fn new(
        pa5: Pin<'A', 5>,
        pa7: Pin<'A', 7>,
        spi: SPI1,
        buffer: &'a mut [u8; (LED_NUM * 12) + 30],
        clocks: &Clocks,
        sys_timer: &'a Counter<TIM2, 1000>,
    ) -> Self {
        // SPI1 with 3Mhz
        let spi: Spi<SPI1> = Spi::new(
            spi,
            (pa5.into_alternate(), NoPin::new(), pa7.into_alternate()),
            ws2812::MODE,
            3_000_000.Hz(),
            clocks,
        );

        let data = [RGB8::new(0x00, 0x00, 0x00); LED_NUM];

        // Create Ws2812 instance with the mutable reference to the buffer
        let ws = Ws2812::new(spi, buffer);

        // Return the LightPorts instance
        Self {
            led_data: data,
            blink_mask: [false; LED_NUM],
            blink_on: false,
            blink_next: sys_timer.now(),
            ws,
            sys_timer
        }
    }

    fn get_next_blink(&self) -> Instant<u32, 1, 1000> {
        self.sys_timer.now() + BLINK_MSEC.millis()
    }

    pub fn set_blade(&mut self, blade: u8, color: RGB8, blink: bool) -> Result<(), &'static str>{
        let blade = blade as usize;
        if blade >= LED_NUM {
            return Err("blade index out of range")
        }

        self.led_data[blade] = color;
        self.blink_mask[blade] = blink;

        Ok(())
    }

    pub fn refresh(&mut self, updated: bool)  {

        let mut updated = updated;
        if self.sys_timer.now() > self.blink_next {
            self.blink_next = self.get_next_blink();
            self.blink_on ^= true;
            updated = true;
        }

        if !updated {
            return;
        }

        let mut current_leds = self.led_data.clone();
        if self.blink_on == false {
            for i in 0..LED_NUM {
                if self.blink_mask[i] == true {
                    current_leds[i] = RGB8::default();
                }
            }
        }

        self.ws.write(current_leds.iter().cloned()).unwrap();

    }

}
