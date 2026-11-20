//! Wire structs to domain model and back.
//!
//! The wire layer asserts every flag range while it reads, so the mapping
//! from wire to domain cannot fail.

use ftl_core::Difficulty;
use ftl_core::profile::{
    AchievementRecord, CrewRecord, CrewRecords, NewbieTipLevel, Profile, ProfileStats, Score,
    ShipAvailability,
};

use super::wire::{
    SHIP_SLOTS, WireAchievement, WireCrewRecord, WireProfile, WireScore, WireScoreList,
    WireShipUnlock,
};
use crate::wire::{Bool32, FtlString};

/// The ship id of each slot, as ProfileParser.java names them.
pub fn ship_slot_ids(format: i32) -> [&'static str; SHIP_SLOTS] {
    let [a, b, c] = if format >= 9 {
        ["PLAYER_SHIP_ANAEROBIC", "UNKNOWN_BETA", "UNKNOWN_GAMMA"]
    } else {
        ["UNKNOWN_ALPHA", "UNKNOWN_BETA", "UNKNOWN_GAMMA"]
    };
    [
        "PLAYER_SHIP_HARD",
        "PLAYER_SHIP_STEALTH",
        "PLAYER_SHIP_MANTIS",
        "PLAYER_SHIP_CIRCLE",
        "PLAYER_SHIP_FED",
        "PLAYER_SHIP_JELLY",
        "PLAYER_SHIP_ROCK",
        "PLAYER_SHIP_ENERGY",
        "PLAYER_SHIP_CRYSTAL",
        a,
        b,
        c,
    ]
}

fn difficulty(flag: i32) -> Difficulty {
    match flag {
        0 => Difficulty::Easy,
        1 => Difficulty::Normal,
        _ => Difficulty::Hard,
    }
}

fn flag(d: Difficulty) -> i32 {
    match d {
        Difficulty::Easy => 0,
        Difficulty::Normal => 1,
        Difficulty::Hard => 2,
    }
}

/// Scores in format 4 store 0 for Normal and 1 for Easy.
fn score_difficulty(format: i32, f: i32) -> Difficulty {
    match (format, f) {
        (4, 0) => Difficulty::Normal,
        (4, _) => Difficulty::Easy,
        _ => difficulty(f),
    }
}

fn score_flag(format: i32, d: Difficulty) -> i32 {
    match (format, d) {
        (4, Difficulty::Normal) => 0,
        (4, _) => 1,
        _ => flag(d),
    }
}

fn crew(c: WireCrewRecord) -> CrewRecord {
    CrewRecord {
        value: c.value,
        name: c.name.0,
        race: c.race.0,
        male: c.male.0,
    }
}

fn wire_crew(c: &CrewRecord) -> WireCrewRecord {
    WireCrewRecord {
        value: c.value,
        name: FtlString(c.name.clone()),
        race: FtlString(c.race.clone()),
        male: Bool32(c.male),
    }
}

fn scores(format: i32, list: WireScoreList) -> Vec<Score> {
    list.scores
        .into_iter()
        .map(|s| Score {
            ship_name: s.ship_name.0,
            ship_id: s.ship_id.0,
            value: s.value,
            sector: s.sector,
            victory: s.victory.0,
            difficulty: score_difficulty(format, s.difficulty),
            dlc_enabled: s.dlc_enabled.is_some_and(|b| b.0),
        })
        .collect()
}

fn wire_scores(format: i32, list: &[Score]) -> WireScoreList {
    WireScoreList {
        scores: list
            .iter()
            .map(|s| WireScore {
                ship_name: FtlString(s.ship_name.clone()),
                ship_id: FtlString(s.ship_id.clone()),
                value: s.value,
                sector: s.sector,
                victory: Bool32(s.victory),
                difficulty: score_flag(format, s.difficulty),
                dlc_enabled: (format >= 9).then_some(Bool32(s.dlc_enabled)),
            })
            .collect(),
    }
}

pub fn to_domain(w: WireProfile) -> Profile {
    let f = w.format;
    Profile {
        file_format: f,
        newbie_tip_level: w.newbie_tip_level.map(|v| match v {
            0 => NewbieTipLevel::ShipsUnlocked,
            1 => NewbieTipLevel::ShipListIntro,
            _ => NewbieTipLevel::Veteran,
        }),
        achievements: w
            .achievements
            .into_iter()
            .map(|a| AchievementRecord {
                id: a.id.0,
                difficulty: difficulty(a.difficulty),
                layouts: a
                    .layouts
                    .map(|l| l.map(|v| (v >= 0).then(|| difficulty(v)))),
            })
            .collect(),
        ships: ship_slot_ids(f)
            .into_iter()
            .zip(w.ships)
            .map(|(id, s)| ShipAvailability {
                ship_id: id.to_owned(),
                unlocked_a: s.unlocked_a.0,
                unlocked_c: s.unlocked_c.is_some_and(|b| b.0),
            })
            .collect(),
        top_scores: scores(f, w.top_scores),
        ship_best: scores(f, w.ship_best),
        stats: ProfileStats {
            most_ships_defeated: w.most_ships_defeated,
            total_ships_defeated: w.total_ships_defeated,
            most_beacons_explored: w.most_beacons_explored,
            total_beacons_explored: w.total_beacons_explored,
            most_scrap_collected: w.most_scrap_collected,
            total_scrap_collected: w.total_scrap_collected,
            most_crew_hired: w.most_crew_hired,
            total_crew_hired: w.total_crew_hired,
            total_games_played: w.total_games_played,
            total_victories: w.total_victories,
        },
        crew_records: CrewRecords {
            most_repairs: crew(w.most_repairs),
            most_combat_kills: crew(w.most_combat_kills),
            most_piloted_evasions: crew(w.most_piloted_evasions),
            most_jumps_survived: crew(w.most_jumps_survived),
            most_skill_masteries: crew(w.most_skill_masteries),
        },
        undecoded: w.rest,
    }
}

/// Writes the slots by id in file order, as ProfileParser.java does: a slot
/// the profile does not list is written locked.
pub fn to_wire(p: &Profile) -> WireProfile {
    let f = p.file_format;
    let s = &p.stats;
    let c = &p.crew_records;
    WireProfile {
        format: f,
        newbie_tip_level: (f >= 9).then_some(match p.newbie_tip_level {
            Some(NewbieTipLevel::ShipsUnlocked) | None => 0,
            Some(NewbieTipLevel::ShipListIntro) => 1,
            Some(NewbieTipLevel::Veteran) => 2,
        }),
        achievements: p
            .achievements
            .iter()
            .map(|a| WireAchievement {
                id: FtlString(a.id.clone()),
                difficulty: flag(a.difficulty),
                layouts: a.layouts.map(|l| l.map(|d| d.map_or(-1, flag))),
            })
            .collect(),
        ships: ship_slot_ids(f)
            .into_iter()
            .map(|id| {
                let slot = p.ships.iter().find(|s| s.ship_id == id);
                WireShipUnlock {
                    unlocked_a: Bool32(slot.is_some_and(|s| s.unlocked_a)),
                    unlocked_c: (f >= 9).then_some(Bool32(slot.is_some_and(|s| s.unlocked_c))),
                }
            })
            .collect(),
        top_scores: wire_scores(f, &p.top_scores),
        ship_best: wire_scores(f, &p.ship_best),
        most_ships_defeated: s.most_ships_defeated,
        total_ships_defeated: s.total_ships_defeated,
        most_beacons_explored: s.most_beacons_explored,
        total_beacons_explored: s.total_beacons_explored,
        most_scrap_collected: s.most_scrap_collected,
        total_scrap_collected: s.total_scrap_collected,
        most_crew_hired: s.most_crew_hired,
        total_crew_hired: s.total_crew_hired,
        total_games_played: s.total_games_played,
        total_victories: s.total_victories,
        most_repairs: wire_crew(&c.most_repairs),
        most_combat_kills: wire_crew(&c.most_combat_kills),
        most_piloted_evasions: wire_crew(&c.most_piloted_evasions),
        most_jumps_survived: wire_crew(&c.most_jumps_survived),
        most_skill_masteries: wire_crew(&c.most_skill_masteries),
        rest: p.undecoded.clone(),
    }
}
