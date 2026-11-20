//! A crew member aboard a ship (`readCrewMember` in SavedGameParser.java).
//!
//! Fields that exist only in some file formats hold their default (false,
//! 0, empty) when the file does not store them.

use serde::{Deserialize, Serialize};

/// Race id of the Crystal crew; only they store lockdown fields.
pub const CRYSTAL_RACE: &str = "crystal";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrewMember {
    pub name: String,
    /// Race id as FTL writes it: `human`, `engi`, `mantis`, `slug`, `rock`,
    /// `energy` (Zoltan), `crystal`, `anaerobic` (Lanius), `ghost` (the
    /// ghost crew of an event; it has no crew blueprint and wears the human
    /// sprite) and `battle` (a boarding drone).
    pub race: String,
    /// An enemy boarding drone, stored in the crew list.
    pub enemy_boarding_drone: bool,
    pub health: i32,
    /// Sprite position in pixels on the ship floorplan.
    pub sprite_x: i32,
    pub sprite_y: i32,
    pub room_id: i32,
    pub room_square: i32,
    pub player_controlled: bool,
    /// Format 7 and later. Clone progress; Java names it "Clone Ready?".
    pub clone_ready: i32,
    /// Format 7 and later. Java calls it redundant with
    /// [`CrewMember::clonebay_priority`] and writes that value here; it is
    /// kept apart so that a file with two different values round-trips.
    pub death_order: i32,
    /// Format 7 and later. Indices into the tint list of the race sprite.
    pub sprite_tints: Vec<i32>,
    /// Format 7 and later.
    pub mind_controlled: bool,
    /// Format 7 and later. Note the file order: square first, then room.
    pub saved_room_square: i32,
    /// Format 7 and later.
    pub saved_room_id: i32,
    pub skills: Skills,
    pub male: bool,
    pub repairs: i32,
    pub combat_kills: i32,
    pub piloted_evasions: i32,
    pub jumps_survived: i32,
    pub skill_masteries_earned: i32,
    /// Format 7 and later. Decrements to 0.
    pub stun_ticks: i32,
    /// Format 7 and later. Subtracted from health when mind control ends.
    pub health_boost: i32,
    /// Format 7 and later. -1 for a living crew member.
    pub clonebay_priority: i32,
    /// Format 7 and later. Fixed point, 1000 = 1.0.
    pub damage_boost: i32,
    /// Format 7 and later. `CrewMember::fCloneDying` (+0x878,
    /// `CrewMember::SaveState`, FTL 1.6.13 macOS 0x10004fc20), times 1000:
    /// the progress of dying while waiting in the clone queue (Hyperspace
    /// sets it from `CloneSystem::GetDeathProgress`). Java: lambda.
    pub clone_dying: i32,
    /// Format 7 and later. Shared across all crew for the next clone priority.
    pub universal_death_count: i32,
    /// Format 7 and later. `crewAnim->status == 6`: the crew member is
    /// teleporting. Java: nu.
    pub teleporting: bool,
    /// Format 7 and later.
    pub teleport_anim: Anim,
    /// Format 7 and later. `tracker.reverse` of the teleport animation
    /// (`crewAnim->anims[0][6]`): it plays backwards, which Hyperspace reads
    /// as arriving rather than leaving. Java: phi.
    pub teleport_anim_reverse: bool,
    /// Format 7 and later, Crystal crew only: time spent recharging lockdown
    /// (`CrystalAlien::powerCooldown.first`). FTL writes the float with
    /// `FileHelper::writeFloat`, which stores `(int)(f * 1000)` (FTL 1.6.13
    /// macOS, 0x10014f120), so 1000 is one second.
    pub lockdown_recharge_ticks: i32,
    /// Format 7 and later, Crystal crew only: the time the recharge needs
    /// (`powerCooldown.second`), fixed point like
    /// [`CrewMember::lockdown_recharge_ticks`].
    pub lockdown_recharge_ticks_goal: i32,
    /// Format 7 and later, Crystal crew only. `CrewMember::bCloned` (+0x8f0),
    /// set by `CrewMember::Clone`; only `CrystalAlien::SaveState` writes it.
    /// Java: omega.
    pub cloned: i32,
    /// Not in the file: a removal staged by `crew.<n>.remove`, which the
    /// end of a patch carries out ([`crate::crew_list::drop_removed`]).
    #[serde(skip)]
    pub removed: bool,
}

/// A race the editor can give a crew member: the playable races of FTL
/// 1.6.13, in the order of `data/blueprints.xml` and `data/dlcBlueprints.xml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Race {
    pub id: &'static str,
    /// Colors per tint layer of the crew blueprint (`<colorList>`, comments
    /// left out), read from `ftl.dat` of FTL 1.6.13 on 2026-11-19. FTL writes
    /// one tint index per layer ([`CrewMember::sprite_tints`]).
    pub color_layers: &'static [usize],
}

/// The races [`CrewMember::change_race`] accepts. `ghost` and `battle` are
/// left out: neither has a crew blueprint a player can hire.
pub const RACES: [Race; 8] = [
    Race {
        id: "human",
        color_layers: &[1, 4],
    },
    Race {
        id: "engi",
        color_layers: &[],
    },
    Race {
        id: "mantis",
        color_layers: &[5],
    },
    Race {
        id: "slug",
        color_layers: &[6],
    },
    Race {
        id: "rock",
        color_layers: &[4],
    },
    Race {
        id: "energy",
        color_layers: &[5],
    },
    Race {
        id: CRYSTAL_RACE,
        color_layers: &[4],
    },
    Race {
        id: "anaerobic",
        color_layers: &[4],
    },
];

/// The race of [`RACES`] with this id.
pub fn race(id: &str) -> Option<&'static Race> {
    RACES.iter().find(|r| r.id == id)
}

/// The lockdown recharge goal of a fresh player Crystal: 50.0 seconds as
/// `writeFloat` stores it, times 1000 (`CrystalAlien::CrystalAlien` of FTL
/// 1.6.13 stores 50.0 for ship 0; an enemy Crystal gets 10.0 already spent
/// and a random goal).
pub const CRYSTAL_LOCKDOWN_GOAL: i32 = 50_000;

impl CrewMember {
    /// Gives this crew member another race and fits what depends on it.
    ///
    /// What FTL 1.6.13 derives from the race when it loads a save
    /// (`ShipManager::ImportShip`, read 2026-11-19): the class and with it
    /// the full health (`*::GetMaxHealth`), the skill maximum (two mastery
    /// intervals), the tint layers of the blueprint, and for Crystal the
    /// lockdown fields its `LoadState` reads. FTL stores health and skill
    /// points as written, without a check. So this method fits them:
    ///
    /// - health keeps its share of the full health (rounded, at least 1);
    ///   the Java editor resets it to the new full health instead, which
    ///   heals a wounded crew member;
    /// - each skill keeps its mastery level and the progress inside it, cut
    ///   to the new interval: `GetSkillModifier` reads mastery 2 only at
    ///   exactly the maximum and mastery 1 from half of it, so points of the
    ///   old race can show a wrong level;
    /// - the tints get one index per layer of the new race: a layer of the
    ///   same size keeps its index, as FTL: Hyperspace's
    ///   `CrewMember_Extend::TransformColors` does; any other layer takes 0.
    ///   `ImportShip` sets a choice only for an existing layer and never
    ///   checks the index against the colors of that layer;
    /// - a Crystal gets the lockdown fields of a fresh player Crystal, any
    ///   other race zeros (the codec writes them for Crystal only).
    ///
    /// Name, gender (`male`; FTL draws a female sprite for humans only) and
    /// the mastery announcement flags stay. The same race changes nothing.
    pub fn change_race(&mut self, to: &Race) {
        if self.race == to.id {
            return;
        }
        let from = self.race.clone();
        let (old_max, new_max) = (max_health(&from), max_health(to.id));
        let scaled = (i64::from(self.health) * i64::from(new_max) + i64::from(old_max) / 2)
            / i64::from(old_max);
        self.health = (scaled as i32).clamp(1, new_max);
        for kind in SkillKind::ALL {
            let skill = self.skills.get_mut(kind);
            let level = kind.mastery_level(&from, skill.points);
            let new_interval = kind.interval(to.id);
            skill.points = if level == 2 {
                2 * new_interval
            } else {
                let progress = skill.points.max(0) - kind.mastery_points(&from, level);
                kind.mastery_points(to.id, level) + progress.min(new_interval - 1)
            };
        }
        let old_layers = race(&from).map_or(&[][..], |r| r.color_layers);
        self.sprite_tints = to
            .color_layers
            .iter()
            .enumerate()
            .map(
                |(i, &size)| match (old_layers.get(i), self.sprite_tints.get(i)) {
                    (Some(&old), Some(&index))
                        if old == size && (0..size as i32).contains(&index) =>
                    {
                        index
                    }
                    _ => 0,
                },
            )
            .collect();
        let crystal = to.id == CRYSTAL_RACE;
        self.lockdown_recharge_ticks = 0;
        self.lockdown_recharge_ticks_goal = if crystal { CRYSTAL_LOCKDOWN_GOAL } else { 0 };
        self.cloned = 0;
        self.race = to.id.to_owned();
    }
}

/// The six skills of a crew member.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Skills {
    pub pilot: Skill,
    pub engine: Skill,
    pub shield: Skill,
    pub weapon: Skill,
    pub repair: Skill,
    pub combat: Skill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Skill {
    /// Experience points. Each mastery level takes [`SkillKind::interval`]
    /// points; the skill is full at twice that.
    pub points: i32,
    /// Format 8 and later: whether FTL has announced the first mastery.
    pub mastery_one: bool,
    /// Format 8 and later: whether FTL has announced the second mastery.
    pub mastery_two: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillKind {
    Pilot,
    Engine,
    Shield,
    Weapon,
    Repair,
    Combat,
}

impl SkillKind {
    pub const ALL: [SkillKind; 6] = [
        SkillKind::Pilot,
        SkillKind::Engine,
        SkillKind::Shield,
        SkillKind::Weapon,
        SkillKind::Repair,
        SkillKind::Combat,
    ];

    pub fn id(self) -> &'static str {
        match self {
            SkillKind::Pilot => "pilot",
            SkillKind::Engine => "engine",
            SkillKind::Shield => "shield",
            SkillKind::Weapon => "weapon",
            SkillKind::Repair => "repair",
            SkillKind::Combat => "combat",
        }
    }

    pub fn from_id(id: &str) -> Option<SkillKind> {
        SkillKind::ALL.into_iter().find(|k| k.id() == id)
    }

    /// Points per mastery level for a race in FTL 1.5.4 and later.
    ///
    /// Source: `AdvancedFTLConstants.getMasteryInterval*` of the Java editor.
    /// Humans learn faster; every other race uses the same intervals.
    pub fn interval(self, race: &str) -> i32 {
        let human = race == "human";
        match (self, human) {
            (SkillKind::Pilot | SkillKind::Engine, true) => 13,
            (SkillKind::Pilot | SkillKind::Engine, false) => 15,
            (SkillKind::Shield, true) => 50,
            (SkillKind::Shield, false) => 55,
            (SkillKind::Weapon, true) => 58,
            (SkillKind::Weapon, false) => 65,
            (SkillKind::Repair, true) => 16,
            (SkillKind::Repair, false) => 18,
            (SkillKind::Combat, true) => 7,
            (SkillKind::Combat, false) => 8,
        }
    }

    /// The mastery level FTL shows for these points: 0, 1 or 2.
    pub fn mastery_level(self, race: &str, points: i32) -> u8 {
        (points / self.interval(race)).clamp(0, 2) as u8
    }

    /// The points of mastery `level` (0, 1 or 2): `level` intervals.
    pub fn mastery_points(self, race: &str, level: u8) -> i32 {
        i32::from(level.min(2)) * self.interval(race)
    }

    /// The points one mastery step sets: level 0 goes to 1, 1 to 2 and 2
    /// back to 0, each on the exact interval value.
    pub fn next_mastery_points(self, race: &str, points: i32) -> i32 {
        self.mastery_points(race, (self.mastery_level(race, points) + 1) % 3)
    }
}

/// The full health of a crew member of this race.
///
/// FTL keeps it in the game binary, not in `ftl.dat`. Source:
/// `SavedGameParser.CrewType` of the Java editor, except Crystal: Java says
/// 120, the game and every guide show 125. A race this table does not know
/// (a mod) gets the human value.
pub fn max_health(race: &str) -> i32 {
    match race {
        "rock" | "battle" => 150,
        "crystal" => 125,
        "energy" => 70,
        "ghost" => 50,
        _ => 100,
    }
}

impl Skills {
    pub fn get(&self, kind: SkillKind) -> &Skill {
        match kind {
            SkillKind::Pilot => &self.pilot,
            SkillKind::Engine => &self.engine,
            SkillKind::Shield => &self.shield,
            SkillKind::Weapon => &self.weapon,
            SkillKind::Repair => &self.repair,
            SkillKind::Combat => &self.combat,
        }
    }

    pub fn get_mut(&mut self, kind: SkillKind) -> &mut Skill {
        match kind {
            SkillKind::Pilot => &mut self.pilot,
            SkillKind::Engine => &mut self.engine,
            SkillKind::Shield => &mut self.shield,
            SkillKind::Weapon => &mut self.weapon,
            SkillKind::Repair => &mut self.repair,
            SkillKind::Combat => &mut self.combat,
        }
    }
}

/// State of an animation (`readAnim` in SavedGameParser.java), as
/// `Animation::SaveState` of FTL 1.6.13 (macOS 0x10013fb10) writes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Anim {
    /// `tracker.running`.
    pub playing: bool,
    /// `tracker.loop`.
    pub looping: bool,
    /// `currentFrame`.
    pub current_frame: i32,
    /// The share of the animation played (`current_time / time`, at least
    /// 0, 0 when not playing), times 1000.
    pub progress_ticks: i32,
    /// `fScale`, times 1000.
    pub scale: i32,
    /// `position`, times 1000.
    pub x: i32,
    pub y: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_mastery_level_follows_the_interval_of_the_race() {
        let p = SkillKind::Pilot;
        assert_eq!(
            [0, 12, 13, 25, 26].map(|n| p.mastery_level("human", n)),
            [0, 0, 1, 1, 2]
        );
        assert_eq!(
            [14, 15, 29, 30].map(|n| p.mastery_level("engi", n)),
            [0, 1, 1, 2]
        );
        assert_eq!(SkillKind::Weapon.mastery_level("engi", 130), 2);
    }

    #[test]
    fn a_mastery_step_cycles_0_1_2_0_on_the_interval_values() {
        let p = SkillKind::Pilot;
        assert_eq!(
            [0, 5, 13, 20, 26].map(|n| p.next_mastery_points("human", n)),
            [13, 13, 26, 26, 0]
        );
        assert_eq!(SkillKind::Weapon.next_mastery_points("engi", 130), 0);
        assert_eq!(SkillKind::Shield.next_mastery_points("anaerobic", 56), 110);
        assert_eq!(SkillKind::Combat.mastery_points("anaerobic", 2), 16);
    }

    #[test]
    fn the_maximum_health_depends_on_the_race() {
        let races = [
            "human",
            "engi",
            "mantis",
            "slug",
            "rock",
            "energy",
            "crystal",
            "anaerobic",
            "ghost",
            "battle",
        ];
        assert_eq!(
            races.map(max_health),
            [100, 100, 100, 100, 150, 70, 125, 100, 50, 150]
        );
        assert_eq!(max_health("unknown_mod_race"), 100);
    }
}
