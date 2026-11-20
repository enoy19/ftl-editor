//! Game data from the local `ftl.dat`: where it is, the confirmed path, and
//! images as PNG bytes. A thin adapter over `ftl_app::assets`, which holds
//! the opened game data and its image cache for the whole process.

use ftl_app::assets::{self, AssetsConfig, AssetsError, ImageId, Source};

use super::editor::EditorError;

impl From<AssetsError> for EditorError {
    fn from(e: AssetsError) -> Self {
        EditorError {
            code: e.code().to_owned(),
            path: e.path().map(|p| p.display().to_string()),
            message: e.to_string(),
        }
    }
}

/// Where a located `ftl.dat` came from.
pub enum DatSource {
    /// The path the person confirmed earlier.
    Remembered,
    /// Found in a Steam, GOG or Humble folder; not confirmed yet.
    Found,
}

/// What the editor knows about `ftl.dat` right now.
pub struct FtlDatStatus {
    /// The located path; `None` when no candidate exists.
    pub path: Option<String>,
    pub source: Option<DatSource>,
    /// Every place that was searched when nothing was found.
    pub tried: Vec<String>,
    /// The path whose game data is open (images available), if any.
    pub open_path: Option<String>,
}

/// The kinds of [`game_image_of`]. Augments have no image in `ftl.dat`.
pub enum GameImageKind {
    /// The hull of a ship blueprint id.
    Ship,
    /// A crew member's look by its look id (`CrewMember::look_id`): race,
    /// sex and tints, with its visible pixels centered (portraits).
    CrewLook,
    /// The same look anchored as FTL anchors the cell (floorplan).
    CrewLookFrame,
    SystemIcon,
    /// The first frame of a weapon blueprint id.
    WeaponFrame,
    Drone,
    Achievement,
}

fn config(config_dir: Option<String>) -> Result<AssetsConfig, AssetsError> {
    match config_dir {
        Some(dir) => Ok(AssetsConfig::in_dir(dir)),
        None => AssetsConfig::open_default(),
    }
}

/// Locates `ftl.dat`: the remembered path first, then the candidates of this
/// OS. `config_dir` defaults to the editor's OS config folder; tests pass
/// their own.
pub fn ftl_dat_status(config_dir: Option<String>) -> Result<FtlDatStatus, EditorError> {
    let open_path = assets::shared().map(|a| a.path().display().to_string());
    match config(config_dir)?.locate() {
        Ok(located) => Ok(FtlDatStatus {
            path: Some(located.path.display().to_string()),
            source: Some(match located.source {
                Source::Remembered => DatSource::Remembered,
                Source::Found => DatSource::Found,
            }),
            tried: Vec::new(),
            open_path,
        }),
        Err(AssetsError::NotFound { tried }) => Ok(FtlDatStatus {
            path: None,
            source: None,
            tried: tried.iter().map(|p| p.display().to_string()).collect(),
            open_path,
        }),
        Err(e) => Err(e.into()),
    }
}

/// Remembers the `ftl.dat` the person confirmed or picked (the file, its
/// folder or `FTL.app`) and answers the path of the file.
pub fn remember_ftl_dat(config_dir: Option<String>, picked: String) -> Result<String, EditorError> {
    let dat = assets::dat_in(picked.as_ref())?;
    config(config_dir)?.remember(&dat)?;
    Ok(dat.display().to_string())
}

/// Reads the game data of `path` once for the whole process.
pub fn open_game_data(path: String) -> Result<(), EditorError> {
    assets::open_shared(path.as_ref())?;
    Ok(())
}

fn opened() -> Result<std::sync::Arc<assets::GameAssets>, EditorError> {
    Ok(assets::shared().ok_or(AssetsError::NotOpen)?)
}

/// The bytes of an entry of the open `ftl.dat` by its inner path (`img/...`).
pub fn game_image(inner_path: String) -> Result<Vec<u8>, EditorError> {
    Ok(opened()?.image(&inner_path)?.to_vec())
}

/// The PNG bytes of a game image by blueprint id.
pub fn game_image_of(kind: GameImageKind, id: String) -> Result<Vec<u8>, EditorError> {
    let id = match kind {
        GameImageKind::Ship => ImageId::Ship(id),
        GameImageKind::CrewLook => ImageId::CrewLook(id),
        GameImageKind::CrewLookFrame => ImageId::CrewLookFrame(id),
        GameImageKind::SystemIcon => ImageId::SystemIcon(id),
        GameImageKind::WeaponFrame => ImageId::WeaponFrame(id),
        GameImageKind::Drone => ImageId::Drone(id),
        GameImageKind::Achievement => ImageId::Achievement(id),
    };
    Ok(opened()?.image_of(&id)?.to_vec())
}

/// One glyph of a [`GameFont`]: its cell in the atlas and its placement.
pub struct GameGlyph {
    pub code_point: u32,
    pub x: u16,
    pub y: u16,
    pub width: u8,
    pub height: u8,
    /// Rows above the baseline.
    pub top: i8,
    /// Columns before the glyph.
    pub left: i8,
    /// Columns after the glyph; the pen moves `left + width + right`.
    pub right: i8,
}

/// A bitmap font of the open `ftl.dat`: a white atlas PNG whose alpha is the
/// coverage (tint it with any color) and the glyphs that point into it.
pub struct GameFont {
    pub atlas_png: Vec<u8>,
    pub line_height: u8,
    pub baseline: u8,
    pub glyphs: Vec<GameGlyph>,
}

/// A font of the open `ftl.dat` by its inner path (`fonts/JustinFont8.font`).
pub fn game_font(inner_path: String) -> Result<GameFont, EditorError> {
    let font = opened()?.font(&inner_path)?;
    Ok(GameFont {
        atlas_png: font.atlas_png(),
        line_height: font.line_height,
        baseline: font.baseline,
        glyphs: font
            .glyphs
            .iter()
            .map(|g| GameGlyph {
                code_point: g.code_point,
                x: g.x,
                y: g.y,
                width: g.width,
                height: g.height,
                top: g.top,
                left: g.left,
                right: g.right,
            })
            .collect(),
    })
}

/// Pixels from each edge of a frame image that a 9-slice must not stretch.
pub struct ImageInsets {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

/// The 9-slice insets of a frame or button image of the open `ftl.dat`,
/// measured from its pixels.
pub fn game_image_insets(inner_path: String) -> Result<ImageInsets, EditorError> {
    let i = opened()?.insets(&inner_path)?;
    Ok(ImageInsets {
        left: i.left,
        top: i.top,
        right: i.right,
        bottom: i.bottom,
    })
}
