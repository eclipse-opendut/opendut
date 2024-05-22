use crate::arxml_structs::*;
use crate::restbus_utils::*;

/*
 Restbus simulation that makes use of the structures parsed by the ARXML parser. Makes use of the Linux Kernel CAN Broadcast Manager 
*/

/* 
- TODO: 
    - create restbus simulation based on parsed data in a different source code file
    - use 100ms for NM pdus
*/


pub struct RestbusSimulation {
}

impl RestbusSimulation {
    /*pub fn play_from_can_cluster(&self, can_cluster: &CanCluster, interface: &String) {
        let mut pdus: Vec<&PDU> = Vec::new();

        for can_frame_triggering in can_cluster.can_frame_triggerings.values() {
            for pdu_mapping in &can_frame_triggering.pdu_mappings { 
                pdus.push(&pdu_mapping.pdu);
            }
        }
    }*/

    pub fn play_single_bcm_frame(&self, ifname: &String, count: u32, ival1_tv_sec: i64, ival1_tv_usec: i64, 
        ival2_tv_sec: i64, ival2_tv_usec: i64, can_id: u32, can_dlc: u8, addressing_mode: bool, data_vector: &Vec<u8>) -> Result<bool, String> 
        {

        let sock = create_socket()?;
        
        connect_socket(sock, ifname)?;

        let mut write_bytes: Vec<u8> = Vec::new();

        let mut can_frames: Vec<CanFrame> = Vec::new();

        can_frames.push(create_can_frame_structure(can_id, can_dlc, addressing_mode, data_vector));

        create_bcm_structure_bytes(count, ival1_tv_sec, ival1_tv_usec, ival2_tv_sec, ival2_tv_usec, can_id, &can_frames, &mut write_bytes);

        println!("write byte is {}", write_bytes.len());
        for byte in &write_bytes {
            print!("{:02x} ", byte);
        }
        println!("");

        write_socket(sock, &write_bytes, write_bytes.len())?;

        return Ok(true);
    
    }
}