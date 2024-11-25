/*
    HELPER METHODS just for restbus-simulation 
*/

use crate::restbus_structs::*;

use std::{mem, slice};
use std::io::Error;
use std::os::raw::c_void;
use std::ffi::CString;

use nix::libc::{__c_anonymous_sockaddr_can_can_addr, __c_anonymous_sockaddr_can_tp, c_int, connect, if_nametoindex, sockaddr, sockaddr_can, socket, socklen_t, timeval, write, AF_CAN, CAN_BCM, CAN_EFF_FLAG, SOCK_DGRAM};

/* 
    Create a socket using libc's socket function. This is required since not Rust equivalent library or method exists for establishing a BCM CAN socket
*/
pub fn create_socket() -> Result<i32, String>  {
    let sock = unsafe {
        socket(AF_CAN, SOCK_DGRAM, CAN_BCM)
    };

    if sock < 0 {
        return Err(format!("Could not create socket due to {}", Error::last_os_error()));
    }

    Ok(sock)
}

/*
    1. Get the interface index from the interface string
    2. Setup libc sockaddr_can structure 
    3. Connect the existing socket
*/
pub fn connect_socket(sock: i32, ifname: &String) -> Result<i32, String>  {
    let ifindex = unsafe {
        if let Ok(c_ifname) = CString::new(ifname.as_str()) {
            if_nametoindex(c_ifname.as_ptr())
        } else {
            return Err(format!("Could not get ifindex from {}", ifname));
        }
    };

    let sockaddr: sockaddr_can = sockaddr_can {
        can_family: AF_CAN as u16,
        can_ifindex: ifindex as i32,
        can_addr: __c_anonymous_sockaddr_can_can_addr {
            tp: __c_anonymous_sockaddr_can_tp {
                rx_id: 0,
                tx_id: 0
            }
        }
    };

    let sockaddr_ptr = &sockaddr as *const sockaddr_can as *const sockaddr;
   
    let connect_res = unsafe {
        connect(sock, sockaddr_ptr, mem::size_of::<&sockaddr_can>() as socklen_t)
    };

    if connect_res < 0 {
        return Err(format!("Could not connect socket due to {}", Error::last_os_error()));
    }

    Ok(connect_res)
}

/*
    Writes to existing socket 
*/
pub fn write_socket(sock: i32, buf: &[u8]) -> Result<isize, String>  {
    let wres = unsafe {
        write(
            sock as c_int, 
            buf.as_ptr() as *const c_void, 
            buf.len()
        )
    };
    if wres < 0 {
        return Err(format!("Could not write to socket due to {}", Error::last_os_error()));
    }

    Ok(wres)
}

/*
    Fills u8 values from a vector into an array
*/
fn fill_data_array(data: &mut [u8], data_vector: &[u8]) {
    let mut i = 0;
    while i < data_vector.len() {
        data[i] = data_vector[i];
        i += 1;
    }
}

/*
    Creates a self-defined CanFrame structure that is either a CanFdFrame or a Can20Frame
 */
pub fn create_can_frame_structure(can_id: u32, len: u8, addressing_mode: bool, frame_tx_behavior: bool, data_vector: &[u8]) -> CanFrame {
    let eflag: u32 = if addressing_mode { CAN_EFF_FLAG } else { 0 };

    if frame_tx_behavior { 
        let mut data: [u8; 64] = [0; 64];

        fill_data_array(&mut data, data_vector);

        CanFrame::CanFdFrame( CanFdFrame {
            can_id: can_id | eflag,
            len,
            flags: 0, // are there any relevant flags?
            __res0: 0,
            __res1: 0,
            data,
        })
    } else {
        let mut data: [u8; 8] = [0; 8];

        fill_data_array(&mut data, data_vector);

        CanFrame::Can20Frame( Can20Frame {
            can_id: can_id | eflag,
            len,
            __pad: 0,
            __res0: 0,
            __res1: 0,
            data,
        })
    }
}

/*
    Creates a self-defined TimedCanFrame structure that holds the necessary data used for creating a CanFrame later
*/
pub fn create_time_can_frame_structure(count: u32, ivals: &[timeval], can_id: u32, len: u8, addressing_mode: bool, frame_tx_behavior: bool, data_vector: &Vec<u8>) -> TimedCanFrame {
    let mut copy_data_vector: Vec<u8> = Vec::new();

    for data in data_vector {
        copy_data_vector.push(*data);
    }

    TimedCanFrame {
        can_id,
        len,
        addressing_mode,
        frame_tx_behavior,
        data_vector: copy_data_vector,
        count,
        ival1: ivals[0],
        ival2: ivals[1]
    }
}

/*
    Creates a BcmMsgHead structure, which is a header of messages send to/from the CAN Broadcast Manager 
    See also https://github.com/linux-can/can-utils/blob/master/include/linux/can/bcm.h
*/
pub fn create_bcm_head(count: u32, ival1: timeval, ival2: timeval, can_id: u32, frame_tx_behavior: bool, frames: &[CanFrame]) -> BcmMsgHead {
    let mut canfd_flag: u32 = 0x0;
    
    if frame_tx_behavior {
        canfd_flag = BCMFlags::CanFdFrame as u32;
    }
    BcmMsgHead {
        opcode: Opcode::TxSetup as u32,
        flags: BCMFlags::SetTimer as u32 | BCMFlags::StartTimer as u32 | canfd_flag,
        count,
        ival1,
        ival2,
        can_id,
        nframes: frames.len() as u32,
    }
}

/*
    Converts a BcmMsgHead structure and the payload (which are CanFrames) to a byte representation. 
    The write_bytes vector is filled with the bytes and can be then later be used by the caller.
*/
pub fn create_bcm_structure_bytes(count: u32, ival1: timeval, ival2: timeval, can_id: u32, frame_tx_behavior: bool, frames: &Vec<CanFrame>, write_bytes: &mut Vec<u8>) {
    let head: BcmMsgHead = create_bcm_head(count, ival1, ival2, can_id, frame_tx_behavior, frames);

    let ptr: *const u8 = &head as *const BcmMsgHead as *const u8;
    let bytes: &[u8] = unsafe { slice::from_raw_parts(ptr, mem::size_of::<BcmMsgHead>()) };

    write_bytes.extend_from_slice(bytes);
    
    for frame in frames {
        match frame {
            CanFrame::Can20Frame(can20_frame) => {
                let ptr: *const u8 = can20_frame as *const Can20Frame as *const u8;
                let bytes: &[u8] = unsafe { slice::from_raw_parts(ptr, mem::size_of::<Can20Frame>()) };
                write_bytes.extend_from_slice(bytes);
            }
            CanFrame::CanFdFrame(canfd_frame) => {
                let ptr: *const u8 = canfd_frame as *const CanFdFrame as *const u8;
                let bytes: &[u8] = unsafe { slice::from_raw_parts(ptr, mem::size_of::<CanFdFrame>()) };
                write_bytes.extend_from_slice(bytes);
            }
        }
    }
}