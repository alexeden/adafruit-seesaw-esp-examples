use adafruit_seesaw::{devices::ArcadeButton1x4, prelude::*, SeesawDriver};
use embedded_hal_bus::i2c::MutexDevice;
use esp_idf_hal::{
    delay::Delay,
    gpio::{PinDriver, Pull},
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    prelude::*,
};
use log::*;
use std::{sync::Mutex, time::Duration};

fn main() -> anyhow::Result<(), anyhow::Error> {
    esp_idf_hal::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    info!("Starting button test");
    // System
    let peripherals = Peripherals::take().unwrap();
    // Hardware Reset
    let mut rst = PinDriver::input(peripherals.pins.gpio13)?;
    rst.set_pull(Pull::Up)?;
    // I2C Power
    let mut i2c_power = PinDriver::output(peripherals.pins.gpio7)?;
    i2c_power.set_low()?;
    std::thread::sleep(Duration::from_millis(50));

    info!("Powering on I2C");

    // I2C
    let (sda, scl) = (peripherals.pins.gpio3, peripherals.pins.gpio4);
    let config = I2cConfig::new().baudrate(400u32.kHz().into());
    let delay = Delay::new_default();
    let i2c = I2cDriver::<'static>::new(peripherals.i2c0, sda, scl, &config)?;
    info!("I2C driver created");
    i2c_power.set_high()?;
    std::thread::sleep(Duration::from_millis(500));
    let i2c = Mutex::new(i2c);

    let seesaw = SeesawDriver::new(delay, MutexDevice::new(&i2c));
    info!("Seesaw driver created");
    let mut arcade = ArcadeButton1x4::new(0x3a, seesaw)
        .init()
        .expect("Failed to start ArcadeButton1x4");
    info!("Arcade button driver created");

    loop {
        let buttons =
            arcade.button_values().expect("Failed to get button values");
        log::info!("Buttons: {:?}", buttons);
        arcade
            .set_led_duty_cycles(
                &buttons.map(|on| if on { 0xFFu8 } else { 0x1F }),
            )
            .expect("Failed to set LED duty cycles");
    }
}
