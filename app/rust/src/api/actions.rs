//! The one-click actions for the app. A thin adapter over
//! `ftl_app::actions`: the core plans the patch with the limits of the open
//! `ftl.dat`; the app puts its edits into the pending edits and shows them in
//! the change sheet before anything is written.

use flutter_rust_bridge::frb;
use ftl_app::actions::{self, Action};
use ftl_app::assets;

use super::editor::{EditorError, FieldEdit, SaveSession};

/// What an action could not do or where it fell back to an editor cap.
pub struct ActionNoteInfo {
    /// `hull_at_editor_cap`, `systems_at_editor_cap`,
    /// `rooms_not_decoded`.
    pub code: String,
    /// The cap the note is about; 0 for none.
    pub count: i32,
    /// The English text of the core, as detail.
    pub message: String,
}

/// The edits of an action on the open file, only values it raises.
pub struct PlannedAction {
    pub id: String,
    pub edits: Vec<FieldEdit>,
    pub notes: Vec<ActionNoteInfo>,
}

/// The ids of the actions, in the order of the buttons:
/// `max-everything`, `crew-fully-trained`, `repair-ship`, `heal-crew`.
#[frb(sync)]
pub fn action_ids() -> Vec<String> {
    Action::ALL.iter().map(|a| a.id().to_owned()).collect()
}

/// Plans `action` on the file as it was read; writes nothing.
#[frb(sync)]
pub fn plan_action(session: &SaveSession, action: String) -> Result<PlannedAction, EditorError> {
    let action: Action = action
        .parse()
        .map_err(|e: actions::UnknownAction| EditorError {
            code: "edit".into(),
            path: None,
            message: e.to_string(),
        })?;
    let data = assets::shared();
    let plan = actions::plan(
        action,
        session.file.game(),
        data.as_ref().map(|a| a.data().as_ref()),
    );
    Ok(PlannedAction {
        id: action.id().to_owned(),
        edits: plan
            .patch
            .assignments
            .into_iter()
            .map(|a| FieldEdit {
                path: a.field.path(),
                value: a.value.as_int().expect("an action assigns numbers only"),
            })
            .collect(),
        notes: plan
            .notes
            .iter()
            .map(|n| ActionNoteInfo {
                code: n.code().to_owned(),
                count: n.count(),
                message: n.to_string(),
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::editor::open_save;

    const FIXTURE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../crates/ftl-sav/tests/fixtures/continue-1.6.13-format11.sav"
    );

    #[test]
    fn a_planned_action_reaches_the_app_and_applies() {
        let mut session = open_save(FIXTURE.to_owned()).unwrap();
        assert_eq!(action_ids()[0], "max-everything");
        let plan = plan_action(&session, "max-everything".into()).unwrap();
        let scrap = plan.edits.iter().find(|e| e.path == "ship.scrap").unwrap();
        assert_eq!(scrap.value, 99_999);
        assert!(
            plan.edits
                .iter()
                .any(|e| e.path == "reactor" && e.value == 25)
        );
        session.apply(plan.edits, vec![]).unwrap();
        assert!(
            plan_action(&session, "max-everything".into())
                .unwrap()
                .edits
                .is_empty()
        );
        assert_eq!(
            plan_action(&session, "nope".into()).err().unwrap().code,
            "edit"
        );
    }
}
