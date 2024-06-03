/*
    Arxml parser that is able to extract all values necessary for a restbus simulation.
    Uses autosar-data library to parse data like in this example:
    https://github.com/DanielT/autosar-data/blob/main/autosar-data/examples/businfo/main.rs
    Ideas for improvement:
        - Provide options to store parsed data for quicker restart
*/

use crate::arxml_structs::*;
use crate::arxml_utils::*;

use std::time::Instant;
use std::collections::HashMap;

use core::panic;

use autosar_data::{AutosarModel, CharacterData, Element, ElementName, EnumItem};


pub struct ArxmlParser {
}

impl ArxmlParser {
    /*
        1. Parses an Autosar ISignalToIPduMapping. 
        2. Extracts Autosar ISignal and ISignalGroup elements.
        2. Fills the important extracted data into the signals HashMap and signal_groups vectors. 
    */
    fn handle_isignal_to_pdu_mappings(&self, mapping: &Element, 
        signals: &mut HashMap<String, (String, String, u64, u64, InitValues)>, 
        signal_groups: &mut Vec<Element>) 
        {
        if let Some(signal) = mapping
            .get_sub_element(ElementName::ISignalRef)
            .and_then(|elem| elem.get_reference_target().ok())
        {
            let refpath = get_required_string(&mapping, 
                ElementName::ISignalRef);

            let name = get_required_item_name(&signal, "ISignalRef");

            let byte_order = get_required_string(&mapping, ElementName::PackingByteOrder);

            let start_pos = get_required_int_value(&mapping, 
                ElementName::StartPosition);
            
            let length = get_required_int_value(&signal, 
                ElementName::Length);

            let mut init_values: InitValues = InitValues::NotExist(true);

            if let Some(mut init_value_elem) = signal.get_sub_element(ElementName::InitValue) {
                process_init_value(&mut init_value_elem, &mut init_values, &name);
            }                     
            signals.insert(refpath, (name, byte_order, start_pos, length, init_values));
        } else if let Some(signal_group) = mapping
            .get_sub_element(ElementName::ISignalGroupRef)
            .and_then(|elem| elem.get_reference_target().ok())
        {
            // store the signal group for now
            signal_groups.push(signal_group);
        }
    }

    /*
        1. Parses and processes all the ISignals defined in the parent ISignalIPdu.
        2. Fills the important extracted data into the grouped_signals and ungrouped_signals vectors of structures. 
    */
    fn handle_isignals(&self, pdu: &Element, grouped_signals: &mut Vec<ISignalGroup>, ungrouped_signals: &mut Vec<ISignal>) -> Option<()> {
        //let mut signals: HashMap<String, (String, Option<i64>, Option<i64>)> = HashMap::new();
        let mut signals: HashMap<String, (String, String, u64, u64, InitValues)> = HashMap::new();
        let mut signal_groups = Vec::new();


        if let Some(isignal_to_pdu_mappings) = pdu.get_sub_element(ElementName::ISignalToPduMappings) {
            // collect information about the signals and signal groups
            for mapping in isignal_to_pdu_mappings.sub_elements() {
                self.handle_isignal_to_pdu_mappings(&mapping, &mut signals, &mut signal_groups);
            }
        }

        for signal_group in &signal_groups {
            process_signal_group(signal_group, &mut signals, grouped_signals);
        }

        let remaining_signals: Vec<(String, String, u64, u64, InitValues)> = signals.values().cloned().collect();
        if remaining_signals.len() > 0 {
            for (name, byte_order, start_pos, length, init_values) in remaining_signals {
                let isignal_struct: ISignal = ISignal {
                    name: name,
                    byte_order: get_byte_order(&byte_order),
                    start_pos: start_pos,
                    length: length,
                    init_values: init_values
                };
                ungrouped_signals.push(isignal_struct);
            }
        }
            
        ungrouped_signals.sort_by(|a, b| a.start_pos.cmp(&b.start_pos));
        
        Some(())
    }

    /*
        1. Parses an Autosar ISignalIPdu element.
        2. Returns important data in a self-defined ISignalIPDU structure.
    */
    fn handle_isignal_ipdu(&self, pdu: &Element) -> Option<ISignalIPDU> {
        // Find out these values: ...
        let mut cyclic_timing_period_value: f64 = 0_f64;
        let mut cyclic_timing_period_tolerance: Option<TimeRangeTolerance> = None; 

        let mut cyclic_timing_offset_value: f64 = 0_f64;
        let mut cyclic_timing_offset_tolerance: Option<TimeRangeTolerance> = None;
                
        let mut number_of_repetitions: u64 = 0;
        let mut repetition_period_value: f64 = 0_f64;
        let mut repetition_period_tolerance: Option<TimeRangeTolerance> = None;

        if let Some(tx_mode_true_timing) = pdu
            .get_sub_element(ElementName::IPduTimingSpecifications)
            .and_then(|elem| elem.get_sub_element(ElementName::IPduTiming))
            .and_then(|elem| elem.get_sub_element(ElementName::TransmissionModeDeclaration))
            .and_then(|elem| elem.get_sub_element(ElementName::TransmissionModeTrueTiming)) 
        {
            if let Some(cyclic_timing) = tx_mode_true_timing
                    .get_sub_element(ElementName::CyclicTiming) 
            {
                get_sub_element_and_time_range(&cyclic_timing, ElementName::TimePeriod, &mut cyclic_timing_period_value, &mut cyclic_timing_period_tolerance);

                get_sub_element_and_time_range(&cyclic_timing, ElementName::TimeOffset, &mut cyclic_timing_offset_value, &mut cyclic_timing_offset_tolerance);
            }
            if let Some(event_timing) = tx_mode_true_timing
                .get_sub_element(ElementName::EventControlledTiming) 
            {
                number_of_repetitions = get_optional_int_value(&event_timing, 
                    ElementName::NumberOfRepetitions);
                
                get_sub_element_and_time_range(&event_timing, ElementName::RepetitionPeriod, &mut repetition_period_value, &mut repetition_period_tolerance);
            }
        }

        let unused_bit_pattern = get_unused_bit_pattern(&pdu);

        let mut grouped_signals: Vec<ISignalGroup> = Vec::new();
        
        let mut ungrouped_signals: Vec<ISignal> = Vec::new();

        self.handle_isignals(pdu, &mut grouped_signals, &mut ungrouped_signals);

        let isginal_ipdu: ISignalIPDU = ISignalIPDU {
            cyclic_timing_period_value: cyclic_timing_period_value,
            cyclic_timing_period_tolerance: cyclic_timing_period_tolerance,
            cyclic_timing_offset_value: cyclic_timing_offset_value,
            cyclic_timing_offset_tolerance: cyclic_timing_offset_tolerance,
            number_of_repetitions: number_of_repetitions,
            repetition_period_value: repetition_period_value,
            repetition_period_tolerance: repetition_period_tolerance,
            unused_bit_pattern: unused_bit_pattern,
            ungrouped_signals: ungrouped_signals, 
            grouped_signals: grouped_signals 
        };

        return Some(isginal_ipdu);
    }
    
    /*
        1. Parses an Autosar NmPdu element
        2. Returns important data in a self-defined NMPDU structure.
    */
    fn handle_nm_pdu(&self, pdu: &Element) -> Option<NMPDU> {
        let unused_bit_pattern = get_unused_bit_pattern(&pdu);

        let mut grouped_signals: Vec<ISignalGroup> = Vec::new();
        
        let mut ungrouped_signals: Vec<ISignal> = Vec::new();

        self.handle_isignals(pdu, &mut grouped_signals, &mut ungrouped_signals);
        
        let nm_pdu: NMPDU = NMPDU {
            unused_bit_pattern: unused_bit_pattern,
            ungrouped_signals: ungrouped_signals, 
            grouped_signals: grouped_signals 
        };

        return Some(nm_pdu);
    }

    /*
        1. Resolves the reference inside a PduToFrameMapping to get the PDU element.
        2. Parses the Autosar PDU element
        3. Returns important data in a self-defined PDU mapping structure.
    */
    fn handle_pdu_mapping(&self, pdu_mapping: &Element) -> Result<PDUMapping, String> {
        let pdu = get_required_reference(
            pdu_mapping,
            ElementName::PduRef);
        
        let pdu_name = get_required_item_name(
            &pdu, "Pdu");

        //let byte_order = get_required_string(pdu_mapping, 
        let byte_order = get_optional_string(pdu_mapping, 
            ElementName::PackingByteOrder);

        let pdu_length = get_required_int_value(&pdu, 
            ElementName::Length);
        
        let pdu_dynamic_length = get_optional_string(&pdu, 
            ElementName::HasDynamicLength);
        
        let pdu_category = get_optional_string(&pdu, 
            ElementName::Category);
        
        let pdu_contained_header_id_short = get_subelement_optional_string(&pdu, 
            ElementName::ContainedIPduProps, ElementName::HeaderIdShortHeader);
        
        let pdu_contained_header_id_long = get_subelement_optional_string(&pdu, 
            ElementName::ContainedIPduProps, ElementName::HeaderIdLongHeader);

        //let mut pdu_specific: PDU = PDU::Temp(0);
        let pdu_specific: PDU;

        match pdu.element_name() {
            ElementName::ISignalIPdu => {
                if let Some(value) = self.handle_isignal_ipdu(&pdu) {
                    pdu_specific = PDU::ISignalIPDU(value);
                } else {
                    panic!("Error in handle_isignal_ipdu");
                }
            }
            ElementName::NmPdu => {
                if let Some(value) = self.handle_nm_pdu(&pdu) {
                    pdu_specific = PDU::NMPDU(value);
                } else {
                    panic!("Error in handle_nm_pdu");
                }
            }
            /*ElementName::ContainerIPdu => { // Add support in future if needed
                panic!("endounter containerpdu");
                //self.handle_container_ipdu(&pdu);
            }*/
            /*ElementName::SecuredIPdu => { // Add support in future if needed
                self.handle_secured_ipdu(&pdu);
            }*/
            // Handle more?
            _ => {
                let error = format!("PDU type {} not supported. Will skip it.", pdu.element_name().to_string());
                return Err(error)
            }
        }

        let pdu_mapping: PDUMapping = PDUMapping {
            name: pdu_name,
            byte_order: get_byte_order(&byte_order),
        //    start_position: start_position,
            length: pdu_length,
            dynamic_length: pdu_dynamic_length,
            category: pdu_category,
            contained_header_id_short: pdu_contained_header_id_short,
            contained_header_id_long: pdu_contained_header_id_long,
            pdu: pdu_specific 
        };

        return Ok(pdu_mapping);     
    }
    
    /*
        1. Parses an Autosar CanFrameTriggering element.
        2. Returns important data in a self-defined CanFrameTriggering structure.
    */
    fn handle_can_frame_triggering(&self, can_frame_triggering: &Element, has_fd_baudrate: bool) -> Result<CanFrameTriggering, String> {
        let can_frame_triggering_name= get_required_item_name(
            can_frame_triggering, "CanFrameTriggering");

        let can_id = get_required_int_value(
            &can_frame_triggering,
            ElementName::Identifier);

        let frame = get_required_reference(
            can_frame_triggering,
            ElementName::FrameRef);

        let frame_name = get_required_item_name(
            &frame, "Frame");

        let addressing_mode_str = if let Some(CharacterData::Enum(value)) = can_frame_triggering
            .get_sub_element(ElementName::CanAddressingMode)
            .and_then(|elem| elem.character_data()) 
        {
            value.to_string()
        } else {
            EnumItem::Standard.to_string()
        };

        let mut addressing_mode: bool = false;
        if addressing_mode_str.to_uppercase() == String::from("EXTENDED") {
            addressing_mode = true;
        }

        // allow it to be missing. When missing, then derive value from CanCluster
        let mut frame_rx_behavior = false; 
        let frame_rx_behavior_str = get_optional_string(
            can_frame_triggering,
            ElementName::CanFrameRxBehavior);
        if frame_rx_behavior_str.to_uppercase() == String::from("CAN-FD") {
            frame_rx_behavior = true;
        } else if frame_rx_behavior_str == "" && has_fd_baudrate {
            frame_rx_behavior = true;
        }
        
        // allow it to be missing. When missing, then derive value from CanCluster
        let mut frame_tx_behavior = false; 
        let frame_tx_behavior_str = get_optional_string(
            can_frame_triggering,
            ElementName::CanFrameTxBehavior);
        if frame_tx_behavior_str.to_uppercase() == String::from("CAN-FD") {
            frame_tx_behavior = true;
        } else if frame_tx_behavior_str == "" && has_fd_baudrate {
            frame_tx_behavior = true;
        }

        let mut rx_range_lower: u64 = 0;
        let mut rx_range_upper: u64 = 0;
        if let Some(range_elem) = can_frame_triggering.get_sub_element(ElementName::RxIdentifierRange) {
            rx_range_lower = get_required_int_value(&range_elem, ElementName::LowerCanId);
            rx_range_upper = get_required_int_value(&range_elem, ElementName::UpperCanId);
        }

        let mut rx_ecus: Vec<String> = Vec::new();
        let mut tx_ecus: Vec<String> = Vec::new();

        match process_frame_ports(can_frame_triggering, &can_frame_triggering_name, &mut rx_ecus, &mut tx_ecus) {
            Err(err) => return Err(err),
            _ => {}
        }

        let frame_length = get_optional_int_value(
            &frame,
            ElementName::FrameLength);

        let mut pdu_mappings_vec: Vec<PDUMapping> = Vec::new();

        // assign here and other similar variable?
        if let Some(mappings) = frame.get_sub_element(ElementName::PduToFrameMappings) {
            for pdu_mapping in mappings.sub_elements() {
                match self.handle_pdu_mapping(&pdu_mapping) {
                    Ok(value) => pdu_mappings_vec.push(value),
                    Err(error) => return Err(error) 
                }
            }
        }

        let can_frame_triggering_struct: CanFrameTriggering = CanFrameTriggering {
            frame_triggering_name: can_frame_triggering_name,
            frame_name: frame_name,
            can_id: can_id,
            addressing_mode: addressing_mode,
            frame_rx_behavior: frame_rx_behavior,
            frame_tx_behavior: frame_tx_behavior,
            rx_range_lower: rx_range_lower,
            rx_range_upper: rx_range_upper,
            receiver_ecus: rx_ecus,
            sender_ecus: tx_ecus,
            frame_length: frame_length,
            pdu_mappings: pdu_mappings_vec 
        };

        return Ok(can_frame_triggering_struct);
    }

    /*
        1. Parses an Autosar CanCluster element
        2. Returns important data in a self-defined CanCluster structure.
    */
    fn handle_can_cluster(&self, can_cluster: &Element) -> Result<CanCluster, String> {
        let can_cluster_name = get_required_item_name(
            can_cluster, "CanCluster");

        let can_cluster_conditional = get_required_sub_subelement(
            can_cluster, 
            ElementName::CanClusterVariants,
            ElementName::CanClusterConditional);

        //let can_cluster_baudrate =  self.get_required_subelement_int_value(
        let can_cluster_baudrate = get_optional_int_value(
            &can_cluster_conditional,
            ElementName::Baudrate);
        
        let can_cluster_fd_baudrate = get_optional_int_value(
            &can_cluster_conditional,
            ElementName::CanFdBaudrate);

        let has_fd_baudrate = can_cluster_baudrate > 0;

        if can_cluster_baudrate == 0 && can_cluster_fd_baudrate == 0 {
            let msg = format!("Baudrate and FD Baudrate of CanCluster {} do not exist or are 0. Skipping this CanCluster.", can_cluster_name);
            return Err(msg.to_string());
        }

        // iterate over PhysicalChannels and handle the CanFrameTriggerings inside them
        let physical_channels;
        if let Some(value) = can_cluster_conditional
            .get_sub_element(ElementName::PhysicalChannels).map(|elem| {
                elem.sub_elements().filter(|se| se.element_name() == ElementName::CanPhysicalChannel)
            }) 
        {
            physical_channels = value;
        } else {
            let msg = format!("Cannot handle physical channels of CanCluster {}", can_cluster_name);
            return Err(msg.to_string());
        }

        let mut can_frame_triggerings: HashMap<u64, CanFrameTriggering> = HashMap::new(); 
        for physical_channel in physical_channels {
            if let Some(frame_triggerings) = physical_channel.get_sub_element(ElementName::FrameTriggerings) {
                for can_frame_triggering in frame_triggerings.sub_elements() {
                    match self.handle_can_frame_triggering(&can_frame_triggering, has_fd_baudrate) {
                        Ok(value) => {
                            can_frame_triggerings.insert(value.can_id.clone(), value);
                        }
                        Err(error) => println!("[-] WARNING: {}", error),
                    }
                }
            }
        }

        let can_cluster_struct: CanCluster = CanCluster {
            name: can_cluster_name,
            baudrate: can_cluster_baudrate,
            canfd_baudrate: can_cluster_fd_baudrate,
            can_frame_triggerings: can_frame_triggerings
        };
        
        return Ok(can_cluster_struct);
    }

    /*
        Main parsing method. Uses autosar-data libray for parsing ARXML.
        In the future, it might be extended to support Ethernet, Flexray, ... 
        The resources to develop that should not be thaat high, since it is basically just extending the current parser.
        Param file_name: ARXML target file name without ".ser" extension
        Param safe_or_load_serialized: First look if serialized parsed data already exists by looking for file_name + ".ser". 
            If not exists, then parse and safe parsed structures as serialized data in file_name + ".ser"
        Returns a vector of CanCluster structures.
    */
    pub fn parse_file(&self, file_name: &String, safe_or_load_serialized: bool) -> Option<HashMap<String, CanCluster>> {
        if safe_or_load_serialized {
            println!("[+] Loading data from serialized file");
            match load_serialized_data(file_name) {
                Ok(value) => {
                    println!("[+] Successfully loaded serialized data.");
                    return Some(value)
                }
                _ => println!("[-] Could not load serialized data. Will continue parsing.")
            }
        }

        let start = Instant::now();

        let model = AutosarModel::new();

        if let Err(err) = model.load_file(file_name, false) {
            panic!("Parsing failed. Error: {}", err.to_string());
        }

        // DEBUG 
        println!("[+] Duration of loading was: {:?}", start.elapsed());
        // DEBUG END

        let mut can_clusters: HashMap<String, CanCluster> = HashMap::new();

        // Iterate over Autosar elements and handle CanCluster elements
        for element in model
            .identifiable_elements()
            .iter()
            .filter_map(|path| model.get_element_by_path(&path))
        {
            match element.element_name() {
                ElementName::CanCluster => {
                    let result: Result<CanCluster, String> = self.handle_can_cluster(&element);
                    match result {
                        Ok(value) => {
                            can_clusters.insert(value.name.clone(), value);
                        }
                        Err(error) => println!("[-] WARNING: {}", error)
                    }
                }
                _ => {}
            }
        }

        println!("[+] Duration of parsing: {:?}", start.elapsed());

        if safe_or_load_serialized {
            println!("[+] Storing serialized data to file");
            match store_serialized_data(file_name, &can_clusters) {
                Ok(()) => println!("[+] Successfully stored serialized data."),
                _ => println!("[-] Could not store serialized data.")
            }
        }

        return Some(can_clusters);
    }
}
