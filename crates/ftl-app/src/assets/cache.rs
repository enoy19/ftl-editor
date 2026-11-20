//! The opened game data and a small cache of the images read from `ftl.dat`.
//!
//! [`GameAssets`] reads the blueprints once ([`GameData`]) and keeps them as an
//! `Arc`, so every reader (the bridge, a save that needs ship layouts) shares
//! one copy. An image that [`GameData`] does not keep (ship hulls, drones,
//! achievements, any inner path) is read from the package on demand.
//! `ftldat::Package` is not `Send`, so the package is not kept open: a cache
//! miss opens it again, a hit does not touch it. The cache keeps the last
//! [`IMAGE_CACHE_ENTRIES`] images and drops the one used least recently.
//!
//! [`open_shared`] holds one `GameAssets` for the whole process.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use super::{AssetsError, BitmapFont, GameData, Insets, ReadError};

/// How many images the cache keeps. A player ship hull is about 100 KB.
pub const IMAGE_CACHE_ENTRIES: usize = 128;

/// Where the game data and the bytes of an entry come from. The one
/// implementation outside tests is the `ftl.dat` file; tests count the calls.
pub trait Archive: Send + Sync {
    fn load(&self) -> Result<GameData, AssetsError>;
    fn read(&self, inner_path: &str) -> Result<Vec<u8>, AssetsError>;
}

struct DatFile(PathBuf);

impl Archive for DatFile {
    fn load(&self) -> Result<GameData, AssetsError> {
        Ok(GameData::open(&self.0)?)
    }

    fn read(&self, inner_path: &str) -> Result<Vec<u8>, AssetsError> {
        Ok(ftl_assets::read_entry(&self.0, inner_path)?)
    }
}

/// A game image by blueprint id. Augments have no image in `ftl.dat`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageId {
    /// The hull (`_base`) of a ship blueprint.
    Ship(String),
    /// The standing frame of a crew race, as FTL anchors it (floorplan).
    CrewFrame(String),
    /// The standing frame with its visible pixels centered (crew card).
    CrewPortrait(String),
    /// A crew portrait with its look: sex and tints ([`crew_look_id`]).
    CrewLook(String),
    /// The same look anchored as FTL anchors the cell (floorplan).
    CrewLookFrame(String),
    /// The white overlay icon of a system.
    SystemIcon(String),
    /// The first frame of a weapon's strip.
    WeaponFrame(String),
    /// The sprite of a drone blueprint.
    Drone(String),
    /// The image of an achievement.
    Achievement(String),
}

impl ImageId {
    /// The kind as a word for messages and cache keys.
    pub fn kind(&self) -> &'static str {
        match self {
            ImageId::Ship(_) => "ship",
            ImageId::CrewFrame(_) => "crew race",
            ImageId::CrewPortrait(_) => "crew portrait",
            ImageId::CrewLook(_) => "crew look",
            ImageId::CrewLookFrame(_) => "crew look frame",
            ImageId::SystemIcon(_) => "system",
            ImageId::WeaponFrame(_) => "weapon",
            ImageId::Drone(_) => "drone",
            ImageId::Achievement(_) => "achievement",
        }
    }

    pub fn id(&self) -> &str {
        match self {
            ImageId::Ship(id)
            | ImageId::CrewFrame(id)
            | ImageId::CrewPortrait(id)
            | ImageId::CrewLook(id)
            | ImageId::CrewLookFrame(id)
            | ImageId::SystemIcon(id)
            | ImageId::WeaponFrame(id)
            | ImageId::Drone(id)
            | ImageId::Achievement(id) => id,
        }
    }

    fn unknown(&self) -> AssetsError {
        AssetsError::UnknownId {
            kind: self.kind(),
            id: self.id().to_owned(),
        }
    }
}

/// The id of [`ImageId::CrewLook`]: race, `m` or `f`, then one tint index
/// per layer, joined by `.` (`human.f.0.2`).
pub fn crew_look_id(race: &str, male: bool, tints: &[i32]) -> String {
    let mut id = format!("{race}.{}", if male { "m" } else { "f" });
    for t in tints {
        id.push_str(&format!(".{t}"));
    }
    id
}

fn parse_crew_look(id: &str) -> Option<(&str, bool, Vec<i32>)> {
    let mut parts = id.split('.');
    let race = parts.next().filter(|r| !r.is_empty())?;
    let male = match parts.next()? {
        "m" => true,
        "f" => false,
        _ => return None,
    };
    let tints = parts
        .map(|t| t.parse().ok())
        .collect::<Option<Vec<i32>>>()?;
    Some((race, male, tints))
}

/// The game data of one `ftl.dat` and its image cache.
pub struct GameAssets {
    path: PathBuf,
    data: Arc<GameData>,
    archive: Box<dyn Archive>,
    /// Most recently used first.
    cache: Mutex<VecDeque<(String, Arc<[u8]>)>>,
}

impl std::fmt::Debug for GameAssets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameAssets")
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

impl GameAssets {
    /// Reads the game data of `path` once.
    pub fn open(path: impl Into<PathBuf>) -> Result<GameAssets, AssetsError> {
        let path = path.into();
        GameAssets::with_archive(path.clone(), Box::new(DatFile(path)))
    }

    /// [`GameAssets::open`] over another [`Archive`]; `path` only names it.
    pub fn with_archive(
        path: PathBuf,
        archive: Box<dyn Archive>,
    ) -> Result<GameAssets, AssetsError> {
        Ok(GameAssets {
            data: Arc::new(archive.load()?),
            path,
            archive,
            cache: Mutex::new(VecDeque::new()),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The blueprints, names, layouts and sprites, shared.
    pub fn data(&self) -> &Arc<GameData> {
        &self.data
    }

    /// The bytes of an entry of the package by its inner path (`img/...`).
    pub fn image(&self, inner_path: &str) -> Result<Arc<[u8]>, AssetsError> {
        self.cached(inner_path, || self.archive.read(inner_path))
    }

    /// A bitmap font of the package (`fonts/JustinFont8.font` and so on),
    /// decoded from the cached bytes.
    pub fn font(&self, inner_path: &str) -> Result<BitmapFont, AssetsError> {
        BitmapFont::parse(&self.image(inner_path)?).map_err(|e| {
            ReadError::Parse {
                inner_path: inner_path.to_owned(),
                detail: e.to_string(),
            }
            .into()
        })
    }

    /// The 9-slice insets of a frame or button image, measured from its pixels.
    pub fn insets(&self, inner_path: &str) -> Result<Insets, AssetsError> {
        Ok(ftl_assets::measure_insets(
            inner_path,
            &self.image(inner_path)?,
        )?)
    }

    /// The PNG bytes of a game image by blueprint id.
    pub fn image_of(&self, id: &ImageId) -> Result<Arc<[u8]>, AssetsError> {
        // Typed keys contain a colon; inner paths of ftl.dat never do.
        self.cached(&format!("{}:{}", id.kind(), id.id()), || self.decode(id))
    }

    fn decode(&self, id: &ImageId) -> Result<Vec<u8>, AssetsError> {
        let d = &self.data;
        let no_sprite = |e: ReadError| match e {
            ReadError::NoSprite { .. } => id.unknown(),
            e => e.into(),
        };
        match id {
            ImageId::Ship(ship) => {
                self.first_of(&d.ship_image_paths(ship).ok_or_else(|| id.unknown())?)
            }
            ImageId::Drone(drone) => {
                self.first_of(&d.drone_image_paths(drone).ok_or_else(|| id.unknown())?)
            }
            ImageId::Achievement(a) => self
                .archive
                .read(d.achievement_image_path(a).ok_or_else(|| id.unknown())?),
            ImageId::CrewFrame(race) => d.crew_frame(race).map_err(no_sprite),
            ImageId::CrewPortrait(race) => d.crew_portrait(race).map_err(no_sprite),
            ImageId::CrewLook(look) => {
                let (race, male, tints) = parse_crew_look(look).ok_or_else(|| id.unknown())?;
                d.crew_look(race, male, &tints).map_err(no_sprite)
            }
            ImageId::CrewLookFrame(look) => {
                let (race, male, tints) = parse_crew_look(look).ok_or_else(|| id.unknown())?;
                d.crew_look_frame(race, male, &tints).map_err(no_sprite)
            }
            ImageId::WeaponFrame(weapon) => d.weapon_frame(weapon).map_err(no_sprite),
            ImageId::SystemIcon(system) => d
                .system_icon(system)
                .map(<[u8]>::to_vec)
                .ok_or_else(|| id.unknown()),
        }
    }

    /// The first candidate the package holds; when it holds none, the error
    /// names every candidate.
    fn first_of(&self, candidates: &[String]) -> Result<Vec<u8>, AssetsError> {
        for inner_path in candidates {
            match self.archive.read(inner_path) {
                Err(AssetsError::Read(ReadError::Missing { .. })) => continue,
                found => return found,
            }
        }
        Err(ReadError::Missing {
            inner_path: candidates.join(" or "),
        }
        .into())
    }

    fn lock(&self) -> MutexGuard<'_, VecDeque<(String, Arc<[u8]>)>> {
        self.cache.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The cached bytes for `key`, else `read` and keep them. An error is not
    /// kept. `read` runs outside the lock, so a slow read does not block hits.
    fn cached(
        &self,
        key: &str,
        read: impl FnOnce() -> Result<Vec<u8>, AssetsError>,
    ) -> Result<Arc<[u8]>, AssetsError> {
        {
            let mut cache = self.lock();
            if let Some(i) = cache.iter().position(|(k, _)| k == key) {
                let entry = cache.remove(i).expect("position is in range");
                let bytes = entry.1.clone();
                cache.push_front(entry);
                return Ok(bytes);
            }
        }
        let bytes: Arc<[u8]> = read()?.into();
        let mut cache = self.lock();
        cache.push_front((key.to_owned(), bytes.clone()));
        cache.truncate(IMAGE_CACHE_ENTRIES);
        Ok(bytes)
    }
}

static SHARED: Mutex<Option<Arc<GameAssets>>> = Mutex::new(None);

/// The process-wide [`GameAssets`]: the first call opens `path`; a call with
/// the same path answers the same value; another path replaces it. The lock
/// is held while the package is read, so two calls never read it twice.
pub fn open_shared(path: &Path) -> Result<Arc<GameAssets>, AssetsError> {
    let mut shared = SHARED.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(open) = shared.as_ref().filter(|a| a.path() == path) {
        return Ok(open.clone());
    }
    let opened = Arc::new(GameAssets::open(path)?);
    *shared = Some(opened.clone());
    Ok(opened)
}

/// The value [`open_shared`] opened last, if any.
pub fn shared() -> Option<Arc<GameAssets>> {
    SHARED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Empty game data; every read answers the inner path as bytes and counts.
    #[derive(Default)]
    struct Counting {
        loads: Arc<AtomicUsize>,
        reads: Arc<AtomicUsize>,
    }

    impl Archive for Counting {
        fn load(&self) -> Result<GameData, AssetsError> {
            self.loads.fetch_add(1, Ordering::SeqCst);
            Ok(GameData::default())
        }
        fn read(&self, inner_path: &str) -> Result<Vec<u8>, AssetsError> {
            self.reads.fetch_add(1, Ordering::SeqCst);
            match inner_path.starts_with("img/") {
                true => Ok(inner_path.as_bytes().to_vec()),
                false => Err(ReadError::Missing {
                    inner_path: inner_path.to_owned(),
                }
                .into()),
            }
        }
    }

    fn counting() -> (GameAssets, Arc<AtomicUsize>, Arc<AtomicUsize>) {
        let archive = Counting::default();
        let (loads, reads) = (archive.loads.clone(), archive.reads.clone());
        let assets = GameAssets::with_archive("test.dat".into(), Box::new(archive)).unwrap();
        (assets, loads, reads)
    }

    #[test]
    fn a_cache_hit_does_not_read_the_archive_again() {
        let (assets, loads, reads) = counting();
        assert_eq!(
            &*assets.image("img/ship/a_base.png").unwrap(),
            b"img/ship/a_base.png"
        );
        assert_eq!(
            &*assets.image("img/ship/a_base.png").unwrap(),
            b"img/ship/a_base.png"
        );
        assert_eq!(reads.load(Ordering::SeqCst), 1);
        assert_eq!(
            loads.load(Ordering::SeqCst),
            1,
            "the game data is read once, at open"
        );
    }

    #[test]
    fn the_least_recently_used_image_leaves_the_cache_first() {
        let (assets, _, reads) = counting();
        let path = |i: usize| format!("img/{i}.png");
        for i in 0..IMAGE_CACHE_ENTRIES {
            assets.image(&path(i)).unwrap();
        }
        assets.image(&path(0)).unwrap(); // 0 is now the most recent, 1 the least
        assets.image(&path(IMAGE_CACHE_ENTRIES)).unwrap(); // drops 1
        let before = reads.load(Ordering::SeqCst);
        assets.image(&path(0)).unwrap();
        assert_eq!(
            reads.load(Ordering::SeqCst),
            before,
            "0 stayed in the cache"
        );
        assets.image(&path(1)).unwrap();
        assert_eq!(reads.load(Ordering::SeqCst), before + 1, "1 was dropped");
    }

    #[test]
    fn an_error_is_not_cached() {
        let (assets, _, reads) = counting();
        let err = assets.image("data/none.xml").unwrap_err();
        assert_eq!(err.code(), "entry_missing");
        assert!(err.to_string().contains("data/none.xml"), "{err}");
        assets.image("data/none.xml").unwrap_err();
        assert_eq!(reads.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn an_unknown_id_names_its_kind_and_id_without_reading_the_archive() {
        let (assets, _, reads) = counting();
        for (id, text) in [
            (
                ImageId::Ship("NOPE".into()),
                "ftl.dat has no ship with the id NOPE",
            ),
            (
                ImageId::CrewFrame("nope".into()),
                "ftl.dat has no crew race with the id nope",
            ),
            (
                ImageId::SystemIcon("nope".into()),
                "ftl.dat has no system with the id nope",
            ),
            (
                ImageId::WeaponFrame("NOPE".into()),
                "ftl.dat has no weapon with the id NOPE",
            ),
            (
                ImageId::Drone("NOPE".into()),
                "ftl.dat has no drone with the id NOPE",
            ),
            (
                ImageId::Achievement("NOPE".into()),
                "ftl.dat has no achievement with the id NOPE",
            ),
        ] {
            let err = assets.image_of(&id).unwrap_err();
            assert_eq!(
                (err.code(), err.to_string().as_str()),
                ("unknown_image_id", text)
            );
        }
        assert_eq!(reads.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn the_real_game_answers_every_kind_of_image_id_and_caches_it() {
        let Some(path) = super::super::candidates()
            .unwrap()
            .into_iter()
            .find(|p| p.is_file())
        else {
            println!(
                "skip: no ftl.dat installed at {:?}",
                super::super::candidates().unwrap()
            );
            return;
        };
        let assets = GameAssets::open(&path).unwrap();
        const PNG: &[u8] = b"\x89PNG\r\n\x1a\n";
        for id in [
            ImageId::Ship("PLAYER_SHIP_HARD".into()),
            ImageId::Ship("JELLY_BUTTON_P_DLC".into()),
            ImageId::CrewFrame("engi".into()),
            ImageId::SystemIcon("shields".into()),
            ImageId::WeaponFrame("LASER_BURST_1".into()),
            ImageId::Drone("DRONE_SHIELD_PLAYER".into()),
            ImageId::Achievement("ACH_UNITED_FEDERATION".into()),
        ] {
            let png = assets
                .image_of(&id)
                .unwrap_or_else(|e| panic!("{id:?}: {e}"));
            assert!(png.starts_with(PNG), "{id:?}");
            assert!(
                Arc::ptr_eq(&png, &assets.image_of(&id).unwrap()),
                "{id:?} is cached"
            );
        }
        assert!(
            assets
                .image("img/ship/kestral_base.png")
                .unwrap()
                .starts_with(PNG)
        );
        let font = assets.font("fonts/JustinFont8.font").unwrap();
        assert!(font.glyph('A').is_some());
        assert_eq!(assets.insets("img/window_outline.png").unwrap().left, 31);
    }

    #[test]
    fn a_damaged_font_is_unreadable_and_names_the_entry() {
        let (assets, _, _) = counting();
        // The counting archive answers the inner path as bytes: no FONT signature.
        let err = assets.font("img/x.font").unwrap_err();
        assert_eq!(err.code(), "ftl_dat_unreadable");
        assert_eq!(err.to_string(), "img/x.font: no FONT signature");
    }
}
