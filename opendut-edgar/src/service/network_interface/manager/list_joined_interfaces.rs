use netlink_packet_route::link::{InfoKind, LinkAttribute, LinkInfo};
use rtnetlink::LinkGetRequest;

pub trait ShowJoinedInterfaces {
    fn filter_interfaces_joined_to(self, index: u32) -> Self;
    fn filter_gre_interfaces(self) -> Self;

}

impl ShowJoinedInterfaces for LinkGetRequest {
    fn filter_interfaces_joined_to(mut self, index: u32) -> Self {
        // ip link show master br-opendut  # list devices controlled by br-opendut bridge
        self.message_mut().attributes.extend(vec![
            LinkAttribute::Controller(index),
        ]);
        self
    }
    
    fn filter_gre_interfaces(mut self) -> Self {
        // ip l show type gretap  # show all interfaces with given gretap type
        self.message_mut().attributes.extend(vec![
            LinkAttribute::LinkInfo(vec![
                LinkInfo::Kind(InfoKind::GreTap),
            ]),
        ]);
        self
    }
    
}

