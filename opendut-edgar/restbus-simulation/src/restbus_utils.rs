/*
    HELPER METHODS just for restbus-simulation 
*/

use crate::restbus_structs::*;

use std::{mem, slice};
use std::io::Error;
use std::os::raw::c_void;
use std::ffi::CString;

use nix::libc::{__c_anonymous_sockaddr_can_can_addr, __c_anonymous_sockaddr_can_tp, connect, if_nametoindex, sockaddr, sockaddr_can, socket, timeval, write, AF_CAN, CAN_BCM, CAN_EFF_FLAG, SOCK_DGRAM};


fn vec_to_c_void(vec: &Vec<u8>) -> *const c_void {
    vec.as_ptr() as *const c_void
}

pub fn create_socket() -> Result<i32, String>  {
    let sock = unsafe {
        socket(AF_CAN, SOCK_DGRAM, CAN_BCM)
    };

    if sock < 0 {
        return Err(format!("Could not create socket due to {}", Error::last_os_error()));
    }

    return Ok(sock);
}

pub fn connect_socket(sock: i32, ifname: &String) -> Result<i32, String>  {
    let ifindex = unsafe {
        if let Ok(c_ifname) = CString::new(ifname.as_str()) {
            if_nametoindex(c_ifname.as_ptr())
        } else {
            return Err(format!("Could not get ifindex from {}", ifname));
        }
    };

    let sock_addr_can_tp = __c_anonymous_sockaddr_can_tp {
        rx_id: 0,
        tx_id: 0
    };

    let can_addr = __c_anonymous_sockaddr_can_can_addr {
        tp: sock_addr_can_tp
    };

    let my_sockaddr: sockaddr_can = sockaddr_can {
        can_family: AF_CAN as u16,
        can_ifindex: ifindex as i32,
        can_addr: can_addr
    };

    let sockaddr_can_ptr: *const sockaddr_can = &my_sockaddr as *const sockaddr_can;
    let sockaddr_ptr = sockaddr_can_ptr as *const sockaddr;
   
    //let conv_addr = SockaddrLike::from_raw(my_sockaddr, Some(mem::size_of::<&sockaddr_can>() as u32));
    let connect_res = unsafe {
        connect(sock, sockaddr_ptr, mem::size_of::<&sockaddr_can>() as u32)
    };

    if connect_res < 0 {
        return Err(format!("Could not connect socket due to {}", Error::last_os_error()));
    }

    return Ok(connect_res);
}

pub fn write_socket(sock: i32, write_bytes: &Vec<u8>, count: usize) -> Result<isize, String>  {
    let wres = unsafe {
        write(sock, vec_to_c_void(&write_bytes), count)
    };
    if wres < 0 {
        return Err(format!("Could not write to socket due to {}", Error::last_os_error()));
    }

    return Ok(wres);
}

pub fn create_can_frame_structure(can_id: u32, can_dlc: u8, addressing_mode: bool, data_vector: &Vec<u8>) -> CanFrame {
    let mut data: [u8; 8] = [0; 8];

    let mut i = 0;
    while i < data_vector.len() {
        data[i] = data_vector[i];
        i += 1;
    }

    let mut eflag: u32 = 0x0;
    
    if addressing_mode {
        eflag = CAN_EFF_FLAG;
    }

    return CanFrame {
        can_id: can_id | eflag,
        can_dlc: can_dlc,
        __pad: 0,
        __res0: 0,
        __res1: 0,
        data: data,
    };
}

pub fn create_time_can_frame_structure(count: u32, ival1_tv_sec: i64, ival1_tv_usec: i64, ival2_tv_sec: i64, 
    ival2_tv_usec: i64, can_id: u32, can_dlc: u8, addressing_mode: bool, data_vector: &Vec<u8>) -> TimedCanFrame {
    let mut copy_data_vector: Vec<u8> = Vec::new();

    for data in data_vector {
        copy_data_vector.push(*data);
    }

    return TimedCanFrame {
        can_id: can_id,
        can_dlc: can_dlc,
        addressing_mode: addressing_mode,
        data_vector: copy_data_vector,
        count: count,
        ival1: timeval { tv_sec: ival1_tv_sec, tv_usec: ival1_tv_usec },
        ival2: timeval { tv_sec: ival2_tv_sec, tv_usec: ival2_tv_usec },
    }
    
}

pub fn create_bcm_head(count: u32, ival1_tv_sec: i64, ival1_tv_usec: i64
    , ival2_tv_sec: i64, ival2_tv_usec: i64, can_id: u32, frames: &Vec<CanFrame>) -> BcmMsgHead{
    return BcmMsgHead {
        opcode: OPCODE::TxSetup as u32,
        flags: BCMFlags::SetTimer as u32 | BCMFlags::StartTimer as u32,
        count: count,
        ival1: timeval { tv_sec: ival1_tv_sec, tv_usec: ival1_tv_usec },
        ival2: timeval { tv_sec: ival2_tv_sec, tv_usec: ival2_tv_usec },
        can_id: can_id,
        nframes: frames.len() as u32,
    };
}

pub fn create_bcm_structure_bytes(count: u32, ival1_tv_sec: i64, ival1_tv_usec: i64
    , ival2_tv_sec: i64, ival2_tv_usec: i64, can_id: u32, frames: &Vec<CanFrame>, write_bytes: &mut Vec<u8>) {
    let head: BcmMsgHead = create_bcm_head(count, ival1_tv_sec, ival1_tv_usec, ival2_tv_sec, ival2_tv_usec, can_id, frames);

    let ptr: *const u8 = &head as *const BcmMsgHead as *const u8;
    let bytes: &[u8] = unsafe { slice::from_raw_parts(ptr, mem::size_of::<BcmMsgHead>()) };

    write_bytes.extend_from_slice(bytes);
    
    for frame in frames {
        let ptr: *const u8 = frame as *const CanFrame as *const u8;
        let bytes: &[u8] = unsafe { slice::from_raw_parts(ptr, mem::size_of::<CanFrame>()) };
        write_bytes.extend_from_slice(bytes);
    }
}