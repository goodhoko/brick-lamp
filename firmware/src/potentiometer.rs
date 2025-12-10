use panic_halt as _;

use attiny_hal::{
    adc::{AdcChannel, AdcSettings, ClockDivider, ReferenceVoltage},
    pac::ADC,
    port::{
        mode::{Analog, Io},
        Pin, PinOps,
    },
    Adc, Attiny,
};

use crate::CoreClockFrequency;

/// The maximum value this potentiometer can produce. This is dictated by the ADC converter which
/// in ATTiny85 has 10 bits of resolution and thus 2^10-1 is the maximum value.
pub const MAX_VALUE: u16 = 0x3ff;

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

    pub fn measure(&mut self) -> u16 {
        self.pin.analog_read(&mut self.adc)
    }
}
