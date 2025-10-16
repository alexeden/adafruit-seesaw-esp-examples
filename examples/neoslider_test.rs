use adafruit_seesaw::{
    devices::{NeoSlider, NeoSliderColor},
    prelude::*,
    SeesawDriver,
};
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
    i2c_power.set_high()?;
    // i2c_power.set_low()?;
    // std::thread::sleep(Duration::from_millis(50));

    info!("Powering on I2C");

    // I2C
    let (sda, scl) = (peripherals.pins.gpio3, peripherals.pins.gpio4);
    let config = I2cConfig::new().baudrate(100u32.kHz().into());
    let delay = Delay::new_default();
    let i2c = I2cDriver::<'static>::new(peripherals.i2c0, sda, scl, &config)?;
    info!("I2C driver created");
    std::thread::sleep(Duration::from_millis(50));
    let i2c = Mutex::new(i2c);

    let seesaw = SeesawDriver::new(delay, MutexDevice::new(&i2c));
    info!("Seesaw driver created");
    let mut neoslider = NeoSlider::new_with_default_addr(seesaw)
        .init()
        .expect("Failed to start NeoSlider");

    let mut prev_color = color_wheel(0);
    loop {
        let value = neoslider.slider_value().expect("Failed to read slider");
        let color = color_wheel(((value / 3) & 0xFF) as u8);

        neoslider
            .set_neopixel_colors(&[color, color, color, color])
            .and_then(|_| neoslider.sync_neopixel())
            .expect("Failed to set neopixel colors");

        if color != prev_color {
            prev_color = color;
            info!("Color changed to {:?}", color);
        }
    }
}

fn color_wheel(byte: u8) -> NeoSliderColor {
    match byte {
        0..=84 => NeoSliderColor {
            r: 255 - byte * 3,
            g: 0,
            b: byte * 3,
        },
        85..=169 => NeoSliderColor {
            r: 0,
            g: (byte - 85) * 3,
            b: 255 - (byte - 85) * 3,
        },
        _ => NeoSliderColor {
            r: (byte - 170) * 3,
            g: 255 - (byte - 170) * 3,
            b: 0,
        },
    }
}
