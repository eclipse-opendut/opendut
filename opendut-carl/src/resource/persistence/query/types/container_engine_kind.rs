use std::io::Write;

use diesel::deserialize::FromSql;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::sql_types::Text;
use diesel::{AsExpression, FromSqlRow};
use opendut_types::peer::executor::container;

#[derive(Debug, PartialEq, Eq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Text)]
pub enum PersistableContainerEngineKind {
    Docker,
    Podman,
}
impl ToSql<Text, Pg> for PersistableContainerEngineKind {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
        match *self {
            PersistableContainerEngineKind::Docker => out.write_all(DOCKER)?,
            PersistableContainerEngineKind::Podman => out.write_all(PODMAN)?,
        }
        Ok(IsNull::No)
    }
}
impl FromSql<Text, Pg> for PersistableContainerEngineKind {
    fn from_sql(bytes: PgValue<'_>) -> diesel::deserialize::Result<Self> {
        match bytes.as_bytes() {
            DOCKER => Ok(PersistableContainerEngineKind::Docker),
            PODMAN => Ok(PersistableContainerEngineKind::Podman),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

const DOCKER: &[u8] = b"docker";
const PODMAN: &[u8] = b"podman";

impl From<container::Engine> for PersistableContainerEngineKind {
    fn from(value: container::Engine) -> Self {
        match value {
            container::Engine::Docker => PersistableContainerEngineKind::Docker,
            container::Engine::Podman => PersistableContainerEngineKind::Podman,
        }
    }
}
impl From<PersistableContainerEngineKind> for container::Engine {
    fn from(value: PersistableContainerEngineKind) -> Self {
        match value {
            PersistableContainerEngineKind::Docker => container::Engine::Docker,
            PersistableContainerEngineKind::Podman => container::Engine::Podman,
        }
    }
}
