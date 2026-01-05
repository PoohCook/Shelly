use crate::light_ports::LightPorts;
use crate::pallet::{get_temperature, adjust_temperature, get_color_bright};
use smart_leds::RGB8;
use fugit::Instant;
use crate::hal::pac::TIM2;
use crate::hal::timer::Counter;

const NUM_BLADES: usize = 32;

pub enum Effect {
    ShellFire(ShellFireEffect),
    ShellSparkFire(ShellSparkFireEffect),
    ShellSpiral(ShellSpiralEffect),
}

pub struct EffectManager {
    current_effect: Effect,
    effect_index: usize,
    effect_duration_sec: u32,
    effect_start_time: Instant<u32, 1, 1000>,
}

impl EffectManager {
    pub fn new(sys_timer: &Counter<TIM2, 1000>) -> Self {
        Self {
            //current_effect: Effect::ShellFire(ShellFireEffect::new(120, 60)),
            current_effect: Effect::ShellSparkFire(ShellSparkFireEffect::new(100, 50)),
            //current_effect: Effect::ShellSpiral(ShellSpiralEffect::new(80, 50)),
            effect_index: 2,
            effect_duration_sec: 60,
            effect_start_time: sys_timer.now(),
        }
    }

    pub fn update(&mut self, lights: &mut LightPorts, sys_timer: &Counter<TIM2, 1000>) -> bool {
        // Check if we should switch to the next effect
        let elapsed = (sys_timer.now() - self.effect_start_time).to_millis() / 1000;
        if elapsed >= self.effect_duration_sec {
            self.clear_all_blades(lights);
            self.next_effect(sys_timer);
        }

        // Run the current effect
        match &mut self.current_effect {
            Effect::ShellFire(effect) => effect.update(lights, sys_timer),
            Effect::ShellSparkFire(effect) => effect.update(lights, sys_timer),
            Effect::ShellSpiral(effect) => effect.update(lights, sys_timer),
        }
    }

    fn next_effect(&mut self, sys_timer: &Counter<TIM2, 1000>) {
        self.effect_index = (self.effect_index + 1) % 3;

        self.current_effect = match self.effect_index {
            0 => Effect::ShellFire(ShellFireEffect::new(120, 60)),
            1 => Effect::ShellSpiral(ShellSpiralEffect::new(80, 50)),
            _ => Effect::ShellSparkFire(ShellSparkFireEffect::new(100, 50)),
        };

        self.effect_start_time = sys_timer.now();
    }

    fn clear_all_blades(&self, lights: &mut LightPorts) {
        for blade in 0..NUM_BLADES {
            let _ = lights.set_blade(blade as u8, RGB8::new(0, 0, 0), false);
        }
    }
}

// Shell Spark Fire Effect
pub struct ShellSparkFireEffect {
    temperatures: [u8; NUM_BLADES],
    brightness: u8,
    delay_ms: u32,
    last_update: Instant<u32, 1, 1000>,
    spark_odds: u32,
    random_state: u32,
}

impl ShellSparkFireEffect {
    pub fn new(brightness: u8, delay_ms: u32) -> Self {
        let temps = [0u8; NUM_BLADES];

        Self {
            temperatures: temps,
            brightness,
            delay_ms,
            last_update: Instant::<u32, 1, 1000>::from_ticks(0),
            spark_odds: 30,
            random_state: 0x12345678,  // Initial seed
        }
    }

    pub fn update(&mut self, lights: &mut LightPorts, sys_timer: &Counter<TIM2, 1000>) -> bool {
        let now = sys_timer.now();

        if (now - self.last_update).to_millis() < self.delay_ms {
            return false;
        }

        self.last_update = now;

        // Update random state with LFSR and mix in timer
        self.random_state = self.random_state.wrapping_mul(1664525).wrapping_add(1013904223).wrapping_add(now.ticks());

        // Random spark at position 0
        let rand_val = self.random_state % self.spark_odds;
        if rand_val == 0 {
            // Use a different part of random state for tint selection
            let tint = ((self.random_state >> 16) % 6) as u8 + 1;
            self.temperatures[0] = get_temperature(tint, 15);
        }

        // Output colors for all blades (1-indexed in Arduino, 0-indexed here)
        for blade in 0..NUM_BLADES {
            let color = get_color_bright(self.temperatures[blade], self.brightness);
            let _ = lights.set_blade(blade as u8, color, false);
        }

        // Animate colors - shift down and decay
        for blade in (1..NUM_BLADES).rev() {
            self.temperatures[blade] = self.temperatures[blade - 1];
        }
        self.temperatures[0] = adjust_temperature(self.temperatures[0], -1);

        true
    }
}

// Shell Fire Effect (different from spark fire - uses temperature gradient)
pub struct ShellFireEffect {
    temperatures: [u8; NUM_BLADES],
    brightness: u8,
    delay_ms: u32,
    last_update: Instant<u32, 1, 1000>,
    fire_beat: u32,
    fire_spark_odds: u32,
    random_state: u32,
}

impl ShellFireEffect {
    pub fn new(brightness: u8, delay_ms: u32) -> Self {
        // Initialize with hot core at lower blades
        let mut temps = [0u8; NUM_BLADES];
        // Set initial base heat in lower blades
        for i in 0..10 {
            temps[i] = 6 - (i / 2) as u8;
        }

        Self {
            temperatures: temps,
            brightness,
            delay_ms,
            last_update: Instant::<u32, 1, 1000>::from_ticks(0),
            fire_beat: 0,
            fire_spark_odds: 8,
            random_state: 0xDEADBEEF,
        }
    }

    fn get_rand_temperature_color(&self, temperature: u8, major_flicker: f32, now_ticks: u32) -> RGB8 {
        // Return black for temperature 0
        if temperature == 0 {
            return RGB8::new(0, 0, 0);
        }

        // Use ticks for pseudo-random
        let rand = ((now_ticks % 20) as f32) / 20.0;
        let flicker = self.brightness as f32 * rand * major_flicker;

        let (r_level, g_level, b_level) = match temperature {
            1 => (1.0, 0.1, 0.0),
            2 => (1.0, 0.2, 0.0),
            3 => (1.0, 0.3, 0.0),
            4 => (1.0, 0.4, 0.0),
            5 => (1.0, 0.5, 0.0),
            6 => (1.0, 0.6, 0.05),
            7 => (1.0, 0.7, 0.1),
            8 => (1.0, 0.8, 0.15),
            9 => (1.0, 0.9, 0.2),
            _ => (1.0, 1.0, 0.25),
        };

        RGB8::new(
            (r_level * flicker) as u8,
            (g_level * flicker) as u8,
            (b_level * flicker) as u8,
        )
    }

    pub fn update(&mut self, lights: &mut LightPorts, sys_timer: &Counter<TIM2, 1000>) -> bool {
        let now = sys_timer.now();

        if (now - self.last_update).to_millis() < self.delay_ms {
            return false;
        }

        self.last_update = now;
        self.fire_beat += 1;

        // Update random state
        self.random_state = self.random_state.wrapping_mul(1664525).wrapping_add(1013904223).wrapping_add(now.ticks());

        // Animate - heat rises and diminishes as it goes up
        if self.fire_beat % 1 == 0 {
            // Shift temperatures upward (from low blade numbers to high)
            // Process from high to low to avoid overwriting
            for blade in (1..NUM_BLADES).rev() {
                // Temperature from below - only decay every 3 blades so sparks travel further
                if self.temperatures[blade - 1] > 0 {
                    // Decay based on blade position (every 3rd blade loses 1 temperature)
                    let decay = if blade % 3 == 0 { 1 } else { 0 };
                    self.temperatures[blade] = if self.temperatures[blade - 1] > decay {
                        self.temperatures[blade - 1] - decay
                    } else {
                        0
                    };
                } else {
                    // Natural cooling for blades with no heat from below
                    self.temperatures[blade] = if self.temperatures[blade] > 0 {
                        self.temperatures[blade] - 1
                    } else {
                        0
                    };
                }
            }

            // Random spark - can occur anywhere in first third of blades (hot core)
            let rand_val = self.random_state % self.fire_spark_odds;
            if rand_val == 0 {
                let spark_pos = ((self.random_state >> 8) % (NUM_BLADES / 3) as u32) as usize;
                let spark_temp = (((self.random_state >> 16) % 5) + 8) as u8;  // Higher starting temp (8-12)
                self.temperatures[spark_pos] = spark_temp;
            }

            // Keep the core hot - always maintain some heat at the base
            let base_heat_val = (self.random_state >> 20) % 3;
            if base_heat_val == 0 {
                let base_pos = ((self.random_state >> 12) % 5) as usize;
                if self.temperatures[base_pos] < 5 {
                    self.temperatures[base_pos] = 5 + ((self.random_state >> 18) % 3) as u8;
                }
            }
        }

        // Flicker the fire
        for blade in 0..NUM_BLADES {
            let flicker_seed = self.random_state.wrapping_add(blade as u32 * 7919);
            let flicker_val = ((flicker_seed % 20) as f32) / 20.0;
            let color = self.get_rand_temperature_color(
                self.temperatures[blade],
                flicker_val,
                flicker_seed,
            );
            let _ = lights.set_blade(blade as u8, color, false);
        }

        true
    }
}

// Shell Spiral Effect
pub struct ShellSpiralEffect {
    spiral_index: usize,
    brightness: u8,
    delay_ms: u32,
    last_update: Instant<u32, 1, 1000>,
    cur_color: usize,
    cur_band_cnt: usize,
    color_band_size: usize,
    last_color: Option<RGB8>,
}

impl ShellSpiralEffect {
    pub fn new(brightness: u8, delay_ms: u32) -> Self {
        Self {
            spiral_index: 0,
            brightness,
            delay_ms,
            last_update: Instant::<u32, 1, 1000>::from_ticks(0),
            cur_color: 0,
            cur_band_cnt: 0,
            color_band_size: NUM_BLADES,  // One full cycle before changing color
            last_color: None,
        }
    }

    fn get_next_color(&mut self) -> RGB8 {
        let b = self.brightness;
        let color = match self.cur_color {
            0 => RGB8::new(b, 0, 0),
            1 => RGB8::new(0, b, 0),
            2 => RGB8::new(0, 0, b),
            3 => RGB8::new(b, 0, b),
            4 => RGB8::new(b, b, 0),
            5 => RGB8::new(0, b, b),
            _ => RGB8::new(0, 0, b),
        };

        // After returning current color, increment and check if we should change for next time
        self.cur_band_cnt += 1;
        if self.cur_band_cnt >= self.color_band_size {
            self.cur_color = (self.cur_color + 1) % 6;
            self.cur_band_cnt = 0;
        }

        color
    }

    pub fn update(&mut self, lights: &mut LightPorts, sys_timer: &Counter<TIM2, 1000>) -> bool {
        let now = sys_timer.now();

        if (now - self.last_update).to_millis() < self.delay_ms {
            return false;
        }

        self.last_update = now;

        let this_color = self.get_next_color();

        // Set current blade to color
        let _ = lights.set_blade(self.spiral_index as u8, this_color, false);

        // Move to next blade
        self.spiral_index = (self.spiral_index + 1) % NUM_BLADES;

        self.last_color = Some(this_color);

        true
    }
}
