pub mod api;
mod frb_generated;

/// The fake layout port of the fixtures, so the bridge tests decode the
/// fixture's world without a local `ftl.dat`.
#[cfg(test)]
#[path = "../../../crates/ftl-sav/tests/common/layouts.rs"]
mod fake;
