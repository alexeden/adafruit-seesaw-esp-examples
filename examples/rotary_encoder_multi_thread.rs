#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
use adafruit_seesaw::{devices::RotaryEncoder, prelude::*, SeesawStdMutex};
use esp_idf_hal::{
    delay::Delay,
    gpio::PinDriver,
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    prelude::*,
};
use log::*;
use shared_bus::once_cell;
use std::time::Duration;

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
    let i2c = I2cDriver::<'static>::new(peripherals.i2c0, sda, scl, &config)?;
    i2c_power.set_high()?;
    std::thread::sleep(Duration::from_millis(50));

    debug!("Initializing seesaw");
    let seesaw: &'static _ = {
        use once_cell::sync::OnceCell;

        static MANAGER: OnceCell<SeesawStdMutex<(Delay, I2cDriver<'_>)>> =
            OnceCell::new();

        let m = SeesawStdMutex::new(Delay::new_default(), i2c);
        match MANAGER.set(m) {
            Ok(_) => MANAGER.get(),
            Err(_) => None,
        }
    }
    .unwrap();

    debug!("Seesaw initialized");

    let mut encoder =
        RotaryEncoder::new_with_default_addr(seesaw.acquire_driver())
            .init()
            .expect("Failed to start RotaryEncoder");

    loop {
        let position = encoder.position().expect("Failed to get position");
        let c = color_wheel(((position & 0xFF) as u8).wrapping_mul(3));
        let Color(r, g, b) = c.set_brightness(255);

        encoder
            .set_neopixel_color(r, g, b)
            .and_then(|_| encoder.sync_neopixel())
            .expect("Failed to set neopixel");

        if let Ok(true) = encoder.button() {
            encoder.set_position(0).expect("Failed to set position");
        }
    }
}

fn color_wheel(byte: u8) -> Color {
    match byte {
        0..=84 => Color(255 - byte * 3, 0, byte * 3),
        85..=169 => Color(0, (byte - 85) * 3, 255 - (byte - 85) * 3),
        _ => Color((byte - 170) * 3, 255 - (byte - 170) * 3, 0),
    }
}

struct Color(pub u8, pub u8, pub u8);

impl Color {
    pub fn set_brightness(self, brightness: u8) -> Self {
        Self(
            ((self.0 as u16 * brightness as u16) >> 8) as u8,
            ((self.1 as u16 * brightness as u16) >> 8) as u8,
            ((self.2 as u16 * brightness as u16) >> 8) as u8,
        )
    }
}
