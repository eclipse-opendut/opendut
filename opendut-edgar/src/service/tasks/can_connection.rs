use opendut_model::peer::configuration::parameter;
use crate::service::can::can_manager::CanManagerRef;

pub struct CanConnection {
    pub parameter: parameter::CanConnection,
    pub can_manager_ref: CanManagerRef,
}