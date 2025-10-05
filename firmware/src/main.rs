#![no_std]
#![no_main]

use embedded_hal::delay::DelayNs;
use panic_halt as _;

use attiny_hal::{
    self as hal, clock,
    simple_pwm::{IntoPwmPin, Prescaler, Timer0Pwm},
};
use hal::delay::Delay;

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

    // let max_duty: u8 = (pwm.get_max_duty() as f32 / 100.0 * MAX_POWER_PERCENT) as u8;

    let mut delay: Delay<CoreClockFrequency> = hal::delay::Delay::new();

    let inc_button = pins.pb3.into_pull_up_input();
    let dec_button = pins.pb4.into_pull_up_input();

    let mut duty: u8 = 0;

    loop {
        if inc_button.is_low() {
            duty += 1;
        } else if dec_button.is_low() {
            duty -= 1;
        }

        pwm.set_duty(duty);
        delay.delay_ms(10);
    }
}
