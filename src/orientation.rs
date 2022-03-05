// x-axis is along the length of the Arduino.
// x-axis plus is away from the usb, y is on the power led side, z is down

pub enum Orientation {
    RightSideUp,
    UpsideDown,
    USBUp,
    USBDown,
    LEDUp,
    PowerLEDUp,
}

pub fn display_orientation(orientation: Orientation) -> &'static str {
    match orientation {
        Orientation::RightSideUp => "Right side up",
        Orientation::UpsideDown => "Upside down",
        Orientation::USBUp => "USB up",
        Orientation::USBDown => "USB down",
        Orientation::LEDUp => "LED up",
        Orientation::PowerLEDUp => "Power LED up",
    }
}

/// Returns the orientation based on the accelerometer readings for
/// all three axes.
pub fn orientation(gx: f32, gy: f32, gz: f32) -> Orientation {
    if gx.abs() > gy.abs() {
        if gx.abs() > gz.abs() {
            if gx.is_sign_positive() {
                Orientation::USBUp
            } else {
                Orientation::USBDown
            }
        } else if gz.is_sign_positive() {
            Orientation::RightSideUp
        } else {
            Orientation::UpsideDown
        }
    } else if gy.abs() > gz.abs() {
        if gy.is_sign_positive() {
            Orientation::LEDUp
        } else {
            Orientation::PowerLEDUp
        }
    } else if gz.is_sign_positive() {
        Orientation::RightSideUp
    } else {
        Orientation::UpsideDown
    }
}

trait Abs {
    fn abs(&self) -> Self;
}

impl Abs for f32 {
    fn abs(&self) -> f32 {
        if self.is_sign_positive() {
            *self
        } else {
            -1.0 * self
        }
    }
}
