use std::fmt::{Display, Formatter};

pub type RevisionHash = u128;

pub const ROOT_REVISION_HASH: RevisionHash = 0;

pub trait Versioned {

    fn current_hash(&self) -> &RevisionHash;

    fn parent_hash(&self) -> &RevisionHash;
}

pub trait VersionedMut: Versioned {

    fn current_hash_mut(&mut self) -> &mut RevisionHash;

    fn parent_hash_mut(&mut self) -> &mut RevisionHash;

    fn clear_revision(&mut self) {
        *self.current_hash_mut() = ROOT_REVISION_HASH;
        *self.parent_hash_mut() = ROOT_REVISION_HASH;
    }

    fn derive_revision(&mut self) {
        self.reset_revision(ROOT_REVISION_HASH, *self.current_hash())
    }

    fn reset_revision(&mut self, hash: RevisionHash, parent: RevisionHash) {
        *self.current_hash_mut() = hash;
        *self.parent_hash_mut() = parent;
    }

    fn update_revision(&mut self, hash: RevisionHash) {
        *self.current_hash_mut() = hash;
    }
}

pub trait ToRevision {
    fn revision(&self) -> Revision;
}

pub trait BorrowRevision: Versioned {
    fn borrow_revision(&self) -> BorrowedRevision<Self>;
}

pub trait BorrowMutRevision: VersionedMut {
    fn borrow_mut_revision(&mut self) -> BorrowedMutRevision<Self>;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Revision {
    pub current: RevisionHash,
    pub parent: RevisionHash,
}

impl Revision {
    pub fn new(current: RevisionHash, parent: RevisionHash) -> Self {
        Self { current, parent }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct BorrowedRevision<'a, V>
where
    V: Versioned + ?Sized
{
    pub inner: &'a V,
}

impl <'a, V> BorrowedRevision<'a, V>
where
    V: Versioned + ?Sized
{
    pub fn new(inner: &'a V) -> Self {
        Self { inner }
    }
}

#[derive(Debug)]
pub struct BorrowedMutRevision<'a, V>
where
    V: VersionedMut + ?Sized
{
    inner: &'a mut V,
}

impl <'a, V> BorrowedMutRevision<'a, V>
where
    V: VersionedMut + ?Sized
{
    pub fn new(inner: &'a mut V) -> Self {
        Self { inner }
    }
}

impl Versioned for Revision {

    fn current_hash(&self) -> &RevisionHash {
        &self.current
    }

    fn parent_hash(&self) -> &RevisionHash {
        &self.parent
    }

    // fn derived_revision(&self) -> Revision {
    //     Self::new(ROOT_REVISION_HASH, self.current)
    // }
}

impl VersionedMut for Revision {

    fn current_hash_mut(&mut self) -> &mut RevisionHash {
        &mut self.current
    }

    fn parent_hash_mut(&mut self) -> &mut RevisionHash {
        &mut self.parent
    }
}

impl <V> Versioned for BorrowedRevision<'_, V>
where
    V: Versioned
{
    fn current_hash(&self) -> &RevisionHash {
        self.inner.current_hash()
    }

    fn parent_hash(&self) -> &RevisionHash {
        self.inner.parent_hash()
    }

    // fn derived_revision(&self) -> Revision {
    //     Revision::new(ROOT_REVISION_HASH, *self.inner.current_hash())
    // }
}

impl <V> Versioned for BorrowedMutRevision<'_, V>
where
    V: VersionedMut
{
    fn current_hash(&self) -> &RevisionHash {
        self.inner.current_hash()
    }

    fn parent_hash(&self) -> &RevisionHash {
        self.inner.parent_hash()
    }

    // fn derived_revision(&self) -> Revision {
    //     Revision::new(ROOT_REVISION_HASH, *self.inner.current_hash())
    // }
}

impl <V> VersionedMut for BorrowedMutRevision<'_, V>
where
    V: VersionedMut
{
    fn current_hash_mut(&mut self) -> &mut RevisionHash {
        self.inner.current_hash_mut()
    }

    fn parent_hash_mut(&mut self) -> &mut RevisionHash {
        self.inner.parent_hash_mut()
    }
}

impl Display for Revision {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.current, self.parent)
    }
}

impl <V> From<BorrowedRevision<'_, V>> for Revision
where
    V: Versioned
{
    fn from(value: BorrowedRevision<'_, V>) -> Self {
        Self::new(*value.inner.current_hash(), *value.inner.parent_hash())
    }
}

impl <V> From<BorrowedMutRevision<'_, V>> for Revision
where
    V: VersionedMut
{
    fn from(value: BorrowedMutRevision<'_, V>) -> Self {
        Self::new(*value.inner.current_hash(), *value.inner.parent_hash())
    }
}

impl <V> Display for BorrowedRevision<'_, V>
where
    V: Versioned
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.current_hash(), self.parent_hash())
    }
}

impl <V> Display for BorrowedMutRevision<'_, V>
where
    V: VersionedMut
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.current_hash(), self.parent_hash())
    }
}
