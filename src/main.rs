//use std::thread;
//use std::time::Duration;

//esp_idf_svc::hal::prelude::Periferal;
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::{
    wifi::EspWifi,
    wifi::ClientConfiguration,
    wifi::Configuration,
    nvs::EspDefaultNvsPartition,
    eventloop::EspSystemEventLoop,
};

use heapless::String;
use std::env;

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71

    esp_idf_svc::sys::link_patches();

    let ssid_str: &'static str = env!("SSID");
    let ssid_password_str: &'static str = env!("SSID_PASSWORD");

    let ssid = String::<32>::try_from(ssid_str).unwrap();
    let password = String::<64>::try_from(ssid_password_str).unwrap();

    let peripherals = Peripherals::take()?;

    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let mut wifi_driver = EspWifi::new(
        peripherals.modem,
        sys_loop,
        Some(nvs)
    ).unwrap();

    wifi_driver.set_configuration(&Configuration::Client(ClientConfiguration{
        ssid,
        password,
        ..Default::default()
    })).unwrap();

    wifi_driver.start().unwrap();
    wifi_driver.connect().unwrap();

    let mut led1 = PinDriver::output(peripherals.pins.gpio2)?;
    let mut led2 = PinDriver::output(peripherals.pins.gpio3)?;

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    while !wifi_driver.is_connected().unwrap(){
        let config = wifi_driver.get_configuration().unwrap();
        println!("Waiting for station {:?}", config);
    }
    loop {
        log::info!("Hello, world!");
        println!("IP info: {:?}", wifi_driver.sta_netif().get_ip_info().unwrap());
        led1.set_high()?;
        led2.set_low()?;
        FreeRtos::delay_ms(1000);
        led1.set_low()?;
        led2.set_high()?;
        FreeRtos::delay_ms(1000);
    }
}
