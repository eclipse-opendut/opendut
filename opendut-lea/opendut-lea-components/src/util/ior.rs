use self::Ior::{Both, Left, Right};

/// Generic representation of an Inclusive-Or.
/// Similar to [std::result::Result], but both values can occur at the same time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ior<L, R> {
    Left(L),
    Right(R),
    Both(L, R),
}

impl<L, R> Ior<L, R> {

    #[inline]
    pub fn is_left(&self) -> bool {
        matches!(self, Ior::Left(_))
    }

    #[inline]
    pub fn is_right(&self) -> bool {
        matches!(self, Ior::Right(_))
    }

    #[inline]
    pub fn is_both(&self) -> bool {
        matches!(self, Ior::Both(_, _))
    }

    #[inline]
    pub fn left(self) -> Option<L> {
        match self {
            Right(_) => None,
            Left(value) => Some(value),
            Both(value, _) => Some(value),
        }
    }

    #[inline]
    pub fn right(self) -> Option<R> {
        match self {
            Left(_) => None,
            Right(value) => Some(value),
            Both(_, value) => Some(value),
        }
    }

    #[inline]
    pub fn both(self) -> Option<(L, R)> {
        match self {
            Left(_) => None,
            Right(_) => None,
            Both(left, right) => Some((left, right)),
        }
    }

    #[inline]
    pub fn map_left<T, F>(self, f: F) -> Ior<T, R>
        where F: FnOnce(L) -> T {
        match self {
            Left(l) => Left(f(l)),
            Right(r) => Right(r),
            Both(l, r) => Both(f(l), r),
        }
    }

    #[inline]
    pub fn map_right<T, F>(self, f: F) -> Ior<L, T>
        where F: FnOnce(R) -> T {
        match self {
            Left(r) => Left(r),
            Right(r) => Right(f(r)),
            Both(l, r) => Both(l, f(r)),
        }
    }

    #[inline]
    pub fn right_ok_or<E>(self, error: E) -> Result<R, E> {
        match self {
            Right(r) | Both(_, r) => Ok(r),
            Left(_) => Err(error),
        }
    }

    #[inline]
    pub fn left_ok_or<E>(self, error: E) -> Result<L, E> {
        match self {
            Left(l) | Both(l, _) => Ok(l),
            Right(_) => Err(error),
        }
    }

    #[inline]
    pub fn as_mut(&mut self) -> Ior<&mut L, &mut R> {
        match *self {
            Left(ref mut l) => Left(l),
            Right(ref mut r) => Right(r),
            Both(ref mut l, ref mut r) => Both(l, r),
        }
    }
}
