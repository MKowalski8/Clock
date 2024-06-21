use crate::{clock::{self, ClockMode}, utils::{matrix_display::MatrixDisplay, shift_bits, symbols::{self, Letters, DIGITS}, Mode}};
use super::{MenuMode, ANIMATION_TIME, BLANK, };


#[derive(Clone, Copy)]
pub enum SettingTime {
    Hour,
    Minute,
}
#[derive(Clone, Copy)]
pub enum SettingDate {
    Day,
    Month,
    Thousand,
    Hundred,
    Ten,
    One,
}
pub trait ModeExt: Mode {
    fn current_index(&self) -> u8;
    fn dot_mode(&self) -> ClockMode; 
}



impl Mode for SettingTime {
    fn next(&self) -> Self {
        match self {
            SettingTime::Hour => SettingTime::Minute,
            SettingTime::Minute => SettingTime::Hour,
        }
    }

    fn prev(&self) -> Self {
        match self {
            SettingTime::Hour => SettingTime::Minute,
            SettingTime::Minute => SettingTime::Hour,
        }
    }
}

impl ModeExt for SettingTime{
    fn current_index(&self) -> u8 {
        match self {
            SettingTime::Hour => 0,
            SettingTime::Minute => 1,
        }
    }

    fn dot_mode(&self) -> ClockMode{
        ClockMode::Time
    }
}

impl Mode for SettingDate {
    fn next(&self) -> Self {
        match self {
            SettingDate::Day => SettingDate::Month,
            SettingDate::Month => SettingDate::Thousand,
            SettingDate::Thousand => SettingDate::Hundred,
            SettingDate::Hundred => SettingDate::Ten,
            SettingDate::Ten => SettingDate::One,
            SettingDate::One => SettingDate::Day,
        }
    }

    fn prev(&self) -> Self {
        match self {
            SettingDate::Day => SettingDate::One,
            SettingDate::Month => SettingDate::Day,
            SettingDate::Thousand => SettingDate::Month,
            SettingDate::Hundred => SettingDate::Thousand,
            SettingDate::Ten => SettingDate::Hundred,
            SettingDate::One => SettingDate::Ten,
        }
    }
}

impl ModeExt for SettingDate {
    fn current_index(&self) -> u8 {
        match self {
            SettingDate::Day => 0,
            SettingDate::Month => 1,
            SettingDate::Thousand => 2,
            SettingDate::Hundred => 3,
            SettingDate::Ten => 4,
            SettingDate::One => 5,
        }
    }

    fn dot_mode(&self) -> ClockMode{
        match self {
            SettingDate::Day | SettingDate::Month => ClockMode::Date,
            _ => ClockMode::Year
        } 
    }
}


pub fn display_menu_date(matrices: &mut MatrixDisplay, ticks: &u16) {
    let digit = DIGITS[2];

    match *ticks {
        ticks if ticks < ANIMATION_TIME => {
            matrices.first_matrix = digit;
            matrices.second_matrix = Letters::D.bytes();
            shift_bits(&mut matrices.second_matrix, 1);
            clock::add_dots(&clock::ClockMode::Date, true, &mut matrices.first_matrix, &mut matrices.second_matrix);

            matrices.third_matrix = Letters::A.bytes();
            matrices.fourth_matrix = Letters::T.bytes();
        },
        ticks if ticks < ANIMATION_TIME * 2 => {
            matrices.first_matrix = Letters::D.bytes();
            matrices.second_matrix = Letters::A.bytes();
            matrices.third_matrix = Letters::T.bytes();
            matrices.fourth_matrix = Letters::E.bytes();
        },
        ticks if ticks < ANIMATION_TIME * 3 => {
            matrices.first_matrix = Letters::A.bytes();
            matrices.second_matrix = Letters::T.bytes();
            matrices.third_matrix = Letters::E.bytes();
            matrices.fourth_matrix = BLANK;
        },
        ticks if ticks < ANIMATION_TIME * 4 => {
            matrices.first_matrix = Letters::T.bytes();
            matrices.second_matrix = Letters::E.bytes();
            matrices.third_matrix = BLANK;
            matrices.fourth_matrix = digit
        },
        ticks if ticks < ANIMATION_TIME * 5 => {
            matrices.first_matrix = Letters::E.bytes();
            matrices.second_matrix = BLANK;
            matrices.third_matrix = digit;
            matrices.fourth_matrix = Letters::A.bytes();
            shift_bits(&mut matrices.fourth_matrix, 1);
            clock::add_dots(&clock::ClockMode::Date, true, &mut matrices.third_matrix, &mut matrices.fourth_matrix);

        }
        _ => {
            matrices.first_matrix = BLANK;
            matrices.second_matrix = digit;
            matrices.third_matrix = Letters::D.bytes();
            matrices.fourth_matrix = Letters::A.bytes();
            shift_bits(&mut matrices.third_matrix, 1);
            clock::add_dots(&clock::ClockMode::Date, true, &mut matrices.second_matrix, &mut matrices.third_matrix);
        }

    }
}

pub fn display_menu_time(matrices: &mut MatrixDisplay, ticks: &u16) {
    let digit = DIGITS[1];

    match *ticks {
        ticks if ticks < ANIMATION_TIME => {
            matrices.first_matrix = digit;
            matrices.second_matrix = Letters::T.bytes();
            shift_bits(&mut matrices.second_matrix, 1);
            clock::add_dots(&clock::ClockMode::Date, true, &mut matrices.first_matrix, &mut matrices.second_matrix);

            matrices.third_matrix = Letters::I.bytes();
            matrices.fourth_matrix = Letters::M.bytes();
        },
        ticks if ticks < ANIMATION_TIME * 2 => {
            matrices.first_matrix = Letters::T.bytes();
            matrices.second_matrix = Letters::I.bytes();
            matrices.third_matrix = Letters::M.bytes();
            matrices.fourth_matrix = Letters::E.bytes();
        },
        ticks if ticks < ANIMATION_TIME * 3 => {
            matrices.first_matrix = Letters::I.bytes();
            matrices.second_matrix = Letters::M.bytes();
            matrices.third_matrix = Letters::E.bytes();
            matrices.fourth_matrix = BLANK;
        },
        ticks if ticks < ANIMATION_TIME * 4 => {
            matrices.first_matrix = Letters::M.bytes();
            matrices.second_matrix = Letters::E.bytes();
            matrices.third_matrix = BLANK;
            matrices.fourth_matrix = digit;
        },
        ticks if ticks < ANIMATION_TIME * 5 => {
            matrices.first_matrix = Letters::E.bytes();
            matrices.second_matrix = BLANK;
            matrices.third_matrix = digit;
            matrices.fourth_matrix = Letters::T.bytes();
            shift_bits(&mut matrices.fourth_matrix, 1);
            clock::add_dots(&clock::ClockMode::Date, true, &mut matrices.third_matrix, &mut matrices.fourth_matrix);

        }
        _ => {
            matrices.first_matrix = BLANK;
            matrices.second_matrix = digit;
            matrices.third_matrix = Letters::T.bytes();
            matrices.fourth_matrix = Letters::I.bytes();
            shift_bits(&mut matrices.third_matrix, 1);
            clock::add_dots(&clock::ClockMode::Date, true, &mut matrices.second_matrix, &mut matrices.third_matrix);
        }

    }
}

pub fn display_menu_alarm(matrices: &mut MatrixDisplay, ticks: &u16) {
    let digit = DIGITS[3];

    match *ticks {
        ticks if ticks < ANIMATION_TIME => {
            matrices.first_matrix = digit;
            matrices.second_matrix = Letters::A.bytes();
            shift_bits(&mut matrices.second_matrix, 1);
            clock::add_dots(&clock::ClockMode::Date, true, &mut matrices.first_matrix, &mut matrices.second_matrix);

            matrices.third_matrix = Letters::L.bytes();
            matrices.fourth_matrix = Letters::A.bytes();
        },
        ticks if ticks < ANIMATION_TIME * 2 => {
            matrices.first_matrix = Letters::A.bytes();
            matrices.second_matrix = Letters::L.bytes();
            matrices.third_matrix = Letters::A.bytes();
            matrices.fourth_matrix = Letters::R.bytes();
        },
        ticks if ticks < ANIMATION_TIME * 3 => {
            matrices.first_matrix = Letters::L.bytes();
            matrices.second_matrix = Letters::A.bytes();
            matrices.third_matrix = Letters::R.bytes();
            matrices.fourth_matrix = Letters::M.bytes();
        },
        ticks if ticks < ANIMATION_TIME * 4 => {
            matrices.first_matrix = Letters::A.bytes();
            matrices.second_matrix = Letters::R.bytes();
            matrices.third_matrix = Letters::M.bytes();
            matrices.fourth_matrix = BLANK;
        },
        ticks if ticks < ANIMATION_TIME * 5 => {
            matrices.first_matrix = Letters::R.bytes();
            matrices.second_matrix = Letters::M.bytes();
            matrices.third_matrix = BLANK;
            matrices.fourth_matrix = digit
        },
        ticks if ticks < ANIMATION_TIME * 6 => {
            matrices.first_matrix = Letters::M.bytes();
            matrices.second_matrix = BLANK;
            matrices.third_matrix = digit;
            matrices.fourth_matrix = Letters::A.bytes();
            shift_bits(&mut matrices.fourth_matrix, 1);
            clock::add_dots(&clock::ClockMode::Date, true, &mut matrices.third_matrix, &mut matrices.fourth_matrix);

        }
        _ => {
            matrices.first_matrix = BLANK;
            matrices.second_matrix = digit;
            matrices.third_matrix = Letters::A.bytes();
            matrices.fourth_matrix = Letters::L.bytes();
            shift_bits(&mut matrices.third_matrix, 1);
            clock::add_dots(&clock::ClockMode::Date, true, &mut matrices.second_matrix, &mut matrices.third_matrix);
        }

    }
}

pub fn display_menu(matrices: &mut MatrixDisplay) {
    matrices.first_matrix = Letters::M.bytes();
    matrices.second_matrix = Letters::E.bytes();
    matrices.third_matrix = Letters::N.bytes();
    matrices.fourth_matrix = Letters::U.bytes();
}

pub fn animation_ticks_set(ticks_before: u16, mode: &MenuMode) -> u16 {
    let mut ticks = ticks_before+1;

    match mode {
        MenuMode::SetHour => {
            if ticks >= ANIMATION_TIME*6  {
                ticks = 0;
            }  
        }
        MenuMode::SetDate => { 
            if ticks >= ANIMATION_TIME*6  {
                ticks = 0;
            }  

        }
        
        MenuMode::SetAlarm => {
            if ticks >= ANIMATION_TIME*7  {
                ticks = 0;
            }  
        }      
    }

    return ticks;
}

pub fn off_display_info(matrices: &mut MatrixDisplay) {
    matrices.first_matrix = Letters::O.bytes();
    matrices.second_matrix = Letters::F.bytes();
    matrices.third_matrix = Letters::F.bytes();
    matrices.fourth_matrix = symbols::EXCLAMETION_MARK;
}

pub fn on_display_info(matrices: &mut MatrixDisplay) {
    matrices.first_matrix = BLANK;
    matrices.second_matrix = Letters::O.bytes();
    matrices.third_matrix = Letters::N.bytes();
    matrices.fourth_matrix = symbols::EXCLAMETION_MARK;
}
