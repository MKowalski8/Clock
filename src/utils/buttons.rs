use super::Mode;
use core::ops::ControlFlow;

use defmt::info;
use embassy_stm32::gpio::Pull;

use embassy_stm32::peripherals::PA4;

use embassy_stm32::peripherals::PA3;

use embassy_stm32::peripherals::PA2;

use embassy_stm32::peripherals::PA1;

use embassy_stm32::gpio::Input;
use embassy_time::Timer;

pub const HOLD_TIME: u16 = 2000;
pub const BUTTON_CLICK_TIME: u16 = 150;

pub struct Buttons<'a> {
    main: Input<'a, PA1>,
    down: Input<'a, PA2>,
    up: Input<'a, PA3>,
    exit: Input<'a, PA4>,
}

impl<'a> Buttons<'a> {
    pub fn new(p1: PA1, p2: PA2, p3: PA3, p4: PA4) -> Self {
        Buttons {
            main: Input::new(p1, Pull::Up),
            down: Input::new(p2, Pull::Up),
            up: Input::new(p3, Pull::Up),
            exit: Input::new(p4, Pull::Up),
        }
    }

    pub async fn main_is_low(&self) -> bool {
        if self.main.is_low() {
            Timer::after_millis(150).await;
            return true;
        }
        false
    }

    pub async fn up_is_low(&self) -> bool {
        if self.up.is_low() {
            Timer::after_millis(150).await;
            return true;
        }
        false
    }

    pub async fn down_is_low(&self) -> bool {
        if self.down.is_low() {
            Timer::after_millis(150).await;
            return true;
        }
        false
    }
    pub async fn exit_is_low(&self) -> bool {
        if self.exit.is_low() {
            Timer::after_millis(150).await;
            return true;
        }
        false
    }

    pub async fn any_pin_is_low(&self) -> bool {
        self.main_is_low().await
            || self.exit_is_low().await
            || self.up_is_low().await
            || self.down_is_low().await
    }

    pub async fn button_hold(&self, hold_time: &mut u16, main: bool) -> ControlFlow<()> {
        if main {
            if self.main_is_low().await {
                if let Some(value) = Buttons::hold_handler(hold_time) {
                    return value;
                }
            } else {
                *hold_time = 0;
            }
        } else {
            if self.exit_is_low().await {
                if let Some(value) = Buttons::hold_handler(hold_time) {
                    return value;
                }
            } else {
                *hold_time = 0;
            }
        }

        ControlFlow::Continue(())
    }

    pub(crate) fn hold_handler(hold_time: &mut u16) -> Option<ControlFlow<()>> {
        *hold_time += 150;
        if *hold_time >= HOLD_TIME {
            info! {"Held for = {}", *hold_time};
            return Some(ControlFlow::Break(()));
        }
        None
    }

    pub async fn mode_change<T: Mode>(&self, mode: &mut T, standard: bool) -> bool {
        let mut changed = false;

        if standard {
            if self.up_is_low().await {
                *mode = mode.next();
                info! {"Mode up"};
                changed = true;
            }
            if self.down_is_low().await {
                *mode = mode.prev();
                info! {"Mode down"};
                changed = true;
            }
        } else {
            if self.exit_is_low().await {
                *mode = mode.next();
                info! {"Mode up"};
                changed = true;
            }
            if self.main_is_low().await {
                *mode = mode.prev();
                info! {"Mode down"};
                changed = true;
            }
        }

        changed
    }
}
