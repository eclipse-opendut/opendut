use googletest::prelude::*;
use rstest::rstest;

use fixtures::*;
use opendut_resources::resource::Resource;
use opendut_resources::repository::{ChangeKind, CommitError, GetError, HeadError, Repository};
use opendut_resources::resource::versioning::{RevisionHash, ROOT_REVISION_HASH, ToRevision, Versioned, VersionedMut};
use uuid::uuid;

#[rstest]
pub fn commit_should_compute_the_revision_of_a_resource(fixture: Fixture) -> Result<()> {

    let mut testee = fixture.testee;

    let awesome_id = AwesomeId {
        id: uuid!("a3afee2c-09a9-42e7-8a5a-e138f421d5b6"),
        current_hash: Default::default(),
        parent_hash: Default::default(),
    };

    let change = testee.commit(
        AwesomeResource {
            id: Clone::clone(&awesome_id),
            value: String::from("Hello Awesome World!")
        }
    )?;

    verify_that!(change.kind(), eq(&ChangeKind::Created))?;
    verify_that!(change.uuid(), eq(&awesome_id.id))?;
    verify_that!(change.revision_hash(), eq(&RevisionHash::from(323428928513327140397858641528357845748_u128)))?;
    verify_that!(change.revision_parent(), eq(&ROOT_REVISION_HASH))?;

    // let next_change = {
    //     testee.commit(
    //         AwesomeResource {
    //             id: change.resource_ref().derived_revision(),
    //             value: String::from("Bye Bye"),
    //         }
    //     )?
    // };

    // verify_that!(next_change.kind(), eq(&ChangeKind::Updated))?;
    // verify_that!(next_change.uuid(), eq(&awesome_id.id))?;
    // verify_that!(next_change.revision_hash(), eq(&RevisionHash::from(192892363451858671320584141235385346449_u128)))?;
    // verify_that!(next_change.revision_parent(), eq(change.revision_hash()))?;

    // verify_that!(testee.get(change.resource_ref()),
    //     ok(matches_pattern!(AwesomeResource {
    //         id: eq(change.into_resource_ref()),
    //         value: eq(String::from("Hello Awesome World!"))
    //     }))
    // )?;

    // verify_that!(testee.get(next_change.resource_ref()),
    //     ok(matches_pattern!(AwesomeResource {
    //         id: eq(next_change.into_resource_ref()),
    //         value: eq(String::from("Bye Bye"))
    //     }))
    // )?;

    Ok(())
}

#[rstest]
pub fn commit_should_compute_a_change_which_can_be_applied_to_different_repository(fixture: Fixture) -> Result<()> {

    // let mut testee = fixture.testee;
    // let mut other = Repository::<2>::new(Box::new(AwesomeMarshaller));
    //
    // let awesome_id = AwesomeId {
    //     id: uuid!("a3afee2c-09a9-42e7-8a5a-e138f421d5b6"),
    //     current_hash: Default::default(),
    //     parent_hash: Default::default(),
    // };
    //
    // let change = testee.commit(
    //     AwesomeResource {
    //         id: Clone::clone(&awesome_id),
    //         value: String::from("Hello Awesome World!")
    //     }
    // )?;
    //
    // let patch = change.into_patch(); // simulate replication
    //
    // verify_that!(other.apply(patch), ok(anything()))?;
    //
    Ok(())
}

#[rstest]
pub fn commit_should_fail_if_the_parent_revision_is_not_the_head_revision(fixture: Fixture) -> Result<()> {

    let mut testee = fixture.testee;

    let resource_ref = testee.commit(Clone::clone(&fixture.awesome_resource_1))?;

    verify_that!(testee.commit(fixture.awesome_resource_1),
        err(matches_pattern!(CommitError::InvalidParentRevision {
            uuid: eq(Clone::clone(&fixture.awesome_id_1.id)),
            actual: eq(fixture.awesome_id_1.revision()),
            head: eq(*resource_ref.revision()),
        }))
    )?;

    Ok(())
}

#[rstest]
pub fn commit_should_fail_if_the_parent_revision_does_not_exist(fixture: Fixture) -> Result<()> {

    let mut testee = fixture.testee;

    let mut resource = fixture.awesome_resource_1;

    let revision = resource.resource_ref_mut();
    revision.reset_revision(*revision.current_hash(), RevisionHash::from(42_u32));

    let revision = resource.resource_ref().revision();

    verify_that!(testee.commit(resource),
        err(matches_pattern!(CommitError::UnknownParentRevision {
            uuid: eq(Clone::clone(&fixture.awesome_id_1.id)),
            actual: eq(revision),
        }))
    )?;

    Ok(())
}

#[rstest]
pub fn commit_should_drop_old_revisions(fixture: Fixture) -> Result<()> {

    let mut testee = fixture.testee;

    let first_change = testee.commit(AwesomeResource {
        id: Clone::clone(&fixture.awesome_id_1),
        value: String::from("first revision")
    })?;

    // let second_change = testee.commit(AwesomeResource {
    //     id: first_change.resource_ref().derived_revision(),
    //     value: String::from("second revision")
    // })?;
    //
    // verify_that!(testee.get(first_change.resource_ref()), ok(anything()))?;
    // verify_that!(testee.head(second_change.resource_ref()),
    //     ok(matches_pattern!(AwesomeResource {
    //         value: eq(String::from("second revision"))
    //     }))
    // )?;
    //
    // let third_change = testee.commit(AwesomeResource {
    //     id: second_change.resource_ref().derived_revision(),
    //     value: String::from("third revision")
    // })?;
    //
    // verify_that!(testee.get(first_change.resource_ref()),
    //     err(matches_pattern!(GetError::UnknownRevision {
    //         uuid: eq(fixture.awesome_id_1.id),
    //         revision: eq(first_change.revision().into())
    //     })))?;
    // verify_that!(testee.head(first_change.resource_ref()),
    //     ok(matches_pattern!(AwesomeResource {
    //         value: eq(String::from("third revision"))
    //     }))
    // )?;
    // verify_that!(testee.get(second_change.resource_ref()),
    //     ok(matches_pattern!(AwesomeResource {
    //         value: eq(String::from("second revision"))
    //     }))
    // )?;
    // verify_that!(testee.get(third_change.resource_ref()),
    //     ok(matches_pattern!(AwesomeResource {
    //         value: eq(String::from("third revision"))
    //     }))
    // )?;

    Ok(())
}

#[rstest]
pub fn commit_should_do_nothing_if_the_given_resource_contains_no_changes_towards_the_head_revision(fixture: Fixture) -> Result<()> {

    let mut testee = fixture.testee;

    let change = testee.commit(Clone::clone(&fixture.awesome_resource_1))?;

    verify_that!(change.kind(), eq(&ChangeKind::Created))?;

    // let change = testee.commit(Clone::clone(change.resource()))?;
    //
    // verify_that!(change.kind(), eq(&ChangeKind::Nothing))?;
    //
    // let mut resource = change.into_resource();
    // resource.resource_ref_mut().derive_revision();
    //
    // let change = testee.commit(resource)?;
    //
    // verify_that!(change.kind(), eq(&ChangeKind::Nothing))?;

    Ok(())
}

#[rstest]
pub fn head_should_fail_if_the_given_resource_does_not_exist(fixture: Fixture) -> Result<()> {

    let testee = fixture.testee;

    verify_that!(testee.head(&fixture.awesome_id_2),
        err(matches_pattern!(HeadError::ResourceNotFound {
            uuid: eq(fixture.awesome_id_2.id),
        }))
    )?;

    Ok(())
}

#[rstest]
pub fn get_should_fail_if_the_given_resource_does_not_exist(fixture: Fixture) -> Result<()> {

    let testee = fixture.testee;

    verify_that!(testee.get(&fixture.awesome_id_2),
        err(matches_pattern!(GetError::ResourceNotFound {
            uuid: eq(fixture.awesome_id_2.id),
        }))
    )?;

    Ok(())
}

#[rstest]
pub fn get_should_fail_if_the_given_revision_does_not_exist(fixture: Fixture) -> Result<()> {

    let mut testee = fixture.testee;

    let change = testee.commit(fixture.awesome_resource_2)?;

    // let resource_ref = change.into_resource_ref().derived_revision();
    //
    // verify_that!(testee.get(&resource_ref),
    //     err(matches_pattern!(GetError::UnknownRevision {
    //         uuid: eq(fixture.awesome_id_2.id),
    //         revision: eq(resource_ref.revision())
    //     }))
    // )?;

    Ok(())
}

mod fixtures {
    use std::any::Any;
    use std::io::{Cursor, Write};
    use rstest::fixture;
    use serde::{Deserialize, Serialize};
    use uuid::uuid;
    use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

    use opendut_resources::prelude::*;
    use opendut_resources::repository::Repository;
    use opendut_resources::resource::generic::GenericResource;
    use opendut_resources::resource::marshalling::{MarshalError, MarshallerIdentifier, ResourceTag, UnmarshalError};

    pub struct Fixture {
        pub testee: Repository<2>,
        pub awesome_id_1: AwesomeId,
        pub awesome_resource_1: AwesomeResource,
        pub awesome_id_2: AwesomeId,
        pub awesome_resource_2: AwesomeResource,
    }

    #[fixture]
    pub fn fixture() -> Fixture {

        let awesome_id_1 = AwesomeId {
            id: uuid!("a3afee2c-09a9-42e7-8a5a-e138f421d5b6"),
            current_hash: Default::default(),
            parent_hash: Default::default(),
        };

        let awesome_resource_1 = AwesomeResource {
            id: Clone::clone(&awesome_id_1),
            value: String::from("I'm the first awesome resource!")
        };

        let awesome_id_2 = AwesomeId {
            id: uuid!("3894f4f3-c514-440c-b88c-459b1a24c3b4"),
            current_hash: Default::default(),
            parent_hash: Default::default(),
        };

        let awesome_resource_2 = AwesomeResource {
            id: Clone::clone(&awesome_id_2),
            value: String::from("I'm late, i'm the second one!")
        };

        Fixture {
            testee: Repository::new(Box::new(AwesomeMarshaller)),
            awesome_id_1,
            awesome_resource_1,
            awesome_id_2,
            awesome_resource_2,
        }
    }

    #[derive(Clone, Debug, PartialEq, ResourceRef, Serialize, Deserialize)]
    pub struct AwesomeId {
        pub id: Uuid,
        pub current_hash: RevisionHash,
        pub parent_hash: RevisionHash,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct AwesomeResource {
        pub id: AwesomeId,
        pub value: String,
    }

    pub struct AwesomeMarshaller;

    impl Marshaller for AwesomeMarshaller {

        const IDENTIFIER: MarshallerIdentifier = 8121;

        fn marshal<'r, W>(&self, resource: &'r dyn GenericResource, writer: W) -> Result<Vec<u8>, MarshalError>
        where
            W: Write
        {
            let resource = resource
                .as_any()
                .downcast_ref::<AwesomeResource>()
                .ok_or_else(|| MarshalError::UnknownResource)?;
            let mut bytes = Vec::new();
            // bytes.write_i32::<LittleEndian>(73).unwrap();
            let bytes = postcard::to_io::<AwesomeResource, W>(resource, bytes)
                .map_err(|cause| MarshalError::MarshallingFailure { cause: Box::new(cause) })?;
            Ok(bytes)
        }

        fn unmarshal(&self, bytes: &[u8]) -> Result<Box<dyn GenericResource>, UnmarshalError> {
            let mut bytes = bytes;
            match bytes.read_i32::<LittleEndian>() {
                Ok(73) => {
                    let resource: AwesomeResource = postcard::from_bytes(bytes)
                        .map_err(|cause| UnmarshalError::UnmarshallingFailure { cause: Box::new(cause) })?;
                    Ok(Box::new(resource))
                }
                _ => Err(UnmarshalError::UnknownResource)
            }
        }
    }

    resource!(AwesomeResource, AwesomeId);
}
