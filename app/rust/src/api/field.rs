//! The one conversion of a bounded field of `ftl-app` into the
//! [`FieldValue`] the app sees, shared by every API module.

use ftl_app::ShipId;
use ftl_app::crew::Bounded;

use super::editor::FieldValue;

/// The ship a `nearby` flag of the API picks.
pub(crate) fn ship(nearby: bool) -> ShipId {
    if nearby {
        ShipId::Nearby
    } else {
        ShipId::Player
    }
}

pub(crate) fn field_value(b: Bounded) -> FieldValue {
    FieldValue {
        path: b.field.path(),
        value: b.value,
        min: b.min,
        max: b.max,
    }
}
