use std::io::Write;

use diesel::deserialize::FromSql;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::{AsExpression, FromSqlRow};
use diesel::sql_types::Text;

#[derive(Clone, Debug, PartialEq, Eq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Text)]
pub enum PersistableNetworkInterfaceKind {
    Ethernet,
    Can,
}
impl ToSql<Text, Pg> for PersistableNetworkInterfaceKind {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
        match *self {
            PersistableNetworkInterfaceKind::Ethernet => out.write_all(ETHERNET)?,
            PersistableNetworkInterfaceKind::Can => out.write_all(CAN)?,
        }
        Ok(IsNull::No)
    }
}
impl FromSql<Text, Pg> for PersistableNetworkInterfaceKind {
    fn from_sql(bytes: PgValue<'_>) -> diesel::deserialize::Result<Self> {
        match bytes.as_bytes() {
            ETHERNET => Ok(PersistableNetworkInterfaceKind::Ethernet),
            CAN => Ok(PersistableNetworkInterfaceKind::Can),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

const ETHERNET: &[u8] = b"ethernet";
const CAN: &[u8] = b"can";
