//! The profile codec contract on real files: decode, encode, same bytes.
//!
//! Every file under tests/fixtures/profile/ is a profile written by FTL
//! itself. Saved games live one folder up; the two folders stay apart so
//! each round-trip test reads only its own kind of file.

use ftl_core::profile::{Layout, NewbieTipLevel, unlock_all_ships};
use ftl_core::{Difficulty, ProfileCodec};
use ftl_sav::ProfCodec;

const AE_PROF: &[u8] = include_bytes!("fixtures/profile/ae_prof-1.6.13.sav");

#[test]
fn every_profile_fixture_round_trips_byte_identical() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/profile");
    let mut seen = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "sav") {
            continue;
        }
        let bytes = std::fs::read(&path).unwrap();
        let profile = ProfCodec
            .decode(&bytes)
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        assert!(
            ProfCodec.encode(&profile) == bytes,
            "{} does not round-trip",
            path.display()
        );
        seen += 1;
    }
    assert!(seen > 0, "no fixtures in {dir}");
}

/// The values the Java editor's ProfileParser prints for the same file.
#[test]
fn decodes_what_the_java_editor_reads() {
    let p = ProfCodec.decode(AE_PROF).unwrap();
    assert_eq!(p.file_format, 9);
    assert_eq!(p.newbie_tip_level, Some(NewbieTipLevel::Veteran));
    assert_eq!(p.achievements.len(), 10);
    let victory = p.achievements.last().unwrap();
    assert_eq!(victory.id, "PLAYER_SHIP_HARD_VICTORY");
    assert_eq!(victory.layouts, Some([Some(Difficulty::Easy), None, None]));
    assert_eq!(p.achievements[0].layouts, None);
    assert_eq!(p.ships.len(), 12);
    assert_eq!(p.ships[9].ship_id, "PLAYER_SHIP_ANAEROBIC");
    let kestrel = p.ship_unlocks("PLAYER_SHIP_HARD").unwrap();
    assert!(kestrel.layout_a && !kestrel.layout_b && !kestrel.layout_c);
    // ACH_ROBOTIC, ACH_ONLY_DRONES and ACH_IONED are all present.
    assert!(p.ship_unlocks("PLAYER_SHIP_CIRCLE").unwrap().layout_b);
    assert_eq!(p.top_scores.len(), 2);
    let best = &p.top_scores[0];
    assert_eq!(
        (
            best.ship_name.as_str(),
            best.value,
            best.sector,
            best.victory,
            best.difficulty,
            best.dlc_enabled
        ),
        ("Wayfarer", 3575, 8, true, Difficulty::Easy, true)
    );
    assert_eq!(p.ship_best, p.top_scores);
    let s = p.stats;
    assert_eq!(
        (
            s.most_ships_defeated,
            s.total_ships_defeated,
            s.most_scrap_collected,
            s.total_games_played,
            s.total_victories
        ),
        (34, 121, 2015, 5, 1)
    );
    let c = &p.crew_records;
    assert_eq!(
        (
            c.most_piloted_evasions.name.as_str(),
            c.most_piloted_evasions.value
        ),
        ("Pell", 231)
    );
    assert_eq!(c.most_jumps_survived.race, "human");
    assert!(p.undecoded.is_empty());
}

#[test]
fn unlock_all_on_the_fixture_decodes_again() {
    let mut p = ProfCodec.decode(AE_PROF).unwrap();
    let changes = unlock_all_ships(&mut p);
    assert!(changes.iter().any(|c| c.layout == Layout::B));
    let bytes = ProfCodec.encode(&p);
    assert_ne!(bytes, AE_PROF);
    let again = ProfCodec.decode(&bytes).unwrap();
    assert_eq!(again, p);
    assert!(again.ships.iter().take(10).all(|s| s.unlocked_a));
}

#[test]
fn an_unknown_format_names_the_known_ones() {
    let err = ProfCodec.decode(&11i32.to_le_bytes()).unwrap_err();
    assert_eq!(err.offset, 0);
    assert!(err.problem.contains("known: 4, 9"), "{err}");
}

/// No format 4 file written by FTL is at hand; this converts the fixture to
/// format 4 and checks that encode and decode agree on it.
#[test]
fn a_format_4_profile_survives_encode_and_decode() {
    let mut p = ProfCodec.decode(AE_PROF).unwrap();
    p.file_format = 4;
    p.newbie_tip_level = None;
    p.achievements.retain(|a| a.layouts.is_none());
    for (slot, id) in p
        .ships
        .iter_mut()
        .zip(ftl_sav::profile::map::ship_slot_ids(4))
    {
        slot.ship_id = id.into();
        slot.unlocked_c = false;
    }
    for s in p.top_scores.iter_mut().chain(p.ship_best.iter_mut()) {
        s.dlc_enabled = false;
        s.difficulty = Difficulty::Normal;
    }
    let bytes = ProfCodec.encode(&p);
    assert_eq!(&bytes[..4], &4i32.to_le_bytes());
    assert_eq!(ProfCodec.decode(&bytes).unwrap(), p);
}
