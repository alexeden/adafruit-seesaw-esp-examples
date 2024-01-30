#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
use adafruit_seesaw::{devices::NeoKey1x4, prelude::*, SeesawRefCell};
use esp_idf_hal::{
    delay::Delay,
    gpio::PinDriver,
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    prelude::*,
};
use log::*;
use std::time::Duration;

const RED: (u8, u8, u8) = (255, 0, 0);
const GREEN: (u8, u8, u8) = (0, 255, 0);

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
    debug!("I2C on!");

    // Seesaw
    debug!("Initializing seesaw");
    let seesaw = SeesawRefCell::new(Delay::new_default(), i2c);
    debug!("Seesaw initialized");

    let mut neokeys = NeoKey1x4::new(0x33, seesaw.acquire_driver())
        .init()
        .expect("Failed to start NeoKey1x4");

    debug!("Neokeys inited");
    loop {
        let keys = neokeys.keys().expect("Failed to read keys");

        neokeys
            .set_neopixel_colors(&[
                if (keys >> 0) & 1 == 0 { GREEN } else { RED },
                if (keys >> 1) & 1 == 0 { GREEN } else { RED },
                if (keys >> 2) & 1 == 0 { GREEN } else { RED },
                if (keys >> 3) & 1 == 0 { GREEN } else { RED },
            ])
            .and_then(|_| neokeys.sync_neopixel())
            .expect("Failed to update neopixels");
    }
}
