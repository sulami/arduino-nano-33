use arduino_nano33iot as bsp;
use bsp::hal;
use hal::clock::GenericClockController;
use hal::pac::{interrupt, CorePeripherals};
use hal::usb::UsbBus;
use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use cortex_m::peripheral::NVIC;

static mut USB_ALLOCATOR: Option<UsbBusAllocator<UsbBus>> = None;
static mut USB_BUS: Option<UsbDevice<UsbBus>> = None;
static mut USB_SERIAL: Option<SerialPort<UsbBus>> = None;

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

#[allow(non_snake_case)]
#[interrupt]
fn USB() {
    poll_usb();
}

/// Setup a USB client device.
pub unsafe fn setup_usb(
    clocks: &mut GenericClockController,
    usb: bsp::pac::USB,
    pm: &mut bsp::pac::PM,
    usb_dm: impl Into<bsp::UsbDm>,
    usb_dp: impl Into<bsp::UsbDp>,
    core: &mut CorePeripherals,
) {
    USB_ALLOCATOR = Some(bsp::usb_allocator(usb, clocks, pm, usb_dm, usb_dp));
    let bus_allocator = USB_ALLOCATOR.as_ref().unwrap();
    USB_SERIAL = Some(SerialPort::new(bus_allocator));
    USB_BUS = Some(
        UsbDeviceBuilder::new(bus_allocator, UsbVidPid(0x2222, 0x3333))
            .manufacturer("Fake company")
            .product("Serial port")
            .serial_number("TEST")
            .device_class(USB_CLASS_CDC)
            .build(),
    );
    core.NVIC.set_priority(interrupt::USB, 1);
    NVIC::unmask(interrupt::USB);
}

/// Log to the USB host via serial.
pub fn usb_log(s: &str) {
    cortex_m::interrupt::free(|_| unsafe {
        USB_BUS.as_mut().map(|_| {
            if let Some(serial) = USB_SERIAL.as_mut() {
                // Skip errors so we can continue the program
                let _ = serial.write(s.as_bytes());
            };
        })
    });
}
