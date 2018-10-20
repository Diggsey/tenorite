use crate::{AnyComponent, Voltage, CircuitBuilder};
use crate::gates::Constant;

fn simulate_component_internal(
    component: Box<AnyComponent>,
    inputs: &[(usize, u64, u64)],
    outputs: &[usize],
) -> Vec<(u64, u64)> {
    let mut builder = CircuitBuilder::new();
    let power = builder.add_wire();
    let ground = builder.add_wire();
    let floating = builder.add_wire();
    let error = builder.add_wire();

    // Set up inputs
    let mut input_wires = Vec::new();
    for &(bits, v1, v2) in inputs {
        for bit in 0..bits {
            let a = (v1 >> bit) & 1 == 1;
            let b = (v2 >> bit) & 1 == 1;
            input_wires.push(match (a, b) {
                (false, false) => ground,
                (true, false) => power,
                (false, true) => floating,
                (true, true) => error,
            });
        }
    }

    // Set up outputs
    let num_outputs = outputs.iter().sum();
    let output_wires: Vec<_> = (0..num_outputs).map(|_| {
        builder.add_wire()
    }).collect();

    builder.add_component(Constant::new(Voltage::High.into()), &[], &[power]);
    builder.add_component(Constant::new(Voltage::Low.into()), &[], &[ground]);
    builder.add_component(Constant::new(Voltage::Error.into()), &[], &[error]);
    builder.add_boxed_component(component, &input_wires, &output_wires);

    let mut circuit = builder.build();
    assert!(circuit.propagate(10));

    // Decode result
    let mut result = Vec::new();
    let mut index = 0;
    for &bits in outputs {
        let mut v1 = 0;
        let mut v2 = 0;
        for bit in 0..bits {
            let (a, b) = match circuit.wire(output_wires[index]).voltage {
                Voltage::Low => (false, false),
                Voltage::High => (true, false),
                Voltage::Floating => (false, true),
                Voltage::Error => (true, true),
            };

            if a {
                v1 |= 1 << bit;
            }
            if b {
                v2 |= 1 << bit;
            }

            index += 1;
        }
        result.push((v1, v2));
    }

    result
}

pub fn simulate_component<T: AnyComponent>(
    component: T,
    inputs: &[(usize, u64, u64)],
    outputs: &[usize],
) -> Vec<(u64, u64)> {
    simulate_component_internal(Box::new(component), inputs, outputs)
}

pub const INTERESTING_VALUES: &[u64] = &[
    0, 1, !0, !1, !0 >> 1, !(!0 >> 1), 2, 7, 100, 997
];
