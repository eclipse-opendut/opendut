use netlink_packet_route::link::{InfoKind, LinkAttribute, LinkInfo};
use rtnetlink::LinkAddRequest;

pub const VIRTUAL_CAN_INTERFACE_TYPE: &str = "vcan";

pub trait VCan {
    fn create_virtual_can_request(self, name: impl Into<String>) -> Self;
}
impl VCan for LinkAddRequest {
    fn create_virtual_can_request(mut self, name: impl Into<String>) -> Self {
        self.message_mut().attributes.extend(vec![
            LinkAttribute::IfName(name.into()),
            LinkAttribute::LinkInfo(vec![
                LinkInfo::Kind(InfoKind::Other(VIRTUAL_CAN_INTERFACE_TYPE.to_string())),
            ]),
        ]);
        self
    }
}
