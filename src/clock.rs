use core::ops::ControlFlow;
use defmt::*;
use embassy_stm32::dma::NoDma;
use embassy_stm32::peripherals::{I2C1, PA5, PA7, PB0};
use ds1307::{DateTimeAccess, Datelike, Ds1307, NaiveDateTime, Timelike};
use max7219::connectors::PinConnector;
use max7219::{DataError, MAX7219};
use embassy_stm32::gpio::Output;
use embassy_stm32::i2c::I2c;
use crate::utils::{self, alarm::Alarm, buttons::Buttons};
use crate::utils::{matrix_display, symbols};
use crate::utils::Mode;
use crate::utils::matrix_display::MatrixDisplay;

pub enum ClockMode {
    Time,
    Date,
    Year,
}

impl Mode for ClockMode {
    fn next(&self) -> Self {
        match self {
            ClockMode::Time => ClockMode::Date,
            ClockMode::Date => ClockMode::Time,
            ClockMode::Year => ClockMode::Date,
        }
    }

    fn prev(&self) -> Self {
        match self {
            ClockMode::Time => ClockMode::Date,
            ClockMode::Date => ClockMode::Time,
            ClockMode::Year => ClockMode::Date,
        }
    }
}


pub async fn clock_mode<'a>(
    rtc: &mut Ds1307<I2c<'a, I2C1, NoDma, NoDma>>, 
    display: &mut MAX7219<PinConnector<Output<'a, PA7>, Output<'a, PB0>, Output<'a, PA5>>>,
    buttons: &utils::buttons::Buttons<'a>,
    alarm: &mut utils::alarm::Alarm<'a>
) {
    let mut matrices = MatrixDisplay::new();
    let mut mode: ClockMode = ClockMode::Time;

    let mut changed = false;
    let mut is_late = false;
    let mut last_second = 0;

    let mut hold_time_main = 0;
    let mut hold_time_exit = 0;

    info!("Clock");
    loop {
        if let ControlFlow::Break(_) = buttons.button_hold(&mut hold_time_main, true).await { break; }
        if let ControlFlow::Break(_) = buttons.button_hold(&mut hold_time_exit, false).await { break; }

        buttons.mode_change(&mut mode, true).await;

        if let Ok(datetime) = rtc_read(rtc, &mut last_second, &mut changed) {
            if changed {
                calc_digits(&mode, &datetime, &mut matrices);
                prepare_display(&mut matrices, &mode, last_second%2==0);
                if let Err(_) = check_intensity(datetime.hour(), &mut is_late, display) {
                    matrices.set_error();
                }

                check_alarm(alarm, last_second, datetime, buttons, display).await;
        }} else {
            matrices.set_error();
        }
        
       matrices.display_update(display);

    }
}

async fn check_alarm<'a>(alarm: &mut Alarm<'_>, last_second: u32, datetime: NaiveDateTime, buttons: &Buttons<'a>,  display: &mut MAX7219<PinConnector<Output<'a, PA7>, Output<'a, PB0>, Output<'a, PA5>>>) {
    if alarm.is_enable() && last_second == 1 {
        if alarm.get_hour() == datetime.hour() && alarm.get_minute() == datetime.minute() {
            alarm.play_alarm(buttons, display).await;
        }
    }
}

fn check_intensity(hour: u32, is_late: &mut bool, display: &mut MAX7219<PinConnector<Output<'_, PA7>, Output<'_, PB0>, Output<'_, PA5>>>) -> Result<(), DataError>{        
    if *is_late {
        if hour < 23 as u32 && hour >= 6 as u32 {
            info!{"Hello at morning"}
            utils::set_display_intensity(display, 3)?;
            *is_late = false;
        }
    } else {
        if hour >= 23 as u32 || hour < 6 as u32 {
            info!{"Hello at night"}

            utils::set_display_intensity(display, 0)?;
            *is_late = true;
        }
    }
    Ok(())
}

pub fn prepare_display(
    matrices: &mut matrix_display::MatrixDisplay, 
    mode: &ClockMode, 
    is_even: bool,  
) {
    utils::shift_bits(&mut matrices.first_matrix, 1);
    utils::shift_bits(&mut matrices.third_matrix, 2);
    utils::shift_bits(&mut matrices.fourth_matrix, 1);

    add_dots(mode, is_even, &mut matrices.second_matrix, &mut matrices.third_matrix);
}

pub fn rtc_read(
    rtc: &mut Ds1307<I2c<'_, I2C1, NoDma, NoDma>>, 
    last_second: &mut u32, 
    changed: &mut bool
) -> Result<NaiveDateTime, ()> {
    match rtc.datetime() {
        Ok(datetime) => {
            if datetime.second() != *last_second {
                *last_second = datetime.second();
                *changed = true;
                
                if let Some(hour) = time_change(&datetime){
                    rtc.set_datetime(&datetime.with_hour(hour).unwrap()).unwrap();
                }
            } 
            Ok(datetime)
        }
        Err(_) => {
            info!("RTC read failed!");
            Err(())
        }
    }
}



pub fn time_change(datetime: &NaiveDateTime) -> Option<u32> {
    let mut to_ret = None;
    
    if datetime.weekday().num_days_from_sunday() == 0 {
        if datetime.day() > 24  {
            if datetime.month() == 10 && datetime.hour() == 3 {
                to_ret = Some(2) ;
            } else if datetime.month() == 3 && datetime.hour() == 2{
                to_ret = Some(3);
            }
        }
    }

    to_ret
}

pub fn calc_digits(
    mode: &ClockMode, 
    datetime: &NaiveDateTime, 
    matrices: &mut matrix_display::MatrixDisplay
) {
    let (first_digit, second_digit, third_digit, fourth_digit) = match mode {
        ClockMode::Time => (
            (datetime.hour() / 10) as usize,
            (datetime.hour() % 10) as usize,
            (datetime.minute() / 10) as usize,
            (datetime.minute() % 10) as usize,
        ),
        ClockMode::Date => (
            (datetime.day() / 10) as usize,
            (datetime.day() % 10) as usize,
            (datetime.month() / 10) as usize,
            (datetime.month() % 10) as usize,
        ),

        ClockMode::Year => (
            (datetime.year() / 1000) as usize,
            (datetime.year() / 100 % 10) as usize,
            (datetime.year() / 10 % 10) as usize,
            (datetime.year() % 10) as usize,
        ),
    };

    matrices.first_matrix = symbols::DIGITS[first_digit];
    matrices.second_matrix = symbols::DIGITS[second_digit];
    matrices.third_matrix = symbols::DIGITS[third_digit];
    matrices.fourth_matrix = symbols::DIGITS[fourth_digit];
}

pub fn add_dots(mode: &ClockMode, is_even: bool, matrix_one: &mut [u8; 8], matrix_two: &mut [u8; 8]) {
    if is_even{
        match mode {
            ClockMode::Time => {
                    matrix_one[1] ^= 1;
                    matrix_one[2] ^= 1;
                    matrix_one[4] ^= 1;
                    matrix_one[5] ^= 1;
            
                    matrix_two[1] ^= 128;
                    matrix_two[2] ^= 128;
                    matrix_two[4] ^= 128;
                    matrix_two[5] ^= 128;
            }
            ClockMode::Year =>  {
    
            }   
            _ => {         
                matrix_one[5] ^= 1;
                matrix_one[6] ^= 1;
    
                matrix_two[5] ^= 128;
                matrix_two[6] ^= 128;}
        }   
    }

}
