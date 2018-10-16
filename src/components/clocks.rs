use crate::{Voltage, Component, ComponentInterface};


#[derive(Debug, Clone)]
pub struct Clock {
    ticks_low: u32,
    ticks_high: u32,
    tick_phase: u32,
    state: bool,
}

impl Clock {
    pub fn set_ticks(&mut self, low: u32, high: u32) {
        self.ticks_low = low;
        self.ticks_high = high;
    }
    pub fn set_phase(&mut self, phase: u32) {
        self.tick_phase = phase;
    }
    pub fn ticks_low(&self) -> u32 { self.ticks_low }
    pub fn ticks_high(&self) -> u32 { self.ticks_high }
    pub fn tick_phase(&self) -> u32 { self.tick_phase }
    pub fn state(&self) -> bool { self.state }
}

impl Component for Clock {
    fn tick(&mut self, tick: u64) -> bool {
        let old_state = self.state;
        let period = (self.ticks_low + self.ticks_high) as u64;
        self.state = (tick + self.tick_phase as u64) % period >= self.ticks_low as u64;
        self.state != old_state
    }
    fn update(&mut self, interface: &mut ComponentInterface) {
        interface.output(0, self.state.into());
    }
}

#[derive(Debug, Clone)]
pub struct ControlledClock {
    ticks_low: u32,
    ticks_high: u32,
    tick_phase: u32,
    state: Voltage,
    enabled: Voltage,
}

impl ControlledClock {
    pub fn set_ticks(&mut self, low: u32, high: u32) {
        self.ticks_low = low;
        self.ticks_high = high;
    }
    pub fn set_phase(&mut self, phase: u32) {
        self.tick_phase = phase;
    }
    pub fn ticks_low(&self) -> u32 { self.ticks_low }
    pub fn ticks_high(&self) -> u32 { self.ticks_high }
    pub fn tick_phase(&self) -> u32 { self.tick_phase }
    pub fn state(&self) -> Voltage { self.state }
    pub fn enabled(&self) -> Voltage { self.enabled }
}

impl Component for ControlledClock {
    fn tick(&mut self, tick: u64) -> bool {
        let old_state = self.state;
        let period = self.ticks_low + self.ticks_high;

        match self.enabled {
            Voltage::Low => { self.tick_phase = (self.tick_phase + period - 1) % period; },
            Voltage::High => {},
            Voltage::Floating | Voltage::Error => { self.state = Voltage::Error },
        }

        if self.state != Voltage::Error {
            self.state = ((tick + self.tick_phase as u64) % (period as u64) >= self.ticks_low as u64).into();
        }
        self.state != old_state
    }
    fn update(&mut self, interface: &mut ComponentInterface) {
        self.enabled = interface.input(0);
        interface.output(0, self.state.into());
    }
}
