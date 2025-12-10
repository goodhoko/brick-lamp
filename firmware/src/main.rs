#![no_std]
#![no_main]

use panic_halt as _;

use attiny_hal::{
    self as hal, clock,
    simple_pwm::{Prescaler, Timer0Pwm},
};

use crate::{inverted_pwm::InvertedPwm, potentiometer::Potentiometer};

mod inverted_pwm;
mod potentiometer;

type CoreClockFrequency = clock::MHz8;

// The maximum power pumped to the LEDs. Lower than 100 for longevity.
const MAX_POWER_PERCENT: u8 = 80;

const DITHER_PATTERNS: [[u8; 4]; 4] = [[0, 0, 0, 0], [1, 0, 0, 0], [1, 0, 1, 0], [1, 1, 1, 0]];

#[arduino_hal::entry]
fn main() -> ! {
    let dp = hal::Peripherals::take().unwrap();
    let pins = hal::pins!(dp);

    // We use a potentiometer with the wiper connected to the PB3 pin as user input for brightness.
    let mut potentiometer = Potentiometer::new(pins.pb3, dp.ADC);

    // Use the direct prescaler to achieve highest possible PWM frequency for flicker-free light.
    let pwm_timer = Timer0Pwm::new(dp.TC0, Prescaler::Direct);
    let mut pwm = InvertedPwm::new(pwm_timer, pins.pb0);

    let mut n: usize = 0;
    let mut input: u16 = 0;
    loop {
        let measured = potentiometer.measure();
        if measured > input.saturating_add(1) {
            input = measured - 1;
        } else if measured < input {
            input = measured;
        }

        let gamma_corrected = correct_gamma(input, potentiometer::MAX_VALUE);
        let power_capped = scale(gamma_corrected, MAX_POWER_PERCENT);

        // Temporal dithering
        let base_duty = (power_capped >> 2) as u8;
        let dither_pattern_index = (power_capped % 4) as usize;
        let duty = base_duty + DITHER_PATTERNS[dither_pattern_index][n];

        n = (n + 1) % 4;

        pwm.set_duty(duty);
    }
}

/// Integer-only gamma correction with gamma = 2.0
pub fn correct_gamma(input: u16, max: u16) -> u16 {
    let input = input as u32;
    let max = max as u32;

    ((input * input) / max).clamp(0, max) as u16
}

pub fn scale(value: u16, percent: u8) -> u16 {
    (value as u32 * percent as u32 / 100) as u16
}
