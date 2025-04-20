//use std::thread;
//use std::time::Duration;

//esp_idf_svc::hal::prelude::Periferal;
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::PinDriver;


fn main() -> anyhow::Result<()>  {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();


    let peripherals = Peripherals::take()?;
    let mut led = PinDriver::output(peripherals.pins.gpio2)?;

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    loop {
        log::info!("Hello, world!");
        led.set_high()?;
        FreeRtos::delay_ms(1000);
        led.set_low()?;
        FreeRtos::delay_ms(1000);
    }
}
