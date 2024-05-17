/*
    HELPER METHODS 
*/
use autosar_data::{CharacterData, Element, ElementName, EnumItem};

use std::collections::HashMap;

use crate::arxml_structs::*;

pub fn decode_integer(cdata: &CharacterData) -> Option<i64> {
    if let CharacterData::String(text) = cdata {
        if text == "0" {
            Some(0)
        } else if text.starts_with("0x") {
            let hexstr = text.strip_prefix("0x").unwrap();
            Some(i64::from_str_radix(hexstr, 16).ok()?)
        } else if text.starts_with("0X") {
            let hexstr = text.strip_prefix("0X").unwrap();
            Some(i64::from_str_radix(hexstr, 16).ok()?)
        } else if text.starts_with("0b") {
            let binstr = text.strip_prefix("0b").unwrap();
            Some(i64::from_str_radix(binstr, 2).ok()?)
        } else if text.starts_with("0B") {
            let binstr = text.strip_prefix("0B").unwrap();
            Some(i64::from_str_radix(binstr, 2).ok()?)
        } else if text.starts_with('0') {
            let octstr = text.strip_prefix('0').unwrap();
            Some(i64::from_str_radix(octstr, 8).ok()?)
        } else {
            Some(text.parse().ok()?)
        }
    } else {
        None
    }
}

pub fn get_time_range(base: &Element) -> Option<TimeRange> {
    let value = base
        .get_sub_element(ElementName::Value)
        .and_then(|elem| elem.character_data())
        .and_then(|cdata| cdata.double_value())?;

    let tolerance = if let Some(absolute_tolerance) = base
        .get_sub_element(ElementName::AbsoluteTolerance)
        .and_then(|elem| elem.get_sub_element(ElementName::Absolute))
        .and_then(|elem| elem.character_data())
        .and_then(|cdata| cdata.double_value())
    {
        Some(TimeRangeTolerance::Absolute(absolute_tolerance))
    } else {
        base.get_sub_element(ElementName::RelativeTolerance)
            .and_then(|elem| elem.get_sub_element(ElementName::Relative))
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| decode_integer(&cdata))
            .map(TimeRangeTolerance::Relative)
    };

    Some(TimeRange { tolerance, value })
}

pub fn get_sub_element_and_time_range(base: &Element, sub_elem_name: ElementName, value: &mut f64, tolerance: &mut Option<TimeRangeTolerance>) {
    if let Some(time_range) = base 
        .get_sub_element(sub_elem_name)
        .and_then(|elem| get_time_range(&elem)) 
    {
        *value = time_range.value;
        *tolerance = time_range.tolerance;
    }
}

pub fn get_required_item_name(element: &Element, element_name: &str) -> String {
    if let Some(item_name) = element.item_name() {
        return item_name; 
    } else {
        panic!("Error getting required item name of {}", element_name);
    } 
}

pub fn get_required_sub_subelement(element: &Element, subelement_name: ElementName, sub_subelement_name: ElementName) -> Element {
    if let Some(sub_subelement) = element 
        .get_sub_element(subelement_name)
        .and_then(|elem| elem.get_sub_element(sub_subelement_name)) 
    {
        return sub_subelement;
    } else {
        panic!("Error getting sub_subelement. Tried to retrieve {} and then {}",
            subelement_name,
            sub_subelement_name);
    } 
}

pub fn get_subelement_int_value(element: &Element, subelement_name: ElementName) -> Option<i64> {
    return element 
        .get_sub_element(subelement_name)
        .and_then(|elem| elem.character_data())
        .and_then(|cdata| decode_integer(&cdata));
} 

pub fn get_required_int_value(element: &Element, subelement_name: ElementName) -> i64 {
    if let Some(int_value) = get_subelement_int_value(element, subelement_name) {
        return int_value;
    } else {
        panic!("Error getting required integer value of {}", subelement_name);
    }
}

pub fn get_optional_int_value(element: &Element, subelement_name: ElementName) -> i64 {
    if let Some(int_value) = get_subelement_int_value(element, subelement_name) {
        return int_value;
    } else {
        return 0;
    }
}

pub fn get_required_reference(element: &Element, subelement_name: ElementName) -> Element {
    if let Some(subelement) = element.get_sub_element(subelement_name) {
        match subelement.get_reference_target() {
            Ok(reference) => return reference,
            Err(_) => {} 
        }
    }
    
    panic!("Error getting required reference for {}", subelement_name);
}

pub fn get_subelement_string_value(element: &Element, subelement_name: ElementName) -> Option<String> {
    return element 
        .get_sub_element(subelement_name)
        .and_then(|elem| elem.character_data())
        .map(|cdata| cdata.to_string());
}

pub fn get_required_string(element: &Element, subelement_name: ElementName) -> String {
    if let Some(value) = get_subelement_string_value(element, subelement_name) {
        return value;
    } else {
        panic!("Error getting required String value of {}", subelement_name);
    }
}

pub fn get_optional_string(element: &Element, subelement_name: ElementName) -> String {
    if let Some(value) = get_subelement_string_value(element, subelement_name) {
        return value;
    } else {
        return String::from("");
    }
}

pub fn get_subelement_optional_string(element: &Element, subelement_name: ElementName, sub_subelement_name: ElementName) -> String {
    if let Some(value) = element.get_sub_element(subelement_name)
        .and_then(|elem| elem.get_sub_element(sub_subelement_name))
        .and_then(|elem| elem.character_data())
        .map(|cdata| cdata.to_string()) 
    {
        return value;     
    } else {
        return String::from("");
    }
}

pub fn ecu_of_frame_port(frame_port: &Element) -> Option<String> {
    let ecu_comm_port_instance = frame_port.parent().ok()??;
    let comm_connector = ecu_comm_port_instance.parent().ok()??;
    let connectors = comm_connector.parent().ok()??;
    let ecu_instance = connectors.parent().ok()??;
    ecu_instance.item_name()
}

// 1: Big Endian, 0: Little Endian
pub fn get_byte_order(byte_order: &String) -> bool {
    if byte_order.eq("MOST-SIGNIFICANT-BYTE-LAST") {
        return false;
    }
    return true;
}

// See how endianess affects PDU in 6.2.2 https://www.autosar.org/fileadmin/standards/R22-11/CP/AUTOSAR_TPS_SystemTemplate.pdf
// Currenlty assumes Little Endian byte ordering and has support for signals that are Little Endian or Big Endian
// Bit positions in undefined ranges are set to 1
pub fn extract_init_values(unused_bit_pattern: bool, ungrouped_signals: &Vec<ISignal>, grouped_signals: &Vec<ISignalGroup>, length: i64, byte_order: &bool) -> Vec<u8> {
    // pre checks
    if grouped_signals.len() > 0 && ungrouped_signals.len() > 0 {
        panic!("both signal vectors are > 0");
    }

    let isignals: &Vec<ISignal>;

    if grouped_signals.len() > 0 {
        if grouped_signals.len() > 1 {
            panic!("Grouped signals > 0");
        }
        isignals = &grouped_signals[0].isignals;
    } else {
        isignals = ungrouped_signals;
    }

    let dlc: usize = length.try_into().unwrap();

    let mut bits = vec![unused_bit_pattern; dlc * 8]; // Using unusued_bit_pattern for undefined bits 

    for isignal in isignals {
        let mut tmp_bit_array: Vec<bool> = Vec::new();
        let init_values = &isignal.init_values;
        let isignal_byte_order = isignal.byte_order;
        let isignal_length: usize = isignal.length.try_into().unwrap();
        let isignal_start: usize = isignal.start_pos.try_into().unwrap();

        match init_values {
            InitValues::Single(value) => {
                let mut n = value.clone();

                while n != 0 {
                    tmp_bit_array.push(n & 1 != 0);
                    n >>= 1;
                }

                while tmp_bit_array.len() < isignal_length {
                    tmp_bit_array.push(false);
                }
        
                if isignal_byte_order {
                    tmp_bit_array.reverse();
                }
            }
            InitValues::Array(values) => {
                if isignal_length % 8 != 0 {
                    panic!("ISignal length for array is not divisable through 8. Length is {}", isignal_length);
                }

                for isignal_value in values {
                    let byte_len: usize = 8;
                    let mut n = isignal_value.clone();
                    let mut tmp_tmp_bit_array: Vec<bool> = Vec::new();

                    while n != 0 {
                        tmp_tmp_bit_array.push(n & 1 != 0);
                        n >>= 1;
                    }

                    while tmp_tmp_bit_array.len() < byte_len {
                        tmp_tmp_bit_array.push(false);
                    }
                        
                    tmp_tmp_bit_array.reverse();

                    tmp_bit_array.extend(tmp_tmp_bit_array);
                }
            }
            _ => continue
        }

        if tmp_bit_array.len() != isignal.length.try_into().unwrap() {
            panic!("Miscalculation for tmp_bit_array");
        }

        let mut index: usize = 0;

        while index < isignal_length {
            bits[isignal_start + index] = tmp_bit_array[index];
            index += 1;
        } 
    }

    let mut init_values: Vec<u8> = Vec::new();
    let mut current_byte: u8 = 0;
    let mut bit_count = 0;
        
    for bit in bits {
        current_byte <<= 1;
        if bit {
            current_byte |= 1;
        }
        bit_count += 1;
   
        if bit_count == 8 {
            init_values.push(current_byte);
            current_byte = 0;
            bit_count = 0;
        }
    }
    if bit_count > 0 {
        current_byte <<= 8 - bit_count;
        init_values.push(current_byte);
    }

    if !byte_order {
        for init_value in init_values.iter_mut() {
            *init_value = init_value.reverse_bits(); // reverse bits of each byte
        }
    }

    if init_values.len() != dlc {
        panic!("Error creating byte array");
    }

    /*if !byte_order { 
        init_values.reverse();
    }*/

    return init_values;
}

pub fn get_unused_bit_pattern(pdu: &Element) -> bool {
    let unused_bit_pattern_int = get_required_int_value(&pdu, ElementName::UnusedBitPattern);
    let unused_bit_pattern: bool;

    if unused_bit_pattern_int == 0 {
        unused_bit_pattern = false;
    } else if unused_bit_pattern_int == 1 {
        unused_bit_pattern = true;
    } else {
        panic!("Error reading unused_bit_pattern. Value is {}", unused_bit_pattern_int);
    }

    return unused_bit_pattern;
}

pub fn process_frame_ports(can_frame_triggering: &Element, can_frame_triggering_name: &String, rx_ecus: &mut Vec<String>, tx_ecus: &mut Vec<String>) -> Result<(), String> {
    if let Some(frame_ports) = can_frame_triggering.get_sub_element(ElementName::FramePortRefs) {
        let frame_ports: Vec<Element> = frame_ports.sub_elements()
            .filter(|se| se.element_name() == ElementName::FramePortRef)
            .filter_map(|fpr| fpr.get_reference_target().ok())
            .collect();

        for frame_port in frame_ports {
            if let Some(ecu_name) = ecu_of_frame_port(&frame_port) {
                if let Some(CharacterData::Enum(direction)) = frame_port
                    .get_sub_element(ElementName::CommunicationDirection)
                    .and_then(|elem| elem.character_data())
                {
                    match direction {
                        EnumItem::In => rx_ecus.push(ecu_name), 
                        EnumItem::Out => tx_ecus.push(ecu_name), 
                        _ => return Err(format!("Invalid direction ID encountered in FramePort. Skipping CanFrameTriggering {}", can_frame_triggering_name))
                    }
                } else {
                    return Err(format!("No CommunicationDirection encountered in FramePort. Skipping CanFrameTriggering {}", can_frame_triggering_name)) 
                }
            } else {
                return Err(format!("Could not extract ECUName in FramePort. Skipping CanFrameTriggering {}", can_frame_triggering_name)) ;
            }
        }
    } else {
        return Err(format!("FramePortRefs in CanFrameTriggering not found. Skipping CanFrameTriggering {}", can_frame_triggering_name));
    }

    Ok(())
}

pub fn process_init_value(init_value_elem: &Element, init_values: &mut InitValues, signal_name: &String) {
    let init_value_single: bool;

    let subelement_name = init_value_elem.get_sub_element_at(0).unwrap();
    
    if subelement_name.element_name().eq(&ElementName::NumericalValueSpecification) {
        init_value_single = true; 
    } else if subelement_name.element_name().eq(&ElementName::ArrayValueSpecification) {
        init_value_single = false; 
    } else {
        panic!("Unrecognized sublement {} for init-value", subelement_name.element_name());
    }

    if init_value_single {
        if let Some(num_val) = init_value_elem.get_sub_element(ElementName::NumericalValueSpecification) {
            let init_value = get_required_int_value(&num_val, ElementName::Value);
            *init_values = InitValues::Single(init_value);
        } else {
            panic!("InitValue element does not have NumercialValueSpecification for signal {}", signal_name);
        }

    } else {
        let mut init_value_array: Vec<i64> = Vec::new();
        let num_val_elements = get_required_sub_subelement(&init_value_elem, 
            ElementName::ArrayValueSpecification, 
            ElementName::Elements);

        for num_val_elem in num_val_elements.sub_elements() {
            init_value_array.push(get_required_int_value(&num_val_elem, ElementName::Value));
        }
        
        *init_values = InitValues::Array(init_value_array);
    }
}

pub fn process_signal_group(signal_group: &Element, 
    signals: &mut HashMap<String, (String, String, i64, i64, InitValues)>, 
    grouped_signals: &mut Vec<ISignalGroup>) -> Option<()> 
    {
    let group_name = get_required_item_name(&signal_group, "ISignalGroupRef"); 
    
    let mut signal_group_signals: Vec<ISignal> = Vec::new();

    let isignal_refs = signal_group.get_sub_element(ElementName::ISignalRefs)?;

    // Removing ok and needed?
    for isignal_ref in isignal_refs.sub_elements()
        .filter(|elem| elem.element_name() == ElementName::ISignalRef) {
        if let Some(CharacterData::String(path)) = isignal_ref.character_data() {
            if let Some(siginfo) = signals.get(&path) {
                let siginfo_tmp = siginfo.clone();
                let isginal_tmp: ISignal = ISignal {
                    name: siginfo_tmp.0,
                    byte_order: get_byte_order(&siginfo_tmp.1),
                    start_pos: siginfo_tmp.2,
                    length: siginfo_tmp.3,
                    init_values: siginfo_tmp.4
                };

                signal_group_signals.push(isginal_tmp);
                signals.remove(&path);
            }
        }
    }

    signal_group_signals.sort_by(|a, b| a.start_pos.cmp(&b.start_pos));

    let mut data_transformations: Vec<String> = Vec::new();

    if let Some(com_transformations) = signal_group
        .get_sub_element(ElementName::ComBasedSignalGroupTransformations) 
    {
        for elem in com_transformations.sub_elements() {
            let data_transformation = get_required_reference(&elem,
                ElementName::DataTransformationRef);
            
            data_transformations.push(get_required_item_name(
                    &data_transformation,
                    "DataTransformation"));
        }
    }

    let mut props_vector: Vec<E2EDataTransformationProps> = Vec::new();

    if let Some(transformation_props) = signal_group.get_sub_element(ElementName::TransformationISignalPropss) {
        for e2exf_props in transformation_props
            .sub_elements()
            .filter(|elem| elem.element_name() == ElementName::EndToEndTransformationISignalProps)
        {
            if let Some(e2exf_props_cond) = e2exf_props
                .get_sub_element(ElementName::EndToEndTransformationISignalPropsVariants)
                .and_then(|elem| elem.get_sub_element(ElementName::EndToEndTransformationISignalPropsConditional))
            {
                let transformer_reference = get_required_reference(&e2exf_props_cond, 
                    ElementName::TransformerRef);
                
                let transformer_name = get_required_item_name(&transformer_reference, 
                    "TransformerName");

                let data_ids = e2exf_props_cond
                    .get_sub_element(ElementName::DataIds)?;

                let data_id = get_required_int_value(&data_ids,
                    ElementName::DataId);
                
                let data_length = get_required_int_value(&e2exf_props_cond,
                    ElementName::DataLength);
                
                
                let props_struct: E2EDataTransformationProps = E2EDataTransformationProps {
                    transformer_name: transformer_name,
                    data_id: data_id,
                    data_length: data_length 
                };

                props_vector.push(props_struct);
            }
        }
    }

    let isignal_group_struct: ISignalGroup = ISignalGroup {
        name: group_name,
        isignals: signal_group_signals,
        data_transformations: data_transformations,
        transformation_props: props_vector 
    };

    grouped_signals.push(isignal_group_struct);

    Some(())
}