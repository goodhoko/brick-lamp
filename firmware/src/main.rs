#![no_std]
#![no_main]

use embedded_hal::delay::DelayNs;
use panic_halt as _;

use attiny_hal::{
    self as hal, clock,
    simple_pwm::{IntoPwmPin, Prescaler, Timer0Pwm},
};
use hal::delay::Delay;

use crate::potentiometer::Potentiometer;

mod potentiometer;

type CoreClockFrequency = clock::MHz8;

/// The maximum power pumped to the LEDs. Lower than 100 for longevity.
// const MAX_POWER_PERCENT: f32 = 90.0;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = hal::Peripherals::take().unwrap();
    let pins = hal::pins!(dp);

    let pwm_timer = Timer0Pwm::new(dp.TC0, Prescaler::Direct);
    let mut pwm = pins.pb0.into_output().into_pwm(&pwm_timer);
    pwm.set_duty(0);
    pwm.enable();

    let mut potentiometer = Potentiometer::new(pins.pb3, dp.ADC);

    let mut delay: Delay<CoreClockFrequency> = hal::delay::Delay::new();

    loop {
        let input = potentiometer.measure();
        let duty = correct_gamma(input);
        pwm.set_duty(duty);
        delay.delay_ms(10);
    }
}

/// Integer-only gamma correction with gamma = 2.0
pub fn correct_gamma(input: u8) -> u8 {
    let input = input as u16;
    let max = u8::MAX as u16;

    ((input * input) / max).clamp(0, max) as u8
}
