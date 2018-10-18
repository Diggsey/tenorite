use crate::{Voltage, Component, ComponentInterface};


#[derive(Debug, Clone)]
pub struct Multiplexer {
    select_bits: u32,
}

impl Multiplexer {
    pub fn new(select_bits: u32) -> Self {
        Self { select_bits }
    }
    pub fn select_bits(&self) -> u32 {
        self.select_bits
    }
}

impl Component for Multiplexer {
    fn update(&mut self, interface: &mut ComponentInterface) {
        let select_bits = self.select_bits as usize;
        let mut index = 0;
        for i in 0..select_bits {
            match interface.input(i) {
                Voltage::Low => {},
                Voltage::High => {
                    index |= 1 << i;
                },
                _ => {
                    interface.output(0, Voltage::Error.into());
                    return;
                }
            }
        }
        interface.output(0, interface.input(select_bits + index).into());
    }
}

#[derive(Debug, Clone)]
pub struct Demultiplexer {
    select_bits: u32,
    three_state: bool,
    changed: bool,
}

impl Demultiplexer {
    pub fn new(select_bits: u32) -> Self {
        Self { select_bits, three_state: false, changed: false }
    }
    pub fn select_bits(&self) -> u32 {
        self.select_bits
    }
    pub fn set_three_state(&mut self, three_state: bool) -> &mut Self {
        if self.three_state != three_state {
            self.three_state = three_state;
            self.changed = true;
        }
        self
    }
    pub fn three_state(&self) -> bool {
        self.three_state
    }
}

impl Component for Demultiplexer {
    fn update(&mut self, interface: &mut ComponentInterface) {
        self.changed = false;
        let select_bits = self.select_bits as usize;
        let mut index = 0;
        for i in 0..select_bits {
            match interface.input(i) {
                Voltage::Low => {},
                Voltage::High => {
                    index |= 1 << i;
                },
                _ => {
                    for j in 0..(1 << select_bits) {
                        interface.output(j, Voltage::Error.into());
                    }
                    return;
                }
            }
        }
        for j in 0..(1 << select_bits) {
            if j == index {
                interface.output(index, interface.input(select_bits).into());
            } else if self.three_state {
                interface.output(index, Voltage::Floating.into());
            } else {
                interface.output(index, Voltage::Low.into());
            }
        }
    }
    fn tick(&mut self, _tick: u64) -> bool {
        self.changed
    }
}

#[derive(Debug, Clone)]
pub struct PriorityEncoder {
    select_bits: u32,
    inverted: bool,
    changed: bool,
}

impl PriorityEncoder {
    pub fn new(select_bits: u32) -> Self {
        Self { select_bits, inverted: false, changed: false }
    }
    pub fn select_bits(&self) -> u32 {
        self.select_bits
    }
    pub fn set_inverted(&mut self, inverted: bool) -> &mut Self {
        if inverted != self.inverted {
            self.inverted = inverted;
            self.changed = true;
        }
        self
    }
    pub fn inverted(&self) -> bool {
        self.inverted
    }
}

impl Component for PriorityEncoder {
    fn update(&mut self, interface: &mut ComponentInterface) {
        self.changed = false;
        let select_bits = self.select_bits as usize;
        let mut maybe_index = None;
        for i in (0..(1 << select_bits)).rev() {
            match interface.input(i) {
                Voltage::Low => {
                    if self.inverted {
                        maybe_index = Some(i);
                        break;
                    }
                },
                Voltage::High => {
                    if !self.inverted {
                        maybe_index = Some(i);
                        break;
                    }
                },
                _ => {
                    for j in 0..(select_bits+1) {
                        interface.output(j, Voltage::Error.into());
                    }
                    return;
                }
            }
        }

        if let Some(index) = maybe_index {
            interface.output(0, Voltage::High.into());
            for j in 0..select_bits {
                interface.output(1+j, ((index >> j) & 1 == 1).into());
            }
        } else {
            interface.output(0, Voltage::Low.into());
            for j in 0..select_bits {
                interface.output(1+j, Voltage::Floating.into());
            }
        }
    }
    fn tick(&mut self, _tick: u64) -> bool {
        self.changed
    }
}
