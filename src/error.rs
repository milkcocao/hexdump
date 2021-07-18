use crate::backtrace::Backtrace;
use crate::chain::Chain;
#[cfg(any(feature = "std", anyhow_no_ptr_addr_of))]
use crate::ptr::Mut;
use crate::ptr::{Own, Ref};
use crate::{Error, StdError};
use alloc::boxed::Box;
#[cfg(backtrace)]
use core::any::Demand;
use core::any::TypeId;
use core::fmt::{self, Debug, Display};
use core::mem::ManuallyDrop;
#[cfg(not(anyhow_no_ptr_addr_of))]
use core::ptr;
use core::ptr::NonNull;

#[cfg(feature = "std")]
use core::ops::{Deref, DerefMut};

impl Error {
    /// Create a new error object from any error type.
    ///
    /// The error type must be threadsafe and `'static`, so that the `Error`
    /// will be as well.
    ///
    /// If the error type does not provide a backtrace, a backtrace will be
    /// created here to ensure that a backtrace exists.
    #[cfg(feature = "std")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
    #[cold]
    #[must_use]
    pub fn new<E>(error: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        let backtrace = backtrace_if_absent!(&error);
        Error::from_std(error, backtrace)
    }

    /// Create a new error object from a printable error message.
    ///
    /// If the argument implements std::error::Error, prefer `Error::new`
    /// instead which preserves the underlying error's cause chain and
    /// backtrace. If the argument may or may not implement std::error::Error
    /// now or in the future, use `anyhow!(err)` which handles either way
    /// correctly.
    ///
    /// `Error::msg("...")` is equivalent to `anyhow!("...")` but occasionally
    /// convenient in places where a function is preferable over a macro, such
    /// as iterator or stream combinators:
    ///
    /// ```
    /// # mod ffi {
    /// #     pub struct Input;
    /// #     pub struct Output;
    /// #     pub async fn do_some_work(_: Input) -> Result<Output, &'static str> {
    /// #         unimplemented!()
    /// #     }
    /// # }
    /// #
    /// # use ffi::{Input, Output};
    /// #
    /// use anyhow::{Error, Result};
    /// use futures::stream::{Stream, StreamExt, TryStreamExt};
    ///
    /// async fn demo<S>(stream: S) -> Result<Vec<Output>>
    /// where
    ///     S: Stream<Item = Input>,
    /// {
    ///     stream
    ///         .then(ffi::do_some_work) // returns Result<Output, &str>
    ///         .map_err(Error::msg)
    ///         .try_collect()
    ///         .await
    /// }
    /// ```
    #[cold]
    #[must_use]
    pub fn msg<M>(message: M) -> Self
    where
        M: Display + Debug + Send + Sync + 'static,
    {
        Error::from_adhoc(message, backtrace!())
    }

    #[cfg(feature = "std")]
    #[cold]
    pub(crate) fn from_std<E>(error: E, backtrace: Option<Backtrace>) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        let vtable = &ErrorVTable {
            object_drop: object_drop::<E>,
            object_ref: object_ref::<E>,
            #[cfg(anyhow_no_ptr_addr_of)]
            object_mut: object_mut::<E>,
            object_boxed: object_boxed::<E>,
            object_downcast: object_downcast::<E>,
            #[cfg(anyhow_no_ptr_addr_of)]
            object_downcast_mut: object_downcast_mut::<E>,
            object_drop_rest: object_drop_front::<E>,
            #[cfg(all(not(backtrace), feature = "backtrace"))]
            object_backtrace: no_backtrace,
        };

        // Safety: passing vtable that operates on the right type E.
        unsafe { Error::construct(error, vtable, backtrace) }
    }

    #[cold]
    pub(crate) fn from_adhoc<M>(message: M, backtrace: Option<Backtrace>) -> Self
    where
        M: Display + Debug + Send + Sync + 'static,
    {
        use crate::wrapper::MessageError;
        let error: MessageError<M> = MessageError(message);
        let vtable = &ErrorVTable {
            object_drop: object_drop::<MessageError<M>>,
            object_ref: object_ref::<MessageError<M>>,
            #[cfg(all(feature = "std", anyhow_no_ptr_addr_of))]
            object_mut: object_mut::<MessageError<M>>,
            object_boxed: object_boxed::<MessageError<M>>,
            object_downcast: object_downcast::<M>,
            #[cfg(anyhow_no_ptr_addr_of)]
            object_downcast_mut: object_downcast_mut::<M>,
            object_drop_rest: object_drop_front::<M>,
            #[cfg(all(not(backtrace), feature = "backtrace"))]
            object_backtrace: no_backtrace,
        };

        // Safety: MessageError is repr(transparent) so it is okay for the
        // vtable to allow casting the MessageError<M> to M.
        unsafe { Error::construct(error, vtable, backtrace) }
    }

    #[cold]
    pub(crate) fn from_display<M>(message: M, backtrace: Option<Backtrace>) -> Self
    where
        M: Display + Send + Sync + 'static,
    {
        use crate::wrapper::DisplayError;
        let error: DisplayError<M> = DisplayError(message);
        let vtable = &ErrorVTable {
            object_drop: object_drop::<DisplayError<M>>,
            object_ref: object_ref::<DisplayError<M>>,
            #[cfg(all(feature = "std", anyhow_no_ptr_addr_of))]
            object_mut: object_mut::<DisplayError<M>>,
            object_boxed: object_boxed::<DisplayError<M>>,
            object_downcast: object_downcast::<M>,
            #[cfg(anyhow_no_ptr_addr_of)]
            object_downcast_mut: object_downcast_mut::<M>,
            object_drop_rest: object_drop_front::<M>,
            #[cfg(all(not(backtrace), feature = "backtrace"))]
            object_backtrace: no_backtrace,
        };

        // Safety: DisplayError is repr(transparent) so it is okay for the
        // vtable to allow casting the DisplayError<M> to M.
        unsafe { Error::construct(error, vtable, backtrace) }
    }

    #[cfg(feature = "std")]
    #[cold]
    pub(crate) fn from_context<C, E>(context: C, error: E, backtrace: Option<Backtrace>) -> Self
    where
        C: Display + Send + Sync + 'static,
        E: StdError + Send + Sync + 'static,
    {
        let error: ContextError<C, E> = ContextError { context, error };

        let vtable = &ErrorVTable {
            object_drop: object_drop::<ContextError<C, E>>,
            object_ref: object_ref::<ContextError<C, E>>,
            #[cfg(anyhow_no_ptr_addr_of)]
            object_mut: object_mut::<ContextError<C, E>>,
            object_boxed: object_boxed::<ContextError<C, E>>,
            object_downcast: context_downcast::<C, E>,
            #[cfg(anyhow_no_ptr_addr_of)]
            object_downcast_mut: context_downcast_mut::<C, E>,
            object_drop_rest: context_drop_rest::<C, E>,
            #[cfg(all(not(backtrace), feature = "backtrace"))]
            object_backtrace: no_backtrace,
        };

        // Safety: passing vtable that operates on the right type.
        unsafe { Error::construct(error, vtable, backtrace) }
    }

    #[cfg(feature = "std")]
    #[cold]
    pub(crate) fn from_boxed(
        error: Box<dyn StdError + Send + Sync>,
        backtrace: Option<Backtrace>,
    ) -> Self {
        use crate::wrapper::BoxedError;
        let error = BoxedError(error);
        let vtable = &ErrorVTable {
            object_drop: object_drop::<BoxedError>,
            object_ref: object_ref::<BoxedError>,
            #[cfg(anyhow_no_ptr_addr_of)]
            object_mut: object_mut::<BoxedError>,
            object_boxed: object_boxed::<BoxedError>,
            object_downcast: object_downcast::<Box<dyn StdError + Send + Sync>>,
            #[cfg(anyhow_no_ptr_addr_of)]
            object_downcast_mut: object_downcast_mut::<Box<dyn StdError + Send + Sync>>,
            object_drop_rest: object_drop_front::<Box<dyn StdError + Send + Sync>>,
            #[cfg(all(not(backtrace), feature = "backtrace"))]
            object_backtrace: no_backtrace,
        };

        // Safety: BoxedError is repr(transparent) so it is okay for the vtable
        // to allow casting to Box<dyn StdError + Send + Sync>.
        unsafe { Error::construct(error, vtable, backtrace) }
    }

    // Takes backtrace as argument rather than capturing it here so that the
    // user sees one fewer layer of wrapping noise in the backtrace.
    //
    // Unsafe because the given vtable must have sensible behavior on the error
    // value of type E.
    #[cold]
    unsafe fn construct<E>(
        error: E,
        vtable: &'static ErrorVTable,
        backtrace: Option<Backtrace>,
    ) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        let inner: Box<ErrorImpl<E>> = Box::new(ErrorImpl {
            vtable,
            backtrace,
            _object: error,
        });
        // Erase the concrete type of E from the compile-time type system. This
        // is equivalent to the safe unsize coercion from Box<ErrorImpl<E>> to
        // Box<ErrorImpl<dyn StdError + Send + Sync + 'static>> except that the
        // result is a thin pointer. The necessary behavior for manipulating the
        // underlying ErrorImpl<E> is preserved in the vtable provided by the
        // caller rather than a builtin fat pointer vtable.
        let inner = Own::new(inner).cast::<ErrorImpl>();
        Error { inner }
    }

    /// Wrap the error value with additional context.
    ///
    /// For attaching context to a `Result` as it is propagated, the
    /// [`Context`][crate::Context] extension trait may be more convenient than
    /// this function.
    ///
    /// The primary reason to use `error.context(...)` instead of
    /// `result.context(...)` via the `Context` trait would be if the context
    /// needs to depend on some data held by the underlying error:
    ///
    /// ```
    /// # use std::fmt::{self, Debug, Display};
    /// #
    /// # type T = ();
    /// #
    /// # impl std::error::Error for ParseError {}
    /// # impl Debug for ParseError {
    /// #     fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    /// #         unimplemented!()
    /// #     }
    /// # }
    /// # impl Display for ParseError {
    /// #     fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    /// #         unimplemented!()
    /// #     }
    /// # }
    /// #
    /// use anyhow::Result;
    /// use std::fs::File;
    /// use std::path::Path;
    ///
    /// struct ParseError {
    ///     line: usize,
    ///     column: usize,
    /// }
    ///
    /// fn parse_impl(file: File) -> Result<T, ParseError> {
    ///     # const IGNORE: &str = stringify! {
    ///     ...
    ///     # };
    ///     # unimplemented!()
    /// }
    ///
    /// pub fn parse(path: impl AsRef<Path>) -> Result<T> {
    ///     let file = File::open(&path)?;
    ///     parse_impl(file).map_err(|error| {
    ///         let context = format!(
    ///             "only the first {} lines of {} are valid",
    ///             error.line, path.as_ref().display(),
    ///         );
    ///         anyhow::Error::new(error).context(context)
    ///     })
    /// }
    /// ```
    #[cold]
    #[must_use]
    pub fn context<C>(self, context: C) -> Self
    where
        C: Display + Send + Sync + 'static,
    {
        let error: ContextError<C, Error> = ContextError {
            context,
            error: self,
        };

        let vtable = &ErrorVTable {
            object_drop: object_drop::<ContextError<C, Error>>,
            object_ref: object_ref::<ContextError<C, Error>>,
            #[cfg(all(feature = "std", anyhow_no_ptr_addr_of))]
            object_mut: object_mut::<ContextError<C, Error>>,
            object_boxed: object_boxed::<ContextError<C, Error>>,
            object_downcast: context_chain_downcast::<C>,
            #[cfg(anyhow_no_ptr_addr_of)]
            object_downcast_mut: context_chain_downcast_mut::<C>,
            object_drop_rest: context_chain_drop_rest::<C>,
            #[cfg(all(not(backtrace), feature = "backtrace"))]
            object_backtrace: context_backtrace::<C>,
        };

        // As the cause is anyhow::Error, we already have a backtrace for it.
        let backtrace = None;

        // Safety: passing vtable that operates on the right type.
        unsafe { Error::construct(error, vtable, backtrace) }
    }

    /// Get the backtrace for this Error.
    ///
    /// In order for the backtrace to be meaningful, one of the two environment
    /// variables `RUST_LIB_BACKTRACE=1` or `RUST_BACKTRACE=1` must be defined
    /// and `RUST_LIB_BACKTRACE` must not be `0`. Backtraces are somewhat
    /// expensive to capture in Rust, so we don't necessarily want to be
    /// capturing them all over the place all the time.
    ///
    /// - If you want panics and errors to both have backtraces, set
    ///   `RUST_BACKTRACE=1`;
    /// - If you want only errors to have backtraces, set
    ///   `RUST_LIB_BACKTRACE=1`;
    /// - If you want only panics to have backtraces, set `RUST_BACKTRACE=1` and
    ///   `RUST_LIB_BACKTRACE=0`.
    ///
    /// # Stability