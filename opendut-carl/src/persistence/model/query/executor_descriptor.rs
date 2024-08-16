use diesel::{Connection, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};
use opendut_types::peer::executor::container::{ContainerCommand, ContainerCommandArgument, ContainerDevice, ContainerEnvironmentVariable, ContainerImage, ContainerName, ContainerPortSpec, ContainerVolume};
use opendut_types::peer::executor::{ExecutorDescriptor, ExecutorId, ExecutorKind, ResultsUrl};
use opendut_types::peer::PeerId;
use tracing::warn;
use uuid::Uuid;

use crate::persistence::database::schema;
use crate::persistence::error::{PersistenceError, PersistenceOperation, PersistenceResult};
use crate::persistence::model::query::types::container_engine_kind::PersistableContainerEngineKind;
use crate::persistence::model::query::types::environment_variable::PersistableEnvironmentVariable;
use crate::persistence::model::query::types::executor_kind::PersistableExecutorKind;
use crate::persistence::model::query::types::null_removing_text_array::NullRemovingTextArray;

#[derive(diesel::Queryable, diesel::Selectable, diesel::Insertable, diesel::AsChangeset)]
#[diesel(table_name = schema::executor_descriptor)]
#[diesel(belongs_to(PeerDescriptor, foreign_key = peer_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PersistableExecutorDescriptor {
    pub executor_id: Uuid,
    pub kind: PersistableExecutorKind,
    pub results_url: Option<String>,
    pub peer_id: Uuid,
}

#[derive(diesel::Queryable, diesel::Selectable, diesel::Insertable, diesel::Identifiable, diesel::Associations, diesel::AsChangeset, Debug, PartialEq)]
#[diesel(table_name = schema::executor_kind_container)]
#[diesel(primary_key(executor_id))]
#[diesel(belongs_to(PersistableExecutorDescriptor, foreign_key = executor_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PersistableExecutorKindContainer {
    pub executor_id: Uuid,
    engine: PersistableContainerEngineKind,
    name: Option<String>,
    image: String,
    volumes: NullRemovingTextArray,
    devices: NullRemovingTextArray,
    envs: Vec<Option<PersistableEnvironmentVariable>>,
    ports: NullRemovingTextArray,
    command: Option<String>,
    args: NullRemovingTextArray,
}

pub fn insert_into_database(executor: ExecutorDescriptor, peer_id: PeerId, connection: &mut PgConnection) -> PersistenceResult<()> {
    let ExecutorDescriptor { id, kind, results_url } = executor;

    let executor_id = id.uuid;

    let (kind, executor_kind_container) = match kind {
        ExecutorKind::Executable => {
            (PersistableExecutorKind::Executable, None)
        }
        ExecutorKind::Container { engine, name, image, volumes, devices, envs, ports, command, args } => {

            let engine = engine.into();
            let name = match name {
                ContainerName::Empty => None,
                ContainerName::Value(value) => Some(value),
            };
            let image = image.value().to_owned();
            let volumes = volumes.into_iter().map(|volume| volume.value().to_owned()).collect();
            let devices = devices.into_iter().map(|device| device.value().to_owned()).collect();
            let envs = envs.into_iter().map(|env| Some(env.into())).collect();
            let ports = ports.into_iter().map(|port| port.value().to_owned()).collect();
            let command = match command {
                ContainerCommand::Default => None,
                ContainerCommand::Value(value) => Some(value),
            };
            let args = args.into_iter().map(|arg| arg.value().to_owned()).collect();

            let executor_kind_container = PersistableExecutorKindContainer {
                executor_id,
                engine,
                name,
                image,
                volumes,
                devices,
                envs,
                ports,
                command,
                args,
            };
            (PersistableExecutorKind::Container, Some(executor_kind_container))
        }
    };

    let results_url = results_url.map(|url| url.to_string());

    let executor_descriptor = PersistableExecutorDescriptor {
        executor_id,
        kind,
        results_url,
        peer_id: peer_id.uuid,
    };

    insert_persistable(executor_descriptor, executor_kind_container, executor.id, connection)
}


fn insert_persistable(
    executor_descriptor: PersistableExecutorDescriptor,
    maybe_executor_kind_container: Option<PersistableExecutorKindContainer>,
    executor_id: ExecutorId,
    connection: &mut PgConnection
) -> PersistenceResult<()> {

    connection.transaction::<_, PersistenceError, _>(|connection| {

        diesel::insert_into(schema::executor_descriptor::table)
            .values(&executor_descriptor)
            .on_conflict(schema::executor_descriptor::executor_id)
            .do_update()
            .set(&executor_descriptor)
            .execute(connection)
            .map_err(|cause| PersistenceError::insert::<ExecutorDescriptor>(executor_id.uuid, cause))?;

        maybe_executor_kind_container.map(|executor_kind_container| {
            diesel::insert_into(schema::executor_kind_container::table)
                .values(&executor_kind_container)
                .on_conflict(schema::executor_kind_container::executor_id)
                .do_update()
                .set(&executor_kind_container)
                .execute(connection)
                .map_err(|cause| PersistenceError::insert::<PersistableExecutorKindContainer>(executor_id.uuid, cause))
        }).transpose()?;

        Ok(())
    })?;

    Ok(())
}


pub fn list_filtered_by_peer(
    peer_id: PeerId,
    connection: &mut PgConnection
) -> PersistenceResult<Vec<ExecutorDescriptor>> {
    let persistables = list_filtered_by_peer_id_persistable(peer_id, connection)?;

    let result = persistables.into_iter().map(|(persistable_executable_descriptor, persistable_executable_kind_container)| {
        let PersistableExecutorDescriptor { executor_id, kind, results_url, peer_id: _ } = persistable_executable_descriptor;

        let id = ExecutorId::from(executor_id);

        let kind = executor_kind_from_persistable(kind, persistable_executable_kind_container)?;

        let results_url = results_url.map(ResultsUrl::try_from).transpose()
            .map_err(PersistenceError::list::<ExecutorDescriptor>)?;

        Ok(ExecutorDescriptor { id, kind, results_url })
    }).collect::<PersistenceResult<_>>()?;

    Ok(result)
}


fn list_filtered_by_peer_id_persistable(
    peer_id: PeerId,
    connection: &mut PgConnection
) -> PersistenceResult<Vec<(
    PersistableExecutorDescriptor,
    Option<PersistableExecutorKindContainer>
)>> {
    schema::executor_descriptor::table
        .left_join(schema::executor_kind_container::table)
        .filter(schema::executor_descriptor::peer_id.eq(peer_id.uuid))
        .select((PersistableExecutorDescriptor::as_select(), Option::<PersistableExecutorKindContainer>::as_select()))
        .get_results(connection)
        .map_err(PersistenceError::list::<ExecutorDescriptor>)
}

fn executor_kind_from_persistable(
    persistable_executor_kind: PersistableExecutorKind,
    persistable_executor_kind_container: Option<PersistableExecutorKindContainer>,
) -> PersistenceResult<ExecutorKind> {
    let result = match persistable_executor_kind {
        PersistableExecutorKind::Executable => ExecutorKind::Executable,
        PersistableExecutorKind::Container => {
            let persistable_executor_kind_container = persistable_executor_kind_container
                .ok_or(PersistenceError::new::<ExecutorKind>(None::<Uuid>, PersistenceOperation::List, Option::<PersistenceError>::None))?;

            let PersistableExecutorKindContainer { executor_id: _, engine, name, image, volumes, devices, envs, ports, command, args } = persistable_executor_kind_container;

            let engine = engine.into();

            let name = match name {
                Some(value) => {
                    ContainerName::try_from(value)
                        .map_err(PersistenceError::list::<ExecutorKind>)?
                }
                None => ContainerName::Empty
            };

            let image = ContainerImage::try_from(image)
                .map_err(PersistenceError::list::<ExecutorKind>)?;

            let volumes = volumes.into_iter()
                .map(ContainerVolume::try_from)
                .collect::<Result<Vec<_>, _>>()
                .map_err(PersistenceError::list::<ExecutorKind>)?;

            let devices = devices.into_iter()
                .map(ContainerDevice::try_from)
                .collect::<Result<Vec<_>, _>>()
                .map_err(PersistenceError::list::<ExecutorKind>)?;

            let envs = envs.into_iter()
                .filter_map(|env| {
                    if env.is_none() {
                        warn!("Database contained a NULL value in a list of envs. Removing the NULL value and loading list without it.");
                    }
                    env
                })
                .map(ContainerEnvironmentVariable::try_from)
                .collect::<Result<Vec<_>, _>>()
                .map_err(PersistenceError::list::<ExecutorKind>)?;

            let ports = ports.into_iter()
                .map(ContainerPortSpec::try_from)
                .collect::<Result<Vec<_>, _>>()
                .map_err(PersistenceError::list::<ExecutorKind>)?;

             let command = match command {
                Some(value) => {
                    ContainerCommand::try_from(value)
                        .map_err(PersistenceError::list::<ExecutorKind>)?
                }
                None => ContainerCommand::Default
            };

            let args = args.into_iter()
                .map(ContainerCommandArgument::try_from)
                .collect::<Result<Vec<_>, _>>()
                .map_err(PersistenceError::list::<ExecutorKind>)?;

            ExecutorKind::Container {
                engine,
                name,
                image,
                volumes,
                devices,
                envs,
                ports,
                command,
                args,
            }
        }
    };
    Ok(result)
}

