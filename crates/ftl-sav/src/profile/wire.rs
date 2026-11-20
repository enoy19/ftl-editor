//! The bytes of a profile, field by field. All ints are little-endian i32.
//!
//! Format 9 stores UTF-8 strings (FTL 1.6.1+; the Java editor assumes UTF-8
//! for every format 9 file, since 1.5.4 profiles carry no other marker).
//! Format 4 stores one byte per character.

use binrw::{binrw, helpers::until_eof};
use ftl_core::profile::is_victory_achievement;

use crate::wire::{Bool32, FtlString};

pub const KNOWN_FORMATS: [i32; 2] = [4, 9];

/// Ship slots in the file, also for formats with fewer ships.
pub const SHIP_SLOTS: usize = 12;

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireProfile {
    #[br(assert(KNOWN_FORMATS.contains(&format), "unsupported profile format {}; known: 4, 9", format))]
    pub format: i32,
    #[br(if(format == 9), assert(newbie_tip_level.is_none_or(|v| (0..=2).contains(&v)), "unknown newbie tip level {:?}", newbie_tip_level))]
    #[bw(if(*format == 9))]
    pub newbie_tip_level: Option<i32>,
    #[br(temp)]
    #[bw(calc = achievements.len() as u32)]
    achievement_count: u32,
    #[br(count = achievement_count, args { inner: (format,) })]
    #[bw(args(*format))]
    pub achievements: Vec<WireAchievement>,
    #[br(count = SHIP_SLOTS, args { inner: (format,) })]
    #[bw(args(*format))]
    pub ships: Vec<WireShipUnlock>,
    #[br(args(format))]
    #[bw(args(*format))]
    pub top_scores: WireScoreList,
    #[br(args(format))]
    #[bw(args(*format))]
    pub ship_best: WireScoreList,
    pub most_ships_defeated: i32,
    pub total_ships_defeated: i32,
    pub most_beacons_explored: i32,
    pub total_beacons_explored: i32,
    pub most_scrap_collected: i32,
    pub total_scrap_collected: i32,
    pub most_crew_hired: i32,
    pub total_crew_hired: i32,
    pub total_games_played: i32,
    pub total_victories: i32,
    #[br(args(format >= 9))]
    #[bw(args(*format >= 9))]
    pub most_repairs: WireCrewRecord,
    #[br(args(format >= 9))]
    #[bw(args(*format >= 9))]
    pub most_combat_kills: WireCrewRecord,
    #[br(args(format >= 9))]
    #[bw(args(*format >= 9))]
    pub most_piloted_evasions: WireCrewRecord,
    #[br(args(format >= 9))]
    #[bw(args(*format >= 9))]
    pub most_jumps_survived: WireCrewRecord,
    #[br(args(format >= 9))]
    #[bw(args(*format >= 9))]
    pub most_skill_masteries: WireCrewRecord,
    #[br(parse_with = until_eof)]
    pub rest: Vec<u8>,
}

/// Format 4 knows Easy and Normal, format 9 adds Hard.
pub fn max_difficulty(format: i32) -> i32 {
    if format >= 9 { 2 } else { 1 }
}

#[binrw]
#[brw(little, import(format: i32))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireAchievement {
    #[brw(args(format >= 9))]
    pub id: FtlString,
    #[br(assert((0..=max_difficulty(format)).contains(&difficulty), "unknown difficulty flag {} of achievement {}", difficulty, id.0))]
    pub difficulty: i32,
    /// Format 9, victory achievements only: difficulty flag per layout A, B,
    /// C, or -1 for none.
    #[br(if(format == 9 && is_victory_achievement(&id.0)))]
    #[br(assert(layouts.is_none_or(|l| l.iter().all(|v| (-1..=2).contains(v))), "unknown layout difficulty flags {:?} of achievement {}", layouts, id.0))]
    #[bw(if(format == 9 && is_victory_achievement(&id.0)))]
    pub layouts: Option<[i32; 3]>,
}

#[binrw]
#[brw(little, import(format: i32))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireShipUnlock {
    pub unlocked_a: Bool32,
    #[brw(if(format == 9))]
    pub unlocked_c: Option<Bool32>,
}

#[binrw]
#[brw(little, import(format: i32))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireScoreList {
    #[br(temp)]
    #[bw(calc = scores.len() as u32)]
    count: u32,
    #[br(count = count, args { inner: (format,) })]
    #[bw(args(format))]
    pub scores: Vec<WireScore>,
}

#[binrw]
#[brw(little, import(format: i32))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireScore {
    #[brw(args(format >= 9))]
    pub ship_name: FtlString,
    #[brw(args(format >= 9))]
    pub ship_id: FtlString,
    pub value: i32,
    pub sector: i32,
    pub victory: Bool32,
    /// Format 4 swaps 0 and 1: 0 is Normal, 1 is Easy.
    #[br(assert((0..=max_difficulty(format)).contains(&difficulty), "unknown difficulty flag {} of score {}", difficulty, ship_name.0))]
    pub difficulty: i32,
    #[brw(if(format == 9))]
    pub dlc_enabled: Option<Bool32>,
}

#[binrw]
#[brw(little, import(utf8: bool))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireCrewRecord {
    pub value: i32,
    #[brw(args(utf8))]
    pub name: FtlString,
    #[brw(args(utf8))]
    pub race: FtlString,
    pub male: Bool32,
}
