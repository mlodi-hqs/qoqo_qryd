// Copyright © 2023 HQS Quantum Simulations GmbH. All Rights Reserved.
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

use bincode::serialize;
use ndarray::Array2;
use std::collections::HashMap;
use std::env;

use roqoqo::{devices::Device, RoqoqoBackendError};
use roqoqo_qryd::{
    phi_theta_relation, PragmaChangeQRydLayout, PragmaShiftQRydQubit, PragmaShiftQubitsTweezers,
    PragmaSwitchDeviceLayout, TweezerDevice,
};

#[cfg(feature = "web-api")]
use wiremock::matchers::method;
#[cfg(feature = "web-api")]
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Test TweezerDevice new()
#[test]
fn test_new() {
    let device = TweezerDevice::new(Some(2), None, None);

    assert!(device.current_layout.is_none());
    assert!(device.qubit_to_tweezer.is_none());
    assert!(device.layout_register.is_some());
    assert_eq!(device.layout_register.as_ref().unwrap().len(), 0);
    assert_eq!(device.seed(), Some(2));
    assert_eq!(device.qrydbackend(), "qryd_tweezer_device");

    let device_emp = TweezerDevice::new(None, None, None);

    assert_eq!(device_emp.seed(), None);
}

// Test TweezerDevice add_layout(), switch_layout() methods
#[test]
fn test_layouts() {
    let mut device = TweezerDevice::new(None, None, None);

    assert!(device.current_layout.is_none());
    assert!(device.available_layouts().is_empty());

    device.add_layout("default").unwrap();
    device.current_layout = Some("default".to_string());

    assert!(device.available_layouts().contains(&"default"));

    device.add_layout("Test").unwrap();

    assert!(device.add_layout("Test").is_err());

    assert_eq!(device.layout_register.as_ref().unwrap().len(), 2);
    assert!(device
        .layout_register
        .as_ref()
        .unwrap()
        .contains_key("default"));
    assert!(device
        .layout_register
        .as_ref()
        .unwrap()
        .contains_key("Test"));

    device
        .set_tweezer_single_qubit_gate_time("RotateX", 0, 0.23, None)
        .unwrap();
    device
        .set_tweezer_single_qubit_gate_time("RotateZ", 1, 0.23, Some("Test".to_string()))
        .unwrap();
    device
        .set_tweezer_single_qubit_gate_time("PhaseShiftState1", 2, 0.23, Some("Test".to_string()))
        .unwrap();
    device
        .set_tweezer_two_qubit_gate_time("PhaseShiftedControlledPhase", 0, 1, 0.23, None)
        .unwrap();
    device
        .set_tweezer_two_qubit_gate_time(
            "PhaseShiftedControlledPhase",
            0,
            1,
            0.23,
            Some("Test".to_string()),
        )
        .unwrap();
    device
        .set_tweezer_three_qubit_gate_time("ControlledControlledPauliZ", 0, 1, 2, 0.34, None)
        .unwrap();
    device
        .set_tweezer_three_qubit_gate_time(
            "ControlledControlledPauliZ",
            0,
            1,
            2,
            0.34,
            Some("Test".to_string()),
        )
        .unwrap();
    // TODO: Add this back in when the backend supports multi-qubit gates
    // device
    //     .set_tweezer_multi_qubit_gate_time("MultiQubitZZ", &[0, 1, 2, 3], 0.13, None)
    //     .unwrap();
    // device
    //     .set_tweezer_multi_qubit_gate_time(
    //         "MultiQubitZZ",
    //         &[0, 1, 2, 3],
    //         0.13,
    //         Some("Test".to_string()),
    //     )
    //     .unwrap();

    let default_layout = device
        .layout_register
        .as_ref()
        .unwrap()
        .get("default")
        .unwrap();
    let test_layout = device
        .layout_register
        .as_ref()
        .unwrap()
        .get("Test")
        .unwrap();
    assert!(default_layout
        .tweezer_single_qubit_gate_times
        .contains_key("RotateX"));
    assert!(default_layout
        .tweezer_single_qubit_gate_times
        .get("RotateX")
        .unwrap()
        .get(&0)
        .is_some());
    assert_eq!(
        *default_layout
            .tweezer_single_qubit_gate_times
            .get("RotateX")
            .unwrap()
            .get(&0)
            .unwrap(),
        0.23
    );
    assert_eq!(test_layout.tweezer_single_qubit_gate_times.len(), 2);
    assert_eq!(
        *test_layout
            .tweezer_single_qubit_gate_times
            .get("RotateZ")
            .unwrap()
            .get(&1)
            .unwrap(),
        0.23
    );
    assert_eq!(
        *test_layout
            .tweezer_single_qubit_gate_times
            .get("PhaseShiftState1")
            .unwrap()
            .get(&2)
            .unwrap(),
        0.23
    );

    assert_eq!(
        *default_layout
            .tweezer_two_qubit_gate_times
            .get("PhaseShiftedControlledPhase")
            .unwrap()
            .get(&(0, 1))
            .unwrap(),
        0.23
    );
    assert_eq!(
        *test_layout
            .tweezer_two_qubit_gate_times
            .get("PhaseShiftedControlledPhase")
            .unwrap()
            .get(&(0, 1))
            .unwrap(),
        0.23
    );

    assert_eq!(
        *default_layout
            .tweezer_three_qubit_gate_times
            .get("ControlledControlledPauliZ")
            .unwrap()
            .get(&(0, 1, 2))
            .unwrap(),
        0.34
    );
    assert_eq!(
        *test_layout
            .tweezer_three_qubit_gate_times
            .get("ControlledControlledPauliZ")
            .unwrap()
            .get(&(0, 1, 2))
            .unwrap(),
        0.34
    );

    // TODO: Add this back in when the backend supports multi-qubit gates
    // assert_eq!(
    //     *default_layout
    //         .tweezer_multi_qubit_gate_times
    //         .get("MultiQubitZZ")
    //         .unwrap()
    //         .get(&[0, 1, 2, 3].to_vec())
    //         .unwrap(),
    //     0.13
    // );
    // assert_eq!(
    //     *test_layout
    //         .tweezer_multi_qubit_gate_times
    //         .get("MultiQubitZZ")
    //         .unwrap()
    //         .get(&[0, 1, 2, 3].to_vec())
    //         .unwrap(),
    //     0.13
    // );

    assert_eq!(device.current_layout, Some("default".to_string()));
    assert!(device.qubit_to_tweezer.is_none());

    device.switch_layout("Test", None).unwrap();
    assert_eq!(device.current_layout, Some("Test".to_string()));
    assert!(device.qubit_to_tweezer.is_some());
    assert_eq!(device.qubit_to_tweezer.clone().unwrap().len(), 3);

    assert!(device.switch_layout("Error", None).is_err());

    assert!(device.available_layouts().contains(&"Test"));

    device.add_layout("test_trivial_population").unwrap();
    device
        .set_tweezer_single_qubit_gate_time(
            "RotateZ",
            1,
            0.23,
            Some("test_trivial_population".to_string()),
        )
        .unwrap();
    device
        .set_tweezer_single_qubit_gate_time(
            "RotateX",
            2,
            0.23,
            Some("test_trivial_population".to_string()),
        )
        .unwrap();
    device
        .switch_layout("test_trivial_population", Some(false))
        .unwrap();

    assert!(device.qubit_to_tweezer.is_none());

    device
        .switch_layout("test_trivial_population", Some(true))
        .unwrap();

    assert_eq!(
        device.qubit_to_tweezer.unwrap(),
        HashMap::from([(0, 0), (1, 1), (2, 2)])
    );
}

// Test TweezerDevice add_qubit_tweezer_mapping(), get_tweezer_from_qubit() methods
#[test]
fn test_qubit_tweezer_mapping() {
    let mut device = TweezerDevice::new(None, None, None);
    device.add_layout("default").unwrap();
    device.current_layout = Some("default".to_string());

    assert!(device.add_qubit_tweezer_mapping(0, 0).is_err());
    assert!(device.get_tweezer_from_qubit(&0).is_err());

    device
        .set_tweezer_single_qubit_gate_time("RotateX", 0, 0.0, None)
        .unwrap();
    device
        .set_tweezer_three_qubit_gate_time("ControlledControlledPauliZ", 1, 2, 3, 0.1, None)
        .unwrap();

    assert!(device.add_qubit_tweezer_mapping(0, 0).is_ok());
    assert!(device.add_qubit_tweezer_mapping(2, 3).is_ok());

    assert_eq!(device.get_tweezer_from_qubit(&0).unwrap(), 0);
    assert_eq!(device.get_tweezer_from_qubit(&2).unwrap(), 3);
    assert!(device.get_tweezer_from_qubit(&4).is_err());

    let add_01 = device.add_qubit_tweezer_mapping(0, 1);
    assert!(add_01.is_ok());
    assert_eq!(add_01.unwrap(), vec![(0, 1), (2, 3)].into_iter().collect());
}

/// Test TweezerDevice set_allowed_tweezer_shifts_from_rows() method
#[test]
fn test_allowed_tweezer_shifts_from_rows() {
    let mut device = TweezerDevice::new(None, None, None);
    device.add_layout("triangle").unwrap();
    device
        .set_tweezer_single_qubit_gate_time("RotateX", 0, 0.0, Some("triangle".to_string()))
        .unwrap();
    device
        .set_tweezer_single_qubit_gate_time("RotateX", 1, 0.0, Some("triangle".to_string()))
        .unwrap();
    device
        .set_tweezer_single_qubit_gate_time("RotateX", 2, 0.0, Some("triangle".to_string()))
        .unwrap();
    device
        .set_tweezer_single_qubit_gate_time("RotateX", 3, 0.0, Some("triangle".to_string()))
        .unwrap();
    device
        .set_tweezer_single_qubit_gate_time("RotateX", 4, 0.0, Some("triangle".to_string()))
        .unwrap();
    device
        .set_tweezer_single_qubit_gate_time("RotateX", 5, 0.0, Some("triangle".to_string()))
        .unwrap();

    assert!(device
        .set_allowed_tweezer_shifts_from_rows(
            &[&[0, 1, 2], &[3, 4, 5]],
            Some("triangle".to_string())
        )
        .is_ok());

    let saved_shifts = &device
        .layout_register
        .as_ref()
        .unwrap()
        .get("triangle")
        .unwrap()
        .allowed_tweezer_shifts;
    assert!(!saved_shifts.is_empty());
    assert!(saved_shifts.contains_key(&0));
    assert!(saved_shifts.get(&0).unwrap().contains(&vec![1, 2]));
    assert!(saved_shifts.get(&1).unwrap().contains(&vec![0]));
    assert!(saved_shifts.get(&1).unwrap().contains(&vec![2]));
    assert!(saved_shifts.get(&2).unwrap().contains(&vec![1, 0]));
    assert!(saved_shifts.get(&3).unwrap().contains(&vec![4, 5]));
    assert!(saved_shifts.get(&4).unwrap().contains(&vec![3]));
    assert!(saved_shifts.get(&4).unwrap().contains(&vec![5]));
    assert!(saved_shifts.get(&5).unwrap().contains(&vec![4, 3]));

    let incorrect_tweezer =
        device.set_allowed_tweezer_shifts_from_rows(&[&[9]], Some("triangle".to_string()));
    assert!(incorrect_tweezer.is_err());
    assert_eq!(
        incorrect_tweezer.unwrap_err(),
        RoqoqoBackendError::GenericError {
            msg: "A given Tweezer is not present in the device Tweezer data.".to_string(),
        }
    );

    let adding_to_already_present_key = device.set_allowed_tweezer_shifts_from_rows(
        &[&[0, 3], &[1, 4], &[2, 5]],
        Some("triangle".to_string()),
    );
    assert!(adding_to_already_present_key.is_ok());

    let saved_shifts = &device
        .layout_register
        .as_ref()
        .unwrap()
        .get("triangle")
        .unwrap()
        .allowed_tweezer_shifts;
    assert!(saved_shifts.get(&0).unwrap().contains(&vec![1, 2]));
    assert!(saved_shifts.get(&0).unwrap().contains(&vec![3]));
    assert!(saved_shifts.get(&1).unwrap().contains(&vec![0]));
    assert!(saved_shifts.get(&1).unwrap().contains(&vec![2]));
    assert!(saved_shifts.get(&1).unwrap().contains(&vec![4]));
    assert!(saved_shifts.get(&2).unwrap().contains(&vec![1, 0]));
    assert!(saved_shifts.get(&2).unwrap().contains(&vec![5]));
    assert!(saved_shifts.get(&3).unwrap().contains(&vec![4, 5]));
    assert!(saved_shifts.get(&3).unwrap().contains(&vec![0]));
    assert!(saved_shifts.get(&4).unwrap().contains(&vec![3]));
    assert!(saved_shifts.get(&4).unwrap().contains(&vec![5]));
    assert!(saved_shifts.get(&4).unwrap().contains(&vec![1]));
    assert!(saved_shifts.get(&5).unwrap().contains(&vec![4, 3]));
    assert!(saved_shifts.get(&5).unwrap().contains(&vec![2]));
}

/// Test TweezerDevice set_allowed_tweezer_shifts() method
#[test]
fn test_allowed_tweezer_shifts() {
    let mut device = TweezerDevice::new(None, None, None);
    device.add_layout("OtherLayout").unwrap();
    device
        .set_tweezer_single_qubit_gate_time("RotateX", 0, 0.0, Some("OtherLayout".to_string()))
        .unwrap();
    device
        .set_tweezer_single_qubit_gate_time("RotateX", 1, 0.0, Some("OtherLayout".to_string()))
        .unwrap();
    device
        .set_tweezer_single_qubit_gate_time("RotateX", 2, 0.0, Some("OtherLayout".to_string()))
        .unwrap();
    device
        .set_tweezer_single_qubit_gate_time("RotateX", 3, 0.0, Some("OtherLayout".to_string()))
        .unwrap();
    device
        .set_tweezer_single_qubit_gate_time("RotateX", 4, 0.0, Some("OtherLayout".to_string()))
        .unwrap();
    device.switch_layout("OtherLayout", None).unwrap();

    assert!(device
        .set_allowed_tweezer_shifts(&0, &[&[1], &[2]], Some("OtherLayout".to_string()))
        .is_ok());

    let saved_shifts = &device
        .layout_register
        .as_ref()
        .unwrap()
        .get("OtherLayout")
        .unwrap()
        .allowed_tweezer_shifts;
    assert!(!saved_shifts.is_empty());
    assert!(saved_shifts.contains_key(&0));
    assert!(saved_shifts.get(&0).unwrap().contains(&vec![1]));
    assert!(saved_shifts.get(&0).unwrap().contains(&vec![2]));

    let incorrect_shift_list =
        device.set_allowed_tweezer_shifts(&0, &[&[0]], Some("OtherLayout".to_string()));
    assert!(incorrect_shift_list.is_err());
    assert_eq!(
        incorrect_shift_list.unwrap_err(),
        RoqoqoBackendError::GenericError {
            msg: "The allowed shifts contain the given tweezer.".to_string(),
        }
    );

    let incorrect_tweezer = device.set_allowed_tweezer_shifts(&5, &[&[0]], None);
    assert!(incorrect_tweezer.is_err());
    assert_eq!(
        incorrect_tweezer.unwrap_err(),
        RoqoqoBackendError::GenericError {
            msg:
                "The given tweezer, or shifts tweezers, are not present in the device Tweezer data."
                    .to_string(),
        }
    );

    let incorrect_shift_list_2 = device.set_allowed_tweezer_shifts(&2, &[&[5]], None);
    assert!(incorrect_shift_list_2.is_err());
    assert_eq!(
        incorrect_shift_list_2.unwrap_err(),
        RoqoqoBackendError::GenericError {
            msg:
                "The given tweezer, or shifts tweezers, are not present in the device Tweezer data."
                    .to_string(),
        }
    );

    assert!(device
        .set_allowed_tweezer_shifts(&0, &[&[3], &[4]], Some("OtherLayout".to_string()))
        .is_ok());

    let saved_shifts = &device
        .layout_register
        .as_ref()
        .unwrap()
        .get("OtherLayout")
        .unwrap()
        .allowed_tweezer_shifts;
    assert!(saved_shifts.contains_key(&0));
    assert!(saved_shifts.get(&0).unwrap().contains(&vec![3]));
    assert!(saved_shifts.get(&0).unwrap().contains(&vec![4]));
}

/// Test TweezerDevice deactivate_qubit()
#[test]
fn test_deactivate_qubit() {
    let mut device = TweezerDevice::new(None, None, None);
    device.add_layout("default").unwrap();
    device.current_layout = Some("default".to_string());

    assert!(device.deactivate_qubit(0).is_err());

    device
        .set_tweezer_single_qubit_gate_time("RotateX", 1, 0.1, None)
        .unwrap();
    device.add_qubit_tweezer_mapping(0, 1).unwrap();

    assert!(device.deactivate_qubit(0).is_ok());
    assert!(device.deactivate_qubit(0).is_err());
}

/// Test TweezerDevice ..._qubit_gate_time() methods
#[test]
fn test_qubit_times() {
    let mut device = TweezerDevice::new(None, None, None);
    device.add_layout("default").unwrap();
    device.current_layout = Some("default".to_string());

    assert!(device.single_qubit_gate_time("RotateX", &0).is_none());

    // Testing missing qubits
    assert!(device.single_qubit_gate_time("RotateX", &5).is_none());
    assert!(device
        .two_qubit_gate_time("PhaseShiftedControlledPhase", &0, &7)
        .is_none());
    assert!(device
        .three_qubit_gate_time("ControlledControlledPhaseShift", &12, &1, &3)
        .is_none());
    assert!(device
        .multi_qubit_gate_time("MultiQubitZZ", &[6, 2, 3, 4])
        .is_none());

    device
        .set_tweezer_single_qubit_gate_time("RotateX", 1, 0.23, None)
        .unwrap();
    device
        .set_tweezer_two_qubit_gate_time("PhaseShiftedControlledPhase", 0, 1, 0.45, None)
        .unwrap();
    device
        .set_tweezer_three_qubit_gate_time("ControlledControlledPhaseShift", 0, 1, 2, 0.65, None)
        .unwrap();
    // TODO: Add this back in when the backend supports multi-qubit gates
    // device
    //     .set_tweezer_multi_qubit_gate_time("MultiQubitZZ", &[0, 1, 2, 3], 0.34, None)
    //     .unwrap();

    device.add_qubit_tweezer_mapping(0, 1).unwrap();
    device.add_qubit_tweezer_mapping(1, 2).unwrap();
    // TODO: Add this back in when the backend supports multi-qubit gates
    // device.add_qubit_tweezer_mapping(2, 3).unwrap();
    device.add_qubit_tweezer_mapping(3, 0).unwrap();

    assert!(device.single_qubit_gate_time("RotateX", &0).is_some());
    assert_eq!(device.single_qubit_gate_time("RotateX", &0).unwrap(), 0.23);
    assert_eq!(
        device
            .two_qubit_gate_time("PhaseShiftedControlledPhase", &3, &0)
            .unwrap(),
        0.45
    );
    assert_eq!(
        device
            .three_qubit_gate_time("ControlledControlledPhaseShift", &3, &0, &1)
            .unwrap(),
        0.65
    );
    // TODO: Add this back in when the backend supports multi-qubit gates
    // assert_eq!(
    //     device
    //         .multi_qubit_gate_time("MultiQubitZZ", &[3, 0, 1, 2])
    //         .unwrap(),
    //     0.34
    // );
}

/// Test TweezerDevice number_qubits() method
#[test]
fn test_number_qubits() {
    let mut device = TweezerDevice::new(None, None, None);
    device.add_layout("default").unwrap();
    device.current_layout = Some("default".to_string());

    assert_eq!(device.number_qubits(), 0);

    device
        .set_tweezer_single_qubit_gate_time("RotateX", 0, 0.0, None)
        .unwrap();
    device
        .set_tweezer_single_qubit_gate_time("RotateX", 1, 0.0, None)
        .unwrap();

    assert_eq!(device.number_qubits(), 0);

    device.add_qubit_tweezer_mapping(0, 0).unwrap();
    device.add_qubit_tweezer_mapping(1, 1).unwrap();

    assert_eq!(device.number_qubits(), 2)
}

/// Test TweezerDevice number_tweezer_positions() method
#[test]
fn test_number_tweezer_positions() {
    let mut device = TweezerDevice::new(None, None, None);
    device.add_layout("default").unwrap();
    device.add_layout("empty").unwrap();

    assert_eq!(
        device.number_tweezer_positions(Some("empty".to_string())),
        Ok(0)
    );

    assert!(device.number_tweezer_positions(None).is_err());
    assert!(device
        .number_tweezer_positions(Some("error".to_string()))
        .is_err());

    device.current_layout = Some("default".to_string());
    device
        .set_tweezer_single_qubit_gate_time("RotateX", 0, 0.23, None)
        .unwrap();
    device
        .set_tweezer_two_qubit_gate_time("PhaseShiftedControlledPhase", 1, 7, 0.23, None)
        .unwrap();
    device
        .set_tweezer_three_qubit_gate_time("ControlledControlledPhaseShift", 2, 9, 13, 0.34, None)
        .unwrap();
    // TODO: Add this back in when the backend supports multi-qubit gates
    // device
    //     .set_tweezer_multi_qubit_gate_time("MultiQubitZZ", &[1, 12, 5], 0.34, None)
    //     .unwrap();

    assert_eq!(device.number_tweezer_positions(None), Ok(6));
}

/// Test TweezerDevice to_generic_device() method
#[test]
fn test_to_generic_device() {
    let mut device = TweezerDevice::new(None, None, None);
    device.add_layout("default").unwrap();
    device.current_layout = Some("default".to_string());
    device
        .set_tweezer_single_qubit_gate_time("RotateX", 0, 0.23, None)
        .unwrap();
    device
        .set_tweezer_single_qubit_gate_time("RotateZ", 1, 0.23, None)
        .unwrap();
    device
        .set_tweezer_two_qubit_gate_time("PhaseShiftedControlledPhase", 2, 3, 0.34, None)
        .unwrap();
    device
        .set_tweezer_two_qubit_gate_time("PhaseShiftedControlledZ", 1, 2, 0.34, None)
        .unwrap();
    device.add_qubit_tweezer_mapping(0, 0).unwrap();
    device.add_qubit_tweezer_mapping(1, 1).unwrap();
    device.add_qubit_tweezer_mapping(2, 2).unwrap();
    device.add_qubit_tweezer_mapping(3, 3).unwrap();

    let generic_device = device.to_generic_device();

    assert_eq!(
        generic_device
            .single_qubit_gates
            .get("RotateX")
            .unwrap()
            .get(&0)
            .unwrap(),
        &0.23
    );
    assert_eq!(
        generic_device
            .single_qubit_gates
            .get("RotateZ")
            .unwrap()
            .get(&1)
            .unwrap(),
        &0.23
    );
    assert_eq!(
        generic_device
            .two_qubit_gates
            .get("PhaseShiftedControlledPhase")
            .unwrap()
            .get(&(2, 3))
            .unwrap(),
        &0.34
    );
    assert_eq!(
        generic_device
            .two_qubit_gates
            .get("PhaseShiftedControlledZ")
            .unwrap()
            .get(&(1, 2))
            .unwrap(),
        &0.34
    );
    assert_eq!(
        generic_device.qubit_decoherence_rates(&0),
        Some(Array2::zeros((3, 3).to_owned()))
    );
    assert_eq!(
        generic_device.qubit_decoherence_rates(&1),
        Some(Array2::zeros((3, 3).to_owned()))
    );
}

/// Test TweezerDevice change_device() method
#[test]
fn test_change_device() {
    let mut device = TweezerDevice::new(None, None, None);
    device.add_layout("Test").unwrap();
    device
        .set_tweezer_single_qubit_gate_time("RotateXY", 0, 0.23, Some("Test".to_string()))
        .unwrap();
    device
        .set_tweezer_single_qubit_gate_time("RotateZ", 1, 0.23, Some("Test".to_string()))
        .unwrap();
    device
        .set_tweezer_two_qubit_gate_time(
            "PhaseShiftedControlledZ",
            2,
            3,
            0.34,
            Some("Test".to_string()),
        )
        .unwrap();
    device
        .set_tweezer_two_qubit_gate_time(
            "PhaseShiftedControlledPhase",
            1,
            2,
            0.34,
            Some("Test".to_string()),
        )
        .unwrap();
    let pragma_old_c = PragmaChangeQRydLayout::new(0);
    let hm: HashMap<usize, (usize, usize)> = [(0, (1, 2))].into_iter().collect();
    let pragma_old_s = PragmaShiftQRydQubit::new(hm);

    assert!(device.change_device("Error", &Vec::<u8>::new()).is_err());
    assert!(device
        .change_device("PragmaChangeQRydLayout", &Vec::<u8>::new())
        .is_err());
    assert!(device.current_layout.is_none());
    assert!(device
        .change_device("PragmaChangeQRydLayout", &serialize(&pragma_old_c).unwrap())
        .is_err());

    assert!(device
        .change_device("PragmaShiftQRydQubit", &serialize(&pragma_old_s).unwrap())
        .is_err());
}

/// Test TweezerDevice change_device() method with PragmaSwitchDeviceLayout
#[test]
fn test_change_device_switch() {
    let mut device = TweezerDevice::new(None, None, None);
    device.add_layout("two_rows_two_twzrs_0").unwrap();
    device
        .set_tweezer_two_qubit_gate_time(
            "PhaseShiftedControlledZ",
            0,
            1,
            0.23,
            Some("two_rows_two_twzrs_0".to_string()),
        )
        .unwrap();
    device
        .set_tweezer_two_qubit_gate_time(
            "PhaseShiftedControlledZ",
            2,
            3,
            0.23,
            Some("two_rows_two_twzrs_0".to_string()),
        )
        .unwrap();
    device
        .set_tweezer_two_qubit_gate_time(
            "PhaseShiftedControlledZ",
            0,
            2,
            0.34,
            Some("two_rows_two_twzrs_0".to_string()),
        )
        .unwrap();
    device
        .set_tweezer_two_qubit_gate_time(
            "PhaseShiftedControlledZ",
            1,
            3,
            0.34,
            Some("two_rows_two_twzrs_0".to_string()),
        )
        .unwrap();
    device.add_layout("two_rows_two_twzrs_1").unwrap();
    device
        .set_tweezer_two_qubit_gate_time(
            "PhaseShiftedControlledZ",
            0,
            1,
            0.23,
            Some("two_rows_two_twzrs_1".to_string()),
        )
        .unwrap();
    device
        .set_tweezer_two_qubit_gate_time(
            "PhaseShiftedControlledZ",
            2,
            3,
            0.23,
            Some("two_rows_two_twzrs_1".to_string()),
        )
        .unwrap();
    device
        .set_tweezer_two_qubit_gate_time(
            "PhaseShiftedControlledZ",
            0,
            3,
            0.34,
            Some("two_rows_two_twzrs_1".to_string()),
        )
        .unwrap();
    device.add_layout("one_row_three_twzrs").unwrap();
    device
        .set_tweezer_three_qubit_gate_time(
            "ControlledControlledPauliZ",
            0,
            1,
            2,
            0.4,
            Some("one_row_three_twzrs".to_string()),
        )
        .unwrap();
    device.add_layout("no_twzrs_per_row_set").unwrap();

    assert!(device.set_tweezers_per_row(vec![5, 2, 3], None).is_err());

    device
        .switch_layout("two_rows_two_twzrs_0", Some(true))
        .unwrap();

    device
        .set_tweezers_per_row(vec![2, 2], Some("two_rows_two_twzrs_0".to_string()))
        .unwrap();
    device
        .set_tweezers_per_row(vec![2, 2], Some("two_rows_two_twzrs_1".to_string()))
        .unwrap();
    device
        .set_tweezers_per_row(vec![3], Some("one_row_three_twzrs".to_string()))
        .unwrap();

    let pragma_correct = PragmaSwitchDeviceLayout::new("two_rows_two_twzrs_1".to_string());
    let pragma_incorrect_0 = PragmaSwitchDeviceLayout::new("one_row_three_twzrs".to_string());
    let pragma_incorrect_1 = PragmaSwitchDeviceLayout::new("no_twzrs_per_row_set".to_string());
    let pragma_incorrect_2 = PragmaSwitchDeviceLayout::new("non_existant_layout".to_string());

    assert!(device
        .change_device("PragmaSwitchDeviceLayout", &Vec::<u8>::new())
        .is_err());

    assert!(device
        .change_device(
            "PragmaSwitchDeviceLayout",
            &serialize(&pragma_correct).unwrap()
        )
        .is_ok());
    assert_eq!(
        device.current_layout,
        Some("two_rows_two_twzrs_1".to_string())
    );

    let wrong_switch = device.change_device(
        "PragmaSwitchDeviceLayout",
        &serialize(&pragma_incorrect_0).unwrap(),
    );
    assert!(wrong_switch.is_err());
    assert_eq!(
        wrong_switch.unwrap_err().to_string(),
        "An error occured in the backend: Error with dynamic layout switching of TweezerDevice. Current tweezers per row is [2, 2] but switching to a layout with [3] tweezers per row. ".to_string(),
    );

    let wrong_switch = device.change_device(
        "PragmaSwitchDeviceLayout",
        &serialize(&pragma_incorrect_1).unwrap(),
    );
    assert!(wrong_switch.is_err());
    assert_eq!(
        wrong_switch.unwrap_err().to_string(),
        "An error occured in the backend: Error with dynamic layout switching of TweezerDevice. Tweezers per row info missing from current or new layout. ".to_string(),
    );

    let wrong_switch = device.change_device(
        "PragmaSwitchDeviceLayout",
        &serialize(&pragma_incorrect_2).unwrap(),
    );
    assert!(wrong_switch.is_err());
    assert_eq!(
        wrong_switch.unwrap_err().to_string(),
        "An error occured in the backend: Error with dynamic layout switching of TweezerDevice. Layout non_existant_layout is not set. ".to_string(),
    );
}

/// Test TweezerDevice allow_reset field
#[test]
fn test_allow_reset() {
    let mut device = TweezerDevice::new(None, None, None);
    assert!(!device.allow_reset);
    assert!(device.set_allow_reset(true).is_ok());
    assert!(device.allow_reset);
}

/// Test TweezerDevice change_device() method with PragmaShiftQubitsTweezers
#[test]
fn test_change_device_shift() {
    let mut device = TweezerDevice::new(None, None, None);
    device.add_layout("triangle").unwrap();
    device
        .set_tweezer_single_qubit_gate_time("RotateZ", 0, 0.23, Some("triangle".to_string()))
        .unwrap();
    device
        .set_tweezer_single_qubit_gate_time("RotateX", 1, 0.23, Some("triangle".to_string()))
        .unwrap();
    device
        .set_tweezer_two_qubit_gate_time(
            "PhaseShiftedControlledZ",
            2,
            3,
            0.34,
            Some("triangle".to_string()),
        )
        .unwrap();
    device
        .set_tweezer_two_qubit_gate_time(
            "PhaseShiftedControlledPhase",
            1,
            2,
            0.34,
            Some("triangle".to_string()),
        )
        .unwrap();
    device
        .set_tweezer_two_qubit_gate_time(
            "PhaseShiftedControlledPhase",
            4,
            5,
            0.34,
            Some("triangle".to_string()),
        )
        .unwrap();
    device
        .set_allowed_tweezer_shifts(&0, &[&[1, 2], &[3]], Some("triangle".to_string()))
        .unwrap();
    device
        .set_allowed_tweezer_shifts(&1, &[&[4, 5]], Some("triangle".to_string()))
        .unwrap();

    let pragma_s = PragmaShiftQubitsTweezers::new(vec![(0, 1), (2, 3)]);

    let err1 = device.change_device("PragmaShiftQubitsTweezers", &serialize(&pragma_s).unwrap());
    assert!(err1.is_err());
    assert_eq!(
        err1.unwrap_err(),
        RoqoqoBackendError::GenericError {
            msg: "The device qubit -> tweezer mapping is empty: no qubits to shift.".to_string(),
        }
    );

    device.current_layout = Some("triangle".to_string());
    device.add_qubit_tweezer_mapping(0, 0).unwrap();
    device.add_qubit_tweezer_mapping(1, 1).unwrap();
    device.add_qubit_tweezer_mapping(2, 2).unwrap();

    let err2 = device.change_device("PragmaShiftQubitsTweezers", &serialize(&pragma_s).unwrap());
    assert!(err2.is_err());
    assert_eq!(
        err2.unwrap_err(),
        RoqoqoBackendError::GenericError {
            msg: "The PragmaShiftQubitsTweezers operation is not valid on this device.".to_string(),
        }
    );

    device
        .set_allowed_tweezer_shifts(&2, &[&[3]], None)
        .unwrap();

    // Target already occupied
    let err3 = device.change_device("PragmaShiftQubitsTweezers", &serialize(&pragma_s).unwrap());
    assert!(err3.is_err());

    device.deactivate_qubit(1).unwrap();

    let ok = device.change_device("PragmaShiftQubitsTweezers", &serialize(&pragma_s).unwrap());
    assert!(ok.is_ok());
    assert_eq!(device.qubit_to_tweezer.as_ref().unwrap().len(), 2);
    assert_eq!(
        device.qubit_to_tweezer.as_ref().unwrap().get(&0).unwrap(),
        &1
    );
    assert_eq!(
        device.qubit_to_tweezer.as_ref().unwrap().get(&2).unwrap(),
        &3
    );

    device.add_qubit_tweezer_mapping(4, 4).unwrap();

    // Path is blocked
    let pragma_s = PragmaShiftQubitsTweezers::new(vec![(1, 5)]);
    let err4 = device.change_device("PragmaShiftQubitsTweezers", &serialize(&pragma_s).unwrap());
    assert!(err4.is_err());
}

/// Test TweezerDevice change_device() method with PragmaShiftQubitsTweezers (whole row)
#[test]
fn test_change_device_shift_row() {
    let mut device = TweezerDevice::new(None, None, None);
    device.add_layout("row").unwrap();
    device
        .set_tweezer_two_qubit_gate_time(
            "PhaseShiftedControlledZ",
            0,
            1,
            0.34,
            Some("row".to_string()),
        )
        .unwrap();
    device
        .set_tweezer_two_qubit_gate_time(
            "PhaseShiftedControlledPhase",
            1,
            2,
            0.34,
            Some("row".to_string()),
        )
        .unwrap();
    device
        .set_tweezer_two_qubit_gate_time(
            "PhaseShiftedControlledPhase",
            2,
            3,
            0.34,
            Some("row".to_string()),
        )
        .unwrap();
    device
        .set_tweezer_two_qubit_gate_time(
            "PhaseShiftedControlledPhase",
            4,
            1,
            0.34,
            Some("row".to_string()),
        )
        .unwrap();
    device
        .set_allowed_tweezer_shifts(&0, &[&[1, 2, 3]], Some("row".to_string()))
        .unwrap();
    device
        .set_allowed_tweezer_shifts(&1, &[&[0], &[2, 3]], Some("row".to_string()))
        .unwrap();
    device
        .set_allowed_tweezer_shifts(&2, &[&[1, 0], &[3]], Some("row".to_string()))
        .unwrap();
    device
        .set_allowed_tweezer_shifts(&3, &[&[2, 1, 0]], Some("row".to_string()))
        .unwrap();

    device.current_layout = Some("row".to_string());
    device.add_qubit_tweezer_mapping(0, 0).unwrap();
    device.add_qubit_tweezer_mapping(1, 1).unwrap();
    device.add_qubit_tweezer_mapping(2, 2).unwrap();

    let mut cloned = device.clone();

    let pragma_s = PragmaShiftQubitsTweezers::new(vec![(2, 3), (1, 2), (0, 1)]);

    let res = cloned.change_device("PragmaShiftQubitsTweezers", &serialize(&pragma_s).unwrap());

    assert!(res.is_ok());
    assert_eq!(
        cloned.qubit_to_tweezer.unwrap(),
        HashMap::from([(2, 3), (1, 2), (0, 1)])
    );

    device
        .set_allowed_tweezer_shifts(&4, &[&[1]], Some("row".to_string()))
        .unwrap();

    device.add_qubit_tweezer_mapping(4, 4).unwrap();

    let pragma_s = PragmaShiftQubitsTweezers::new(vec![(2, 3), (1, 2), (0, 1), (4, 1)]);

    let res = device.change_device("PragmaShiftQubitsTweezers", &serialize(&pragma_s).unwrap());

    assert!(res.is_err());
}

/// Test TweezerDevice from_api() method
#[tokio::test]
#[cfg(feature = "web-api")]
async fn asnyc_test_from_api() {
    let mut returned_device_default = TweezerDevice::new(None, None, None);
    returned_device_default.add_layout("default").unwrap();
    returned_device_default.current_layout = Some("default".to_string());
    returned_device_default
        .set_tweezer_single_qubit_gate_time("RotateX", 0, 0.23, None)
        .unwrap();
    returned_device_default.device_name = "qryd_emulator".to_string();
    let wiremock_server = MockServer::start().await;
    let port = wiremock_server.address().port().to_string();
    let _mock = Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&returned_device_default))
        .expect(2)
        .mount(&wiremock_server)
        .await;

    let port_cloned = port.clone();
    let response = tokio::task::spawn_blocking(move || {
        TweezerDevice::from_api(None, None, Some(port_cloned), None, None, None)
    })
    .await
    .unwrap();
    assert!(response.is_ok());
    let device = response.unwrap();
    assert_eq!(device, returned_device_default);
    assert_eq!(device.qrydbackend(), "qryd_emulator".to_string());

    let port_cloned = port.clone();
    let response_new_seed = tokio::task::spawn_blocking(move || {
        TweezerDevice::from_api(None, None, Some(port_cloned), Some(42), None, None)
    })
    .await
    .unwrap();
    assert!(response_new_seed.is_ok());
    let device_new_seed = response_new_seed.unwrap();
    assert_eq!(device_new_seed.seed(), Some(42));

    wiremock_server.verify().await;
    wiremock_server.reset().await;

    let _mock = Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(500))
        .expect(1)
        .mount(&wiremock_server)
        .await;

    let port_cloned = port.clone();
    let response = tokio::task::spawn_blocking(move || {
        TweezerDevice::from_api(None, None, Some(port_cloned), None, None, None)
    })
    .await
    .unwrap();
    assert!(response.is_err());
    assert_eq!(
        response.unwrap_err(),
        RoqoqoBackendError::NetworkError {
            msg: format!("Request to server failed with HTTP status code {:?}.", 500),
        }
    );

    wiremock_server.verify().await;
}

#[test]
#[cfg(feature = "web-api")]
fn test_from_api() {
    if env::var("QRYD_API_TOKEN").is_ok() {
        let response = TweezerDevice::from_api(
            None,
            None,
            None,
            None,
            Some(env::var("QRYD_API_HQS").is_ok()),
            None,
        );
        assert!(response.is_ok());
        let device = response.unwrap();
        assert_eq!(device.qrydbackend(), "qryd_emulator".to_string());
        assert!(!device.allow_reset);

        let response_new_seed = TweezerDevice::from_api(
            None,
            None,
            None,
            Some(42),
            Some(env::var("QRYD_API_HQS").is_ok()),
            None,
        );
        assert!(response_new_seed.is_ok());
        let device_new_seed = response_new_seed.unwrap();
        assert_eq!(device_new_seed.seed(), Some(42));

        let response = TweezerDevice::from_api(
            Some("qiskit_emulator".to_string()),
            None,
            None,
            None,
            Some(env::var("QRYD_API_HQS").is_ok()),
            None,
        );
        assert!(response.is_ok());
        let device = response.unwrap();
        assert_eq!(device.qrydbackend(), "qiskit_emulator".to_string());
        assert!(device.allow_reset);
    }
}

/// Test TweezerDevice phase_shift_controlled_...() and gate_time_controlled_...()  methods
#[test]
fn test_phi_theta_relation() {
    let mut device = TweezerDevice::new(None, None, None);
    let mut device_f = TweezerDevice::new(None, Some(2.13.to_string()), Some(2.15.to_string()));
    device.add_layout("default").unwrap();
    device.current_layout = Some("default".to_string());
    device_f.add_layout("default").unwrap();
    device_f.current_layout = Some("default".to_string());

    assert_eq!(
        device.phase_shift_controlled_z().unwrap(),
        phi_theta_relation("DefaultRelation", std::f64::consts::PI).unwrap()
    );
    assert_eq!(
        device.phase_shift_controlled_phase(1.2).unwrap(),
        phi_theta_relation("DefaultRelation", 1.2).unwrap()
    );
    assert_eq!(device_f.phase_shift_controlled_z(), Some(2.13));
    assert_eq!(device_f.phase_shift_controlled_phase(0.3), Some(2.15));

    assert!(device.gate_time_controlled_z(&0, &1, 1.4).is_none());
    assert!(device
        .gate_time_controlled_phase(&0, &1, 1.4, 2.4)
        .is_none());
    assert!(device.gate_time_controlled_z(&0, &7, 1.4).is_none());
    assert!(device
        .gate_time_controlled_phase(&0, &7, 1.4, 2.3)
        .is_none());

    device
        .set_tweezer_two_qubit_gate_time("PhaseShiftedControlledZ", 0, 1, 0.23, None)
        .unwrap();
    device
        .set_tweezer_two_qubit_gate_time("PhaseShiftedControlledPhase", 0, 1, 0.23, None)
        .unwrap();
    device.add_qubit_tweezer_mapping(0, 0).unwrap();
    device.add_qubit_tweezer_mapping(1, 1).unwrap();

    assert!(device
        .gate_time_controlled_z(&0, &1, device.phase_shift_controlled_z().unwrap())
        .is_some());
    assert!(device
        .gate_time_controlled_z(&0, &7, device.phase_shift_controlled_z().unwrap())
        .is_none());
    assert!(device
        .gate_time_controlled_phase(
            &0,
            &1,
            device.phase_shift_controlled_phase(0.1).unwrap(),
            0.1
        )
        .is_some());
    assert!(device
        .gate_time_controlled_phase(
            &0,
            &7,
            device.phase_shift_controlled_phase(0.1).unwrap(),
            0.1
        )
        .is_none());
}

// Test TweezerDevice two_tweezer_edges() method
#[test]
fn test_two_tweezer_edges() {
    let mut device = TweezerDevice::new(None, None, None);
    device.add_layout("default").unwrap();
    device.current_layout = Some("default".to_string());

    assert_eq!(device.two_tweezer_edges().len(), 0);

    device
        .set_tweezer_two_qubit_gate_time("PhaseShiftedControlledPhase", 0, 1, 0.0, None)
        .unwrap();
    device
        .set_tweezer_two_qubit_gate_time("PhaseShiftedControlledPhase", 0, 2, 0.0, None)
        .unwrap();
    device
        .set_tweezer_two_qubit_gate_time("PhaseShiftedControlledPhase", 1, 3, 0.0, None)
        .unwrap();
    device
        .set_tweezer_two_qubit_gate_time("PhaseShiftedControlledPhase", 2, 3, 0.0, None)
        .unwrap();

    assert_eq!(device.two_tweezer_edges().len(), 4);
    assert!(device
        .two_tweezer_edges()
        .iter()
        .all(|el| [(0, 1), (0, 2), (1, 3), (2, 3)].contains(el)));
}

#[test]
fn test_default_layout() {
    let mut device = TweezerDevice::new(None, None, None);
    device.add_layout("triangle").unwrap();
    device
        .set_tweezer_single_qubit_gate_time("RotateZ", 0, 0.23, Some("triangle".to_string()))
        .unwrap();

    assert!(device.set_default_layout("square").is_err());

    assert!(device.set_default_layout("triangle").is_ok());
    assert_eq!(device.default_layout, Some("triangle".to_string()));
    assert_eq!(device.current_layout, Some("triangle".to_string()));
}

#[test]
fn test_setters_native_set_error() {
    let mut device = TweezerDevice::new(None, None, None);
    device.add_layout("layout_name").unwrap();

    let single_setter =
        device.set_tweezer_single_qubit_gate_time("wrong", 0, 1.0, Some("layout_name".to_string()));
    assert!(single_setter.is_err());
    assert!(single_setter.unwrap_err().to_string().contains(
        "Error setting the gate time of a single-qubit gate. Gate wrong is not supported."
    ));

    let two_setter =
        device.set_tweezer_two_qubit_gate_time("wrong", 0, 1, 1.0, Some("layout_name".to_string()));
    assert!(two_setter.is_err());
    assert!(two_setter
        .unwrap_err()
        .to_string()
        .contains("Error setting the gate time of a two-qubit gate. Gate wrong is not supported."));

    let three_setter = device.set_tweezer_three_qubit_gate_time(
        "wrong",
        0,
        1,
        2,
        1.0,
        Some("layout_name".to_string()),
    );
    assert!(three_setter.is_err());
    assert!(three_setter.unwrap_err().to_string().contains(
        "Error setting the gate time of a three-qubit gate. Gate wrong is not supported."
    ));

    let multi_setter = device.set_tweezer_multi_qubit_gate_time(
        "wrong",
        &[0, 1, 2, 3],
        1.0,
        Some("layout_name".to_string()),
    );
    assert!(multi_setter.is_err());
    assert!(multi_setter.unwrap_err().to_string().contains(
        "Error setting the gate time of a multi-qubit gate. Gate wrong is not supported."
    ));
}

#[test]
fn test_available_gate_names() {
    let mut device = TweezerDevice::new(None, None, None);
    device.add_layout("layout_name").unwrap();

    let res = device.get_available_gates_names(None);
    assert!(res
        .unwrap_err()
        .to_string()
        .contains("No layout name provided and no current layout set."));

    let res = device.get_available_gates_names(Some("".to_string()));
    assert_eq!(res.unwrap(), Vec::<&str>::new());

    device
        .set_tweezer_single_qubit_gate_time(
            "PhaseShiftState1",
            0,
            1.0,
            Some("layout_name".to_string()),
        )
        .unwrap();

    assert_eq!(
        device
            .get_available_gates_names(Some("layout_name".to_string()))
            .unwrap(),
        Vec::<&str>::from(&["PhaseShiftState1"])
    );

    let res = device.get_available_gates_names(None);
    assert!(res
        .unwrap_err()
        .to_string()
        .contains("No layout name provided and no current layout set."));

    device
        .set_tweezer_two_qubit_gate_time(
            "PhaseShiftedControlledPhase",
            0,
            1,
            1.0,
            Some("layout_name".to_string()),
        )
        .unwrap();
    device
        .set_tweezer_three_qubit_gate_time(
            "ControlledControlledPauliZ",
            0,
            1,
            2,
            1.0,
            Some("layout_name".to_string()),
        )
        .unwrap();

    let expected_result = Vec::<&str>::from(&[
        "PhaseShiftState1",
        "PhaseShiftedControlledPhase",
        "ControlledControlledPauliZ",
    ]);
    assert_eq!(
        device
            .get_available_gates_names(Some("layout_name".to_string()))
            .unwrap()
            .into_iter()
            .filter(|extracted| expected_result.contains(extracted))
            .count(),
        expected_result.len()
    );

    device.switch_layout("layout_name", None).unwrap();

    assert_eq!(
        device
            .get_available_gates_names(None)
            .unwrap()
            .into_iter()
            .filter(|extracted| expected_result.contains(extracted))
            .count(),
        expected_result.len()
    );
}

#[test]
fn test_render_device() {
    let mut device = TweezerDevice::new(None, None, None);
    device.add_layout("default").unwrap();
    device.current_layout = Some("default".to_string());
    device
        .set_tweezer_single_qubit_gate_time("RotateX", 0, 0.23, None)
        .unwrap();
    device
        .set_tweezer_single_qubit_gate_time("RotateZ", 1, 0.23, None)
        .unwrap();
    device
        .set_tweezer_two_qubit_gate_time("PhaseShiftedControlledPhase", 2, 3, 0.34, None)
        .unwrap();
    device
        .set_tweezer_two_qubit_gate_time("PhaseShiftedControlledZ", 1, 2, 0.34, None)
        .unwrap();
    device
        .set_tweezers_per_row(vec![2, 2], Some("default".to_string()))
        .unwrap();
    device.add_qubit_tweezer_mapping(0, 0).unwrap();
    device.add_qubit_tweezer_mapping(1, 1).unwrap();
    device.add_qubit_tweezer_mapping(2, 2).unwrap();

    let _image = device
        .draw(None, false, &Some("graph_test.png".to_owned()))
        .unwrap();
    assert!(std::path::Path::new("graph_test.png").exists());
    std::fs::remove_file("graph_test.png").unwrap();
}
