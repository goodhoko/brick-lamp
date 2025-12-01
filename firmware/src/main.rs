#![no_std]
#![no_main]

use avr_device::interrupt;
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

    // We use a potentiometer with the wiper connected to the PB3 pin as user input for brightness.
    let mut potentiometer = Potentiometer::new(pins.pb3, dp.ADC);

    let pwm_timer = Timer0Pwm::new(dp.TC0, Prescaler::Direct);
    let mut pwm = pins.pb0.into_output().into_pwm(&pwm_timer);
    pwm.set_duty(invert(0));

    // Change the Compare Output Mode register to "compare_match *inverted*" mode to work around
    // a limitation of the "simple PWM" where it can't reach a pure, constant low-level when the
    // duty is set to 0. By inverting the output we can reach constant low-level at duty 255.
    // In exchange, we can't reach a constant high-level but that's OK as we don't want to drive
    // the LEDs at 100% power anyway. This block functionally replaces a call to pwm.enable().
    interrupt::free(|_| {
        // SAFETY: we are in critical section. This should be fine.
        let timer_control_register_a = &unsafe { &*avr_device::attiny85::TC0::ptr() }.tccr0a;
        const SIMPLE_PWM_INVERTED_MODE: u8 = 3;
        timer_control_register_a.modify(|_, w| w.com0a().bits(SIMPLE_PWM_INVERTED_MODE));
    });

    let mut delay: Delay<CoreClockFrequency> = hal::delay::Delay::new();

    loop {
        let input = potentiometer.measure();
        let duty = correct_gamma(input);
        let inverted = invert(duty);
        pwm.set_duty(inverted);
        delay.delay_ms(10);
    }
}

// TODO: use precomputed table and gamma of 2.2 which matches human perception more closely.
/// Integer-only gamma correction with gamma = 2.0
pub fn correct_gamma(input: u8) -> u8 {
    let input = input as u16;
    let max = u8::MAX as u16;

    ((input * input) / max).clamp(0, max) as u8
}

pub fn invert(value: u8) -> u8 {
    u8::MAX - value
}
