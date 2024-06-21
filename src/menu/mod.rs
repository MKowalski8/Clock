use core::cmp::min;
use core::ops::ControlFlow;

use defmt::*;
use embassy_stm32::peripherals::I2C1;
use embassy_stm32::dma::NoDma;
use embassy_stm32::peripherals::{PA7, PB0, PA5};
use ds1307::{DateTimeAccess, Datelike, Ds1307, Error, NaiveDate, NaiveDateTime, Timelike};
use embassy_stm32::gpio::Output;
use embassy_time::Timer;
use embassy_stm32::i2c::I2c;
use max7219::connectors::PinConnector;
use max7219::MAX7219;

use crate::utils::buttons::BUTTON_CLICK_TIME;
use crate::utils::symbols::BLANK;
use crate::utils::{self, alarm::Alarm, buttons::Buttons};
use crate::utils::Mode;
use crate::utils::matrix_display::MatrixDisplay;
use crate::clock::{self};

mod menu_utils;
use menu_utils::*;

const ANIMATION_TIME: u16 = 200;
const DISPLAY_TIME: u16 = 600;
const BLINK_TIME: u16 = 300;

enum MenuMode {
    SetHour,
    SetDate,
    SetAlarm,
}

impl Mode for MenuMode {
    fn next(&self) -> Self {
        match self {
            MenuMode::SetHour => MenuMode::SetDate,
            MenuMode::SetDate => MenuMode::SetAlarm,
            MenuMode::SetAlarm => MenuMode::SetHour,
        }
    }

    fn prev(&self) -> Self {
        match self {
            MenuMode::SetHour => MenuMode::SetAlarm,
            MenuMode::SetDate => MenuMode::SetHour,
            MenuMode::SetAlarm => MenuMode::SetDate,
        }
    }
        
}


pub async fn main_menu<'a> (
    rtc: &mut Ds1307<I2c<'a, I2C1, NoDma, NoDma>>, 
    display: &mut MAX7219<PinConnector<Output<'a, PA7>, Output<'a, PB0>, Output<'a, PA5>>>,
    buttons: &utils::buttons::Buttons<'a>,
    alarm: &mut utils::alarm::Alarm<'a>,)
{         
    info!{"Menu"}
    let mut mode: MenuMode = MenuMode::SetHour;

    let mut matrices = MatrixDisplay::new();


    display_menu(&mut matrices); 

    if let Err(_) = utils::set_display_intensity(display, 5) {matrices.set_error();};
           
    matrices.display_update(display);

    Timer::after_millis(1500).await;

    let mut hold_time = 0;
    let mut ticks = 0;
    loop {
        if let ControlFlow::Break(_) = buttons.button_hold(&mut hold_time, false).await  {break;}

        if buttons.mode_change(&mut mode, true).await { ticks = 0; }

        if hold_time <= BUTTON_CLICK_TIME { 
            ticks = animation_ticks_set(ticks, &mode);
        } else {
            ticks = 0;
        }

        match mode {
            MenuMode::SetHour => {
                display_menu_time(&mut matrices, &ticks);
                if buttons.main_is_low().await {
                    if let Err(_) = set_time(rtc, display, &buttons, &mut matrices).await {matrices.set_error();};
                }
            }
            MenuMode::SetDate => { 
                display_menu_date(&mut matrices, &ticks);
                if buttons.main_is_low().await   {
                    if let Err(_) = set_date(rtc, display, &buttons, &mut matrices).await {matrices.set_error();};
                }
            }
            MenuMode::SetAlarm => {
                display_menu_alarm(&mut matrices, &ticks);
                if buttons.main_is_low().await  {
                    if let Err(_) = set_alarm(alarm, display, &buttons, &mut matrices).await {matrices.set_error();};
                }                
            }
        }
        
       matrices.display_update(display);
    }

    if let Err(_) = utils::set_display_intensity(display, 3) {matrices.set_error();};
}

async fn set_time<'a> (
    rtc: &mut Ds1307<I2c<'a, I2C1, NoDma, NoDma>>,
    display: &mut MAX7219<PinConnector<Output<'a, PA7>, Output<'a, PB0>, Output<'a, PA5>>>,
    buttons: &utils::buttons::Buttons<'a>,
    matrices: &mut MatrixDisplay,
) -> Result<(), Error<embassy_stm32::i2c::Error>>{
    let mut datetime = rtc.datetime()?;

    let mut hour = datetime.hour();
    let mut minute = datetime.minute();
    
    let mut setting_step = SettingTime::Hour;


    let mut hold_time_main = 0;
    let mut hold_time_exit = 0;

    let mut ticks = 0;

    loop {   
        if hold_time_main <= BUTTON_CLICK_TIME && hold_time_exit <= BUTTON_CLICK_TIME {
            if buttons.mode_change( &mut setting_step, false).await {ticks = 0;}

            (hour, minute) = setting_time(buttons, &setting_step, hour, minute, &mut ticks).await;
            datetime = datetime.with_hour(hour).unwrap().with_minute(minute).unwrap();
        }  else {
            ticks = 0;
        }



        blink_display(setting_step, datetime, matrices, ticks, display);

        if let ControlFlow::Break(_) = buttons.button_hold(&mut hold_time_main, false).await {break;}
       
        if let ControlFlow::Break(_) = buttons.button_hold(&mut hold_time_exit, true).await {
                rtc.set_datetime(&datetime)?;

            break;
        }

        ticks = (ticks + 2) % DISPLAY_TIME;
    }

    display_menu(matrices);
    matrices.display_update(display);
    Timer::after_millis(1000).await;

    Ok(())
}


async fn set_date<'a> (
    rtc: &mut Ds1307<I2c<'a, I2C1, NoDma, NoDma>>,
    display: &mut MAX7219<PinConnector<Output<'a, PA7>, Output<'a, PB0>, Output<'a, PA5>>>,
    buttons: &utils::buttons::Buttons<'a>,
    matrices: &mut MatrixDisplay,
) -> Result<(), Error<embassy_stm32::i2c::Error>>{
    let mut datetime = rtc.datetime()?;

    let mut day = datetime.day();
    let mut month = datetime.month();
    let mut year = datetime.year();
    
    let mut setting_step = SettingDate::Day;


    let mut hold_time_accept = 0;
    let mut hold_time_exit = 0;

    let mut ticks = 0;

    loop {   
        if hold_time_accept <= BUTTON_CLICK_TIME && hold_time_exit <= BUTTON_CLICK_TIME {
            if buttons.mode_change( &mut setting_step, false).await {ticks = 0;}
            
            match setting_step {
                SettingDate::Day | SettingDate::Month => {
                    (day, month) = setting_date(buttons, &setting_step, day, month,  year, &mut ticks).await;
                }
                _  => {
                    year = setting_year(buttons, &setting_step, year, &mut ticks).await;
                }
            }
            datetime = datetime.with_day(day).unwrap()
                .with_month(month).unwrap()
                .with_year(year).unwrap();
        } else {
            ticks = 0;
        }

        blink_display(setting_step, datetime, matrices, ticks, display);

        if let ControlFlow::Break(_) = buttons.button_hold(&mut hold_time_exit, false).await {break;}
       
        if let ControlFlow::Break(_) = buttons.button_hold(&mut hold_time_accept, true).await {
            rtc.set_datetime(&datetime)?;
            break;
        }

        ticks = (ticks + 2) % DISPLAY_TIME;
    }

    display_menu(matrices);
    matrices.display_update(display);
    Timer::after_millis(1000).await;

    Ok(())
}

async fn set_alarm <'a>(
    alarm: &mut Alarm<'a>,
    display: &mut MAX7219<PinConnector<Output<'a, PA7>, Output<'a, PB0>, Output<'a, PA5>>>,
    buttons: &utils::buttons::Buttons<'a>,
    matrices: &mut MatrixDisplay,
) -> Result<(), Error<embassy_stm32::i2c::Error>> {
    let mut hour = alarm.get_hour();
    let mut minute = alarm.get_minute();

    let mut setting_step = SettingTime::Hour;

    let mut hold_time_accept = 0;
    let mut hold_time_exit = 0;

    let mut ticks = 0;
    loop {
        if hold_time_accept <= BUTTON_CLICK_TIME && hold_time_exit <= BUTTON_CLICK_TIME {
            if buttons.mode_change( &mut setting_step, false).await {ticks = 0;};

            (hour, minute) = setting_time(&buttons, &setting_step, hour, minute, &mut ticks).await;         
            
            let datetime = NaiveDate::from_ymd_opt(0, 1, 1)
            .unwrap()
            .and_hms_opt(hour, minute, 00)
            .unwrap();
    
            blink_display(setting_step,  datetime, matrices, ticks, display);
                
        } else {
            ticks = 0;
        }


        if let ControlFlow::Break(_) = buttons.button_hold(&mut hold_time_exit, false).await {
            alarm.enable(false);
            info!{"Alarm disable!"};

            off_display_info(matrices);
            matrices.display_update(display);
            Timer::after_millis(1000).await;

            break;
        }

        if let ControlFlow::Break(_) = buttons.button_hold(&mut hold_time_accept, true).await {
            alarm.enable(true);
            alarm.update_time(hour, minute);

            info!{"Alarm enable!"};

            on_display_info(matrices);
            matrices.display_update(display);
            Timer::after_millis(1000).await;


            break;
        }

        ticks = (ticks + 2) % DISPLAY_TIME
    }

    display_menu(matrices);
    matrices.display_update(display);
    Timer::after_millis(1000).await;
    Ok(())
}
 
async fn setting_time(buttons: &Buttons<'_>, setting_step: &SettingTime, hour: u32, minute: u32, ticks: &mut u16) -> (u32, u32) {
    let mut hour = hour;
    let mut minute = minute;

    if buttons.up_is_low().await {
        match setting_step {
            SettingTime::Hour => {
                hour = (hour + 1) % 24;
            }
            SettingTime::Minute => {
                minute = (minute + 1) % 60;
            }
        }
        *ticks = 0;
    }

    if buttons.down_is_low().await {
        match setting_step {
            SettingTime::Hour => {
                hour = if hour == 0 { 23 } else { hour - 1 };
            }
            SettingTime::Minute => {
                minute = if minute == 0 { 59 } else { minute - 1 };
            }
        }

        *ticks = 0;
    }

    (hour, minute)
}

async fn setting_year(buttons: &Buttons<'_>, setting_step: &SettingDate, year: i32, ticks: &mut u16) -> i32 {
    let mut year = year;

    let mut digits = [
        year / 1000,
        year / 100 % 10,
        year / 10 % 10,
        year % 10,
    ];

    let mut changed = false;

    if buttons.up_is_low().await {
        match setting_step {
            SettingDate::Thousand => digits[0] = (digits[0] + 1) % 3,
            SettingDate::Hundred => digits[1] = if year > 2000 || digits[1] == 9 { 0 } else { digits[1] + 1 },
            SettingDate::Ten => digits[2] = (digits[2] + 1) % 10,
            SettingDate::One => digits[3] = (digits[3] + 1) % 10,
            _ => {}
        }
        changed = true;
    }

    if buttons.down_is_low().await {
        match setting_step {
            SettingDate::Thousand => digits[0] = if digits[0] == 0 { 2 } else { digits[0] - 1 },
            SettingDate::Hundred => {
                if year == 2100 {
                    digits[1] = 0;
                } else if year == 2000 {
                    digits[1] = 1;
                } else {
                    digits[1] = if digits[1] == 0 { 9 } else { digits[1] - 1 };
                }
            }
            SettingDate::Ten => {
                if year >= 2100 {
                    digits[2] = 0;
                } else {
                    digits[2] = if digits[2] == 0 { 9 } else { digits[2] - 1 };
                }
            }
            SettingDate::One => {
                if year >= 2100 {
                    digits[3] = 0;
                } else {
                    digits[3] = if digits[3] == 0 { 9 } else { digits[3] - 1 };
                }
            }
            _ => {}
        }
        changed = true;
    }

    if changed {
        year = digits[0] * 1000 + digits[1] * 100 + digits[2] * 10 + digits[3];
        year = year.min(2100); // Maximum year is 2100
        *ticks = 0;
    }

    year
}

async fn setting_date(buttons: &Buttons<'_>, setting_step: &SettingDate, day: u32, month: u32,  year: i32, ticks: &mut u16) -> (u32, u32) {
    let mut day = day;
    let mut month = month;

    let days = days_in_month(month, year);

    if buttons.up_is_low().await {
        match setting_step {
            SettingDate::Day => {day = (day % days) + 1;}
            SettingDate::Month => {month = (month % 12) + 1;}
            _ => {}
        }

        *ticks = 0;

    }

    if buttons.down_is_low().await {
        match setting_step {
            SettingDate::Day => {day = if day == 1 { days } else { day - 1 };}
            SettingDate::Month => {month = if month == 1 { 12 } else { month - 1 };}
            _ => {}
        }
 
        *ticks = 0;

    }

    // Check if possible value 
    day = min(days_in_month(month, year),day);

    (day, month)
}

fn days_in_month(month: u32, year: i32) -> u32 {
    let days_in_month = match month {
        4 | 6 | 9 | 11 => 30,
        2 => {
        if year % 4 == 0 && year % 100 != 0 || year % 400 == 0 {
                29
            } else {
                28
            }
        }
        1 | 3 | 5 | 7 | 8 | 10 | 12 | _ => 31,
    };
    days_in_month
}

fn blink_display<T: ModeExt + Mode> (setting_step: T, datetime: NaiveDateTime, matrices: &mut MatrixDisplay, ticks: u16, display: &mut MAX7219<PinConnector<Output<'_, PA7>, Output<'_, PB0>, Output<'_, PA5>>>) {
    if setting_step.current_index() < 2 {
        clock::calc_digits(&setting_step.dot_mode(), &datetime, matrices);

    } else {
        clock::calc_digits(&clock::ClockMode::Year, &datetime, matrices);
        matrices.matrix_shift(1);
    }


    if ticks > BLINK_TIME{
        match setting_step.current_index() {
            0 => {
                matrices.first_matrix = BLANK;
                matrices.second_matrix = BLANK;
            },
            1 => {
                matrices.third_matrix = BLANK;
                matrices.fourth_matrix = BLANK;
            },
            2 => matrices.first_matrix = BLANK,
            3 => matrices.second_matrix = BLANK,
            4 => matrices.third_matrix = BLANK,
            5 => matrices.fourth_matrix = BLANK,
            _ => {}
        }
    }

    if setting_step.current_index() < 2 {
        clock::prepare_display(matrices, &setting_step.dot_mode(), true);
    }
        
    matrices.display_update(display);
}
