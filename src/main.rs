#![no_std]
#![no_main]

use core::str;
use core::time::Duration;

use arduino_nano33iot as bsp;
use bsp::hal;
use hal::prelude::*;

use panic_halt as _;

use bsp::entry;
use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::pac::{interrupt, CorePeripherals, Peripherals};

// USB logging
use hal::usb::UsbBus;
use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use cortex_m::peripheral::NVIC;

// WiFi
use hal::sercom::v2::spi;
use wifi_nina::transport;

// Acellerometer
use lsm6ds33;
use lexical_core as lexical;

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
    let mut delay = Delay::new(core.SYST, &mut clocks);

    let bus_allocator = unsafe {
        USB_ALLOCATOR = Some(bsp::usb_allocator(
            peripherals.USB,
            &mut clocks,
            &mut peripherals.PM,
            pins.usb_dm,
            pins.usb_dp,
        ));
        USB_ALLOCATOR.as_ref().unwrap()
    };

    unsafe {
        USB_SERIAL = Some(SerialPort::new(bus_allocator));
        USB_BUS = Some(
            UsbDeviceBuilder::new(bus_allocator, UsbVidPid(0x2222, 0x3333))
                .manufacturer("Fake company")
                .product("Serial port")
                .serial_number("TEST")
                .device_class(USB_CLASS_CDC)
                .build(),
        );
    }

    unsafe {
        core.NVIC.set_priority(interrupt::USB, 1);
        NVIC::unmask(interrupt::USB);
    }

    usb_log("usb initialised\n");

    // Prep Nina
    let nina_cs = pins.nina_cs.into_push_pull_output();
    let nina_ack = pins.nina_ack.into_floating_input();
    let nina_rst = pins.nina_resetn.into_push_pull_output();
    // let nina_gpio0 = pins.nina_gpio0.into_readable_output();

    // Init SPI to Nina
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

    usb_log("SPI initialised\n");

    let transport =
        transport::SpiTransport::start(spi_master, nina_ack, nina_rst, nina_cs, |d: Duration| {
            delay.delay_us(d.as_micros() as u32)
        })
        .unwrap();
    usb_log("transport initialised\n");
    let mut wifi = wifi_nina::Wifi::new(transport);
    usb_log("wifi initialised\n");
    let fw_version = wifi.get_firmware_version();
    usb_log("got wifi firmware version\n");
    match fw_version {
        Ok(version) => {
            usb_log(str::from_utf8(version.as_slice()).unwrap());
            usb_log("\n");
        }
        _ => usb_log("version bad\n"),
    }

    usb_log("scanning networks...\n");
    match wifi.scan_networks() {
        Ok(networks) => {
            usb_log("scanned networks\n");
            for network in networks.flatten() {
                if let Ok(ssid) = str::from_utf8(network.ssid.as_slice()) {
                    usb_log(ssid);
                    usb_log("\n");
                }
            };
        },
        _ => usb_log("failed to scan\n"),
    }

    usb_log("Connecting to Gyro\n");
    let i2c = bsp::i2c_master(
        &mut clocks,
        bsp::hal::time::KiloHertz(100),
        peripherals.SERCOM4,
        &mut peripherals.PM,
        pins.sda,
        pins.scl);
    let mut gyro = lsm6ds33::Lsm6ds33::new(i2c, 0x6A).unwrap();
    gyro.set_gyroscope_output(lsm6ds33::GyroscopeOutput::Rate104).unwrap();
    gyro.set_accelerometer_output(lsm6ds33::AccelerometerOutput::Rate104).unwrap();
    usb_log("Connected to Gyro\n");

    loop {
        // delay.delay_ms(500u16);
        // led.toggle().unwrap();
        // while ! gyro.gyro_data_available().unwrap() {}
        while ! gyro.accel_data_available().unwrap() {}
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

static mut USB_ALLOCATOR: Option<UsbBusAllocator<UsbBus>> = None;
static mut USB_BUS: Option<UsbDevice<UsbBus>> = None;
static mut USB_SERIAL: Option<SerialPort<UsbBus>> = None;

fn usb_log(s: &str) {
    cortex_m::interrupt::free(|_| unsafe {
        USB_BUS.as_mut().map(|_| {
            if let Some(serial) = USB_SERIAL.as_mut() {
                // Skip errors so we can continue the program
                let _ = serial.write(s.as_bytes());
            };
        })
    });
}

fn poll_usb() {
    unsafe {
        if let Some(usb_dev) = USB_BUS.as_mut() {
            if let Some(serial) = USB_SERIAL.as_mut() {
                usb_dev.poll(&mut [serial]);

                // Make the other side happy
                let mut buf = [0u8; 16];
                let _ = serial.read(&mut buf);
            }
        };
    };
}

#[interrupt]
fn USB() {
    poll_usb();
}
