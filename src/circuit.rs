use std::collections::HashMap;
use std::any::Any;
use std::fmt;

use smallvec::{SmallVec, smallvec};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Voltage {
    Floating = 0,
    Low = 1,
    High = 2,
    Error = 3,
}

impl From<u8> for Voltage {
    fn from(v: u8) -> Self {
        match v {
            0 => Voltage::Floating,
            1 => Voltage::Low,
            2 => Voltage::High,
            3 => Voltage::Error,
            _ => unreachable!()
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct VoltageInput {
    pub voltage: Voltage,
    pub resistor: bool,
}

impl From<Voltage> for VoltageInput {
    fn from(voltage: Voltage) -> Self {
        VoltageInput { voltage, resistor: false }
    }
}

impl From<bool> for VoltageInput {
    fn from(voltage: bool) -> Self {
        VoltageInput { voltage: voltage.into(), resistor: false }
    }
}

impl Voltage {
    pub fn pull(self, other: Voltage) -> Voltage {
        (self as u8 | other as u8).into()
    }
}

impl From<bool> for Voltage {
    fn from(voltage: bool) -> Self {
        if voltage { Voltage::High } else { Voltage::Low }
    }
}

pub trait Component: fmt::Debug + Any {
    fn update(&mut self, interface: &mut ComponentInterface);
    fn tick(&mut self, _tick: u64) -> bool { false }

}

pub trait AnyComponent: Component {
    fn as_any_mut(&mut self) -> &mut Any;
    fn as_any_ref(&self) -> &Any;
    fn clone_box(&self) -> Box<AnyComponent>;
}

impl<C: Component + Clone> AnyComponent for C {
    fn as_any_mut(&mut self) -> &mut Any { self }
    fn as_any_ref(&self) -> &Any { self }
    fn clone_box(&self) -> Box<AnyComponent> { Box::new(self.clone()) }
}

#[derive(Debug, Clone)]
struct Wire {
    voltage: Voltage,
    inputs: SmallVec<[VoltageInput; 2]>,
    invalidation_id: usize,
    next: usize,
}

impl Wire {
    fn state(&self) -> WireState {
        WireState {
            voltage: self.voltage,
            unstable: self.next != NULL_INDEX,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct WireState {
    pub voltage: Voltage,
    pub unstable: bool,
}

#[derive(Debug, Copy, Clone)]
struct Pin {
    wire_id: usize,
    input_id: usize,
}

#[derive(Debug)]
pub struct ComponentInterface<'a> {
    first_wire: &'a mut usize,
    wires: &'a mut [Wire],
    inputs: &'a [usize],
    outputs: &'a [Pin],
}

const NULL_INDEX: usize = !0;
const TAIL_INDEX: usize = !0-1;

impl<'a> ComponentInterface<'a> {
    pub fn input(&self, index: usize) -> Voltage {
        self.wires[self.inputs[index]].voltage
    }
    pub fn output(&mut self, index: usize, voltage: VoltageInput) {
        let pin = self.outputs[index];
        if pin.wire_id != NULL_INDEX {
            let wire = &mut self.wires[pin.wire_id];
            if wire.inputs[pin.input_id] != voltage {
                wire.inputs[pin.input_id] = voltage;
                if wire.next == NULL_INDEX {
                    wire.next = *self.first_wire;
                    *self.first_wire = pin.wire_id;
                }
            }
        }
    }
}

#[derive(Debug)]
struct ComponentWrapper {
    iteration: usize,
    inputs: SmallVec<[usize; 4]>,
    outputs: SmallVec<[Pin; 2]>,
    component_impl: Box<AnyComponent>,
}

impl Clone for ComponentWrapper {
    fn clone(&self) -> Self {
        ComponentWrapper {
            iteration: self.iteration,
            inputs: self.inputs.clone(),
            outputs: self.outputs.clone(),
            component_impl: self.component_impl.clone_box(),
        }
    }
}

#[derive(Clone, Debug)]
struct ComponentSet {
    components: SmallVec<[usize; 4]>,
    next: usize,
}

#[derive(Clone, Debug)]
pub struct Circuit {
    iteration_count: usize,
    tick_count: u64,
    first_component_set: usize,
    first_wire: usize,
    wires: Vec<Wire>,
    component_sets: Vec<ComponentSet>,
    components: Vec<ComponentWrapper>,
}

impl Circuit {
    fn update_wire(&mut self, wire_id: usize) -> usize {
        let wire = &mut self.wires[wire_id];
        let mut new_voltage = Voltage::Floating;

        // Normal inputs
        for input in &wire.inputs {
            if !input.resistor {
                new_voltage = new_voltage.pull(input.voltage);
            }
        }

        // Pull resistors
        if new_voltage == Voltage::Floating {
            for input in &wire.inputs {
                if input.resistor {
                    new_voltage = new_voltage.pull(input.voltage);
                }
            }
        }

        // Invalidate components
        if wire.voltage != new_voltage {
            wire.voltage = new_voltage;
            if wire.invalidation_id != NULL_INDEX {
                let component_set = &mut self.component_sets[wire.invalidation_id];
                if component_set.next == NULL_INDEX {
                    component_set.next = self.first_component_set;
                    self.first_component_set = wire.invalidation_id;
                }
            }
        }

        let result = wire.next;
        wire.next = NULL_INDEX;
        result
    }
    fn update_wires(&mut self) {
        while self.first_wire != TAIL_INDEX {
            self.first_wire = self.update_wire(self.first_wire);
        }
    }
    fn update_components(&mut self, iteration: usize) {
        while self.first_component_set != TAIL_INDEX {
            let component_set = &mut self.component_sets[self.first_component_set];

            for &component_id in &component_set.components {
                let component = &mut self.components[component_id];
                if component.iteration == iteration {
                    continue;
                }
                component.iteration = iteration;
                let mut interface = ComponentInterface {
                    first_wire: &mut self.first_wire,
                    wires: &mut self.wires,
                    inputs: &component.inputs,
                    outputs: &component.outputs,
                };
                component.component_impl.update(&mut interface);
            }

            self.first_component_set = component_set.next;
            component_set.next = NULL_INDEX;
        }
    }
    pub fn propagate(&mut self, max_iterations: usize) -> bool {
        // Propagate changes
        for _ in 0..max_iterations {
            self.update_wires();
            self.update_components(self.iteration_count);
            self.iteration_count += 1;

            // No more changes to propagate
            if self.first_wire == TAIL_INDEX && self.first_component_set == TAIL_INDEX {
                return true;
            }
        }

        false
    }
    pub fn tick(&mut self) {
        // Tick all the components
        self.iteration_count = 0;
        for component in &mut self.components {
            component.iteration = NULL_INDEX;
            if component.component_impl.tick(self.tick_count) {
                let mut interface = ComponentInterface {
                    first_wire: &mut self.first_wire,
                    wires: &mut self.wires,
                    inputs: &component.inputs,
                    outputs: &component.outputs,
                };
                component.component_impl.update(&mut interface);
            }
        }
        self.tick_count += 1;
    }
    fn init(&mut self) {
        // Initialize all the components
        for component in &mut self.components {
            component.iteration = NULL_INDEX;
            let mut interface = ComponentInterface {
                first_wire: &mut self.first_wire,
                wires: &mut self.wires,
                inputs: &component.inputs,
                outputs: &component.outputs,
            };
            component.component_impl.update(&mut interface);
        }
    }
    pub fn component_mut<C: Component>(&mut self, cref: ComponentRef) -> &mut C {
        self.components[cref.0].component_impl.as_any_mut().downcast_mut().unwrap()
    }
    pub fn component_ref<C: Component>(&self, cref: ComponentRef) -> &C {
        self.components[cref.0].component_impl.as_any_ref().downcast_ref().unwrap()
    }
    pub fn wire(&self, wref: WireRef) -> WireState {
        self.wires[wref.0].state()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct WireRef(usize);

impl WireRef {
    pub const NONE: WireRef = WireRef(NULL_INDEX);
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ComponentRef(usize);

#[derive(Default, Debug, Clone)]
struct PreparedWire {
    components: SmallVec<[usize; 4]>,
    num_inputs: usize,
}

impl PreparedWire {
    fn build(mut self, component_sets: &mut HashMap<SmallVec<[usize; 4]>, usize>) -> Wire {
        // Compute invalidation ID
        let invalidation_id = if self.components.is_empty() {
            NULL_INDEX
        } else {
            self.components.sort_unstable();

            let next_id = component_sets.len();
            *component_sets.entry(self.components).or_insert(next_id)
        };

        Wire {
            voltage: if self.num_inputs > 0 { Voltage::Low } else { Voltage::Floating },
            inputs: smallvec![VoltageInput {
                voltage: Voltage::Low,
                resistor: false,
            }; self.num_inputs],
            invalidation_id,
            next: NULL_INDEX,
        }
    }
    fn add_input(&mut self) -> usize {
        let result = self.num_inputs;
        self.num_inputs += 1;
        result
    }
}

#[derive(Debug)]
struct PreparedComponent {
    inputs: SmallVec<[usize; 4]>,
    outputs: SmallVec<[Pin; 2]>,
    component_impl: Box<AnyComponent>,
}

impl PreparedComponent {
    fn build(self) -> ComponentWrapper {
        ComponentWrapper {
            iteration: NULL_INDEX,
            inputs: self.inputs,
            outputs: self.outputs,
            component_impl: self.component_impl,
        }
    }
}

impl Clone for PreparedComponent {
    fn clone(&self) -> Self {
        PreparedComponent {
            inputs: self.inputs.clone(),
            outputs: self.outputs.clone(),
            component_impl: self.component_impl.clone_box(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CircuitBuilder {
    wires: Vec<PreparedWire>,
    components: Vec<PreparedComponent>,
}

impl CircuitBuilder {
    pub fn new() -> Self {
        CircuitBuilder {
            wires: Vec::new(),
            components: Vec::new(),
        }
    }
    pub fn add_wire(&mut self) -> WireRef {
        let result = WireRef(self.wires.len());
        self.wires.push(PreparedWire::default());
        result
    }
    pub fn add_component(&mut self, component: Box<AnyComponent>, inputs: &[WireRef], outputs: &[WireRef]) -> ComponentRef {
        let result = ComponentRef(self.components.len());

        let mut comp = PreparedComponent {
            inputs: SmallVec::with_capacity(inputs.len()),
            outputs: SmallVec::with_capacity(outputs.len()),
            component_impl: component,
        };

        for input in inputs {
            comp.inputs.push(input.0);
            let wire = &mut self.wires[input.0];
            if !wire.components.contains(&result.0) {
                wire.components.push(result.0);
            }
        }

        for output in outputs {
            let input_id = if output.0 == NULL_INDEX { 0 } else {self.wires[output.0].add_input() };
            comp.outputs.push(Pin {
                wire_id: output.0,
                input_id,
            });
        }

        self.components.push(comp);
        result
    }
    pub fn build(self) -> Circuit {
        let mut component_sets_map = HashMap::new();
        let wires = self.wires.into_iter().map(|w| w.build(&mut component_sets_map)).collect();
        let components = self.components.into_iter().map(PreparedComponent::build).collect();

        let mut component_sets = vec![ComponentSet {
            components: SmallVec::new(),
            next: NULL_INDEX,
        }; component_sets_map.len()];

        for (component_set, index) in component_sets_map {
            component_sets[index].components = component_set;
        }

        let mut circuit = Circuit {
            iteration_count: 0,
            tick_count: 0,
            first_component_set: TAIL_INDEX,
            first_wire: TAIL_INDEX,
            wires,
            component_sets,
            components,
        };
        circuit.init();
        circuit
    }
}
