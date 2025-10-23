#![no_std]
#![no_main]

use core::array::from_fn;

use embedded_hal::delay::DelayNs;
use panic_halt as _;

use attiny_hal::{
    self as hal,
    adc::{AdcChannel, AdcSettings, ClockDivider, ReferenceVoltage},
    clock,
    pac::ADC,
    port::{
        mode::{Analog, Io},
        Pin, PinOps,
    },
    simple_pwm::{IntoPwmPin, Prescaler, Timer0Pwm},
    Adc, Attiny,
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

    let mut potentiometer = Potentiometer::new(pins.pb3, dp.ADC);

    let mut delay: Delay<CoreClockFrequency> = hal::delay::Delay::new();

    loop {
        let input = potentiometer.measure();
        pwm.set_duty(input);
        delay.delay_ms(10);
    }
}

struct Potentiometer<P> {
    pin: Pin<Analog, P>,
    adc: Adc<CoreClockFrequency>,
}

impl<P> Potentiometer<P>
where
    P: PinOps,
    Pin<Analog, P>: AdcChannel<Attiny, ADC>,
{
    pub fn new<MODE: Io>(pin: Pin<MODE, P>, adc: ADC) -> Self {
        let mut adc = Adc::<CoreClockFrequency>::new(
            adc,
            AdcSettings {
                // For best precision the ADC needs between 50 and 200 kHz. Our system clock is 8MHz.
                // 8 MHZ / 64 = 128 kHz.
                clock_divider: ClockDivider::Factor64,
                // The potentiometer ends are wired to GND and 3.3V a.k.a. Vcc.
                ref_voltage: ReferenceVoltage::AVcc,
            },
        );

        let pin = pin.into_analog_input(&mut adc);

        Self { pin, adc }
    }

    pub fn measure(&mut self) -> u8 {
        let mut delay: Delay<CoreClockFrequency> = hal::delay::Delay::new();
        // Measure 5 samples.
        let mut samples: [u16; 5] = from_fn(|_| {
            delay.delay_ms(10);
            self.pin.analog_read(&mut self.adc)
        });

        samples.as_mut_slice().sort_unstable();
        // Drop the max and min samples and average the rest
        let avg = samples[1..4].iter().sum::<u16>() / 3;

        // The ADC has 10 bits of precision but our PWM has only 8bits.
        // Convert by simply flooring to u8;
        (avg >> 2) as u8
    }
}
