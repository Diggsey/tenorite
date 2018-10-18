mod circuit;
mod components;

pub use self::circuit::*;
pub use self::components::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gates::*;

    #[test]
    fn it_works() {
        let mut builder = CircuitBuilder::new();
        let power_line = builder.add_wire();
        builder.add_component(Constant::new(Voltage::High.into()), &[], &[power_line]);
    
        let mut circuit = builder.build();
        assert!(circuit.propagate(10));
        assert_eq!(circuit.wire(power_line), WireState {
            voltage: Voltage::High,
            unstable: false,
        });
    }

    #[test]
    fn basic_gate() {
        let mut builder = CircuitBuilder::new();
        let power = builder.add_wire();
        let ground = builder.add_wire();
        let or_result = builder.add_wire();
        let and_result = builder.add_wire();
        builder.add_component(Constant::new(Voltage::High.into()), &[], &[power]);
        builder.add_component(Constant::new(Voltage::Low.into()), &[], &[ground]);
        builder.add_component(OrGate::default(), &[power, ground], &[or_result]);
        builder.add_component(AndGate::default(), &[power, ground], &[and_result]);

        let mut circuit = builder.build();
        assert!(circuit.propagate(10));
        assert_eq!(circuit.wire(or_result), WireState {
            voltage: Voltage::High,
            unstable: false,
        });
        assert_eq!(circuit.wire(and_result), WireState {
            voltage: Voltage::Low,
            unstable: false,
        });
    }

    #[test]
    fn error_state() {
        let mut builder = CircuitBuilder::new();
        let power = builder.add_wire();
        let ground = builder.add_wire();
        let floating = builder.add_wire();
        let error_result = builder.add_wire();
        let low_result = builder.add_wire();
        let high_result = builder.add_wire();
        let floating_result = builder.add_wire();
        builder.add_component(Constant::new(Voltage::High.into()), &[], &[power]);
        builder.add_component(Constant::new(Voltage::Low.into()), &[], &[ground]);
        builder.add_component(Buffer::default(), &[ground], &[error_result]);
        builder.add_component(Buffer::default(), &[power], &[error_result]);
        builder.add_component(Buffer::default(), &[ground], &[low_result]);
        builder.add_component(Buffer::default(), &[ground], &[low_result]);
        builder.add_component(Buffer::default(), &[power], &[high_result]);
        builder.add_component(Buffer::default(), &[power], &[high_result]);
        builder.add_component(Buffer::default(), &[floating], &[floating_result]);
        builder.add_component(Buffer::default(), &[floating], &[floating_result]);

        let mut circuit = builder.build();
        assert!(circuit.propagate(10));
        assert_eq!(circuit.wire(error_result), WireState {
            voltage: Voltage::Error,
            unstable: false,
        });
        assert_eq!(circuit.wire(low_result), WireState {
            voltage: Voltage::Low,
            unstable: false,
        });
        assert_eq!(circuit.wire(high_result), WireState {
            voltage: Voltage::High,
            unstable: false,
        });
        assert_eq!(circuit.wire(floating_result), WireState {
            voltage: Voltage::Floating,
            unstable: false,
        });
    }

    #[test]
    fn unstable_state() {
        let mut builder = CircuitBuilder::new();
        let pull_up = builder.add_wire();
        let ground = builder.add_wire();
        builder.add_component(Constant::new(VoltageInput {
            voltage: Voltage::High,
            resistor: true,
        }), &[], &[pull_up]);
        builder.add_component(Constant::new(Voltage::Low.into()), &[], &[ground]);
        builder.add_component(ControlledBuffer::default(), &[ground, pull_up], &[pull_up]);

        let mut circuit = builder.build();
        assert!(!circuit.propagate(10));
        assert!(circuit.wire(pull_up).unstable);
    }
}
