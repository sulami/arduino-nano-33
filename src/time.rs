use arduino_nano33iot as bsp;
use bsp::hal::clock::{GClock, GenericClockController};
use bsp::hal::prelude::*;
use bsp::hal::timer::{TimerCounter, TimerCounter5};

pub struct Timer {
    tc: TimerCounter5,
    millis: u64,
}

impl Timer {
    pub fn new(
        tc5: bsp::pac::TC5,
        clocks: &mut GenericClockController,
        gclk0: &GClock,
        pm: &mut bsp::pac::PM,
    ) -> Self {
        let timer_clock = clocks.tc4_tc5(gclk0).unwrap();
        let mut timer = TimerCounter::tc5_(&timer_clock, tc5, pm);
        timer.start(1.khz());
        Timer {
            tc: timer,
            millis: 0,
        }
    }

    pub fn tick(&mut self) {
        nb::block!(self.tc.wait()).ok();
        self.millis += 1;
    }

    pub fn millis(&self) -> u64 {
        self.millis
    }
}
