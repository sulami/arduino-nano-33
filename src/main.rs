#![no_std]
#![no_main]

use core::time::Duration;

use panic_halt as _;

use arduino_nano33iot as bsp;
use bsp::entry;
use bsp::hal;
use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::pac::{CorePeripherals, Peripherals};
use hal::prelude::*;

mod gyro;
mod orientation;
mod usb;
use usb::usb_log;
mod wifi;

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
    // let mut led: bsp::Led = pins.led_sck.into();

    let mut gyro = gyro::setup_gyro(
        &mut clocks,
        peripherals.SERCOM4,
        &mut peripherals.PM,
        pins.sda,
        pins.scl,
    )
    .unwrap();

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

    let gclk0 = clocks.gclk0();
    let clock = clocks.sercom2_core(&gclk0).unwrap();

    let wifi_pins = wifi::WifiPins {
        miso: pins.nina_miso,
        mosi: pins.nina_mosi,
        sck: pins.nina_sck,
        ack: pins.nina_ack,
        rst: pins.nina_resetn,
        cs: pins.nina_cs,
    };
    let _wifi = wifi::setup_wifi(
        &clock,
        &peripherals.PM,
        peripherals.SERCOM2,
        wifi_pins,
        |d: Duration| delay.delay_us(d.as_micros() as u32),
    );

    // match wifi.scan_networks() {
    //     Ok(networks) => {
    //         usb_log("scanned networks\n");
    //         for network in networks.flatten() {
    //             if let Ok(ssid) = str::from_utf8(network.ssid.as_slice()) {
    //                 usb_log(ssid);
    //                 usb_log("\n");
    //             }
    //         }
    //     }
    //     _ => usb_log("failed to scan\n"),
    // }

    loop {
        unsafe {
            // NB Must be called at least once every 10ms to stay
            // USB-compliant.
            poll_usb(&mut gyro);
        }
        // delay.delay_ms(500u16);
        // led.toggle().unwrap();
        // while ! gyro.gyro_data_available().unwrap() {}
        // while !gyro.accel_data_available().unwrap() {}
    }
}

unsafe fn poll_usb(gyro: &mut gyro::Gyro) {
    if let Some(usb_dev) = usb::USB_BUS.as_mut() {
        if let Some(serial) = usb::USB_SERIAL.as_mut() {
            usb_dev.poll(&mut [serial]);

            let mut buf = [0u8; 127];
            if let Ok(count) = serial.read(&mut buf) {
                handle_serial(core::str::from_utf8(&buf[..count - 1]).unwrap(), gyro);
            }
        }
    };
}

fn handle_serial(s: &str, gyro: &mut gyro::Gyro) {
    match s {
        "ping" => usb_log("pong\n"),
        "gyro" => log_gyro(gyro),
        _ => usb_log("unknown command\n"),
    };
}

fn log_gyro(gyro: &mut gyro::Gyro) {
    if let Ok((gx, gy, gz)) = gyro.read_accelerometer() {
        usb_log(orientation::display_orientation(orientation::orientation(
            gx, gy, gz,
        )));
        usb_log("\n");
    }
}
