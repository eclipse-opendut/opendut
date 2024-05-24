use nix::libc::timeval;

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

#[repr(C, packed)]
#[derive(Debug)]
pub struct CanFrame {
    pub can_id: u32,
    pub can_dlc: u8,
    pub __pad: u8,
    pub __res0: u8,
    pub __res1: u8,
    pub data: [u8; 8],
}

#[derive(Debug)]
pub struct TimedCanFrame {
    pub can_id: u32,
    pub can_dlc: u8,
    pub addressing_mode: bool,
    pub data_vector: Vec<u8>,
    pub count: u32,
    pub ival1: timeval,
    pub ival2: timeval,
}

pub enum OPCODE {
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
    RxRtrFrame = 0x0400,
    CanFdFrame = 0x0800*/
}
