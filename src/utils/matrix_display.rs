use super::shift_bits;
use defmt::info;
use embassy_stm32::peripherals::PA5;
use embassy_stm32::peripherals::PB0;
use embassy_stm32::peripherals::PA7;
use embassy_stm32::gpio::Output;
use max7219::connectors::PinConnector;
use max7219::DataError;
use max7219::MAX7219;
use super::symbols;


pub struct MatrixDisplay {
    pub first_matrix: [u8; 8],
    pub second_matrix: [u8; 8],
    pub third_matrix: [u8; 8],
    pub fourth_matrix: [u8; 8],
}

impl MatrixDisplay {
    pub fn new() -> Self {
        Self {
            first_matrix: symbols::BLANK,
            second_matrix: symbols::BLANK,
            third_matrix: symbols::BLANK,
            fourth_matrix: symbols::BLANK,
        }
    }

    pub fn display_update(
        &mut self,
        display: &mut MAX7219<PinConnector<Output<'_, PA7>, Output<'_, PB0>, Output<'_, PA5>>>,
    ) {
        if let Err(_) = self.write_to_display(display) {
            self.set_error();
        }
    }

    pub(crate) fn write_to_display(
        &self,
        display: &mut MAX7219<PinConnector<Output<'_, PA7>, Output<'_, PB0>, Output<'_, PA5>>>,
    ) -> Result<(), DataError> {
        display.write_raw(0, &self.first_matrix)?;
        display.write_raw(1, &self.second_matrix)?;
        display.write_raw(2, &self.third_matrix)?;
        display.write_raw(3, &self.fourth_matrix)?;
        Ok(())
    }

    pub fn set_error(&mut self) {
        self.first_matrix = symbols::Letters::E.bytes();
        self.second_matrix = symbols::Letters::R.bytes();
        self.third_matrix = symbols::Letters::R.bytes();
        self.fourth_matrix = symbols::EXCLAMETION_MARK;
        info! {"Error"};
    }

    pub fn matrix_shift(&mut self, shift: u8) {
        shift_bits(&mut self.first_matrix, shift);
        shift_bits(&mut self.second_matrix, shift);
        shift_bits(&mut self.third_matrix, shift);
        shift_bits(&mut self.fourth_matrix, shift);
    }
}
