use defmt::info;
use embassy_stm32::gpio::Output;
use embassy_stm32::time::Hertz;

use embassy_time::Timer;

use embassy_stm32::timer::Channel;

use embassy_stm32::peripherals::{PA5, PA7, PB0, TIM1};

use embassy_stm32::timer::simple_pwm::SimplePwm;
use max7219::connectors::PinConnector;
use max7219::MAX7219;

pub struct Alarm<'a> {
    hour: u32,
    minute: u32,
    enabled: bool,
    pwm: SimplePwm<'a, TIM1>,
}

impl<'a> Alarm<'a> {
    pub fn new(pwm: SimplePwm<'a, TIM1>) -> Self {
        Alarm {
            hour: 0,
            minute: 0,
            enabled: false,
            pwm,
        }
    }

    pub fn update_time(&mut self, hour: u32, minute: u32) {
        self.hour = hour;
        self.minute = minute;
    }

    pub fn enable(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn get_hour(&self) -> u32 {
        self.hour
    }

    pub fn get_minute(&self) -> u32 {
        self.minute
    }

    pub fn is_enable(&self) -> bool {
        self.enabled
    }

    pub fn set_volume(&mut self, level: u16) {
        let max_duty = self.pwm.get_max_duty() / 2;
        self.pwm.set_duty(Channel::Ch2, max_duty);

        if level <= 100 {
            let duty: u16 = max_duty / 100 * level;

            self.pwm.set_duty(Channel::Ch2, duty);
        } else {
            self.pwm.set_duty(Channel::Ch2, max_duty);
        }
    }

    pub async fn play_alarm<'b>(&mut self, buttons: &super::buttons::Buttons<'b>, display: &mut MAX7219<PinConnector<Output<'b, PA7>, Output<'b, PB0>, Output<'b, PA5>>>) {
        let sound = super::symbols::FREQUENCIES[21].1;
        let buzz_length = 100;

        info! {"Alarm!!!"};
        let mut ticks = 0;
        loop {

            for _ in 0..3 {
                if self.check_off_and_play(buttons, sound, buzz_length, display).await {
                    display.power_on().unwrap();

                    return;
                }
            }
            for _ in 0..3 {
                Timer::after_millis(50).await;
                if buttons.any_pin_is_low().await {
                    display.power_on().unwrap();

                    return;
                }
            } 

            ticks += 1;
            if ticks == 1000 {
                break;
            }
        }
    }

    pub async fn check_off_and_play<'b>(
        &mut self,
        buttons: &super::buttons::Buttons<'b>,
        sound: Hertz,
        buzz_length: u64,
        display: &mut MAX7219<PinConnector<Output<'b, PA7>, Output<'b, PB0>, Output<'b, PA5>>>
    ) -> bool {
        display.power_on().unwrap();
        self.play_sound(sound, buzz_length).await;
        display.power_off().unwrap();

        Timer::after_millis(50).await;
        buttons.any_pin_is_low().await
    }

    pub async fn play_sound(&mut self, sound: Hertz, buzz_length: u64) {
        self.pwm.set_frequency(sound);
        self.pwm.enable(Channel::Ch2);
        Timer::after_millis(buzz_length).await;
        self.pwm.disable(Channel::Ch2);
    }
}
