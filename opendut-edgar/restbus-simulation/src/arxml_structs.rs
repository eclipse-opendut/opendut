/*
    Also see https://www.autosar.org/fileadmin/standards/R22-11/CP/AUTOSAR_TPS_SystemTemplate.pdf.
*/ 


use std::collections::HashMap;

/*
    Represents important data from Autosar CanCluster element.
*/
#[derive(Debug)]
pub struct CanCluster {
    pub name: String,
    pub baudrate: i64,
    pub canfd_baudrate: i64,
    pub can_frame_triggerings: HashMap<i64, CanFrameTriggering>
}

/*
    Represents important data from Autosar CanFrameTriggering element.
*/
#[derive(Debug)]
pub struct CanFrameTriggering {
    pub frame_triggering_name: String,
    pub frame_name: String,
    pub can_id: i64,
    pub addressing_mode: bool,
    pub frame_rx_behavior: bool,
    pub frame_tx_behavior: bool,
    pub rx_range_lower: i64,
    pub rx_range_upper: i64,
    pub sender_ecus: Vec<String>,
    pub receiver_ecus: Vec<String>,
    pub frame_length: i64,
    pub pdu_mappings: Vec<PDUMapping>
}

/*
    Represents important parent data from an Autosar *PDU element.
*/
#[derive(Debug)]
pub struct PDUMapping {
    pub name: String,
    pub byte_order: bool,
    pub start_position: i64,
    pub length: i64,
    pub dynamic_length: String,
    pub category: String,
    pub contained_header_id_short: String,
    pub contained_header_id_long: String,
    pub pdu: PDU
}

/*
    Enum of all supported PDU types. 
*/
#[derive(Debug)]
pub enum PDU {
    ISignalIPDU(ISignalIPDU),
    NMPDU(NMPDU),
//     DCMIPDU(DCMIPDU),
//    NMPDU(NMPDU),
//     ContaineredPDU(XY),
//    Temp(i64)
}


/*
    Represents important data from an Autosar ISignalIPDU element.
*/
#[derive(Debug)]
pub struct ISignalIPDU {
    pub cyclic_timing_period_value: f64,
    pub cyclic_timing_period_tolerance: Option<TimeRangeTolerance>,
    pub cyclic_timing_offset_value: f64,
    pub cyclic_timing_offset_tolerance: Option<TimeRangeTolerance>,
    pub number_of_repetitions: i64,
    pub repetition_period_value: f64,
    pub repetition_period_tolerance: Option<TimeRangeTolerance>,
    pub unused_bit_pattern: bool,
    pub ungrouped_signals: Vec<ISignal>,
    pub grouped_signals: Vec<ISignalGroup>,
}

/*
    Represents important data from an Autosar NmPdu element.
*/
#[derive(Debug)]
pub struct NMPDU {
    pub unused_bit_pattern: bool,
    pub ungrouped_signals: Vec<ISignal>,
    pub grouped_signals: Vec<ISignalGroup>,
}

/*
    Represents important data from an Autosar ISignal element.
*/
#[derive(Debug)]
pub struct ISignal {
    pub name: String,
    pub byte_order: bool,
    pub start_pos: i64,
    pub length: i64,
    pub init_values: InitValues
}

/*
    Enum of the initial values of an ISignal.
*/
#[derive(Debug)]
#[derive(Clone)]
pub enum InitValues {
    Single(i64),
    Array(Vec<i64>),
    NotExist(bool),
}

/*
    Represents important data from an Autosar ISignal element.
*/
#[derive(Debug)]
pub struct E2EDataTransformationProps {
    pub transformer_name: String,
    pub data_id: i64,
    pub data_length: i64
}

#[derive(Debug)]
pub struct ISignalGroup {
    pub name: String,
    pub isignals: Vec<ISignal>,
    pub data_transformations: Vec<String>,
    pub transformation_props: Vec<E2EDataTransformationProps>
}

#[derive(Debug)]
pub enum TimeRangeTolerance {
    Relative(i64),
    Absolute(f64),
}

#[derive(Debug)]
pub struct TimeRange {
    pub tolerance: Option<TimeRangeTolerance>,
    pub value: f64,
}    