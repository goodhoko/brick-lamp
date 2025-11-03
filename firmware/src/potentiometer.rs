use core::array::from_fn;

use embedded_hal::delay::DelayNs;
use panic_halt as _;

use attiny_hal::{
    self as hal,
    adc::{AdcChannel, AdcSettings, ClockDivider, ReferenceVoltage},
    pac::ADC,
    port::{
        mode::{Analog, Io},
        Pin, PinOps,
    },
    Adc, Attiny,
};
use hal::delay::Delay;

use crate::CoreClockFrequency;

/// How many samples to take and average to get a single measurement.
const SAMPLE_COUNT: usize = 5;
/// How long we wait in milliseconds between individual samples.
const SAMPLE_INTERVAL_MS: u32 = 5;

pub struct Potentiometer<P> {
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
        let avg = self.sample();

        // TODO: offset and multiply

        // The ADC has 10 bits of precision but our PWM has only 8bits.
        // Convert by simply flooring to u8;
        (avg >> 2) as u8
    }

    /// Takes SAMPLE_COUNT individual samples and returns their average discarding outliers.
    fn sample(&mut self) -> u16 {
        let mut delay: Delay<CoreClockFrequency> = hal::delay::Delay::new();
        let mut samples: [u16; SAMPLE_COUNT] = from_fn(|_| {
            delay.delay_ms(SAMPLE_INTERVAL_MS);
            self.pin.analog_read(&mut self.adc)
        });

        samples.as_mut_slice().sort_unstable();
        // Drop the max and min samples and average the rest
        samples[1..4].iter().sum::<u16>() / 3
    }
}
