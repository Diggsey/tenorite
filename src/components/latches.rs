use crate::{Voltage, Component, ComponentInterface};


// SR-NOR latch
#[derive(Debug, Copy, Clone)]
pub struct SrNorLatch {
    s: bool,
    r: bool,
}

impl Component for SrNorLatch {
    fn update(&mut self, interface: &mut ComponentInterface) {
        let s = interface.input(0);
        let r = interface.input(1);
        let (new_s, new_r) = match (self.s, self.r, s, r) {
            (_, _, Voltage::High, Voltage::Low) | (true, false, _, Voltage::Low) => (true, false),
            (_, _, Voltage::Low, Voltage::High) | (false, true, Voltage::Low, _) => (false, true),
            (_, _, Voltage::High, Voltage::High) => (true, true),
            _ => (false, false),
        };
        self.s = new_s;
        self.r = new_r;
        if new_s || new_r {
            interface.output(0, if new_r { Voltage::Low } else { Voltage::High }.into());
            interface.output(1, if new_s { Voltage::Low } else { Voltage::High }.into());
        } else {
            interface.output(0, Voltage::Error.into());
            interface.output(1, Voltage::Error.into());
        }
    }
}

impl Default for SrNorLatch {
    fn default() -> Self {
        SrNorLatch {
            s: false,
            r: true,
        }
    }
}
