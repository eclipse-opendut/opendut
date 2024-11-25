use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::thread;
use std::time::Duration;

mod arxml_parser;
mod arxml_structs;
mod arxml_utils;
mod restbus_simulation;
mod restbus_utils;
mod restbus_structs;

use arxml_parser::*;
use arxml_structs::*;
use arxml_utils::*;
use restbus_simulation::*;
use restbus_structs::TimedCanFrame;


/*
    Debug method. Combines the CAN ID and intial values as hex of a PDU. Returns this as string.
*/
fn get_pdu_hex(can_id: &u64, init_values: &Vec<u8>) -> String {
    let mut hex_string = String::new();
    hex_string.push_str(&format!("{}:", can_id));
    for element in init_values {
        hex_string.push_str(&format!("{:02X}", element));
    }
    println!("Values: {}", hex_string);

    hex_string
}


/*
    Debug method. Iterates over all PDUs of the target bus and returns all combinations of CAN IDs and initial values as string.
*/
fn collect_pdus(can_clusters: &HashMap<String, CanCluster>, bus_name: String) -> Vec<String> {
    let mut init_values_strings: Vec<String> = Vec::new();
    if let Some(can_cluster) = can_clusters.get(&bus_name) {
        for can_frame_triggering in can_cluster.can_frame_triggerings.values() {
            for pdu_mapping in &can_frame_triggering.pdu_mappings {
                match &pdu_mapping.pdu {
                    Pdu::ISignalIPdu(pdu) => {
                        let init_values = extract_init_values(pdu.unused_bit_pattern,
                                &pdu.ungrouped_signals,
                                &pdu.grouped_signals,
                                pdu_mapping.length,
                                &pdu_mapping.byte_order).unwrap();
                        init_values_strings.push(get_pdu_hex(&can_frame_triggering.can_id, &init_values))
                    }
                    Pdu::NmPdu(pdu) => {
                        let init_values = extract_init_values(pdu.unused_bit_pattern,
                                &pdu.ungrouped_signals,
                                &pdu.grouped_signals,
                                pdu_mapping.length,
                                &pdu_mapping.byte_order).unwrap();
                        init_values_strings.push(get_pdu_hex(&can_frame_triggering.can_id, &init_values))
                    }
                }
            }
        }
    }
    init_values_strings
}


/*
    Play a single CAN frame from the target bus periodcically (if it is periodic) to the bus to which ifname is connected to.
*/
fn test_find_frame_and_play(restbus_simulation: &RestbusSimulation, ifname: &String, can_clusters: &HashMap<String, CanCluster>, bus_name: String, can_id: u64) {
    let timed_can_frames: Vec<TimedCanFrame> = get_timed_can_frame_from_id(can_clusters, bus_name, can_id).unwrap();

    match restbus_simulation.play_all(&timed_can_frames, ifname) {
        Ok(_val) => println!("Successfully sent message with can id {}", can_id),
        Err(error) => println!("Could not send message with can id {} because: {}", can_id, error),
    }

}


/*
    Play all CAN frames from the target bus periodcically (only periodic frames are sent periodically) to the bus to which ifname is connected to.
*/
fn test_bus_play_all(restbus_simulation: &RestbusSimulation, ifname: &String, can_clusters: &HashMap<String, CanCluster>, bus_name: String) {
    let timed_can_frames: Vec<TimedCanFrame> = get_timed_can_frames_from_bus(can_clusters, bus_name).unwrap();

    match restbus_simulation.play_all(&timed_can_frames, ifname) {
        Ok(_val) => println!("Successfully established restbus simulation"),
        Err(error) => println!("Could not establish restbus simulation because: {}", error)
    }
}


/*
    This shows an example of how to create a restbus-simulation with an ARXML file.
*/ 
fn main() -> std::io::Result<()> {
    println!("[+] Starting openDuT ARXML parser over main method.");

    // Define file name. It is used as a path.
    let file_name = "system-4.2.arxml"; // from https://github.com/cantools/cantools/blob/master/tests/files/arxml/system-4.2.arxml

    // Create ArxmlParser struct, which contains all the parsing methods.
    let arxml_parser: ArxmlParser = ArxmlParser {};

    // Parse the ARXML file. Use serialized file if it exists. Parsed data is stored in can cluster ARXML element representations.
    if let Ok(can_clusters) = arxml_parser
        .parse_file(&file_name.to_string(), true) 
    {
        let bus_name = "Cluster0";
        let target_file = "system.txt";
        let play_single_or_all = true;
        let ifname = String::from("vcan0");

        // Debug output 
        for cluster in can_clusters.values() {
            println!("CanCluster: {cluster:?}");
        }
         
        // Debug output 
        let mut frames = String::new();
        for frame in collect_pdus(&can_clusters, String::from(bus_name)) {
            frames.push_str(frame.as_str());
            frames.push('\n');
        }

        // Debug. Found PDUs of target bus are written to file.
        let mut f = File::create(target_file)?;
        f.write_all(frames.as_bytes())?;

        println!("Trying to setup up restbus simulation");

        // Create RestbusSimulation structure, which contains all the relevant methods for the restbus simulation.
        let restbus_simulation: RestbusSimulation = RestbusSimulation {};

        if !play_single_or_all {
            // Play a single CAN frame from the target bus periodcically (if it is periodic) to the bus to which ifname is connected to.
            test_find_frame_and_play(&restbus_simulation, &ifname, &can_clusters, String::from(bus_name), 0x3E9);

        } else {
            // Play all CAN frames from the target bus periodcically (only periodic frames are sent periodically) to the bus to which ifname is connected to.
            test_bus_play_all(&restbus_simulation, &ifname, &can_clusters, String::from(bus_name));
        }

        // Sleep as long as restbus-simulation should run.
        thread::sleep(Duration::from_secs(300));
    }
    Ok(())
}

