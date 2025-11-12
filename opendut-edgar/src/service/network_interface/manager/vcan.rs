use rtnetlink::packet_route::link::{InfoKind, LinkAttribute, LinkInfo};
use rtnetlink::LinkMessageBuilder;

pub const VIRTUAL_CAN_INTERFACE_TYPE: &str = "vcan";

pub trait Vcan {
    fn vcan(self) -> Self;
}
impl<T> Vcan for LinkMessageBuilder<T> {
    fn vcan(self) -> Self {
        self.append_extra_attribute(
            LinkAttribute::LinkInfo(vec![
                LinkInfo::Kind(InfoKind::Other(VIRTUAL_CAN_INTERFACE_TYPE.to_string())),
            ])
        )
    }
}
