use std::mem::size_of;
use std::net::Ipv4Addr;

use netlink_packet_route::link::{InfoData, InfoKind, LinkAttribute, LinkInfo};
use netlink_packet_utils::{Emitable, Parseable};
use netlink_packet_utils::byteorder::{ByteOrder, NativeEndian, WriteBytesExt};
use netlink_packet_utils::nla::NlaBuffer;
use rtnetlink::LinkAddRequest;

pub trait Gretap {
    fn gretap_v4(self, name: impl Into<String>, local_ip: &Ipv4Addr, remote_ip: &Ipv4Addr) -> Self;
}
impl Gretap for LinkAddRequest {
    fn gretap_v4(mut self, name: impl Into<String>, local_ip: &Ipv4Addr, remote_ip: &Ipv4Addr) -> Self {

        // Byte-values extracted from WireShark via nlmon-interface
        // and command `ip link add name <NAME> type gretap local <LOCAL_IP> remote <REMOTE_IP>`.
        // Compare with implementation of ip-command: https://github.com/shemminger/iproute2/blob/040325f543a1f7e6bb336355c136984e9bbe00d6/ip/link_gre.c#L394
        let attributes = [
            InfoGreTap::IKey(0),
            InfoGreTap::OKey(0),
            InfoGreTap::IFlags(0),
            InfoGreTap::OFlags(0),
            InfoGreTap::Local(u32::from_le_bytes(local_ip.octets())),
            InfoGreTap::Remote(u32::from_le_bytes(remote_ip.octets())),
            InfoGreTap::Pmtudisc(1),
            InfoGreTap::Tos(0),
            InfoGreTap::Ttl(0),
            InfoGreTap::FwMark(0),
            InfoGreTap::EncapType(0),
            InfoGreTap::EncapFlags(0),
            InfoGreTap::EncapSPort(0),
            InfoGreTap::EncapDPort(0),
        ];

        let attributes = attributes.map(|attribute| {
            let mut buffer = vec![0u8; attribute.buffer_len()];
            attribute.emit(&mut buffer);
            let buffer = NlaBuffer::new(&buffer);
            netlink_packet_route::link::InfoGreTap::parse(&buffer)
                .expect("GRE attribute should be parseable from constant") //if not, this is a bug in how we specify the attribute
        });

        self.message_mut().attributes.extend(vec![
            LinkAttribute::IfName(name.into()),
            LinkAttribute::LinkInfo(vec![
                LinkInfo::Kind(InfoKind::GreTap),
                LinkInfo::Data(InfoData::GreTap(attributes.to_vec())),
            ]),
        ]);
        self
    }
}

#[allow(dead_code)]
enum InfoGreTap { // https://elixir.bootlin.com/linux/v6.5.3/source/include/uapi/linux/if_tunnel.h#L117
    Unspec,
    Link,
    IFlags(u16),
    OFlags(u16),
    IKey(u32),
    OKey(u32),
    Local(u32),
    Remote(u32),
    Ttl(u8),
    Tos(u8),
    Pmtudisc(u8),
    EncapLimit,
    FlowInfo,
	Flags,
	EncapType(u16),
	EncapFlags(u16),
	EncapSPort(u16),
	EncapDPort(u16),
	CollectMetadata,
	IgnoreDf,
	FwMark(u32),
	ErspanIndex,
	ErspanVer,
	ErspanDir,
	ErspanHwid,
	Max,
}
impl netlink_packet_utils::nla::Nla for InfoGreTap {
    fn value_len(&self) -> usize {
        match self {
            Self::Unspec => unimplemented!(),
            Self::Link => unimplemented!(),
            Self::IFlags(_) => size_of::<u16>(),
            Self::OFlags(_) => size_of::<u16>(),
            Self::IKey(_) => size_of::<u32>(),
            Self::OKey(_) => size_of::<u32>(),
            Self::Local(_) => size_of::<u32>(),
            Self::Remote(_) => size_of::<u32>(),
            Self::Ttl(_) => size_of::<u8>(),
            Self::Tos(_) => size_of::<u8>(),
            Self::Pmtudisc(_) => size_of::<u8>(),
            Self::EncapLimit => unimplemented!(),
            Self::FlowInfo => unimplemented!(),
            Self::Flags => unimplemented!(),
            Self::EncapType(_) => size_of::<u16>(),
            Self::EncapFlags(_) => size_of::<u16>(),
            Self::EncapSPort(_) => size_of::<u16>(),
            Self::EncapDPort(_) => size_of::<u16>(),
            Self::CollectMetadata => unimplemented!(),
            Self::IgnoreDf => unimplemented!(),
            Self::FwMark(_) => size_of::<u32>(),
            Self::ErspanIndex => unimplemented!(),
            Self::ErspanVer => unimplemented!(),
            Self::ErspanDir => unimplemented!(),
            Self::ErspanHwid => unimplemented!(),
            Self::Max => unimplemented!(),
        }
    }
    fn kind(&self) -> u16 {
        match self {
            Self::Unspec          => 0x00,
            Self::Link            => 0x01,
            Self::IFlags(_)       => 0x02,
            Self::OFlags(_)       => 0x03,
            Self::IKey(_)         => 0x04,
            Self::OKey(_)         => 0x05,
            Self::Local(_)        => 0x06,
            Self::Remote(_)       => 0x07,
            Self::Ttl(_)          => 0x08,
            Self::Tos(_)          => 0x09,
            Self::Pmtudisc(_)     => 0x0a,
            Self::EncapLimit      => 0x0b,
            Self::FlowInfo        => 0x0c,
            Self::Flags           => 0x0d,
            Self::EncapType(_)    => 0x0e,
            Self::EncapFlags(_)   => 0x0f,
            Self::EncapSPort(_)   => 0x10,
            Self::EncapDPort(_)   => 0x11,
            Self::CollectMetadata => 0x12,
            Self::IgnoreDf        => 0x13,
            Self::FwMark(_)       => 0x14,
            Self::ErspanIndex     => 0x15,
            Self::ErspanVer       => 0x16,
            Self::ErspanDir       => 0x17,
            Self::ErspanHwid      => 0x18,
            Self::Max             => 0x19,
        }
    }
    fn emit_value(&self, mut buffer: &mut [u8]) {
        match self {
            Self::Unspec => unimplemented!(),
            Self::Link => unimplemented!(),
            Self::IFlags(bytes) => NativeEndian::write_u16(buffer, *bytes),
            Self::OFlags(bytes) => NativeEndian::write_u16(buffer, *bytes),
            Self::IKey(bytes) => NativeEndian::write_u32(buffer, *bytes),
            Self::OKey(bytes) => NativeEndian::write_u32(buffer, *bytes),
            Self::Local(bytes) => NativeEndian::write_u32(buffer, *bytes),
            Self::Remote(bytes) => NativeEndian::write_u32(buffer, *bytes),
            Self::Ttl(byte) => WriteBytesExt::write_u8(&mut buffer, *byte).unwrap(),
            Self::Tos(byte) => WriteBytesExt::write_u8(&mut buffer, *byte).unwrap(),
            Self::Pmtudisc(byte) => WriteBytesExt::write_u8(&mut buffer, *byte).unwrap(),
            Self::EncapLimit => unimplemented!(),
            Self::FlowInfo => unimplemented!(),
            Self::Flags => unimplemented!(),
            Self::EncapType(bytes) => NativeEndian::write_u16(buffer, *bytes),
            Self::EncapFlags(bytes) => NativeEndian::write_u16(buffer, *bytes),
            Self::EncapSPort(bytes) => NativeEndian::write_u16(buffer, *bytes),
            Self::EncapDPort(bytes) =>  NativeEndian::write_u16(buffer, *bytes),
            Self::CollectMetadata => unimplemented!(),
            Self::IgnoreDf => unimplemented!(),
            Self::FwMark(bytes) => NativeEndian::write_u32(buffer, *bytes),
            Self::ErspanIndex => unimplemented!(),
            Self::ErspanVer => unimplemented!(),
            Self::ErspanDir => unimplemented!(),
            Self::ErspanHwid => unimplemented!(),
            Self::Max => unimplemented!(),
        }
    }
}
