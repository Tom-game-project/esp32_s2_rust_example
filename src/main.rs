use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::rmt::{config::TransmitConfig, FixedLengthSignal, PinState, Pulse, TxRmtDriver};
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
use std::time::Duration;

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

    // NeoPixel (WS2812B) on GPIO18
    let led_pin = peripherals.pins.gpio18;
    let channel = peripherals.rmt.channel0;
    let config = TransmitConfig::new().clock_divider(1);
    let mut neopixel_tx = TxRmtDriver::new(channel, led_pin, &config)?;

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

                            match request {
                                "on" => {
                                    led1.set_high()?;
                                    led2.set_low()?;
                                    stream.write_all(b"LED ON\n")?;
                                    log::info!("LED ON");
                                },
                                "off" => {
                                    led1.set_low()?;
                                    led2.set_high()?;
                                    stream.write_all(b"LED OFF\n")?;
                                    log::info!("LED OFF");
                                },
                                "red" => {
                                    neopixel(Rgb::new(255, 0, 0), &mut neopixel_tx)?;
                                    stream.write_all(b"NeoPixel RED\n")?;
                                    log::info!("NeoPixel RED");
                                },
                                "green" => {
                                    neopixel(Rgb::new(0, 255, 0), &mut neopixel_tx)?;
                                    stream.write_all(b"NeoPixel GREEN\n")?;
                                    log::info!("NeoPixel GREEN");
                                },
                                "blue" => {
                                    neopixel(Rgb::new(0, 0, 255), &mut neopixel_tx)?;
                                    stream.write_all(b"NeoPixel BLUE\n")?;
                                    log::info!("NeoPixel BLUE");
                                },
                                "neopixel_off" => {
                                    neopixel(Rgb::new(0, 0, 0), &mut neopixel_tx)?;
                                    stream.write_all(b"NeoPixel OFF\n")?;
                                    log::info!("NeoPixel OFF");
                                },
                                _ => {
                                    stream.write_all(b"Invalid command. Use 'on', 'off', 'red', 'green', 'blue', or 'neopixel_off'.\n")?;
                                    log::warn!("Invalid command received: {}", request);
                                }
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

fn neopixel(rgb: Rgb, tx: &mut TxRmtDriver) -> anyhow::Result<()> {
    let color: u32 = rgb.into();
    let ticks_hz = tx.counter_clock()?;

    let (t0h, t0l, t1h, t1l) = (
        Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(350))?,
        Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(800))?,
        Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(700))?,
        Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(600))?,
    );

    let mut signal = FixedLengthSignal::<24>::new();

    for i in (0..24).rev() {
        let p = 2_u32.pow(i);
        let bit: bool = p & color != 0;

        let (high_pulse, low_pulse) = if bit { (t1h, t1l) } else { (t0h, t0l) };

        signal.set(23 - i as usize, &(high_pulse, low_pulse))?;
    }

    tx.start_blocking(&signal)?;

    Ok(())
}

struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl From<Rgb> for u32 {
    /// Convert RGB to u32 color value
    ///
    /// e.g. rgb: (1,2,4)
    /// G R B
    /// 7 0 7 0 7 0
    /// 00000010 00000001 00000100
    fn from(rgb: Rgb) -> Self {
        ((rgb.g as u32) << 16) | ((rgb.r as u32) << 8) | rgb.b as u32
    }
}
