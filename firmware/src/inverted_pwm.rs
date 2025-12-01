use attiny_hal::{
    port::{
        mode::{self, Io},
        Pin, PinOps,
    },
    simple_pwm::{IntoPwmPin, PwmPinOps, Timer0Pwm},
};
use avr_device::interrupt;

/// A wrapper around the "simple PWM" implementation from attiny-hal that inverts the PWM output to
/// workaround simple pwm's limitation where it can't reach a pure, constant low-level output when
/// the duty is set to 0. By inverting the output we can reach constant low-level at duty 255. In
/// exchange, we can't reach a constant high-level but that's OK as we don't want to drive the LEDs
/// at 100% power anyway.
///
/// InvertedPwm configures the inverting and exposes `set_duty()` which internally inverts the
/// passed in duty so that users can pass eg. 0 to get a constant low-level as usual.
pub struct InvertedPwm<PIN, TIMER> {
    pwm: Pin<mode::PwmOutput<TIMER>, PIN>,
}

impl<PIN: PwmPinOps<Timer0Pwm> + PinOps> InvertedPwm<PIN, Timer0Pwm> {
    /// Configure and enable the pwm with duty of 0.
    pub fn new<MODE: Io>(timer: Timer0Pwm, pin: Pin<MODE, PIN>) -> Self {
        let mut pwm = pin.into_output().into_pwm(&timer);
        pwm.set_duty(invert(0));

        // This block functionally replaces a call to pwm.enable().
        interrupt::free(|_| {
            // SAFETY: we are in critical section. This should be fine.
            let timer_control_register_a = &unsafe { &*avr_device::attiny85::TC0::ptr() }.tccr0a;
            const SIMPLE_PWM_INVERTED_MODE: u8 = 3;
            timer_control_register_a.modify(|_, w| w.com0a().bits(SIMPLE_PWM_INVERTED_MODE));
        });

        Self { pwm }
    }

    pub fn set_duty(&mut self, duty: u8) {
        self.pwm.set_duty(invert(duty));
    }
}

fn invert(value: u8) -> u8 {
    u8::MAX - value
}
