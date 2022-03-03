#![no_std]
#![no_main]

use core::str;
use core::time::Duration;

use panic_halt as _;

use arduino_nano33iot as bsp;
use bsp::entry;
use bsp::hal;
use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::pac::{CorePeripherals, Peripherals};
use hal::prelude::*;

// WiFi
use hal::sercom::v2::spi;
use wifi_nina::transport;

// Acellerometer
use lexical_core as lexical;

mod usb;
use usb::usb_log;

mod gyro;

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let mut core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );
    let pins = bsp::Pins::new(peripherals.PORT);
    let mut led: bsp::Led = pins.led_sck.into();

    unsafe {
        usb::setup_usb(
            &mut clocks,
            peripherals.USB,
            &mut peripherals.PM,
            pins.usb_dm,
            pins.usb_dp,
            &mut core,
        );
    }
    let mut delay = Delay::new(core.SYST, &mut clocks);

    usb_log("usb initialised\n");

    let nina_cs = pins.nina_cs.into_push_pull_output();
    let nina_ack = pins.nina_ack.into_floating_input();
    let nina_rst = pins.nina_resetn.into_push_pull_output();

    let gclk0 = clocks.gclk0();
    let clock = clocks.sercom2_core(&gclk0).unwrap();
    let miso = pins.nina_miso;
    let mosi = pins.nina_mosi;
    let sck = pins.nina_sck;
    let pads = spi::Pads::default().data_in(miso).data_out(mosi).sclk(sck);

    delay.delay_ms(1000u16);

    let spi_master = spi::Config::new(&peripherals.PM, peripherals.SERCOM2, pads, clock.freq())
        .baud(hal::time::MegaHertz(8))
        .spi_mode(spi::MODE_0)
        .enable();

    let transport =
        transport::SpiTransport::start(spi_master, nina_ack, nina_rst, nina_cs, |d: Duration| {
            delay.delay_us(d.as_micros() as u32)
        })
        .unwrap();
    usb_log("transport initialised\n");
    let mut wifi = wifi_nina::Wifi::new(transport);
    usb_log("wifi initialised\n");

    usb_log("scanning networks...\n");
    match wifi.scan_networks() {
        Ok(networks) => {
            usb_log("scanned networks\n");
            for network in networks.flatten() {
                if let Ok(ssid) = str::from_utf8(network.ssid.as_slice()) {
                    usb_log(ssid);
                    usb_log("\n");
                }
            }
        }
        _ => usb_log("failed to scan\n"),
    }

    let mut gyro = gyro::setup_gyro(
        &mut clocks,
        peripherals.SERCOM4,
        &mut peripherals.PM,
        pins.sda,
        pins.scl,
    )
    .unwrap();

    loop {
        // delay.delay_ms(500u16);
        led.toggle().unwrap();
        // while ! gyro.gyro_data_available().unwrap() {}
        while !gyro.accel_data_available().unwrap() {}
        // x-axis is along the length of the Arduino.
        // x-axis plus is away from the usb, y is on the power led side, z is down
        if let Ok((gx, gy, gz)) = gyro.read_accelerometer() {
            let mut buf = [0u8; lexical::BUFFER_SIZE];
            usb_log("\rgyro: ");
            lexical::write(gx, &mut buf);
            usb_log(str::from_utf8(&buf).unwrap());
            usb_log(", ");
            lexical::write(gy, &mut buf);
            usb_log(str::from_utf8(&buf).unwrap());
            usb_log(", ");
            lexical::write(gz, &mut buf);
            usb_log(str::from_utf8(&buf).unwrap());
            usb_log("\n");
        }
    }
}
