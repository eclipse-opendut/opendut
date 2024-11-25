use nix::libc::timeval;


/*
    Header of messages sent to or received from the BCM..
    Same as in https://github.com/linux-can/can-utils/blob/master/include/linux/can/bcm.h.
*/
#[repr(C)]
#[derive(Debug)]
pub struct BcmMsgHead {
    pub opcode: u32,
    pub flags: u32,
    pub count: u32,
    pub ival1: timeval,
    pub ival2: timeval,
    pub can_id: u32,
    pub nframes: u32,
}

/*
    Enum of all supported Can frames. Currently, only Can-20 and Can-FD are supported.
 */
#[derive(Debug)]
pub enum CanFrame {
    Can20Frame(Can20Frame),
    CanFdFrame(CanFdFrame)
}

/*
    A Can-20 Frame. Same as in https://github.com/linux-can/can-utils/blob/master/include/linux/can.h.
*/
#[repr(C, packed)]
#[derive(Debug)]
pub struct Can20Frame {
    pub can_id: u32,
    pub len: u8,
    pub __pad: u8,
    pub __res0: u8,
    pub __res1: u8,
    pub data: [u8; 8],
}

/*
    A Can-Fd Frame. Same as in https://github.com/linux-can/can-utils/blob/master/include/linux/can.h.
*/
#[repr(C, packed)]
#[derive(Debug)]
pub struct CanFdFrame {
    pub can_id: u32,
    pub len: u8,
    pub flags: u8,
    pub __res0: u8,
    pub __res1: u8,
    pub data: [u8; 64],
}

/*
    A structure holding can frame information including timing, which is later used to create Can frames sent to the BCM.
*/
#[derive(Debug)]
pub struct TimedCanFrame {
    pub can_id: u32,
    pub len: u8,
    pub addressing_mode: bool,
    pub frame_tx_behavior: bool,
    pub data_vector: Vec<u8>,
    pub count: u32,
    pub ival1: timeval,
    pub ival2: timeval,
}

/*
    Opcodes defining the operation that the BCM should do. Same as in https://github.com/linux-can/can-utils/blob/master/include/linux/can/bcm.h. 
*/
pub enum Opcode {
        TxSetup = 1,
/*        TxDelete,
        TxRead,
        TxSend,
        RxSetup,
        RxDelete,
        RxRead,
        TxStatus,
        TxExpired,
        RxStatus,
        RxTimeout,
        RxChanged*/
}

/*
    BCM flags used in messages sent to the BCM. Same as in https://github.com/linux-can/can-utils/blob/master/include/linux/can/bcm.h.
*/
pub enum BCMFlags {
    SetTimer = 0x0001,
    StartTimer = 0x0002,
/*    TxCountEvt = 0x0004,
    TxAnnounce = 0x0008,
    TxCpCanId = 0x0010,
    RxFilterId = 0x0020,
    RxCheckDlc = 0x0040,
    RxNoAutotimer = 0x0080,
    RxAnnounceResume = 0x0100,
    TxResetMultiIdx = 0x0200,
    RxRtrFrame = 0x0400,*/
    CanFdFrame = 0x0800
}

#[cfg(target_pointer_width = "64")]
pub type TimevalNum = i64;

#[cfg(target_pointer_width = "32")]
pub type TimevalNum = i32;