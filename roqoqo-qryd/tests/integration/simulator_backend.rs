// Copyright © 2021-2022 HQS Quantum Simulations GmbH. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the
// License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
// express or implied. See the License for the specific language governing permissions and
// limitations under the License.

use ndarray::{array, Array2};
use roqoqo::prelude::*;
use roqoqo::{operations::*, Circuit};
use roqoqo_qryd::qryd_devices::FirstDevice;
use roqoqo_qryd::{SimulatorBackend, TweezerDevice};
use roqoqo_test::prepare_monte_carlo_gate_test;

/// Test SimulatorBackend initialization with FirstDevice.
#[test]
fn test_init_backend_fd() {
    let device = FirstDevice::new(
        2,
        2,
        &[1, 1],
        3.0,
        array![[0.0, 1.0], [0.0, 1.0]],
        None,
        None,
        None,
        None,
    )
    .unwrap();
    let _backend = SimulatorBackend::new(device.into(), None);
}

/// Test SimulatorBackend initialization with TweezerDevice.
#[test]
fn test_init_backend_tw() {
    let mut device = TweezerDevice::new(None, None, None);
    device.add_layout("square").unwrap();
    device
        .set_tweezer_two_qubit_gate_time("CNOT", 0, 1, 1.0, Some("square".to_string()))
        .unwrap();
    device
        .set_tweezer_two_qubit_gate_time("CNOT", 0, 2, 1.0, Some("square".to_string()))
        .unwrap();
    device
        .set_tweezer_two_qubit_gate_time("CNOT", 1, 3, 1.0, Some("square".to_string()))
        .unwrap();
    device
        .set_tweezer_two_qubit_gate_time("CNOT", 2, 3, 1.0, Some("square".to_string()))
        .unwrap();
    device.switch_layout("square", None).unwrap();

    let _backend = SimulatorBackend::new(device.into(), None);
}

#[test]
fn test_to_qryd_json() {}

/// Test SimulatorBackend standard derived traits (Debug, Clone, PartialEq)
#[test]
fn test_simple_traits() {
    let layout: Array2<f64> = array![[0.0, 1.0], [0.0, 1.0]];
    let device_fd = FirstDevice::new(
        2,
        2,
        &[1, 1],
        3.0,
        array![[0.0, 1.0], [0.0, 1.0]],
        None,
        None,
        None,
        None,
    )
    .unwrap()
    .add_layout(1, layout.clone())
    .unwrap();
    let backend_fd = SimulatorBackend::new(device_fd.clone().into(), None);
    let mut device_tw = TweezerDevice::new(None, None, None);
    device_tw.add_layout("test").unwrap();
    device_tw.add_layout("test2").unwrap();
    let backend_tw = SimulatorBackend::new(device_tw.clone().into(), None);

    // Test Debug trait
    assert_eq!(
        format!("{:?}", backend_fd),
        format!(
            "SimulatorBackend {{ device: QRydDevice(FirstDevice({:?})), number_qubits: 2 }}",
            device_fd
        )
    );
    assert_eq!(
        format!("{:?}", backend_tw),
        format!(
            "SimulatorBackend {{ device: TweezerDevice({:?}), number_qubits: 0 }}",
            device_tw
        )
    );

    // Test Clone trait
    assert_eq!(backend_fd.clone(), backend_fd);
    assert_eq!(backend_tw.clone(), backend_tw);

    // Test PartialEq trait
    let device_0 = FirstDevice::new(
        2,
        2,
        &[1, 1],
        3.0,
        array![[0.0, 1.0], [0.0, 1.0]],
        None,
        None,
        None,
        None,
    )
    .unwrap()
    .add_layout(1, layout.clone())
    .unwrap();
    let backend_0 = SimulatorBackend::new(device_0.into(), None);
    let device_1 = FirstDevice::new(
        2,
        2,
        &[1, 1],
        2.0,
        array![[0.0, 1.0], [0.0, 1.0]],
        None,
        None,
        None,
        None,
    )
    .unwrap()
    .add_layout(1, layout)
    .unwrap();
    let backend_1 = SimulatorBackend::new(device_1.into(), None);
    assert!(backend_0 == backend_fd);
    assert!(backend_fd == backend_0);
    assert!(backend_1 != backend_fd);
    assert!(backend_fd != backend_1);

    let mut device_0 = TweezerDevice::new(None, None, None);
    device_0.add_layout("test").unwrap();
    device_0.add_layout("test2").unwrap();
    let mut device_1 = TweezerDevice::new(None, None, None);
    device_1.add_layout("different").unwrap();
    let backend_0 = SimulatorBackend::new(device_0.into(), None);
    let backend_1 = SimulatorBackend::new(device_1.into(), None);
    assert!(backend_0 == backend_tw);
    assert!(backend_tw == backend_0);
    assert!(backend_1 != backend_tw);
    assert!(backend_tw != backend_1);
}

/// Test .run_circuit() with a simple circuit 
#[test]
fn test_simple_circuit() {
    let layout: Array2<f64> = array![[0.0, 1.0], [0.0, 1.0]];
    let mut device = FirstDevice::new(
        2,
        2,
        &[1, 1],
        3.0,
        array![[0.0, 1.0], [0.0, 1.0]],
        None,
        None,
        None,
        None,
    )
    .unwrap()
    .add_layout(1, layout)
    .unwrap();
    device.switch_layout(&1).unwrap();
    let backend_fd = SimulatorBackend::new(device.into(), None);

    let mut device = TweezerDevice::new(None, None, None);
    device.add_layout("test").unwrap();
    device
        .set_tweezer_single_qubit_gate_time("RotateX", 0, 1.0, Some("test".to_string()))
        .unwrap();
    device
        .set_tweezer_single_qubit_gate_time("RotateX", 1, 1.0, Some("test".to_string()))
        .unwrap();
    device.switch_layout("test", None).unwrap();
    let backend_tw = SimulatorBackend::new(device.into(), None);

    let mut circuit = Circuit::new();
    circuit += DefinitionBit::new("ro".to_string(), 2, true);
    circuit += RotateX::new(0, std::f64::consts::FRAC_PI_2.into());
    circuit += RotateX::new(1, std::f64::consts::FRAC_PI_2.into());
    circuit += PragmaRepeatedMeasurement::new("ro".to_string(), 20, None);
    let (bit_registers_fd, _float_registers, _complex_registers) =
        backend_fd.run_circuit(&circuit).unwrap();
    let (bit_registers_tw, _float_registers, _complex_registers) =
        backend_tw.run_circuit(&circuit).unwrap();

    assert!(bit_registers_fd.contains_key("ro"));
    assert!(bit_registers_tw.contains_key("ro"));

    let out_reg_fd = bit_registers_fd.get("ro").unwrap();
    let out_reg_tw = bit_registers_tw.get("ro").unwrap();

    assert_eq!(out_reg_fd.len(), 20);
    assert_eq!(out_reg_tw.len(), 20);

    for reg in out_reg_fd.iter() {
        assert_eq!(reg.len(), 2);
    }
    for reg in out_reg_tw.iter() {
        assert_eq!(reg.len(), 2);
    }
}

/// Simply test measurement process, not that gate is translated correclty
#[test]
fn test_measurement() {
    let gate: GateOperation = PhaseShiftState1::new(0, std::f64::consts::FRAC_PI_2.into()).into();
    let preparation_gates: Vec<SingleQubitGateOperation> =
        vec![RotateX::new(0, std::f64::consts::FRAC_PI_2.into()).into()];
    let basis_rotation_gates: Vec<SingleQubitGateOperation> =
        vec![RotateY::new(0, std::f64::consts::FRAC_PI_2.into()).into()];
    let (measurement, exp_vals) =
        prepare_monte_carlo_gate_test(gate, preparation_gates, basis_rotation_gates, None, 1, 200);
    let device = FirstDevice::new(1, 1, &[1], 3.0, array![[0.0],], None, None, None, None).unwrap();
    let backend = SimulatorBackend::new(device.into(), None);
    let measured_exp_vals = backend.run_measurement(&measurement).unwrap().unwrap();
    for (key, val) in exp_vals.iter() {
        assert!((val - measured_exp_vals.get(key).unwrap()).abs() < 1.0);
    }
}

/// Test full gate with stochastic application of gates, ignore normally because of length and load
#[test]
fn test_full_simple_gate() {
    let gate: GateOperation = RotateX::new(0, std::f64::consts::FRAC_PI_2.into()).into();
    let preparation_gates: Vec<SingleQubitGateOperation> = vec![
        RotateX::new(0, std::f64::consts::FRAC_PI_2.into()).into(),
        RotateY::new(0, std::f64::consts::FRAC_PI_2.into()).into(),
        PhaseShiftState1::new(0, std::f64::consts::FRAC_PI_2.into()).into(),
    ];
    let basis_rotation_gates: Vec<SingleQubitGateOperation> = vec![
        RotateX::new(0, std::f64::consts::FRAC_PI_2.into()).into(),
        RotateX::new(0, std::f64::consts::FRAC_PI_2.into()).into(),
        PhaseShiftState1::new(0, std::f64::consts::FRAC_PI_2.into()).into(),
    ];
    let (measurement, exp_vals) =
        prepare_monte_carlo_gate_test(gate, preparation_gates, basis_rotation_gates, None, 5, 200);

    let device =
        FirstDevice::new(1, 1, &[1], 3.0, array![[0.0,],], None, None, None, None).unwrap();
    let backend = SimulatorBackend::new(device.into(), None);
    let measured_exp_vals = backend.run_measurement(&measurement).unwrap().unwrap();
    for (key, val) in exp_vals.iter() {
        assert!((val - measured_exp_vals.get(key).unwrap()).abs() < 1.0);
    }
}
