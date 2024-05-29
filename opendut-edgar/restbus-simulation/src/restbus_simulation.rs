/*
    Restbus simulation that makes use of the structures parsed by the ARXML parser. Makes use of the Linux Kernel CAN Broadcast Manager.
    Ideas for improvement:
        - Be able to manually add stuff to restbus -> provide interface
*/

use crate::restbus_utils::*;
use crate::restbus_structs::*;


pub struct RestbusSimulation {
}

impl RestbusSimulation {
    /*
        1. Creates a BCM socket.
        2. Converts all TimedCanFrames into regular CAN frames and puts each CAN frame into a BCM struct.
        3. All created BCM structs are written to the BCM socket (sent to the Broadcast Manager).
    */
    pub fn play_all(&self, timed_can_frames: &Vec<TimedCanFrame>, ifname: &String) -> Result<bool, String> {
        let sock = create_socket()?;
        
        connect_socket(sock, ifname)?;

        let mut write_bytes_global: Vec<Vec<u8>> = Vec::new();

        for timed_can_frame in timed_can_frames { 
            let mut write_bytes: Vec<u8> = Vec::new();

            let mut can_frames: Vec<CanFrame> = Vec::new();

            can_frames.push(
                create_can_frame_structure(timed_can_frame.can_id, timed_can_frame.len, timed_can_frame.addressing_mode, timed_can_frame.frame_tx_behavior, &timed_can_frame.data_vector));
        
            create_bcm_structure_bytes(timed_can_frame.count, timed_can_frame.ival1.tv_sec as u64, timed_can_frame.ival1.tv_usec as u64, 
                timed_can_frame.ival2.tv_sec as u64, timed_can_frame.ival2.tv_usec as u64, timed_can_frame.can_id, timed_can_frame.frame_tx_behavior, &can_frames, &mut write_bytes);
            
            write_bytes_global.push(write_bytes);

        }

        for write_bytes in write_bytes_global {
            /*println!("write byte is {}", write_bytes.len());
            for byte in &write_bytes {
                print!("{:02x} ", byte);
            }
            println!("");*/

            write_socket(sock, &write_bytes, write_bytes.len())?;
            //println!("successfully wrote to socket");
        } 

        return Ok(true);
    }

    /*pub fn play_single_bcm_frame(&self, ifname: &String, count: u32, ival1_tv_sec: i64, ival1_tv_usec: i64, 
        ival2_tv_sec: i64, ival2_tv_usec: i64, can_id: u32, len: u8, addressing_mode: bool, data_vector: &Vec<u8>) -> Result<bool, String> 
        {

        let sock = create_socket()?;
        
        connect_socket(sock, ifname)?;

        let mut write_bytes: Vec<u8> = Vec::new();

        let mut can_frames: Vec<CanFrame> = Vec::new();

        can_frames.push(create_can_frame_structure(can_id, len, addressing_mode, data_vector));

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