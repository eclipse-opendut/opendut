use crate::restbus_utils::*;
use crate::restbus_structs::*;

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
    pub fn play_all(&self, timed_can_frames: &Vec<TimedCanFrame>, ifname: &String) -> Result<bool, String> {
        let sock = create_socket()?;
        
        connect_socket(sock, ifname)?;

        let mut write_bytes_global: Vec<Vec<u8>> = Vec::new();

        for timed_can_frame in timed_can_frames { 
            let mut write_bytes: Vec<u8> = Vec::new();

            let mut can_frames: Vec<CanFrame> = Vec::new();

            can_frames.push(
                create_can_frame_structure(timed_can_frame.can_id, timed_can_frame.can_dlc, timed_can_frame.addressing_mode, &timed_can_frame.data_vector));
        
            create_bcm_structure_bytes(timed_can_frame.count, timed_can_frame.ival1.tv_sec, timed_can_frame.ival1.tv_usec, 
                timed_can_frame.ival2.tv_sec, timed_can_frame.ival2.tv_usec, timed_can_frame.can_id, &can_frames, &mut write_bytes);
            
            println!("write byte is {}", write_bytes.len());
            for byte in &write_bytes {
                print!("{:02x} ", byte);
            }
            println!("");

            write_bytes_global.push(write_bytes);

        }

        for write_bytes in write_bytes_global {
            write_socket(sock, &write_bytes, write_bytes.len())?;
        } 

        return Ok(true);
    }

    /*pub fn play_single_bcm_frame(&self, ifname: &String, count: u32, ival1_tv_sec: i64, ival1_tv_usec: i64, 
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
    
    }*/ 
}