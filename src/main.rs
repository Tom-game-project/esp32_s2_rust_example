use esp_idf_svc::hal::prelude::Peripherals;
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
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

const SSID_STR: &'static str = env!("SSID");
const SSID_PASSWORD_STR: &'static str = env!("SSID_PASSWORD");

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();

    let ssid = String::<32>::try_from(SSID_STR).unwrap();
    let password = String::<64>::try_from(SSID_PASSWORD_STR).unwrap();

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

    esp_idf_svc::log::EspLogger::initialize_default();

    while !wifi_driver.is_connected().unwrap(){
        let config = wifi_driver.get_configuration().unwrap();
        println!("Waiting for station {:?}", config);
    }

    let ip_info = wifi_driver.sta_netif().get_ip_info().unwrap();
    log::info!("Wi-Fi connected, IP: {:?}", ip_info.ip);

    let listener = TcpListener::bind("0.0.0.0:8080")?;
    log::info!("TCP server listening on 0.0.0.0:8080");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                log::info!("New connection from {:?}", stream.peer_addr()?);
                let mut buffer = [0; 64];
                match stream.read(&mut buffer) {
                    Ok(bytes_read) => {
                        if let Ok(request_str) = std::str::from_utf8(&buffer[..bytes_read]) {
                            let request = request_str.trim();
                            log::info!("Received: {}", request);

                            if request == "on" {
                                led1.set_high()?;
                                led2.set_low()?;
                                stream.write_all(b"LED ON\n")?;
                                log::info!("LED ON");
                            } else if request == "off" {
                                led1.set_low()?;
                                led2.set_high()?;
                                stream.write_all(b"LED OFF\n")?;
                                log::info!("LED OFF");
                            } else {
                                stream.write_all(b"Invalid command. Use 'on' or 'off'.\n")?;
                                log::warn!("Invalid command received: {}", request);
                            }
                        } else {
                            stream.write_all(b"Invalid UTF-8 sequence\n")?;
                            log::error!("Received invalid UTF-8 sequence");
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to read from stream: {}", e);
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to accept connection: {}", e);
            }
        }
    }

    Ok(())
}
