//! One-click actions for the common wishes of a player, as pure functions
//! that build a [`Patch`].
//!
//! An action reads the game and the limits of the ship blueprint
//! ([`ShipLimits`], filled by `ftl-app` from `ftl.dat`) and answers the
//! assignments it would make, plus [`ActionNote`]s for what it could not do
//! or had to guess. It only improves the ship (raises resources, levels,
//! skills, health and room oxygen; puts out fires and seals breaches) and lists only
//! fields whose value changes, so running an action twice gives an empty
//! patch. The patch goes
//! through [`Patch::apply`] like every other edit, with the same ranges.

use std::fmt;
use std::str::FromStr;

use crate::edit::{Assignment, Field, MAX_REACTOR, MAX_ROOM_OXYGEN, Patch, Resource};
use crate::model::{SavedGame, SkillKind, SystemKind, max_health};

/// The one-click actions. [`Action::id`] is the name `ftled action` takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    /// Fuel, drone parts, missiles and scrap to the editor caps, the hull to
    /// the ship's maximum, every installed system to its blueprint maximum,
    /// the reactor to [`MAX_REACTOR`].
    MaxEverything,
    /// Every skill of every crew member to the second mastery of its race.
    CrewFullyTrained,
    /// The hull to the ship's maximum, every fire out, every breach sealed,
    /// every room refilled with oxygen (100, as `OxygenSystem::Restart`
    /// fills them; only on a ship with the oxygen system, [`Field::RoomOxygen`]),
    /// every broken bar mended and every ion lock cleared
    /// (`systems.<id>.damage` and `systems.<id>.ion` to 0, [`crate::damage`]).
    RepairShip,
    /// Every crew member to the full health of its race.
    HealCrew,
}

/// The limits of the player ship that the save does not store.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShipLimits {
    /// The hull of the ship blueprint; `None` without game data.
    pub max_hull: Option<i32>,
    /// The highest level of each system on this ship (the blueprint's `max`,
    /// else `maxPower` of the system); empty without game data.
    pub system_levels: Vec<(SystemKind, i32)>,
}

impl ShipLimits {
    fn system_level(&self, kind: SystemKind) -> Option<i32> {
        self.system_levels
            .iter()
            .find(|(k, _)| *k == kind)
            .map(|&(_, level)| level)
    }
}

/// What an action could not do, or where it fell back to an editor cap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionNote {
    /// The ship's maximum hull is unknown (no game data); the hull went to
    /// the editor cap instead.
    HullAtEditorCap { cap: i32 },
    /// At least one system has no blueprint limit (no game data, or the
    /// blueprint does not name it); it went to the cap of unmodded FTL.
    SystemsAtEditorCap,
    /// The rooms are not decoded (no ship layout): fires and breaches stay
    /// as they are.
    RoomsNotDecoded,
}

impl ActionNote {
    /// Stable code for a UI that shows its own text.
    pub fn code(&self) -> &'static str {
        match self {
            ActionNote::HullAtEditorCap { .. } => "hull_at_editor_cap",
            ActionNote::SystemsAtEditorCap => "systems_at_editor_cap",
            ActionNote::RoomsNotDecoded => "rooms_not_decoded",
        }
    }

    /// The number the note is about (the cap); 0 when it has none.
    pub fn count(&self) -> i32 {
        match *self {
            ActionNote::HullAtEditorCap { cap } => cap,
            ActionNote::SystemsAtEditorCap | ActionNote::RoomsNotDecoded => 0,
        }
    }
}

impl fmt::Display for ActionNote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActionNote::HullAtEditorCap { cap } => write!(
                f,
                "the ship's maximum hull is unknown without ftl.dat; hull set to the editor cap {cap}"
            ),
            ActionNote::SystemsAtEditorCap => f.write_str(
                "a system has no blueprint limit; it went to the level cap of unmodded FTL",
            ),
            ActionNote::RoomsNotDecoded => f.write_str(
                "the rooms are not decoded (no ship layout); fires and breaches stay as they are",
            ),
        }
    }
}

/// The patch of an action and its notes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionPlan {
    pub action: Action,
    pub patch: Patch,
    pub notes: Vec<ActionNote>,
}

impl Action {
    pub const ALL: [Action; 4] = [
        Action::MaxEverything,
        Action::CrewFullyTrained,
        Action::RepairShip,
        Action::HealCrew,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Action::MaxEverything => "max-everything",
            Action::CrewFullyTrained => "crew-fully-trained",
            Action::RepairShip => "repair-ship",
            Action::HealCrew => "heal-crew",
        }
    }

    /// The assignments this action makes on `game`, only improving it.
    pub fn plan(self, game: &SavedGame, limits: &ShipLimits) -> ActionPlan {
        let mut plan = Planner {
            game,
            assignments: Vec::new(),
            notes: Vec::new(),
        };
        match self {
            Action::MaxEverything => {
                plan.hull(limits);
                for r in [
                    Resource::Fuel,
                    Resource::DroneParts,
                    Resource::Missiles,
                    Resource::Scrap,
                ] {
                    plan.raise(Field::Resource(r), r.range().1);
                }
                plan.systems(limits);
                plan.raise(Field::Reactor, MAX_REACTOR);
            }
            Action::CrewFullyTrained => plan.crew(|crew, race, plan| {
                for skill in SkillKind::ALL {
                    plan.raise(
                        Field::CrewSkill { crew, skill },
                        skill.mastery_points(race, 2),
                    );
                }
            }),
            Action::RepairShip => {
                plan.hull(limits);
                plan.damage();
                plan.oxygen();
            }
            Action::HealCrew => plan.crew(|crew, race, plan| {
                plan.raise(Field::CrewHealth { crew }, max_health(race));
            }),
        }
        ActionPlan {
            action: self,
            patch: Patch::new(plan.assignments),
            notes: plan.notes,
        }
    }
}

struct Planner<'g> {
    game: &'g SavedGame,
    assignments: Vec<Assignment>,
    notes: Vec<ActionNote>,
}

impl Planner<'_> {
    /// Assigns `target` when the field exists and holds less.
    fn raise(&mut self, field: Field, target: i32) {
        if field.get(self.game).is_ok_and(|v| v < target) {
            self.assignments.push(Assignment::new(field, target));
        }
    }

    /// Assigns 0 when the field holds more (a fire, a breach).
    fn clear(&mut self, field: Field) {
        if field.get(self.game).is_ok_and(|v| v > 0) {
            self.assignments.push(Assignment::new(field, 0));
        }
    }

    fn hull(&mut self, limits: &ShipLimits) {
        let cap = Resource::Hull.range().1;
        let target = match limits.max_hull {
            Some(max) => max.min(cap),
            None => {
                self.notes.push(ActionNote::HullAtEditorCap { cap });
                cap
            }
        };
        self.raise(Field::HULL, target);
    }

    fn systems(&mut self, limits: &ShipLimits) {
        let mut at_editor_cap = false;
        for s in &self.game.player_ship.systems {
            if s.installed.is_none() {
                continue;
            }
            let cap = s.kind.max_level();
            let target = match limits.system_level(s.kind) {
                Some(level) => level.min(cap),
                None => {
                    at_editor_cap = true;
                    cap
                }
            };
            self.raise(Field::SystemLevel(s.kind), target);
        }
        if at_editor_cap {
            self.notes.push(ActionNote::SystemsAtEditorCap);
        }
    }

    /// Refills every room; a ship without the oxygen system or without
    /// decoded rooms has no oxygen field, so nothing is assigned.
    fn oxygen(&mut self) {
        let rooms = self
            .game
            .player_ship
            .body
            .as_ref()
            .map_or(0, |b| b.rooms.len());
        for room in 0..rooms {
            self.raise(Field::RoomOxygen { room }, MAX_ROOM_OXYGEN);
        }
    }

    /// Calls `each` for every crew member; enemy boarding drones share the
    /// crew list and are left out.
    fn crew(&mut self, each: impl Fn(usize, &str, &mut Self)) {
        let game = self.game;
        for (i, c) in game.player_ship.crew.iter().enumerate() {
            if !c.enemy_boarding_drone {
                each(i, &c.race, self);
            }
        }
    }

    /// Puts out every fire, seals every breach, mends every broken bar and
    /// clears every ion lock with a count (a lock of -1 belongs to mind
    /// control or hacking and stays).
    fn damage(&mut self) {
        let ship = &self.game.player_ship;
        match &ship.body {
            Some(body) => {
                for room in &body.rooms {
                    for (s, _) in room.squares.iter().enumerate() {
                        let (x, y) = room.grid_square(s);
                        self.clear(Field::Fire { x, y });
                    }
                }
                for b in &body.breaches {
                    self.clear(Field::Breach { x: b.x, y: b.y });
                }
            }
            None => self.notes.push(ActionNote::RoomsNotDecoded),
        }
        for s in &ship.systems {
            if s.installed.is_some() {
                self.clear(Field::SystemDamage(s.kind));
                self.clear(Field::SystemIon(s.kind));
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error(
    "unknown action '{0}'; known actions: max-everything, crew-fully-trained, repair-ship, heal-crew"
)]
pub struct UnknownAction(pub String);

impl FromStr for Action {
    type Err = UnknownAction;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Action::ALL
            .into_iter()
            .find(|a| a.id() == s)
            .ok_or_else(|| UnknownAction(s.to_owned()))
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.id())
    }
}
