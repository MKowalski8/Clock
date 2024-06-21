#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use ds1307::Ds1307;
use embassy_stm32::dma::NoDma;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::i2c::I2c;
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, i2c, peripherals};
use embassy_time::Timer;
use max7219::*;
use utils::{set_display_intensity, alarm::Alarm};
use {defmt_rtt as _, panic_probe as _};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::time::hz;

mod utils;
mod clock;
mod menu;


bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Init stm32
    let config = Config::default();
    let p = embassy_stm32::init(config);

    //  Giving time for rtc to reset
    Timer::after_millis(100).await;
    

    // Init display
    let cs = Output::new(p.PB0, Level::Low, Speed::Low);
    let clk = Output::new(p.PA5, Level::Low, Speed::Low);
    let din = Output::new(p.PA7, Level::Low, Speed::Low);
    

    let mut display = MAX7219::from_pins(4, din, cs, clk).unwrap();
    
    // Init RTC
    let i2c = I2c::new(p.I2C1, p.PB6, p.PB7, Irqs, NoDma, NoDma,
            Hertz(100_000), Default::default());

    let mut rtc = Ds1307::new(i2c);
    rtc.set_running().unwrap();


    // Init buzzer
    let buzz_pin = PwmPin::new_ch2(p.PA9, embassy_stm32::gpio::OutputType::PushPull);
    let pwm = SimplePwm::new(p.TIM1, None, Some(buzz_pin), None, None, hz(2000), embassy_stm32::timer::CountingMode::EdgeAlignedDown);    
    let mut alarm = Alarm::new(pwm);

    // It is needed for first pwm init 
    alarm.set_volume(1);
    alarm.play_sound(Hertz(440), 0).await; 
    alarm.set_volume(100);

    // Init buttons
    let buttons = utils::buttons::Buttons::new(p.PA1, p.PA2, p.PA3, p.PA4);
    // alarm.play_alarm(&buttons, &mut display).await;

    display.power_on().unwrap();
    let _ = set_display_intensity(&mut display, 4);


    loop {      
        info!("Main");
        clock::clock_mode(&mut rtc, &mut display, &buttons, &mut alarm).await;
        menu::main_menu(&mut rtc, &mut display, &buttons, &mut alarm).await;
    }}



