use crate::{Voltage, Component, ComponentInterface};
use crate::gates::{AndFn, XorFn, BinaryGateFn};


#[derive(Debug, Clone, Default)]
pub struct BitAdder;

impl BitAdder {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Component for BitAdder {
    fn update(&mut self, interface: &mut ComponentInterface) {
        let mut sum = 0;
        for i in 0..interface.num_inputs() {
            match interface.input(i) {
                Voltage::Low => {},
                Voltage::High => { sum += 1; },
                _ => {
                    for j in 0..interface.num_outputs() {
                        interface.output(j, Voltage::Error.into());
                    }
                    return;
                }
            }
        }
        for j in 0..interface.num_outputs() {
            interface.output(j, (sum & 1 == 1).into());
            sum >>= 1;
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct HalfAdder;

impl HalfAdder {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Component for HalfAdder {
    fn update(&mut self, interface: &mut ComponentInterface) {
        let a = interface.input(0);
        let b = interface.input(1);
        interface.output(0, XorFn::call(a, b));
        interface.output(1, AndFn::call(a, b));
    }
}

#[derive(Debug, Clone, Default)]
pub struct FullAdder;

impl FullAdder {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Component for FullAdder {
    fn update(&mut self, interface: &mut ComponentInterface) {
        let a = interface.input(0);
        let b = interface.input(1);
        let c = interface.input(2);
        let (x, y) = match (a, b, c) {
            (Voltage::Low, Voltage::Low, Voltage::Low) => (Voltage::Low, Voltage::Low),
            (Voltage::Low, Voltage::Low, Voltage::High) |
            (Voltage::Low, Voltage::High, Voltage::Low) |
            (Voltage::High, Voltage::Low, Voltage::Low) => (Voltage::High, Voltage::Low),
            (Voltage::Low, Voltage::High, Voltage::High) |
            (Voltage::High, Voltage::Low, Voltage::High) |
            (Voltage::High, Voltage::High, Voltage::Low) => (Voltage::Low, Voltage::High),
            (Voltage::High, Voltage::High, Voltage::High) => (Voltage::High, Voltage::High),
            (Voltage::Low, Voltage::Low, _) |
            (Voltage::Low, _, Voltage::Low) |
            (_, Voltage::Low, Voltage::Low) => (Voltage::Error, Voltage::Low),
            (_, Voltage::High, Voltage::High) |
            (Voltage::High, _, Voltage::High) |
            (Voltage::High, Voltage::High, _) => (Voltage::Error, Voltage::High),
            (_, _, _) => (Voltage::Error, Voltage::Error),
        };
        interface.output(0, x.into());
        interface.output(1, y.into());
    }
}

#[derive(Debug, Clone, Default)]
pub struct Adder;

impl Adder {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Component for Adder {
    fn update(&mut self, interface: &mut ComponentInterface) {
        let bits = interface.num_inputs() / 2;

        // Carry in
        let carry_voltage = interface.input(bits*2);
        let mut carry = match carry_voltage {
            Voltage::Low => false,
            Voltage::High => true,
            _ => {
                for j in 0..(bits+1) {
                    interface.output(j, Voltage::Error.into());
                }
                return;
            }
        };

        // Sum
        for i in 0..bits {
            let a = interface.input(i);
            let b = interface.input(bits+i);
            let (x, y) = match (a, b, carry) {
                (Voltage::Low, Voltage::Low, c) => (false, c),
                (Voltage::Low, Voltage::High, c) | (Voltage::High, Voltage::Low, c) => (c, !c),
                (Voltage::High, Voltage::High, c) => (true, c),
                _ => {
                    for j in i..(bits+1) {
                        interface.output(j, Voltage::Error.into());
                    }
                    return;
                }
            };
            interface.output(i, y.into());
            carry = x;
        }

        // Carry out
        interface.output(bits, carry.into());
    }
}


#[derive(Debug, Clone, Default)]
pub struct Subtractor;

impl Subtractor {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Component for Subtractor {
    fn update(&mut self, interface: &mut ComponentInterface) {
        let bits = interface.num_inputs() / 2;

        // Borrow in
        let borrow_voltage = interface.input(bits*2);
        let mut borrow = match borrow_voltage {
            Voltage::Low => false,
            Voltage::High => true,
            _ => {
                for j in 0..(bits+1) {
                    interface.output(j, Voltage::Error.into());
                }
                return;
            }
        };

        // Subtract
        for i in 0..bits {
            let a = interface.input(i);
            let b = interface.input(bits+i);
            let (x, y) = match (a, b, borrow) {
                (Voltage::Low, Voltage::High, c) => (true, !c),
                (Voltage::Low, Voltage::Low, c) | (Voltage::High, Voltage::High, c) => (c, c),
                (Voltage::High, Voltage::Low, c) => (false, !c),
                _ => {
                    for j in i..(bits+1) {
                        interface.output(j, Voltage::Error.into());
                    }
                    return;
                }
            };
            interface.output(i, y.into());
            borrow = x;
        }

        // Borrow out
        interface.output(bits, borrow.into());
    }
}


#[derive(Debug, Clone, Default)]
pub struct Multiplier;

impl Multiplier {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Component for Multiplier {
    fn update(&mut self, interface: &mut ComponentInterface) {
        let bits = interface.num_inputs() / 3;

        let mut carry = 0;
        let mut has_error = false;

        // Low bits
        for i in 0..bits {
            carry += match interface.input(bits*2+i) {
                Voltage::Low => 0,
                Voltage::High => 1,
                _ => { has_error = true; 1 },
            };
            for j in 0..(i+1) {
                let k = i - j;
                carry += match (interface.input(j), interface.input(bits+k)) {
                    (Voltage::Low, _) | (_, Voltage::Low) => 0,
                    (Voltage::High, Voltage::High) => 1,
                    _ => { has_error = true; 1 },
                }
            }
            if has_error {
                interface.output(i, Voltage::Error.into());
            } else {
                interface.output(i, (carry & 1 == 1).into());
            }
            carry >>= 1;
            if carry == 0 {
                has_error = false;
            }
        }

        // High bits
        for i in bits..(bits*2) {
            for j in (i-bits)..(bits+1) {
                let k = i - j;
                carry += match (interface.input(j), interface.input(bits+k)) {
                    (Voltage::Low, _) | (_, Voltage::Low) => 0,
                    (Voltage::High, Voltage::High) => 1,
                    _ => { has_error = true; 1 }
                }
            }
            if has_error {
                interface.output(i, Voltage::Error.into());
            } else {
                interface.output(i, (carry & 1 == 1).into());
            }
            carry >>= 1;
            if carry == 0 {
                has_error = false;
            }
        }
    }
}


#[derive(Debug, Clone, Default)]
pub struct Negator;

impl Negator {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Component for Negator {
    fn update(&mut self, interface: &mut ComponentInterface) {
        let bits = interface.num_inputs();

        let mut borrow = false;

        for i in 0..bits {
            let b = interface.input(i);
            let (x, y) = match (b, borrow) {
                (Voltage::High, c) => (true, !c),
                (Voltage::Low, c) => (c, c),
                _ => {
                    for j in i..bits {
                        interface.output(j, Voltage::Error.into());
                    }
                    return;
                }
            };
            interface.output(i, y.into());
            borrow = x;
        }
    }
}


#[derive(Debug, Clone, Default)]
pub struct Comparator;

impl Comparator {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Component for Comparator {
    fn update(&mut self, interface: &mut ComponentInterface) {
        let bits = interface.num_inputs()/2;

        let mut req_a_lower = false;
        let mut req_b_lower = false;

        // Compare
        for i in (0..bits).rev() {
            let a = interface.input(i);
            let b = interface.input(bits+i);
            match (a, b) {
                (Voltage::High, Voltage::Low) if !req_a_lower => {
                    interface.output(0, Voltage::Low.into());
                    interface.output(1, Voltage::Low.into());
                    interface.output(2, Voltage::High.into());
                    return;
                },
                (Voltage::Low, Voltage::High) if !req_b_lower => {
                    interface.output(0, Voltage::High.into());
                    interface.output(1, Voltage::Low.into());
                    interface.output(2, Voltage::Low.into());
                    return;
                },
                (Voltage::Low, Voltage::Error) | (Voltage::Low, Voltage::Floating) if !req_b_lower => {
                    req_a_lower = true;
                },
                (Voltage::Error, Voltage::Low) | (Voltage::Error, Voltage::Low) if !req_a_lower => {
                    req_b_lower = true;
                },
                (Voltage::Error, Voltage::High) | (Voltage::Error, Voltage::High) if !req_b_lower => {
                    req_a_lower = true;
                },
                (Voltage::High, Voltage::Error) | (Voltage::High, Voltage::Floating) if !req_a_lower => {
                    req_b_lower = true;
                },
                (Voltage::Low, Voltage::Low) | (Voltage::High, Voltage::High) => {},
                _ => {
                    req_a_lower = true;
                    req_b_lower = true;
                    break;
                }
            }
        }
        if req_a_lower || req_b_lower {
            interface.output(0, if req_b_lower { Voltage::Low.into() } else { Voltage::Error.into() });
            interface.output(1, Voltage::Error.into());
            interface.output(2, if req_a_lower { Voltage::Low.into() } else { Voltage::Error.into() });
        } else {
            interface.output(0, Voltage::Low.into());
            interface.output(1, Voltage::High.into());
            interface.output(2, Voltage::Low.into());
        }
    }
}
