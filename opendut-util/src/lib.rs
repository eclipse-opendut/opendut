#[cfg(feature = "crypto")]
pub mod crypto;

pub use opendut_util_core::future;

#[cfg(all(feature = "telemetry", not(target_arch = "wasm32")))]
pub use opendut_util_telemetry as telemetry;

#[cfg(feature = "project")]
pub use opendut_util_core::project;


#[cfg(feature = "serde")]
pub mod serde;

#[cfg(all(feature = "settings", not(target_arch = "wasm32")))]
pub mod settings;

pub trait ErrorOr<E> {

    fn err_or<T>(self, value: T) -> Result<T, E>;

    fn err_or_else<T, F>(self, value: F) -> Result<T, E>
        where F: FnOnce() -> T;
}

impl <E> ErrorOr<E> for Option<E> {

    /// Transforms the `Option<E>` into a [`Result<T, E>`], mapping [`Some(err)`] to
    /// [`Err(err)`] and [`None`] to [`Ok(value)`].
    ///
    /// Arguments passed to `err_or` are eagerly evaluated; if you are passing the
    /// result of a function call, it is recommended to use [`err_or_else`], which is
    /// lazily evaluated.
    ///
    /// [`Ok(value)`]: Ok
    /// [`Err(err)`]: Err
    /// [`Some(err)`]: Some
    /// [`err_or_else`]: Option::err_or_else
    ///
    /// # Examples
    ///
    /// ```
    /// use opendut_util::ErrorOr;
    ///
    /// let x = Some("foo");
    /// assert_eq!(x.err_or(0), Err("foo"));
    ///
    /// let x: Option<&str> = None;
    /// assert_eq!(x.err_or(0), Ok(0));
    /// ```
    fn err_or<T>(self, value: T) -> Result<T, E> {
        match self {
            Some(err) => Err(err),
            None => Ok(value),
        }
    }

    /// Transforms the `Option<E>` into a [`Result<T, E>`], mapping [`Some(err)`] to
    /// [`Err(err)`] and [`None`] to [`Ok(value())`].
    ///
    /// [`Err(err)`]: Err
    /// [`Ok(value())`]: Ok
    /// [`Some(err)`]: Some
    ///
    /// # Examples
    ///
    /// ```
    /// use opendut_util::ErrorOr;
    ///
    /// let x = Some("foo");
    /// assert_eq!(x.err_or_else(|| 0), Err("foo"));
    ///
    /// let x: Option<&str> = None;
    /// assert_eq!(x.err_or_else(|| 0), Ok(0));
    /// ```
    fn err_or_else<T, F>(self, value: F) -> Result<T, E>
    where
        F: FnOnce() -> T,
    {
        match self {
            Some(err) => Err(err),
            None => Ok(value()),
        }
    }
}
