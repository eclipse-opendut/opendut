use diesel::backend::Backend;
use diesel::deserialize::{FromSql, FromSqlRow};
use diesel::pg::Pg;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::{Array, Nullable, Text};
use diesel::AsExpression;
use std::any::Any;
use std::iter::FilterMap;
use std::vec::IntoIter;
use tracing::warn;

type SelfSql = Array<Nullable<Text>>;

/// It is not possible to specify in Postgres that array elements should be non-null (beyond runtime constraints).
/// Therefore, Diesel always treats array elements as Options.
/// Handling of these can always be the same (remove any NULLs, if they occur).
#[derive(Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = SelfSql)]
pub struct NullRemovingTextArray {
    inner: Vec<Option<String>>,
}

impl FromIterator<String> for NullRemovingTextArray {
    fn from_iter<Iter: IntoIterator<Item=String>>(iter: Iter) -> Self {
        Self {
            inner: iter.into_iter().map(Some).collect(),
        }
    }
}
impl IntoIterator for NullRemovingTextArray {
    type Item = String;
    type IntoIter = FilterMap<IntoIter<Option<String>>, fn(Option<String>) -> Option<String>>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
            .filter_map(|value| {
                if value.is_none() {
                    warn!("Database contained a NULL value in a list of {name:?}. Removing the NULL value and loading list without it.", name=value.type_id());
                }
                value
            })
    }
}

impl FromSql<SelfSql, Pg> for NullRemovingTextArray {
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let inner = FromSql::<SelfSql, Pg>::from_sql(bytes)?;
        Ok(Self { inner })
    }
}
impl ToSql<SelfSql, Pg> for NullRemovingTextArray {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
        ToSql::<SelfSql, Pg>::to_sql(&self.inner, out)
    }
}
