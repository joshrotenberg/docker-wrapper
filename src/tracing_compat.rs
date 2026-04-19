//! Feature-gated re-exports of [`tracing`] macros.
//!
//! Internal compatibility shim that lets call sites emit spans and events without
//! caring whether the `tracing` feature is enabled. When the feature is off,
//! every macro expands to an empty block, and the `tracing` crate is not a
//! compile dependency.
//!
//! Use via `use crate::tracing_compat::{info, debug, ...};` and gate
//! `#[instrument(...)]` attributes with
//! `#[cfg_attr(feature = "tracing", tracing::instrument(...))]` so they also
//! disappear when the feature is off.

#![allow(unused_imports)]

#[cfg(feature = "tracing")]
pub(crate) use tracing::{debug, debug_span, error, info, info_span, trace, warn, Instrument};

#[cfg(not(feature = "tracing"))]
pub(crate) use self::noop::*;

#[cfg(not(feature = "tracing"))]
mod noop {
    /// No-op replacement for `tracing` event macros (`info!`, `debug!`, ...).
    ///
    /// Swallows all tokens (fields, format args, target overrides, ...) and
    /// expands to an empty block.
    #[macro_export]
    #[doc(hidden)]
    macro_rules! __docker_wrapper_tracing_event_noop {
        ($($tt:tt)*) => {{}};
    }

    /// No-op replacement for span-constructor macros (`info_span!`, ...).
    ///
    /// Swallows all tokens and yields a [`NoopSpan`] value so call sites can use
    /// `.entered()` / `.in_scope(...)` / `.record(...)` ergonomics.
    #[macro_export]
    #[doc(hidden)]
    macro_rules! __docker_wrapper_tracing_span_noop {
        ($($tt:tt)*) => {{
            $crate::tracing_compat::NoopSpan
        }};
    }

    pub(crate) use crate::__docker_wrapper_tracing_event_noop as debug;
    pub(crate) use crate::__docker_wrapper_tracing_event_noop as error;
    pub(crate) use crate::__docker_wrapper_tracing_event_noop as info;
    pub(crate) use crate::__docker_wrapper_tracing_event_noop as trace;
    pub(crate) use crate::__docker_wrapper_tracing_event_noop as warn;
    pub(crate) use crate::__docker_wrapper_tracing_span_noop as debug_span;
    pub(crate) use crate::__docker_wrapper_tracing_span_noop as info_span;
}

/// Placeholder span returned by [`info_span!`] and friends when the `tracing`
/// feature is disabled. Mirrors the subset of the `tracing::Span` API that the
/// crate uses.
#[cfg(not(feature = "tracing"))]
#[derive(Copy, Clone, Debug)]
pub(crate) struct NoopSpan;

#[cfg(not(feature = "tracing"))]
#[allow(dead_code, clippy::unused_self, clippy::trivially_copy_pass_by_ref)]
impl NoopSpan {
    /// Run `f` within this (no-op) span. Mirrors `tracing::Span::in_scope`.
    pub(crate) fn in_scope<F: FnOnce() -> R, R>(&self, f: F) -> R {
        f()
    }

    /// Record a field on this (no-op) span. Mirrors `tracing::Span::record`.
    pub(crate) fn record<V>(&self, _field: &'static str, _value: V) -> &Self {
        self
    }
}

/// Helper trait mirroring `tracing::Instrument` when the feature is disabled.
///
/// Provides a no-op `.instrument(span)` method on all futures so call sites can
/// use the same ergonomics regardless of feature state.
#[cfg(not(feature = "tracing"))]
pub(crate) trait Instrument: Sized {
    fn instrument(self, _span: NoopSpan) -> Self {
        self
    }
}

#[cfg(not(feature = "tracing"))]
impl<T> Instrument for T {}
