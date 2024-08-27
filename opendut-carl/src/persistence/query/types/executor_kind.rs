use std::io::Write;

use diesel::deserialize::FromSql;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::sql_types::Text;
use diesel::{AsExpression, FromSqlRow};

#[derive(Debug, PartialEq, Eq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Text)]
pub enum PersistableExecutorKind {
    Executable,
    Container,
}
impl ToSql<Text, Pg> for PersistableExecutorKind {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
        match *self {
            PersistableExecutorKind::Executable => out.write_all(EXECUTABLE)?,
            PersistableExecutorKind::Container => out.write_all(CONTAINER)?,
        }
        Ok(IsNull::No)
    }
}
impl FromSql<Text, Pg> for PersistableExecutorKind {
    fn from_sql(bytes: PgValue<'_>) -> diesel::deserialize::Result<Self> {
        match bytes.as_bytes() {
            EXECUTABLE => Ok(PersistableExecutorKind::Executable),
            CONTAINER => Ok(PersistableExecutorKind::Container),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

const EXECUTABLE: &[u8] = b"executable";
const CONTAINER: &[u8] = b"container";
