//! The API the Flutter app sees. A thin adapter over `ftl-app`: it converts
//! domain values into flat DTOs and holds an opened file as an opaque handle.
//! No rule lives here; every check runs in `ftl-app` and `ftl-core`.

use flutter_rust_bridge::frb;
use ftl_app::{AppError, Assignment, EditError, Field, Patch, SaveFile, SaveOptions, ShipId};

/// An error as the app sees it: a stable code for its own localized text,
/// the file it is about, and the English message of the core as detail.
///
/// Codes: `no_save`, `no_profile`, `changed_on_disk`, `decode`, `verify`, `edit`, `io`,
/// `no_home` (see `AppError::code`); for game data `ftl_dat_not_found`,
/// `config`, `unknown_image_id`, `ftl_dat_not_open`, `entry_missing`,
/// `ftl_dat_unreadable` (see `AssetsError::code`).
#[derive(Debug)]
pub struct EditorError {
    pub code: String,
    pub path: Option<String>,
    pub message: String,
}

impl From<AppError> for EditorError {
    fn from(e: AppError) -> Self {
        EditorError {
            code: e.code().to_owned(),
            path: e.path().map(|p| p.display().to_string()),
            message: e.to_string(),
        }
    }
}

impl From<EditError> for EditorError {
    fn from(e: EditError) -> Self {
        AppError::from(e).into()
    }
}

/// One editable field with its current value and its accepted range.
pub struct FieldValue {
    /// Dotted path, for example `ship.scrap`. The app maps it to a label.
    pub path: String,
    pub value: i32,
    pub min: i32,
    pub max: i32,
}

pub struct SaveSummary {
    pub path: String,
    pub file_format: i32,
    pub dlc_enabled: bool,
    /// `easy`, `normal` or `hard`.
    pub difficulty: String,
    pub ship_name: String,
    pub ship_blueprint_id: String,
    pub sector_number: i32,
    pub fields: Vec<FieldValue>,
    /// The blueprint id of the ship at the current beacon; `None` without
    /// one or when the situation is not decoded.
    pub nearby_blueprint_id: Option<String>,
    /// The resources of that ship (`nearby.ship.<resource>`); empty without one.
    pub nearby_fields: Vec<FieldValue>,
    pub undecoded_bytes: u32,
    pub dirty: bool,
}

pub struct FieldEdit {
    pub path: String,
    pub value: i32,
}

/// A text field edit: `crew.<n>.name` or `crew.<n>.race`.
pub struct TextEdit {
    pub path: String,
    pub text: String,
}

/// A change as the core made it; a number prints as its digits.
pub struct FieldChange {
    pub path: String,
    pub old: String,
    pub new: String,
}

pub struct SaveResult {
    pub path: String,
    pub backup: Option<String>,
    pub bytes: u32,
}

/// An opened `continue.sav`. Dart holds it as an opaque handle.
#[frb(opaque)]
pub struct SaveSession {
    pub(crate) file: SaveFile,
}

impl SaveSession {
    #[frb(sync)]
    pub fn summary(&self) -> SaveSummary {
        let g = self.file.game();
        SaveSummary {
            path: self.file.path().display().to_string(),
            file_format: g.file_format,
            dlc_enabled: g.dlc_enabled,
            difficulty: format!("{:?}", g.difficulty).to_lowercase(),
            ship_name: g.player_ship.name.clone(),
            ship_blueprint_id: g.player_ship.blueprint_id.clone(),
            sector_number: g.sector_number,
            fields: resources(g, ShipId::Player),
            nearby_blueprint_id: g.ship(ShipId::Nearby).map(|s| s.blueprint_id.clone()),
            nearby_fields: resources(g, ShipId::Nearby),
            undecoded_bytes: g.undecoded.len() as u32,
            dirty: self.file.is_dirty(),
        }
    }

    /// Applies all edits in memory, or none when one is invalid.
    #[frb(sync)]
    pub fn apply(
        &mut self,
        edits: Vec<FieldEdit>,
        texts: Vec<TextEdit>,
    ) -> Result<Vec<FieldChange>, EditorError> {
        let changes = self.file.apply(&patch_of(edits, texts)?)?;
        Ok(changes
            .into_iter()
            .map(|c| FieldChange {
                path: c.field.path().to_owned(),
                old: c.old.to_string(),
                new: c.new.to_string(),
            })
            .collect())
    }

    /// Verifies, backs up the old file and writes the file.
    ///
    /// `backup_dir` defaults to the editor's data folder; tests pass their own.
    pub fn save(&mut self, backup_dir: Option<String>) -> Result<SaveResult, EditorError> {
        let backup_dir = match backup_dir {
            Some(d) => d.into(),
            None => ftl_app::default_backup_dir()?,
        };
        let report = self.file.save(&SaveOptions {
            backup_dir: Some(backup_dir),
        })?;
        Ok(SaveResult {
            path: report.path.display().to_string(),
            backup: report.backup.map(|b| b.display().to_string()),
            bytes: report.bytes as u32,
        })
    }
}

/// The resource fields of the ship `on`; empty when the game lacks it.
fn resources(g: &ftl_app::SavedGame, on: ShipId) -> Vec<FieldValue> {
    Field::ALL
        .into_iter()
        .filter_map(|r| {
            let f = Field::from(r).on(on);
            let (min, max) = f.range(g).ok()?;
            Some(FieldValue {
                path: f.path(),
                value: f.get(g).ok()?,
                min,
                max,
            })
        })
        .collect()
}

pub fn open_save(path: String) -> Result<SaveSession, EditorError> {
    Ok(SaveSession {
        file: SaveFile::open(path)?,
    })
}

/// [`open_save`] of the main fixture with the fake layout port: the tests
/// pass with and without a local `ftl.dat` (CI has none).
#[cfg(test)]
pub(crate) fn open_fixture(path: &str) -> SaveSession {
    SaveSession {
        file: SaveFile::open_with_layouts(
            path,
            std::sync::Arc::new(crate::fake::fixture_layouts()),
        )
        .unwrap(),
    }
}

#[frb(sync)]
pub fn default_save_file() -> Result<String, EditorError> {
    Ok(ftl_app::default_save_file()?.display().to_string())
}

#[frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}

/// The one patch of the app's pending edits: the numbers first, then the
/// texts, so a new race fits the skill points the person set, then the crew
/// look ([`Field::is_crew_look`]), so a look picked for a new race stays.
pub(crate) fn patch_of(edits: Vec<FieldEdit>, texts: Vec<TextEdit>) -> Result<Patch, EditorError> {
    let numbers = edits
        .into_iter()
        .map(|e| Ok(Assignment::new(e.path.parse::<Field>()?, e.value)))
        .collect::<Result<Vec<_>, EditorError>>()?;
    let texts = texts
        .into_iter()
        .map(|e| Ok(Assignment::new(e.path.parse::<Field>()?, e.text.as_str())))
        .collect::<Result<Vec<_>, EditorError>>()?;
    // Numbers, then texts (skill edits stay in the units of the file's
    // race), then the crew look, which a race change would reset.
    let (looks, numbers): (Vec<_>, Vec<_>) =
        numbers.into_iter().partition(|a| a.field.is_crew_look());
    Ok(Patch::new(
        numbers.into_iter().chain(texts).chain(looks).collect(),
    ))
}
