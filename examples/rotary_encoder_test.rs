#![allow(incomplete_features)]
use adafruit_seesaw::{
    devices::{RotaryEncoder, RotaryEncoderColor},
    prelude::*,
};
use embedded_hal_bus::i2c::MutexDevice;
use esp_idf_hal::{
    delay::Delay,
    gpio::PinDriver,
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    prelude::*,
};
use log::*;
use std::{sync::Mutex, time::Duration};

fn main() -> anyhow::Result<(), anyhow::Error> {
    esp_idf_hal::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    // System
    let peripherals = Peripherals::take().unwrap();
    let mut i2c_power = PinDriver::output(peripherals.pins.gpio7)?;
    i2c_power.set_low()?;

    // I2C
    let (sda, scl) = (peripherals.pins.gpio3, peripherals.pins.gpio4);
    let config = I2cConfig::new().baudrate(400u32.kHz().into());
    let delay = Delay::new_default();
    let i2c = I2cDriver::<'static>::new(peripherals.i2c0, sda, scl, &config)?;
    i2c_power.set_high()?;
    std::thread::sleep(Duration::from_millis(50));
    let i2c = Mutex::new(i2c);
    let encoder_driver_1 =
        SeesawDriver::new(delay.clone(), MutexDevice::new(&i2c));

    let mut encoder = RotaryEncoder::new(0x3a, encoder_driver_1)
        .init()
        .expect("Failed to start RotaryEncoder 1");

    info!(
        "Capabilities {:#?}",
        encoder.capabilities().expect("Failed to get options")
    );

    info!("Looping...");
    let mut prev_position = 0;
    loop {
        let position = encoder.position(0).expect("Failed to get position");
        let c = color_wheel(((position & 0xFF) as u8).wrapping_mul(3));
        if position != prev_position {
            prev_position = position;
            info!("Position changed to {}, new color is {:?}", position, c);
        }

        encoder
            .set_neopixel_color(c)
            .and_then(|_| encoder.sync_neopixel())
            .expect("Failed to set neopixel");

        if let Ok(true) = encoder.button(0) {
            info!("Button pressed");
            encoder.set_position(0, 0).expect("Failed to set position");
        }
    }
}

fn color_wheel(byte: u8) -> RotaryEncoderColor {
    match byte {
        0..=84 => RotaryEncoderColor {
            r: 255 - byte * 3,
            g: 0,
            b: byte * 3,
        },
        85..=169 => RotaryEncoderColor {
            r: 0,
            g: (byte - 85) * 3,
            b: 255 - (byte - 85) * 3,
        },
        _ => RotaryEncoderColor {
            r: (byte - 170) * 3,
            g: 255 - (byte - 170) * 3,
            b: 0,
        },
    }
}
