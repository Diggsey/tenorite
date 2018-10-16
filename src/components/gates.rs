use std::marker::PhantomData;
use std::fmt;

use crate::{Voltage, VoltageInput, Component, ComponentInterface};

// Constant
#[derive(Debug, Copy, Clone)]
pub struct Constant {
    value: VoltageInput,
    changed: bool,
}

impl Constant {
    pub fn new(value: VoltageInput) -> Self {
        Constant {
            value,
            changed: false,
        }
    }
    pub fn set(&mut self, value: VoltageInput) {
        if value != self.value {
            self.value = value;
            self.changed = true;
        }
    }
    pub fn get(&self) -> VoltageInput {
        self.value
    }
}

impl Component for Constant {
    fn update(&mut self, interface: &mut ComponentInterface) {
        self.changed = false;
        interface.output(0, self.value);
    }
    fn tick(&mut self, _tick: u64) -> bool {
        self.changed
    }
}

// Unary gates
pub trait UnaryGateFn: 'static + fmt::Debug {
    fn call(a: Voltage) -> VoltageInput;
}

#[derive(Debug)]
pub struct UnaryGate<F: UnaryGateFn>(PhantomData<&'static F>);

impl<F: UnaryGateFn> Component for UnaryGate<F> {
    fn update(&mut self, interface: &mut ComponentInterface) {
        let result = F::call(interface.input(0));
        interface.output(0, result);
    }
}

impl<F: UnaryGateFn> Default for UnaryGate<F> {
    fn default() -> Self {
        UnaryGate(PhantomData)
    }
}

impl<F: UnaryGateFn> Clone for UnaryGate<F> {
    fn clone(&self) -> Self {
        UnaryGate(PhantomData)
    }
}

// Buffer
#[derive(Debug)]
pub struct IdentityFn;
impl UnaryGateFn for IdentityFn {
    fn call(a: Voltage) -> VoltageInput {
        a.into()
    }
}
pub type Buffer = UnaryGate<IdentityFn>;

// NOT gate
#[derive(Debug)]
pub struct NotFn;
impl UnaryGateFn for NotFn {
    fn call(a: Voltage) -> VoltageInput {
        match a {
            Voltage::Low => Voltage::High,
            Voltage::High => Voltage::Low,
            _ => Voltage::Error,
        }.into()
    }
}
pub type NotGate = UnaryGate<NotFn>;

// Binary gates
pub trait BinaryGateFn: 'static + fmt::Debug {
    fn call(a: Voltage, b: Voltage) -> VoltageInput;
}

#[derive(Debug)]
pub struct BinaryGate<F: BinaryGateFn>(PhantomData<&'static F>);

impl<F: BinaryGateFn> Component for BinaryGate<F> {
    fn update(&mut self, interface: &mut ComponentInterface) {
        let result = F::call(interface.input(0), interface.input(1));
        interface.output(0, result);
    }
}

impl<F: BinaryGateFn> Default for BinaryGate<F> {
    fn default() -> Self {
        BinaryGate(PhantomData)
    }
}

impl<F: BinaryGateFn> Clone for BinaryGate<F> {
    fn clone(&self) -> Self {
        BinaryGate(PhantomData)
    }
}

// AND gate
#[derive(Debug)]
pub struct AndFn;
impl BinaryGateFn for AndFn {
    fn call(a: Voltage, b: Voltage) -> VoltageInput {
        if a == Voltage::Low || b == Voltage::Low {
            Voltage::Low
        } else if a == Voltage::High && b == Voltage::High {
            Voltage::High
        } else {
            Voltage::Error
        }.into()
    }
}
pub type AndGate = BinaryGate<AndFn>;

// OR gate
#[derive(Debug)]
pub struct OrFn;
impl BinaryGateFn for OrFn {
    fn call(a: Voltage, b: Voltage) -> VoltageInput {
        if a == Voltage::High || b == Voltage::High {
            Voltage::High
        } else if a == Voltage::Low && b == Voltage::Low {
            Voltage::Low
        } else {
            Voltage::Error
        }.into()
    }
}
pub type OrGate = BinaryGate<OrFn>;

// XOR gate
#[derive(Debug)]
pub struct XorFn;
impl BinaryGateFn for XorFn {
    fn call(a: Voltage, b: Voltage) -> VoltageInput {
        match (a, b) {
            (Voltage::Low, Voltage::High) | (Voltage::High, Voltage::Low) => Voltage::High,
            (Voltage::Low, Voltage::Low) | (Voltage::High, Voltage::High) => Voltage::Low,
            _ => Voltage::Error
        }.into()
    }
}
pub type XorGate = BinaryGate<XorFn>;

// NAND gate
#[derive(Debug)]
pub struct NandFn;
impl BinaryGateFn for NandFn {
    fn call(a: Voltage, b: Voltage) -> VoltageInput {
        if a == Voltage::Low || b == Voltage::Low {
            Voltage::High
        } else if a == Voltage::High && b == Voltage::High {
            Voltage::Low
        } else {
            Voltage::Error
        }.into()
    }
}
pub type NandGate = BinaryGate<NandFn>;

// NOR gate
#[derive(Debug)]
pub struct NorFn;
impl BinaryGateFn for NorFn {
    fn call(a: Voltage, b: Voltage) -> VoltageInput {
        if a == Voltage::High || b == Voltage::High {
            Voltage::Low
        } else if a == Voltage::Low && b == Voltage::Low {
            Voltage::High
        } else {
            Voltage::Error
        }.into()
    }
}
pub type NorGate = BinaryGate<NorFn>;

// XNOR gate
#[derive(Debug)]
pub struct XnorFn;
impl BinaryGateFn for XnorFn {
    fn call(a: Voltage, b: Voltage) -> VoltageInput {
        match (a, b) {
            (Voltage::Low, Voltage::High) | (Voltage::High, Voltage::Low) => Voltage::Low,
            (Voltage::Low, Voltage::Low) | (Voltage::High, Voltage::High) => Voltage::High,
            _ => Voltage::Error
        }.into()
    }
}
pub type XnorGate = BinaryGate<XnorFn>;

// IMPLY gate
#[derive(Debug)]
pub struct ImplyFn;
impl BinaryGateFn for ImplyFn {
    fn call(a: Voltage, b: Voltage) -> VoltageInput {
        if a == Voltage::Low || b == Voltage::High {
            Voltage::High
        } else if a == Voltage::High && b == Voltage::Low {
            Voltage::Low
        } else {
            Voltage::Error
        }.into()
    }
}
pub type ImplyGate = BinaryGate<ImplyFn>;

// Controlled buffer
#[derive(Debug)]
pub struct ControlFn;
impl BinaryGateFn for ControlFn {
    fn call(a: Voltage, b: Voltage) -> VoltageInput {
        match b {
            Voltage::High => a,
            Voltage::Low => Voltage::Floating,
            _ => Voltage::Error
        }.into()
    }
}
pub type ControlledBuffer = BinaryGate<ControlFn>;

// Controlled inverter
#[derive(Debug)]
pub struct InvertedControlFn;
impl BinaryGateFn for InvertedControlFn {
    fn call(a: Voltage, b: Voltage) -> VoltageInput {
        match (a, b) {
            (Voltage::Low, Voltage::High) => Voltage::High,
            (Voltage::High, Voltage::High) => Voltage::Low,
            (_, Voltage::Low) => Voltage::Floating,
            _ => Voltage::Error
        }.into()
    }
}
pub type ControlledInverter = BinaryGate<InvertedControlFn>;
