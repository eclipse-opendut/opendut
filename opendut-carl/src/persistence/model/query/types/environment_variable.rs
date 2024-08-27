use diesel::deserialize;
use diesel::deserialize::FromSql;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{Output, ToSql};
use opendut_types::peer::executor::container::ContainerEnvironmentVariable;
use serde_json::json;

type SelfSql = diesel::sql_types::Jsonb;

#[derive(Debug, PartialEq)]
pub(in crate::persistence::model::query) struct PersistableEnvironmentVariable {
    json: serde_json::Value,
}

impl From<ContainerEnvironmentVariable> for PersistableEnvironmentVariable {
    fn from(env: ContainerEnvironmentVariable) -> Self {
        let name = env.name();
        let value = env.value();

        let json = json!({
            "name": name,
            "value": value,
        });

        Self { json }
    }
}
impl From<PersistableEnvironmentVariable> for ContainerEnvironmentVariable {
    fn from(value: PersistableEnvironmentVariable) -> Self {
        let name = value.json.get("name")
            .expect("env JSON in database should have 'name' field")
            .as_str()
            .expect("env JSON in database has 'name' field, but its value is not a string");

        let value = value.json.get("value")
            .expect("env JSON in database should have 'value' field")
            .as_str()
            .expect("env JSON in database has 'value' field, but its value is not a string");

        ContainerEnvironmentVariable::new(name, value)
            .expect("format of envs in database should be legal")
    }
}


impl FromSql<SelfSql, Pg> for PersistableEnvironmentVariable {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        let json = FromSql::<SelfSql, Pg>::from_sql(bytes)?;

        Ok(PersistableEnvironmentVariable { json })
    }
}
impl ToSql<SelfSql, Pg> for PersistableEnvironmentVariable {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
        ToSql::<SelfSql, Pg>::to_sql(&self.json, out)
    }
}
