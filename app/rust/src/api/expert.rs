//! The Expert area: every decoded field of the open save as a tree
//! (`ftl_app::raw`, the same as `ftled dump`). A thin adapter; edits go
//! through [`SaveSession::apply`] with the paths `raw.<path>` and
//! `state.<id>`.

use flutter_rust_bridge::frb;

use super::editor::{FieldValue, SaveSession};

/// One field of the save in the raw view, in the order of the model.
pub struct ExpertEntry {
    /// Dotted path, for example `player_ship.resources.scrap`.
    pub path: String,
    /// 0 for a field of the saved game itself, 1 below it, ...
    pub depth: u32,
    /// `record`, `list`, `i32`, `u32`, `bool`, `text`, `enum` or `none`.
    pub kind: String,
    /// The value as text; the item count of a list; empty for a record.
    pub value: String,
    /// The edit of a number or flag (`raw.<path>`, `state.<id>` for a state
    /// var) with its value and the range of its type; `None` when the
    /// entry is not editable.
    pub field: Option<FieldValue>,
    /// A field whose meaning the codec does not know (`unknown_*`).
    pub unknown: bool,
}

pub struct ExpertDump {
    pub entries: Vec<ExpertEntry>,
    /// The undecoded bytes as hex lines of 16 bytes (offset, bytes, ASCII);
    /// empty when the codec decodes the whole file.
    pub undecoded_hex: Vec<String>,
    pub undecoded_bytes: u32,
}

/// The raw view of the open file as it is on disk or was last applied.
#[frb(sync)]
pub fn expert_of(session: &SaveSession) -> ExpertDump {
    let g = session.file.game();
    ExpertDump {
        entries: ftl_app::raw::entries(g)
            .into_iter()
            .map(|e| ExpertEntry {
                field: match (&e.field, e.range) {
                    (Some(f), Some((min, max))) => Some(FieldValue {
                        path: f.path(),
                        value: f.get(g).unwrap_or_default(),
                        min,
                        max,
                    }),
                    _ => None,
                },
                path: e.path,
                depth: e.depth as u32,
                kind: e.kind.name().to_owned(),
                value: e.value,
                unknown: e.unknown,
            })
            .collect(),
        undecoded_hex: ftl_app::raw::hex_lines(&g.undecoded),
        undecoded_bytes: g.undecoded.len() as u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::editor::{FieldEdit, open_save};

    const FIXTURE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../crates/ftl-sav/tests/fixtures/continue-1.6.13-format11.sav"
    );

    #[test]
    fn the_dump_reaches_the_app_and_a_raw_edit_applies() {
        let mut session = open_save(FIXTURE.to_owned()).unwrap();
        let dump = expert_of(&session);
        let scrap = dump
            .entries
            .iter()
            .find(|e| e.path == "player_ship.resources.scrap")
            .unwrap();
        let field = scrap.field.as_ref().unwrap();
        assert_eq!(
            (field.path.as_str(), field.min, field.max),
            ("raw.player_ship.resources.scrap", i32::MIN, i32::MAX)
        );
        assert_eq!(scrap.value, field.value.to_string());
        let var = dump
            .entries
            .iter()
            .find(|e| e.path == "state_vars.0.value")
            .unwrap();
        let var_path = var.field.as_ref().unwrap().path.clone();
        assert!(var_path.starts_with("state."));
        // Every field of the fixture has a name from the FTL binary
        // (docs/unknown-fields.md), so no entry carries the unknown mark.
        assert!(!dump.entries.iter().any(|e| e.unknown));
        assert_eq!(
            dump.undecoded_hex.len(),
            dump.undecoded_bytes.div_ceil(16) as usize
        );

        session
            .apply(
                vec![
                    FieldEdit {
                        path: field.path.clone(),
                        value: 77,
                    },
                    FieldEdit {
                        path: var_path,
                        value: 4,
                    },
                ],
                vec![],
            )
            .unwrap();
        let after = expert_of(&session);
        let value = |p: &str| {
            after
                .entries
                .iter()
                .find(|e| e.path == p)
                .unwrap()
                .value
                .clone()
        };
        assert_eq!(value("player_ship.resources.scrap"), "77");
        assert_eq!(value("state_vars.0.value"), "4");
    }
}
