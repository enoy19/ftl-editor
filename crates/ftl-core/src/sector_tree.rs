//! The sector tree: the eight columns of sectors a run crosses, which FTL
//! builds from the tree seed and the sector lists of `data/sector_data.xml`,
//! and the sector jump (`sector.jump`).
//!
//! Evidence (FTL 1.6.13, macOS binary, symbols, read with Ghidra 12.1.4 on
//! 2026-11-19):
//!
//! - `StarMap::LoadGame` (0x1001cab20) seeds the RNG with the stored tree
//!   seed (`srandom32`, or `srand` with the header's `random_native`) and
//!   calls `StarMap::GenerateSectorMap` (0x1001c6ac0). `StarMap::NewGame`
//!   draws the seed with `random32` and does the same.
//! - `GenerateSectorMap` reloads `data/sector_data.xml` (so every unique
//!   flag is clear), builds the hidden Crystal sector apart
//!   (`GetSpecificSector("CRYSTAL_HOME")`, no draw), then the first sector:
//!   one type draw it overwrites, point (0, 75), level 0. Six calls of
//!   `StarMap::AddSectorColumn` follow, then the Last Stand: one type draw
//!   it overwrites with hostile, point (x of the last column + 66, 75),
//!   level 7, the description `FINAL`, every sector of the last column
//!   leads to it. Last, the first sector gets `STANDARD_SPACE` with the
//!   name `sectorname_CIVILIAN_SECTOR`, civilian. The sectors are stored in
//!   creation order; the file's visited flags follow that order.
//! - A type draw (`GetRandomSectorChoice`): `r % 10` below 2 is nebula (2),
//!   above 5 hostile (1), else civilian (0).
//! - `AddSectorColumn`: the count is `r % 3 + 2`, drawn again while it
//!   equals the count of the last column, and 2 after the first sector.
//!   x is the last column's + 66, y starts at `(200 - 50 * count) / 2` and
//!   grows by 50 per sector, the level is one more. Each new sector draws
//!   its type when it is built. The joins: 4 to 2 joins old 0 and 1 to new
//!   0, old 2 and 3 to new 1; 2 to 4 joins old `k / 2` to new `k`; else
//!   old `i` leads to the new sector `i` builds, and from old 1 on also to
//!   the sector old `i - 1` built last; growing by one, old 0 builds two,
//!   shrinking by one, the last old sector builds none. After the column,
//!   each new sector in order takes `EventGenerator::GetSectorDescription`
//!   (0x1001a1270) of its type (`CIVILIAN`, `HOSTILE`, `NEBULA`) and level.
//! - `GetSectorDescription`: the sector ids of the `sectorType`, or of
//!   `OVERRIDE_<type>` when the DLC is on and the list exists; it drops
//!   those whose `minSector` exceeds the level and the unique ones already
//!   taken; it picks `r % count` (a unique one is taken from then on), then
//!   a name `r % names` and removes that name from the description when it
//!   has two or more. Every description of 1.6.13 has one name.
//! - `LoadGame` reads one visited flag per sector ("sector count doesn't
//!   match" is logged when the counts differ) and makes the deepest visited
//!   sector current: its level is the score's sector. A sector map of the
//!   hidden Crystal sector is the Crystal node, outside the list.
//! - `StarMap::RenderSectorMap` (0x1001c80d0) draws a line 2 px wide per
//!   join, then a ring of radius 7 (yellow 255,255,50 visited, white
//!   reachable from the current sector, gray 125 else; green 100,255,100
//!   the hovered next sector), a disc of radius 6 in the type color
//!   (`STARMAP_GREEN` 135,199,74 civilian, `STARMAP_RED` 214,50,50
//!   hostile, `STARMAP_PURPLE` 128,51,210 nebula; a quarter of each
//!   channel when neither visited nor reachable), the ship marker at the
//!   current sector and the names of the current and the next sectors.
//!   `ReachSector` marks every sector the joins reach from the current one.
//!
//! The jump ([`jump`]) writes what FTL has after an arrival in a new sector,
//! as `beacons.current` does for a move inside the sector
//! ([`crate::beacons`]): `StarMap::GenerateMap` (0x1001cb870, called with
//! a fresh seed when the ship reaches an exit, `WorldManager::
//! CheckForNewLocation`) draws a new layout seed with `random32`, starts
//! the fleet at -959 with a new fudge, clears the hazards flag (+0x54c),
//! the waiting flag (+0xc48) and the flagship's base turns, puts the ship
//! on the start beacon with one visit and marks it and its neighbors seen;
//! `StarMap::DeleteMap` drops the quests and stores of the old map. The
//! pending quests of the next sector (`distant_quest_events`) would be
//! placed by `AddQuest` with the running RNG, so a game with any is
//! refused. The start beacon gets two visits, so `LoadGame` clears its
//! event as for any return (the editor writes no start event), and the
//! background of the beacon the ship leaves (FTL draws a new one from the
//! running RNG). The situation is an arrival without a ship. The
//! sector's own events, nebulas and stores come back from the layout seed
//! when FTL loads the map; the file holds only what the arrival changed.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::beacons;
use crate::model::{Beacon, FleetPresence, SavedGame};
use crate::sector_map::{self, FLEET_START, LAST_STAND_SECTOR, Random32, fleet_covers};

/// The kind of a sector, as FTL stores it (0, 1, 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SectorKind {
    Civilian,
    Hostile,
    Nebula,
}

impl SectorKind {
    /// The `sectorType` name of `sector_data.xml`.
    pub fn type_name(self) -> &'static str {
        match self {
            SectorKind::Civilian => "CIVILIAN",
            SectorKind::Hostile => "HOSTILE",
            SectorKind::Nebula => "NEBULA",
        }
    }

    /// A stable id for the bridge and the UI.
    pub fn id(self) -> &'static str {
        match self {
            SectorKind::Civilian => "civilian",
            SectorKind::Hostile => "hostile",
            SectorKind::Nebula => "nebula",
        }
    }

    /// `StarMap::GetRandomSectorChoice`.
    fn roll(r: i32) -> SectorKind {
        match r % 10 {
            0 | 1 => SectorKind::Nebula,
            6.. => SectorKind::Hostile,
            _ => SectorKind::Civilian,
        }
    }
}

/// One `<name>` of a `nameList`: a text id (`id`, `short`) or a literal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectorName {
    /// The text id, or the literal name when `is_id` is false.
    pub text: String,
    /// The text id (or literal) of the short name.
    pub short: String,
    pub is_id: bool,
}

/// One `sectorDescription` of `sector_data.xml`, as far as the tree needs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectorDescription {
    pub min_sector: i32,
    pub unique: bool,
    pub names: Vec<SectorName>,
}

/// The sector lists of `ftl.dat`: the `sectorType` lists by name (with the
/// `OVERRIDE_` ones) and the descriptions by id.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SectorCatalog {
    pub types: BTreeMap<String, Vec<String>>,
    pub descriptions: BTreeMap<String, SectorDescription>,
}

/// One sector of the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sector {
    pub kind: SectorKind,
    /// The zero-based sector number (the column).
    pub level: i32,
    /// The point FTL draws the sector at on its sector map.
    pub x: i32,
    pub y: i32,
    /// The `sectorDescription` id, for example `ZOLTAN_HOME`.
    pub description: String,
    pub name: SectorName,
    /// The sectors of the next column this one leads to, by index.
    pub next: Vec<usize>,
}

/// The sectors in FTL's order (the order of the file's visited flags).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectorTree {
    pub sectors: Vec<Sector>,
}

/// The sector columns after the first sector (`AddSectorColumn` calls).
const MIDDLE_COLUMNS: i32 = 6;
/// The x step between columns and the y of the first and the last sector.
const COLUMN_STEP: i32 = 66;
const ROW_STEP: i32 = 50;
const CENTER_Y: i32 = 75;

/// Why FTL could not build the tree from this catalog. FTL itself would
/// divide by zero on an empty list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeError {
    /// No sector of this type fits this level.
    NoSector { kind: SectorKind, level: i32 },
}

impl fmt::Display for TreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TreeError::NoSector { kind, level } => write!(
                f,
                "sector_data.xml has no {} sector for sector {}",
                kind.type_name(),
                level + 1
            ),
        }
    }
}

struct Builder<'c> {
    rng: Random32,
    catalog: &'c SectorCatalog,
    dlc: bool,
    taken: BTreeSet<String>,
    /// Names removed from a description with two or more.
    used_names: BTreeMap<String, Vec<usize>>,
    sectors: Vec<Sector>,
}

/// Builds the tree as `StarMap::GenerateSectorMap` does (see the module
/// doc), with FTL's own RNG.
pub fn generate(seed: i32, catalog: &SectorCatalog, dlc: bool) -> Result<SectorTree, TreeError> {
    let mut b = Builder {
        rng: Random32::new(seed),
        catalog,
        dlc,
        taken: BTreeSet::new(),
        used_names: BTreeMap::new(),
        sectors: Vec::new(),
    };
    b.rng.rand(); // the first sector's type, overwritten below
    b.sectors.push(Sector {
        kind: SectorKind::Civilian,
        level: 0,
        x: 0,
        y: CENTER_Y,
        description: "STANDARD_SPACE".to_owned(),
        name: SectorName {
            text: "sectorname_CIVILIAN_SECTOR".to_owned(),
            short: "sectorname_short_CIVILIAN_SECTOR".to_owned(),
            is_id: true,
        },
        next: Vec::new(),
    });
    let mut column = vec![0];
    for _ in 0..MIDDLE_COLUMNS {
        column = b.add_column(&column)?;
    }
    b.rng.rand(); // the Last Stand's type, overwritten with hostile
    let last = b.sectors.len();
    let (description, name) = specific(catalog, "FINAL");
    b.sectors.push(Sector {
        kind: SectorKind::Hostile,
        level: LAST_STAND_SECTOR,
        x: b.sectors[column[0]].x + COLUMN_STEP,
        y: CENTER_Y,
        description,
        name,
        next: Vec::new(),
    });
    for &c in &column {
        b.sectors[c].next.push(last);
    }
    Ok(SectorTree { sectors: b.sectors })
}

/// `GetSpecificSector`: the description by id (its first name), else
/// `STANDARD_SPACE`, as FTL falls back.
fn specific(catalog: &SectorCatalog, id: &str) -> (String, SectorName) {
    let (id, d) = match catalog.descriptions.get(id) {
        Some(d) => (id, Some(d)),
        None => ("STANDARD_SPACE", catalog.descriptions.get("STANDARD_SPACE")),
    };
    let name = d
        .and_then(|d| d.names.first().cloned())
        .unwrap_or(SectorName {
            text: String::new(),
            short: String::new(),
            is_id: false,
        });
    (id.to_owned(), name)
}

impl Builder<'_> {
    fn new_sector(&mut self, level: i32, x: i32, y: i32) -> usize {
        let kind = SectorKind::roll(self.rng.rand());
        self.sectors.push(Sector {
            kind,
            level,
            x,
            y,
            description: String::new(),
            name: SectorName {
                text: String::new(),
                short: String::new(),
                is_id: false,
            },
            next: Vec::new(),
        });
        self.sectors.len() - 1
    }

    fn add_column(&mut self, prev: &[usize]) -> Result<Vec<usize>, TreeError> {
        let prev_n = prev.len();
        let mut n = loop {
            let n = (self.rng.rand() % 3 + 2) as usize;
            if n != prev_n {
                break n;
            }
        };
        if prev_n == 1 {
            n = 2;
        }
        let x = self.sectors[prev[0]].x + COLUMN_STEP;
        let level = self.sectors[prev[0]].level + 1;
        let mut y = (200 - ROW_STEP * n as i32) >> 1;
        let mut column = Vec::new();
        if n == 2 && prev_n == 4 {
            for k in 0..2 {
                let s = self.new_sector(level, x, y);
                column.push(s);
                self.sectors[prev[2 * k]].next.push(s);
                self.sectors[prev[2 * k + 1]].next.push(s);
                y += ROW_STEP;
            }
        } else if n == 4 && prev_n == 2 {
            for k in 0..4 {
                let s = self.new_sector(level, x, y);
                column.push(s);
                self.sectors[prev[k >> 1]].next.push(s);
                y += ROW_STEP;
            }
        } else {
            let shrink = n <= prev_n;
            let mut built: Option<usize> = None;
            for (i, &p) in prev.iter().enumerate() {
                if let Some(b) = built {
                    self.sectors[p].next.push(b);
                }
                if shrink && i == prev_n - 1 {
                    break;
                }
                let s = self.new_sector(level, x, y);
                self.sectors[p].next.push(s);
                column.push(s);
                if i == 0 && !shrink {
                    y += ROW_STEP;
                    let s = self.new_sector(level, x, y);
                    self.sectors[p].next.push(s);
                    column.push(s);
                }
                built = self.sectors[p].next.last().copied();
                y += ROW_STEP;
            }
        }
        for &s in &column {
            let (description, name) = self.describe(self.sectors[s].kind, level)?;
            self.sectors[s].description = description;
            self.sectors[s].name = name;
        }
        Ok(column)
    }

    /// `EventGenerator::GetSectorDescription`.
    fn describe(
        &mut self,
        kind: SectorKind,
        level: i32,
    ) -> Result<(String, SectorName), TreeError> {
        let none = Vec::new();
        let types = &self.catalog.types;
        let override_list = format!("OVERRIDE_{}", kind.type_name());
        let list = match types.get(&override_list) {
            Some(l) if self.dlc => l,
            _ => types.get(kind.type_name()).unwrap_or(&none),
        };
        let candidates: Vec<&String> = list
            .iter()
            .filter(|id| {
                let d = self.catalog.descriptions.get(*id);
                let fits = d.is_none_or(|d| level >= d.min_sector);
                fits && !self.taken.contains(*id)
            })
            .collect();
        if candidates.is_empty() {
            return Err(TreeError::NoSector { kind, level });
        }
        let id = candidates[(self.rng.rand() % candidates.len() as i32) as usize].clone();
        let d = self.catalog.descriptions.get(&id);
        if d.is_some_and(|d| d.unique) {
            self.taken.insert(id.clone());
        }
        let r = self.rng.rand();
        let names: Vec<(usize, &SectorName)> = d
            .map(|d| {
                let used = self.used_names.get(&id);
                d.names
                    .iter()
                    .enumerate()
                    .filter(|(n, _)| !used.is_some_and(|u| u.contains(n)))
                    .collect()
            })
            .unwrap_or_default();
        let name = if names.is_empty() {
            SectorName {
                text: String::new(),
                short: String::new(),
                is_id: false,
            }
        } else {
            let (n, name) = names[(r % names.len() as i32) as usize];
            let name = name.clone();
            if names.len() >= 2 {
                self.used_names.entry(id.clone()).or_default().push(n);
            }
            name
        };
        Ok((id, name))
    }
}

/// The tree of a game with its visited sectors and the current one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTree {
    pub tree: SectorTree,
    /// One per sector, from the file.
    pub visited: Vec<bool>,
    /// The deepest visited sector (`LoadGame`); `None` in the hidden
    /// Crystal sector, which is not in the tree.
    pub current: Option<usize>,
    /// The sectors the joins reach from the current one (`ReachSector`).
    pub reachable: Vec<bool>,
}

/// Why a game has no tree. Each has a stable code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoTree {
    /// The world after the player ship is not decoded.
    NotDecoded,
    /// The header's `random_native`: FTL used the RNG of the OS.
    NativeRandom,
    /// No `sector_data.xml` (no `ftl.dat`).
    NoGameData,
    /// The catalog cannot build the tree.
    Catalog(TreeError),
    /// The seed builds another sector count than the file's visited flags.
    CountMismatch { generated: usize, stored: usize },
}

impl NoTree {
    pub fn code(&self) -> &'static str {
        match self {
            NoTree::NotDecoded => "tree_not_decoded",
            NoTree::NativeRandom => "tree_native_random",
            NoTree::NoGameData => "tree_no_game_data",
            NoTree::Catalog(_) => "tree_catalog",
            NoTree::CountMismatch { .. } => "tree_count_mismatch",
        }
    }
}

impl fmt::Display for NoTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NoTree::NotDecoded => write!(f, "the sector map is not decoded"),
            NoTree::NativeRandom => write!(
                f,
                "the game uses the random numbers of its OS, so the sector tree is unknown"
            ),
            NoTree::NoGameData => write!(f, "the sector tree needs sector_data.xml from ftl.dat"),
            NoTree::Catalog(e) => write!(f, "{e}"),
            NoTree::CountMismatch { generated, stored } => write!(
                f,
                "the tree seed builds {generated} sectors, the file holds {stored} visited flags"
            ),
        }
    }
}

/// The sector tree of a game, as `StarMap::LoadGame` rebuilds it.
pub fn tree_of(game: &SavedGame, catalog: Option<&SectorCatalog>) -> Result<GameTree, NoTree> {
    let world = game.world.as_ref().ok_or(NoTree::NotDecoded)?;
    if game.random_native {
        return Err(NoTree::NativeRandom);
    }
    let catalog = catalog.ok_or(NoTree::NoGameData)?;
    let tree =
        generate(world.sector_tree_seed, catalog, game.dlc_enabled).map_err(NoTree::Catalog)?;
    let stored = world.sector_visitation.len();
    if tree.sectors.len() != stored {
        return Err(NoTree::CountMismatch {
            generated: tree.sectors.len(),
            stored,
        });
    }
    let visited = world.sector_visitation.clone();
    let current = (!world.sector_is_hidden_crystal_worlds).then(|| deepest(&tree, &visited));
    let reachable = match current {
        Some(c) => reach(&tree, c),
        None => vec![false; tree.sectors.len()],
    };
    Ok(GameTree {
        tree,
        visited,
        current,
        reachable,
    })
}

/// `LoadGame`: the first visited sector of a deeper level than all before.
fn deepest(tree: &SectorTree, visited: &[bool]) -> usize {
    let mut current = 0;
    for (n, s) in tree.sectors.iter().enumerate() {
        if visited[n] && s.level > tree.sectors[current].level {
            current = n;
        }
    }
    current
}

fn reach(tree: &SectorTree, from: usize) -> Vec<bool> {
    let mut reached = vec![false; tree.sectors.len()];
    let mut todo = vec![from];
    while let Some(n) = todo.pop() {
        if !std::mem::replace(&mut reached[n], true) {
            todo.extend(&tree.sectors[n].next);
        }
    }
    reached
}

/// The sector index the ship is in, from the visited flags alone: the last
/// visited flag (the flags follow the levels). 0 without flags.
pub fn current_index(game: &SavedGame) -> Option<i32> {
    let world = game.world.as_ref()?;
    Some(
        world
            .sector_visitation
            .iter()
            .rposition(|&v| v)
            .unwrap_or(0) as i32,
    )
}

/// Why the ship cannot jump to another sector. Each has a stable code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JumpBlock {
    /// The editor knows no tree for this game.
    NoTree(NoTree),
    /// The situation after the sector map is not decoded.
    NotDecoded,
    /// The ship is in the hidden Crystal sector, outside the tree.
    HiddenSector,
    /// The Last Stand: `GenerateMap` builds it with the flagship.
    LastStand,
    /// A crew member stands aboard the other ship.
    CrewAboard,
    /// Quests wait for the next sector; FTL places them with its RNG.
    DistantQuests,
}

impl JumpBlock {
    pub fn code(&self) -> &'static str {
        match self {
            JumpBlock::NoTree(t) => t.code(),
            JumpBlock::NotDecoded => "jump_not_decoded",
            JumpBlock::HiddenSector => "jump_hidden_sector",
            JumpBlock::LastStand => "jump_last_stand",
            JumpBlock::CrewAboard => "jump_crew_aboard",
            JumpBlock::DistantQuests => "jump_distant_quests",
        }
    }
}

impl fmt::Display for JumpBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JumpBlock::NoTree(t) => write!(f, "{t}"),
            JumpBlock::NotDecoded => f.write_str(
                "the situation after the sector map is not decoded; it needs the ship layouts from ftl.dat",
            ),
            JumpBlock::HiddenSector => {
                f.write_str("the ship is in the hidden Crystal sector, which is not in the tree")
            }
            JumpBlock::LastStand => f.write_str(
                "FTL builds the Last Stand with the rebel flagship, which the editor does not place",
            ),
            JumpBlock::CrewAboard => f.write_str(
                "a crew member is aboard the other ship; the jump would leave them there",
            ),
            JumpBlock::DistantQuests => f.write_str(
                "quests wait for the next sector; FTL places them on its beacons with the running random numbers",
            ),
        }
    }
}

/// Why the ship cannot leave its sector at all, or `None`.
pub fn jump_block(game: &SavedGame, tree: &GameTree) -> Option<JumpBlock> {
    let world = game.world.as_ref()?;
    if game.situation.is_none() {
        return Some(JumpBlock::NotDecoded);
    }
    let Some(current) = tree.current else {
        return Some(JumpBlock::HiddenSector);
    };
    if tree.tree.sectors[current].level >= LAST_STAND_SECTOR
        || world.sector_number >= LAST_STAND_SECTOR
    {
        return Some(JumpBlock::LastStand);
    }
    if let Some(beacons::MoveBlock::CrewAboard) = beacons::move_block(game) {
        return Some(JumpBlock::CrewAboard);
    }
    (!world.distant_quest_events.is_empty()).then_some(JumpBlock::DistantQuests)
}

/// Why the ship cannot jump to sector `target`, or `None`.
pub fn jump_block_to(game: &SavedGame, tree: &GameTree, target: usize) -> Option<JumpBlock> {
    if let Some(block) = jump_block(game, tree) {
        return Some(block);
    }
    let sector = tree.tree.sectors.get(target)?;
    (sector.level >= LAST_STAND_SECTOR).then_some(JumpBlock::LastStand)
}

/// The visited flags of a route to `target`: from the target back to the
/// first sector, each step takes a sector leading there that the file marks
/// visited, else the first one.
fn route_to(tree: &GameTree, target: usize) -> Vec<bool> {
    let sectors = &tree.tree.sectors;
    let mut flags = vec![false; sectors.len()];
    let mut at = target;
    flags[at] = true;
    while sectors[at].level > 0 {
        let before: Vec<usize> = (0..sectors.len())
            .filter(|&n| sectors[n].next.contains(&at))
            .collect();
        at = before
            .iter()
            .copied()
            .find(|&n| tree.visited[n])
            .unwrap_or(before[0]);
        flags[at] = true;
    }
    flags
}

/// The layout seed of the new sector. FTL draws it from its running RNG;
/// the editor takes the next `random32` after the old seed, so a preview
/// and the save write the same map.
fn next_layout_seed(old: i32) -> i32 {
    let mut seed = old;
    loop {
        seed = Random32::new(seed).rand();
        if sector_map::generate(seed).is_some() {
            return seed;
        }
    }
}

/// Moves the ship to sector `target` of the tree as an arrival (see the
/// module doc) and answers the old sector index. Call after
/// [`jump_block_to`].
pub fn jump(game: &mut SavedGame, tree: &GameTree, target: usize) -> i32 {
    let old = tree.current.unwrap_or(0) as i32;
    if Some(target) == tree.current {
        return old;
    }
    let level = tree.tree.sectors[target].level;
    let flags = route_to(tree, target);
    let world = game.world.as_mut().expect("jump_block() checked the world");
    let seed = next_layout_seed(world.sector_layout_seed);
    let layout = sector_map::generate(seed).expect("next_layout_seed() built it");
    let left = usize::try_from(world.current_beacon_id)
        .ok()
        .and_then(|n| world.beacons.get(n))
        .and_then(|b| b.background.clone());
    let start = layout.start;
    let mut seen = vec![false; layout.beacons.len()];
    seen[start] = true;
    for &(a, b) in &layout.joins {
        if a == start {
            seen[b] = true;
        } else if b == start {
            seen[a] = true;
        }
    }
    world.sector_visitation = flags;
    world.sector_number = level;
    world.sector_is_hidden_crystal_worlds = false;
    world.sector_layout_seed = seed;
    world.sector_hazards_visible = false;
    world.rebel_fleet.offset = FLEET_START;
    world.rebel_fleet.fudge = layout.fleet_fudge;
    world.rebel_fleet.flagship_base_turns = 0;
    world.current_beacon_id = start as i32;
    world.waiting = false;
    world.quest_events.clear();
    world.beacons = layout
        .beacons
        .iter()
        .enumerate()
        .map(|(n, &p)| Beacon {
            visit_count: if n == start { 2 } else { 0 },
            background: (n == start).then(|| left.clone().unwrap_or_else(beacons::no_background)),
            seen: seen[n],
            enemy: None,
            fleet_presence: if fleet_covers(p, FLEET_START, layout.fleet_fudge) {
                FleetPresence::Rebel
            } else {
                FleetPresence::None
            },
            under_attack: false,
            store: None,
        })
        .collect();
    game.sector_number = level + 1;
    beacons::arrive_without_ship(game);
    old
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(id: &str) -> SectorName {
        SectorName {
            text: format!("sectorname_{id}"),
            short: format!("sectorname_short_{id}"),
            is_id: true,
        }
    }

    fn description(min_sector: i32, unique: bool, id: &str) -> SectorDescription {
        SectorDescription {
            min_sector,
            unique,
            names: vec![name(id)],
        }
    }

    /// The lists of `sector_data.xml` in FTL 1.6.13 (names and the
    /// attributes the tree reads), so the core tests need no `ftl.dat`.
    pub(crate) fn catalog_1_6_13() -> SectorCatalog {
        let list = |ids: &[&str]| ids.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let mut c = SectorCatalog::default();
        c.types.insert(
            "CIVILIAN".into(),
            list(&[
                "CIVILIAN_SECTOR",
                "ENGI_SECTOR",
                "ENGI_HOME",
                "ZOLTAN_SECTOR",
                "ZOLTAN_HOME",
            ]),
        );
        let hostile = [
            "REBEL_SECTOR",
            "REBEL_SECTOR_MINIBOSS",
            "PIRATE_SECTOR",
            "MANTIS_SECTOR",
            "MANTIS_HOME",
            "ROCK_SECTOR",
            "ROCK_HOME",
        ];
        c.types.insert("HOSTILE".into(), list(&hostile));
        let mut over = list(&hostile);
        over.push("LANIUS_SECTOR".into());
        c.types.insert("OVERRIDE_HOSTILE".into(), over);
        c.types.insert(
            "NEBULA".into(),
            list(&["NEBULA_SECTOR", "SLUG_HOME", "SLUG_SECTOR"]),
        );
        c.types.insert("UNKNOWN".into(), list(&["STANDARD_SPACE"]));
        for (id, min, unique) in [
            ("STANDARD_SPACE", 0, false),
            ("FINAL", 7, true),
            ("CIVILIAN_SECTOR", 0, false),
            ("ENGI_SECTOR", 0, false),
            ("ENGI_HOME", 2, true),
            ("PIRATE_SECTOR", 0, false),
            ("REBEL_SECTOR_MINIBOSS", 4, true),
            ("REBEL_SECTOR", 0, false),
            ("MANTIS_SECTOR", 0, false),
            ("MANTIS_HOME", 2, true),
            ("NEBULA_SECTOR", 0, false),
            ("SLUG_HOME", 3, true),
            ("SLUG_SECTOR", 3, false),
            ("ZOLTAN_SECTOR", 1, false),
            ("ZOLTAN_HOME", 2, true),
            ("ROCK_SECTOR", 1, false),
            ("ROCK_HOME", 4, true),
            ("CRYSTAL_HOME", 0, true),
            ("LANIUS_SECTOR", 1, false),
        ] {
            c.descriptions
                .insert(id.into(), description(min, unique, id));
        }
        c
    }

    fn columns(tree: &SectorTree) -> String {
        let mut out = String::new();
        let mut level = -1;
        for s in &tree.sectors {
            if s.level != level {
                out.push_str(" |");
                level = s.level;
            }
            let k = match s.kind {
                SectorKind::Civilian => 'C',
                SectorKind::Hostile => 'H',
                SectorKind::Nebula => 'N',
            };
            out.push_str(&format!(" {k}/{}", s.description));
        }
        out
    }

    /// The Java oracle (`RandomSectorTreeGenerator` with `FTL_1_6_Random`
    /// over the installed 1.6.13 `ftl.dat`, DLC on, printed by
    /// `TreeOracle` on 2026-11-19). It derives from FTL 1.5.13; types and
    /// sector ids agree with the 1.6.13 port on every seed tried. The first
    /// sector is `STANDARD_SPACE` in FTL, `CIVILIAN_SECTOR` in Java.
    #[test]
    fn the_tree_matches_the_java_oracle() {
        for (seed, java) in [
            (
                571_972_150,
                " | C/CIVILIAN_SECTOR | C/CIVILIAN_SECTOR N/NEBULA_SECTOR | H/MANTIS_SECTOR H/MANTIS_HOME N/NEBULA_SECTOR N/NEBULA_SECTOR | H/ROCK_SECTOR C/CIVILIAN_SECTOR | C/CIVILIAN_SECTOR N/SLUG_SECTOR H/ROCK_SECTOR | H/REBEL_SECTOR_MINIBOSS C/ZOLTAN_SECTOR | C/CIVILIAN_SECTOR N/SLUG_HOME N/NEBULA_SECTOR C/ZOLTAN_HOME | H/FINAL",
            ),
            (
                519_279_047,
                " | C/CIVILIAN_SECTOR | C/ZOLTAN_SECTOR H/PIRATE_SECTOR | H/ROCK_SECTOR H/PIRATE_SECTOR C/ZOLTAN_HOME | C/ENGI_HOME H/MANTIS_HOME C/ENGI_SECTOR H/LANIUS_SECTOR | N/NEBULA_SECTOR C/ZOLTAN_SECTOR | H/MANTIS_SECTOR N/NEBULA_SECTOR H/MANTIS_SECTOR | C/CIVILIAN_SECTOR C/ZOLTAN_SECTOR | H/FINAL",
            ),
            (
                1,
                " | C/CIVILIAN_SECTOR | C/CIVILIAN_SECTOR H/REBEL_SECTOR | H/REBEL_SECTOR H/REBEL_SECTOR H/MANTIS_HOME N/NEBULA_SECTOR | H/LANIUS_SECTOR H/LANIUS_SECTOR | C/CIVILIAN_SECTOR C/ENGI_SECTOR H/MANTIS_SECTOR | H/PIRATE_SECTOR C/ZOLTAN_HOME N/SLUG_HOME C/ENGI_SECTOR | H/PIRATE_SECTOR C/ZOLTAN_SECTOR H/LANIUS_SECTOR | H/FINAL",
            ),
            (
                42,
                " | C/CIVILIAN_SECTOR | H/LANIUS_SECTOR C/CIVILIAN_SECTOR | H/REBEL_SECTOR N/NEBULA_SECTOR H/MANTIS_HOME | C/ZOLTAN_SECTOR H/ROCK_SECTOR H/PIRATE_SECTOR C/ZOLTAN_HOME | H/REBEL_SECTOR_MINIBOSS C/ZOLTAN_SECTOR | C/CIVILIAN_SECTOR C/ENGI_HOME N/SLUG_HOME H/PIRATE_SECTOR | N/SLUG_SECTOR H/PIRATE_SECTOR H/ROCK_SECTOR | H/FINAL",
            ),
        ] {
            let tree = generate(seed, &catalog_1_6_13(), true).unwrap();
            let java = java.replacen("C/CIVILIAN_SECTOR", "C/STANDARD_SPACE", 1);
            assert_eq!(columns(&tree), java, "seed {seed}");
        }
    }

    /// The first fixture (sector 7, replaced on 2026-11-19, git 487cbe5^)
    /// was written by FTL: tree seed 519279047, 18 visited flags, set at
    /// 0, 2, 5, 9, 11, 14 and 16, world sector 6. The visited sectors are
    /// one per level 0 to 6 and each leads to the next: a route FTL flew.
    #[test]
    fn the_visited_flags_of_an_ftl_save_form_a_route_through_the_tree() {
        let tree = generate(519_279_047, &catalog_1_6_13(), true).unwrap();
        assert_eq!(tree.sectors.len(), 18);
        let route = [0, 2, 5, 9, 11, 14, 16];
        for (level, &n) in route.iter().enumerate() {
            assert_eq!(tree.sectors[n].level, level as i32, "sector {n}");
        }
        for w in route.windows(2) {
            assert!(
                tree.sectors[w[0]].next.contains(&w[1]),
                "{} to {}",
                w[0],
                w[1]
            );
        }
    }

    #[test]
    fn the_columns_alternate_their_count_and_end_in_the_last_stand() {
        for seed in 1..200 {
            let tree = generate(seed, &catalog_1_6_13(), true).unwrap();
            let count = |level| tree.sectors.iter().filter(|s| s.level == level).count();
            assert_eq!(count(0), 1);
            assert_eq!(count(1), 2);
            assert_eq!(count(7), 1);
            for level in 1..6 {
                assert_ne!(count(level), count(level + 1), "seed {seed}");
                assert!((2..=4).contains(&count(level + 1)));
            }
            for (n, s) in tree.sectors.iter().enumerate() {
                if s.level < 7 {
                    assert!(!s.next.is_empty(), "seed {seed} sector {n} leads nowhere");
                }
                for &m in &s.next {
                    assert_eq!(tree.sectors[m].level, s.level + 1);
                }
            }
            for level in 1..=7 {
                for (n, s) in tree.sectors.iter().enumerate() {
                    if s.level == level {
                        assert!(
                            tree.sectors.iter().any(|p| p.next.contains(&n)),
                            "seed {seed} sector {n} has no way in"
                        );
                    }
                }
            }
            let uniques: Vec<&str> = tree
                .sectors
                .iter()
                .map(|s| s.description.as_str())
                .filter(|d| d.ends_with("_HOME") || d.ends_with("MINIBOSS"))
                .collect();
            let set: BTreeSet<&&str> = uniques.iter().collect();
            assert_eq!(
                set.len(),
                uniques.len(),
                "seed {seed}: a unique sector twice"
            );
        }
    }

    #[test]
    fn without_the_dlc_no_lanius_sector_appears() {
        for seed in 1..200 {
            let tree = generate(seed, &catalog_1_6_13(), false).unwrap();
            assert!(
                tree.sectors
                    .iter()
                    .all(|s| s.description != "LANIUS_SECTOR")
            );
        }
    }
}
