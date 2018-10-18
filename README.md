# tenorite
Fast logic simulation Rust library

## Features
- Four input pin states (high, low, floating, error)
- Seven output pin states (high, low, pull-up, pull-down, floating, error, pull-error)
- Rough approximation of signal delay (all components have equal delays, wires have no delay)
- Fast (only updates what's changed)
- Extensible (component trait is easy to implement)
- Allocation free (whilst simulating)
- Prebuild components
    - Gates
        - Constant
        - Buffer
        - Not gate
        - And gate
        - Or gate
        - Xor gate
        - Nand gate
        - Nor gate
        - Xnor gate
        - Imply gate
        - Controlled buffer
        - Controlled inverter
        - N-ary and gate
        - N-ary or gate
        - Parity gate (N-ary xor gate)
    - Latches
        - SR-Nor latch
    - Memory
        - *TODO: Flip-flops*
        - *TODO: Registers*
        - *TODO: Counter*
        - *TODO: RAM*
        - *TODO: ROM*
    - Clocks
        - Clock
        - Controlled clock
    - Plexers
        - Multiplexer
        - Demultiplexer
        - Priority encoder
    - Arithmetic
        - Half adder
        - Full adder
        - Bit adder
        - Adder
        - Subtractor
        - Multiplier
        - Negator
        - Comparator
        - Shifter

## Out of scope
- Precise signal timing (wires have no delay, components cannot have custom delays)
- Variable voltages or impedences

## Example usage

Simulating an AND and an OR gate:

```rust
let mut builder = CircuitBuilder::new();

// Lay down the wires first...
let power = builder.add_wire();
let ground = builder.add_wire();
let or_result = builder.add_wire();
let and_result = builder.add_wire();

// Add high and low voltage sources for our power and ground lines
builder.add_component(Constant::new(Voltage::High.into()), &[], &[power]);
builder.add_component(Constant::new(Voltage::Low.into()), &[], &[ground]);

// Use the high and low voltages as inputs to our AND and OR gates
builder.add_component(OrGate::default(), &[power, ground], &[or_result]);
builder.add_component(AndGate::default(), &[power, ground], &[and_result]);

// Build the circuit
let mut circuit = builder.build();

// Simulate the circuit, and ensure it reaches equilibrium within 10 iterations
assert!(circuit.propagate(10));

// Check that the results were as expected
assert_eq!(circuit.wire(or_result), WireState {
    voltage: Voltage::High,
    unstable: false,
});
assert_eq!(circuit.wire(and_result), WireState {
    voltage: Voltage::Low,
    unstable: false,
});
```
