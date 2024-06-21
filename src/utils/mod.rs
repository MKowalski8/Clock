use embassy_stm32::gpio::Output;
use embassy_stm32::peripherals::{PA5, PA7, PB0};

use max7219::connectors::PinConnector;
use max7219::{DataError, MAX7219};
pub mod symbols;

pub trait Mode {
    fn next(&self) -> Self;
    fn prev(&self) -> Self;
}

pub mod buttons;
pub mod matrix_display;
pub mod alarm;

pub fn shift_bits(data: &mut [u8], shift: u8) {
    for i in 0..8 {
        data[i] >>= shift;
    }
}

pub fn set_display_intensity(
    display: &mut MAX7219<PinConnector<Output<'_, PA7>, Output<'_, PB0>, Output<'_, PA5>>>,
    intensity: u8,
) -> Result<(), DataError> {
    display.set_intensity(0, intensity)?;
    display.set_intensity(1, intensity)?;
    display.set_intensity(2, intensity)?;
    display.set_intensity(3, intensity)?;

    Ok(())
}
