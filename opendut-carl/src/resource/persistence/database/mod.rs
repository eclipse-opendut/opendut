use crate::resource::storage::DatabaseConnectInfo;
use crate::resource::ConnectError;
use backon::Retryable;
use diesel::{Connection as _, ConnectionError, PgConnection};
use tracing::{info, warn};

pub mod schema;

pub async fn connect(database_connect_info: &DatabaseConnectInfo) -> Result<PgConnection, ConnectError> {
    let DatabaseConnectInfo { url, username, password, .. } = database_connect_info;

    let confidential_url = {
        let mut url = url.clone();
        url.set_username(username)
            .expect("failed to set username on URL while connecting to database");
        url.set_password(Some(password.secret()))
            .expect("failed to set password on URL while connecting to database");
        url
    };

    let connection = (|| async {
        PgConnection::establish(confidential_url.as_str())
    })
        .retry(backon::ExponentialBuilder::default())
        .when(|cause| match &cause {
            ConnectionError::BadConnection(_) => {
                true
            }
            ConnectionError::CouldntSetupConfiguration(_)
            | ConnectionError::InvalidConnectionUrl(_)
            | ConnectionError::InvalidCString(_) => {
                false
            }
            other => {
                warn!("Unhandled Diesel ConnectionError variant: {other:?}");
                false
            }
        })
        .notify(|cause, after| {
            warn!("Connecting to database at {url} failed. Retrying in {after:?}.\n  {cause}");
        })
        .await
        .map_err(|source| ConnectError::Diesel { url: url.to_owned(), source })?;

    info!("Connection to database at {url} established!");

    Ok(connection)
}
