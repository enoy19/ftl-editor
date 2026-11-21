//! Where the beacons of a sector stand and where the rebel fleet reaches:
//! the file stores neither the positions nor the circle, FTL builds both
//! from the sector layout seed when it loads the map.
//!
//! Evidence (FTL 1.6.13, macOS binary, read with Ghidra 12.1.4 on
//! 2026-11-19):
//!
//! - `StarMap::LoadGame` reads the seeds, the fleet offset, fudge and
//!   pursuit, then calls `StarMap::GenerateMap(false, true)`, which seeds
//!   the RNG with the stored layout seed and builds the locations; beacon
//!   `n` of the file is location `n` ("location count doesn't match" is
//!   logged when the counts differ). The current beacon id is clamped to
//!   the last location.
//! - The RNG is `srandom32`/`random32` unless `Globals::RNG` (the
//!   `random_native` flag of the header) picks the C library's
//!   `srand`/`rand`: `state = seed as u32`, `state = state *
//!   0x5851f42d4c957f2d + 1`, the result is `state >> 33`. Java's
//!   `FTL_1_6_Random` computes the same, but sign-extends a negative seed;
//!   FTL draws layout seeds from `random32`, so they are never negative.
//! - `GenerateMap` draws the fleet fudge (`r % 250 + 50`), then calls
//!   `StarMap::PopulateGrid` for the cells of a 6 x 4 grid, column by
//!   column. A cell draws `r % 5`; it stays empty only when that is 0 and
//!   the cells so far divided by the skips so far (both counted after the
//!   draw) is 5 or more. A placed cell builds a `Location`, whose
//!   constructor draws the throb ticks (`r % 2001`), then x is `col * 110 +
//!   10 + r % 90` and y is `row * 110 + 10 + r % 90`, at most 415; in the
//!   top row of the last two columns y is at least 30.
//! - `StarMap::ConnectLocations` joins a new location to the locations of
//!   the eight cells around it that lie at most 165 pixels away;
//!   `StarMap::AddConnections` walks the joins from the first location.
//!   `GenerateMap` deletes the map and populates the grid again, with the
//!   RNG running on, until every location is reached. Java's
//!   `RandomSectorMapGenerator` guesses this test as "every beacon has a
//!   neighbor within 165"; the two disagree on some seeds.
//! - The start beacon: after the grid `GenerateMap` draws `r % 4` and
//!   puts the ship (`StarMap+0x70`) on the location of cell (0, that row),
//!   else on location 0; it gets one visit and it and its neighbors are
//!   seen. Then it draws the exit (column `4 + r % 2`, row `r % 4`, again
//!   while the cell is empty) and asks `StarMap::Dijkstra` for the path;
//!   a path of 4 locations or fewer (100 or more) draws a new exit, and
//!   15 failures build the grid again. Outside the Last Stand that never
//!   happens: a join spans one column at most, so the path from column 0
//!   to column 4 or 5 holds at least 5 locations (start and exit
//!   included). The exit's draws come after the start's, so [`generate`]
//!   stops after the start.
//! - The danger zone: `StarMap::SaveGame` writes a beacon's fleet flag as
//!   "within 767 pixels of (offset, fudge)", the stored offset as x and the
//!   fudge as y (`Pointf::RelativeDistance` against 588289.0), outside the
//!   Last Stand, where the location keeps its own flag. `OnRender` draws the
//!   edge of the zone at x `offset + 767`. `GenerateMap` starts a sector at
//!   offset -959; `StarMap::UpdateDangerZone` advances it per jump (64, 128
//!   while the pursuit is raised, 0 while it is lowered, half in a nebula)
//!   and stops it at 60.
//! - `StarMap::LoadGame` turns every beacon with the fleet flag into a fleet
//!   location (`TurnIntoFleetLocation`: its event becomes a fleet event);
//!   a beacon without it keeps the event `GenerateMap` gave it.
//! - The map background: `StarMap::StarMap` loads `map/zone_1.png` to
//!   `map/zone_3.png` (752 x 534); `OnRender` draws zone
//!   `sector_number % 3 + 1` inside the mask of the map window, tinted
//!   with (0.75, 0.75, 0.75, 1) (`GL_SetColorTint`, constants at
//!   0x1002b5810 and 0x1002b78c0).
//! - What the save holds of the other sectors, and what moving the ship
//!   or changing a beacon writes, is in [`crate::beacons`].

use crate::model::SavedGame;

/// The columns and rows of FTL's sector grid.
pub const GRID: (i32, i32) = (6, 4);

/// The size of the area the beacons stand in, in map pixels (Java's
/// `RandomSectorMapGenerator` for FTL 1.5.4 and later).
pub const MAP_SIZE: (i32, i32) = (640, 488);

/// The radius of the rebel fleet's circle, in map pixels (`sqrt(588289)`).
pub const FLEET_RADIUS: i32 = 767;

/// The fleet offset `GenerateMap` gives a new sector.
pub const FLEET_START: i32 = -959;

/// The fleet offset `UpdateDangerZone` stops at.
pub const FLEET_MAX: i32 = 60;

/// The longest join between two beacons, in map pixels.
const JOIN_DISTANCE: i32 = 165;

/// The zero-based sector of the Last Stand, where a beacon's fleet flag
/// does not follow the circle.
pub const LAST_STAND_SECTOR: i32 = 7;

/// FTL's own RNG since 1.6.1 (`srandom32`, `random32`).
#[derive(Debug, Clone)]
pub struct Random32 {
    state: u64,
}

impl Random32 {
    pub fn new(seed: i32) -> Self {
        Random32 {
            state: u64::from(seed as u32),
        }
    }

    pub fn rand(&mut self) -> i32 {
        self.state = self
            .state
            .wrapping_mul(0x5851_f42d_4c95_7f2d)
            .wrapping_add(1);
        (self.state >> 33) as i32
    }
}

/// A beacon's place on the sector map, in map pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapPoint {
    pub x: i32,
    pub y: i32,
}

/// The beacons FTL builds from a layout seed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectorLayout {
    /// One point per beacon, in the order of the file.
    pub beacons: Vec<MapPoint>,
    /// The joins between beacons, as index pairs (lower index first).
    pub joins: Vec<(usize, usize)>,
    /// The fudge `GenerateMap` draws; the file stores the same value.
    pub fleet_fudge: i32,
    /// How often FTL built the grid until every beacon was reached.
    pub attempts: u32,
    /// The beacon where a ship arrives in the sector: the beacon in cell
    /// (0, `r % 4`) of the first draw after the grid, else beacon 0.
    pub start: usize,
}

/// The attempts after which [`generate`] gives up. FTL tries forever; no
/// seed needs more than a few.
const MAX_ATTEMPTS: u32 = 1000;

/// Builds the beacons of a sector as `StarMap::GenerateMap` does with FTL's
/// own RNG, or `None` when no connected map came within [`MAX_ATTEMPTS`].
pub fn generate(layout_seed: i32) -> Option<SectorLayout> {
    let mut rng = Random32::new(layout_seed);
    let fleet_fudge = rng.rand() % 250 + 50;
    for attempt in 1..=MAX_ATTEMPTS {
        let (beacons, cells) = populate(&mut rng);
        let joins = join(&beacons, &cells);
        if connected(beacons.len(), &joins) {
            let row = rng.rand() % 4;
            let start = cells.iter().position(|&c| c == (0, row)).unwrap_or(0);
            return Some(SectorLayout {
                beacons,
                joins,
                fleet_fudge,
                attempts: attempt,
                start,
            });
        }
    }
    None
}

/// One pass of `PopulateGrid` over the grid: the beacons and their cells.
fn populate(rng: &mut Random32) -> (Vec<MapPoint>, Vec<(i32, i32)>) {
    let mut beacons = Vec::new();
    let mut cells = Vec::new();
    let (mut seen, mut skips) = (0, 0);
    for col in 0..GRID.0 {
        for row in 0..GRID.1 {
            let mut place = rng.rand() % 5 != 0;
            if !place {
                skips += 1;
            }
            if skips > 0 && seen / skips < 5 {
                place = true;
            }
            if place {
                let _throb_ticks = rng.rand() % 2001;
                let x = col * 110 + 10 + rng.rand() % 90;
                let mut y = (row * 110 + 10 + rng.rand() % 90).min(415);
                if col > 3 && row == 0 {
                    y = y.max(30);
                }
                beacons.push(MapPoint { x, y });
                cells.push((col, row));
            }
            seen += 1;
        }
    }
    (beacons, cells)
}

/// The joins of `ConnectLocations`: beacons in neighboring cells at most
/// [`JOIN_DISTANCE`] apart.
fn join(beacons: &[MapPoint], cells: &[(i32, i32)]) -> Vec<(usize, usize)> {
    let mut joins = Vec::new();
    for b in 0..beacons.len() {
        for a in 0..b {
            let (ca, cb) = (cells[a], cells[b]);
            let near = (ca.0 - cb.0).abs() <= 1 && (ca.1 - cb.1).abs() <= 1;
            if near && distance_squared(beacons[a], beacons[b]) <= JOIN_DISTANCE * JOIN_DISTANCE {
                joins.push((a, b));
            }
        }
    }
    joins
}

/// Whether the joins reach every beacon from the first (`AddConnections`).
fn connected(count: usize, joins: &[(usize, usize)]) -> bool {
    if count == 0 {
        return true;
    }
    let mut reached = vec![false; count];
    let mut todo = vec![0];
    reached[0] = true;
    while let Some(n) = todo.pop() {
        for &(a, b) in joins {
            let other = if a == n {
                b
            } else if b == n {
                a
            } else {
                continue;
            };
            if !reached[other] {
                reached[other] = true;
                todo.push(other);
            }
        }
    }
    reached.iter().all(|&r| r)
}

fn distance_squared(a: MapPoint, b: MapPoint) -> i32 {
    let (dx, dy) = (a.x - b.x, a.y - b.y);
    dx * dx + dy * dy
}

/// Whether the rebel fleet at `offset` covers `point`, as `SaveGame`
/// decides a beacon's fleet flag.
pub fn fleet_covers(point: MapPoint, offset: i32, fudge: i32) -> bool {
    let center = MapPoint {
        x: offset,
        y: fudge,
    };
    (distance_squared(point, center) as f32) < (FLEET_RADIUS * FLEET_RADIUS) as f32
}

/// The beacon positions of a game's sector, or why there are none: the
/// header picks the C library's RNG (its sequence differs per OS), or the
/// layout has another beacon count than the file (a map FTL did not build
/// from this seed).
pub fn layout_of(game: &SavedGame) -> Result<SectorLayout, NoLayout> {
    let world = game.world.as_ref().ok_or(NoLayout::NotDecoded)?;
    if game.random_native {
        return Err(NoLayout::NativeRandom);
    }
    let layout = generate(world.sector_layout_seed).ok_or(NoLayout::NoConnectedMap)?;
    if layout.beacons.len() != world.beacons.len() {
        return Err(NoLayout::CountMismatch {
            generated: layout.beacons.len(),
            stored: world.beacons.len(),
        });
    }
    Ok(layout)
}

/// Why [`layout_of`] knows no positions. Each has a stable code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoLayout {
    /// The world after the player ship is not decoded.
    NotDecoded,
    /// The header's `random_native` flag: FTL used the C library's RNG.
    NativeRandom,
    /// No connected map within the attempts.
    NoConnectedMap,
    /// The seed builds another beacon count than the file holds.
    CountMismatch { generated: usize, stored: usize },
}

impl NoLayout {
    pub fn code(self) -> &'static str {
        match self {
            NoLayout::NotDecoded => "map_not_decoded",
            NoLayout::NativeRandom => "map_native_random",
            NoLayout::NoConnectedMap => "map_not_connected",
            NoLayout::CountMismatch { .. } => "map_count_mismatch",
        }
    }
}

impl std::fmt::Display for NoLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NoLayout::NotDecoded => write!(f, "the sector map is not decoded"),
            NoLayout::NativeRandom => write!(
                f,
                "the game uses the random numbers of its OS, so the beacon positions are unknown"
            ),
            NoLayout::NoConnectedMap => write!(f, "the layout seed builds no connected map"),
            NoLayout::CountMismatch { generated, stored } => write!(
                f,
                "the layout seed builds {generated} beacons, the file holds {stored}"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Java `net.blerf.ftl.parser.random.FTL_1_6_Random` of the Java editor
    /// (`reference/ftl-profile-editor`): the first three `rand()` after
    /// `srand(seed)`, printed on 2026-11-19 by a throwaway class on the
    /// classpath of `FTLProfileEditor.jar` (`new FTL_1_6_Random()`, `srand`,
    /// three `rand()`); the same three calls reproduce them.
    #[test]
    fn random32_matches_java_ftl_1_6_random() {
        for (seed, java) in [
            (1, [740_882_966, 1_616_430_695, 1_708_849_955]),
            (12_496_847, [1_424_433_015, 798_565_232, 2_043_768_204]),
            (i32::MAX, [654_075_316, 2_019_367_231, 392_158_185]),
        ] {
            let mut r = Random32::new(seed);
            assert_eq!([r.rand(), r.rand(), r.rand()], java, "seed {seed}");
        }
    }

    /// FTL zero-extends the seed (`srandom32`: `MOV EAX, EDI`); Java's
    /// `BigInteger` sign-extends it, so a negative seed differs from the
    /// Java oracle (seed -5: 590552462 658111617 1154334398).
    #[test]
    fn a_negative_seed_is_taken_as_unsigned() {
        let mut ftl = Random32::new(-5);
        let mut unsigned = Random32 {
            state: u64::from(-5i32 as u32),
        };
        assert_eq!(ftl.rand(), unsigned.rand());
        assert_ne!(Random32::new(-5).rand(), 590_552_462);
    }

    fn points(text: &str) -> Vec<MapPoint> {
        text.split_whitespace()
            .map(|p| {
                let (x, y) = p.split_once(',').unwrap();
                MapPoint {
                    x: x.parse().unwrap(),
                    y: y.parse().unwrap(),
                }
            })
            .collect()
    }

    /// The Java oracle (`RandomSectorMapGenerator` with `FTL_1_6_Random`,
    /// format 11, printed by `MapOracle` on 2026-11-19). Over seeds 1 to
    /// 3000 it agrees on every seed whose first grid is connected; the 85
    /// others differ only in the test that rebuilds the grid.
    #[test]
    fn the_layout_matches_the_java_oracle() {
        for (seed, fudge, java) in [
            (
                1,
                266,
                "58,70 88,152 49,231 40,355 196,28 188,144 126,306 177,375 270,32 316,155 \
                 298,399 371,39 397,162 413,307 412,394 492,30 459,134 517,248 518,415 592,96 \
                 623,193 616,232 620,409",
            ),
            (
                42,
                77,
                "24,89 36,139 47,260 37,415 185,29 133,205 196,236 155,355 230,37 280,204 \
                 239,248 318,362 368,54 416,138 374,415 531,35 478,155 488,372 586,149 566,263 \
                 609,345",
            ),
            // The sector of the first fixture (replaced on 2026-11-19):
            // Java rebuilds it for an isolated
            // beacon, FTL for a beacon the joins do not reach; both get
            // the 21 beacons of the file.
            (
                12_496_847,
                65,
                "81,67 23,194 88,297 19,377 194,22 157,129 165,366 246,93 248,175 266,264 \
                 278,386 386,59 413,157 408,266 368,414 508,74 485,199 458,290 565,30 608,278 \
                 577,415",
            ),
        ] {
            let layout = generate(seed).unwrap();
            assert_eq!(layout.fleet_fudge, fudge, "seed {seed}");
            assert_eq!(layout.beacons, points(java), "seed {seed}");
        }
    }

    /// Seed 11: Java keeps the first grid (20 beacons, each with a
    /// neighbor within 165), but its joins leave a beacon unreached, so
    /// FTL's `GenerateMap` builds the grid again.
    #[test]
    fn a_grid_the_joins_do_not_connect_is_built_again_unlike_java() {
        let layout = generate(11).unwrap();
        assert_eq!(layout.attempts, 2);
        assert_eq!(layout.beacons.len(), 21);
        assert_eq!(layout.beacons[0], MapPoint { x: 61, y: 34 });
    }

    #[test]
    fn the_fleet_covers_within_767_pixels_of_offset_and_fudge() {
        let p = MapPoint { x: 500, y: 65 };
        assert!(fleet_covers(p, -266, 65));
        assert!(!fleet_covers(p, -267, 65));
    }

    #[test]
    fn a_disconnected_grid_is_built_again() {
        let joins = [(0, 1)];
        assert!(!connected(3, &joins));
        assert!(connected(3, &[(0, 1), (1, 2)]));
    }
}
