// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get appTitle => 'FTL Save Editor';

  @override
  String get open => 'Open';

  @override
  String get save => 'Save';

  @override
  String sector(int n) {
    return 'Sector $n';
  }

  @override
  String get difficultyEasy => 'Easy';

  @override
  String get difficultyNormal => 'Normal';

  @override
  String get difficultyHard => 'Hard';

  @override
  String get resources => 'Resources';

  @override
  String get fieldHull => 'Hull';

  @override
  String get fieldFuel => 'Fuel';

  @override
  String get fieldDroneParts => 'Drone parts';

  @override
  String get fieldMissiles => 'Missiles';

  @override
  String get fieldScrap => 'Scrap';

  @override
  String get notANumber => 'Not a number';

  @override
  String outOfRange(int min, int max) {
    return 'Allowed: $min to $max';
  }

  @override
  String saved(String backup) {
    return 'Saved. Backup: $backup';
  }

  @override
  String get closeFtlHint =>
      'Close FTL before you save: the game writes this file when it quits.';

  @override
  String error(String message) {
    return 'Error: $message';
  }

  @override
  String errorNoSave(String path) {
    return 'No saved game at $path. FTL deletes continue.sav when a run ends. Start or continue a run, then quit FTL to write it.';
  }

  @override
  String errorChangedOnDisk(String path) {
    return '$path changed since it was opened (is FTL running?). Nothing was written. Close FTL, reload and repeat the edit.';
  }

  @override
  String errorDecode(String path, String detail) {
    return 'Cannot read $path. Detail: $detail';
  }

  @override
  String errorVerify(String detail) {
    return 'The edited file would not read back correctly. Nothing was written. Detail: $detail';
  }

  @override
  String errorEdit(String detail) {
    return 'Invalid value. Detail: $detail';
  }

  @override
  String errorIo(String path, String detail) {
    return 'Cannot access $path. Detail: $detail';
  }

  @override
  String errorNoHome(String detail) {
    return 'No home folder found. Detail: $detail';
  }

  @override
  String get ftlSaves => 'FTL saved games';

  @override
  String get allFiles => 'All files';

  @override
  String get chooseFile => 'Choose saved game';

  @override
  String get reloadFile => 'Reload the saved game from disk';

  @override
  String get discardEditsTitle => 'Discard changes?';

  @override
  String get discardEditsBody =>
      'Reloading reads the file again. The changes not yet saved are lost.';

  @override
  String get discardEdits => 'Discard and reload';

  @override
  String get noFileOpen => 'No saved game open';

  @override
  String get advancedEdition => 'Advanced Edition';

  @override
  String setToMax(int max) {
    return 'Set to maximum ($max)';
  }

  @override
  String get galleryTitle => 'UI variants';

  @override
  String get galleryDark => 'Dark theme';

  @override
  String get galleryLanguage => 'Switch language';

  @override
  String get areaShip => 'Ship';

  @override
  String get areaCrew => 'Crew';

  @override
  String get areaEquipment => 'Equipment';

  @override
  String get areaMap => 'Map';

  @override
  String get areaProfile => 'Profile';

  @override
  String get areaExpert => 'Expert';

  @override
  String get expertDump => 'Raw dump';

  @override
  String get expertStateVars => 'State vars';

  @override
  String get expertUnknown => 'Unknown fields';

  @override
  String get statusFtlRunning => 'FTL is running';

  @override
  String get statusFtlClosed => 'FTL is closed';

  @override
  String get statusSteamCloud => 'Steam Cloud on';

  @override
  String get skillPilot => 'Pilot';

  @override
  String get skillEngines => 'Engines';

  @override
  String get skillShields => 'Shields';

  @override
  String get skillWeapons => 'Weapons';

  @override
  String get skillRepair => 'Repair';

  @override
  String get skillCombat => 'Combat';

  @override
  String get raceHuman => 'Human';

  @override
  String get raceEngi => 'Engi';

  @override
  String get raceMantis => 'Mantis';

  @override
  String get raceRock => 'Rock';

  @override
  String get raceZoltan => 'Zoltan';

  @override
  String get raceSlug => 'Slug';

  @override
  String get crewTrainAll => 'Train fully';

  @override
  String get crewHeal => 'Heal';

  @override
  String get raceCrystal => 'Crystal';

  @override
  String get raceLanius => 'Lanius';

  @override
  String get raceGhost => 'Ghost';

  @override
  String get raceDrone => 'Anti-personnel drone';

  @override
  String get fieldCrewHealth => 'Health';

  @override
  String get fieldCrewName => 'Name';

  @override
  String get fieldCrewRace => 'Race';

  @override
  String get crewRename => 'Rename';

  @override
  String get crewNameRule => '1 to 32 characters';

  @override
  String get crewChangeRace => 'Change race';

  @override
  String get crewRaceNote =>
      'A new race keeps the share of health and the mastery level of each skill.';

  @override
  String crewField(String name, String field) {
    return '$name: $field';
  }

  @override
  String crewSkillTooltip(String skill, int points, int max, int level) {
    return '$skill: $points of $max, level $level of 2';
  }

  @override
  String get crewSkillHint =>
      'Click: next mastery level. Long press or right click: exact points.';

  @override
  String get crewNone => 'No crew aboard.';

  @override
  String statusLastBackup(String time) {
    return 'Last backup: $time';
  }

  @override
  String get apply => 'Apply';

  @override
  String get cancel => 'Cancel';

  @override
  String resourceTooltip(String label, int min, int max) {
    return '$label, $min to $max. Click to edit.';
  }

  @override
  String resourceEdited(String label, int inFile) {
    return '$label: changed, $inFile in the file';
  }

  @override
  String get laterMap =>
      'The map comes with the map view: sectors, beacons and stores.';

  @override
  String get statusFtlUnknown => 'FTL: unknown';

  @override
  String get statusSteamCloudOff => 'Steam Cloud off';

  @override
  String get statusSteamCloudUnknown => 'Steam Cloud: unknown';

  @override
  String get statusSteamCloudHint =>
      'Steam Cloud can replace the edited file with its own copy when FTL starts.';

  @override
  String get statusNotChecked => 'Open a file to check this.';

  @override
  String get statusNoBackup => 'No backup in this session';

  @override
  String get ftlDatFound => 'Game data found';

  @override
  String get ftlDatFoundHint =>
      'The editor shows ship, crew and item images from this ftl.dat of your FTL installation. Use it?';

  @override
  String get ftlDatConfirm => 'Use this ftl.dat';

  @override
  String get ftlDatChoose => 'Choose ftl.dat';

  @override
  String get ftlDatNotFound => 'No ftl.dat found';

  @override
  String get ftlDatNotFoundHint =>
      'The editor runs in reduced mode without game images. Choose ftl.dat, its folder or FTL.app.';

  @override
  String get ftlDatFiles => 'FTL game data';

  @override
  String get ftlDatApp => 'FTL app';

  @override
  String get reducedMode => 'Reduced mode: no ftl.dat';

  @override
  String errorFtlDatNotFound(String detail) {
    return 'No ftl.dat found. Detail: $detail';
  }

  @override
  String errorConfig(String path, String detail) {
    return 'Cannot use the editor settings in $path. Detail: $detail';
  }

  @override
  String errorUnknownImageId(String detail) {
    return 'The game data has no such image. Detail: $detail';
  }

  @override
  String get errorFtlDatNotOpen =>
      'No ftl.dat is open. The editor runs in reduced mode.';

  @override
  String errorEntryMissing(String detail) {
    return 'The game data has no such file. Detail: $detail';
  }

  @override
  String errorFtlDatUnreadable(String path, String detail) {
    return 'Cannot read the game data $path. Detail: $detail';
  }

  @override
  String get galleryLook => 'FTL look';

  @override
  String get lookNoFtlDat =>
      'No ftl.dat found: the FTL look reads its fonts and frames from the local game.';

  @override
  String lookUnreadable(String detail) {
    return 'The FTL look cannot be read from ftl.dat: $detail';
  }

  @override
  String get equipReactor => 'Reactor';

  @override
  String get equipSystems => 'Systems';

  @override
  String get equipWeapons => 'Weapons';

  @override
  String get equipDrones => 'Drones';

  @override
  String get equipAugments => 'Augments';

  @override
  String equipSlots(int used, int all) {
    return '$used of $all slots';
  }

  @override
  String equipPower(int power) {
    return 'Power $power';
  }

  @override
  String get equipNotInstalled => 'Not installed';

  @override
  String get equipAddAugment => 'Add augment';

  @override
  String get equipEmptySlot => 'Empty slot';

  @override
  String get equipRemove => 'Remove';

  @override
  String get equipCargo => 'Cargo';

  @override
  String get equipAddWeapon => 'Add weapon';

  @override
  String get equipAddDrone => 'Add drone';

  @override
  String get equipAddCargo => 'Add to cargo';

  @override
  String get equipOn => 'on';

  @override
  String get equipPowerOn => 'Power on';

  @override
  String get equipPowerOff => 'Power off';

  @override
  String get equipToCargo => 'Move to cargo';

  @override
  String get equipToSlot => 'Move to a slot';

  @override
  String get equipCargoUnknown =>
      'The cargo needs the sector map after the ship, which needs the ship layout from ftl.dat.';

  @override
  String get equipItemsLocked =>
      'Weapons and drones need ftl.dat and the states after the nearby ship.';

  @override
  String equipPowered(String name) {
    return '$name: powered';
  }

  @override
  String equipInCargo(String name) {
    return '$name in the cargo';
  }

  @override
  String get equipRowHint =>
      'Click a segment to set the level, right-click it to break or repair it.';

  @override
  String get equipNotDecoded =>
      'Weapons, drones and augments need the ship layout from ftl.dat.';

  @override
  String get equipFilter => 'Filter by name';

  @override
  String get statusReducedMode => 'Reduced mode';

  @override
  String get statusReducedModeHint =>
      'No ftl.dat is open: the editor shows icons instead of game images.';

  @override
  String get statusGameData => 'Game data: ftl.dat';

  @override
  String get statusChooseFile => 'Click to open another saved game.';

  @override
  String get statusChooseGameData => 'Click to choose another ftl.dat.';

  @override
  String statusProcessListUnavailable(String detail) {
    return 'The editor cannot read the process list, so a running FTL is unknown. Quit FTL before you save. Detail: $detail';
  }

  @override
  String get ftlRunningTitle => 'FTL is running';

  @override
  String get ftlRunningBody =>
      'FTL is running and overwrites this file when it quits. Quit FTL first, then save.';

  @override
  String get saveAnyway => 'Save anyway';

  @override
  String get changesTitle => 'Changes to write';

  @override
  String get changesHint =>
      'Check each change before the editor writes the file.';

  @override
  String get undo => 'Undo';

  @override
  String get redo => 'Redo';

  @override
  String get writeFile => 'Write file';

  @override
  String backupsTitle(String file) {
    return 'Backups of $file';
  }

  @override
  String get backupsEmpty => 'No backups of this file yet.';

  @override
  String get backupsShow => 'Show the backups of this file';

  @override
  String get restore => 'Restore';

  @override
  String get restoreTitle => 'Restore this backup?';

  @override
  String restoreBody(String time) {
    return 'The editor writes the backup from $time over the open file. It backs up the current file first. Edits that are not saved are lost.';
  }

  @override
  String restored(String backup) {
    return 'Restored the backup. Backup of the replaced file: $backup';
  }

  @override
  String get close => 'Close';

  @override
  String get profileChoose => 'Choose profile';

  @override
  String get profileNoFile => 'No profile open';

  @override
  String get profileShips => 'Player ships';

  @override
  String profileAchievements(int n) {
    return 'Achievements ($n)';
  }

  @override
  String get profileStats => 'Stats';

  @override
  String get profileTopScores => 'Top scores';

  @override
  String get profileNoScores => 'No scores yet';

  @override
  String get profileUnlockAll => 'Unlock all ships';

  @override
  String profileUnlockAdds(String achievements) {
    return 'Adds on Easy: $achievements';
  }

  @override
  String get profileAllUnlocked =>
      'Every layout of every ship is already unlocked.';

  @override
  String profileLayoutUnlocked(String layout) {
    return 'Type-$layout unlocked';
  }

  @override
  String profileLayoutLocked(String layout) {
    return 'Type-$layout locked';
  }

  @override
  String get profileLayoutNone => 'This ship has no Type-C';

  @override
  String get profileCloseFtl =>
      'Close FTL before you save: the game writes the profile when it quits.';

  @override
  String get profileVictory => 'Victory';

  @override
  String get statBest => 'Best';

  @override
  String get statTotal => 'Total';

  @override
  String get statShipsDefeated => 'Ships defeated';

  @override
  String get statBeaconsExplored => 'Beacons explored';

  @override
  String get statScrapCollected => 'Scrap collected';

  @override
  String get statCrewHired => 'Crew hired';

  @override
  String get statGamesPlayed => 'Games played';

  @override
  String get statVictories => 'Victories';

  @override
  String errorNoProfile(String path) {
    return 'No profile at $path. FTL writes ae_prof.sav when it quits for the first time. Start FTL once and quit it.';
  }

  @override
  String get profileBackups => 'Backups';

  @override
  String get floorTitle => 'Floorplan';

  @override
  String get floorHint =>
      'Click a system symbol for its levels, fire and breaches. Click a square to light or put out a fire, add or seal a breach, set the oxygen of its room, or pick a crew member standing there.';

  @override
  String floorRoom(int room) {
    return 'Room $room';
  }

  @override
  String get floorFire => 'Fire';

  @override
  String get floorPutOut => 'Put out';

  @override
  String get floorBreach => 'Hull breach';

  @override
  String get floorSeal => 'Seal';

  @override
  String get squareLightFire => 'Light fire';

  @override
  String get squarePutOutFire => 'Put out fire';

  @override
  String get squareAddBreach => 'Add breach';

  @override
  String get squareSealBreach => 'Seal breach';

  @override
  String get floorOxygen => 'Oxygen';

  @override
  String floorOxygenValue(int percent) {
    return 'Oxygen $percent%';
  }

  @override
  String get floorOxygenSet => 'Set';

  @override
  String fieldOxygen(String name) {
    return 'Oxygen: $name';
  }

  @override
  String get floorClose => 'Close';

  @override
  String fieldFire(String name) {
    return 'Fire: $name';
  }

  @override
  String fieldBreach(String name) {
    return 'Hull breach: $name';
  }

  @override
  String get actionMaxEverything => 'Max everything';

  @override
  String get actionCrewFullyTrained => 'Train crew fully';

  @override
  String get actionRepairShip => 'Repair ship';

  @override
  String get actionHealCrew => 'Heal crew';

  @override
  String actionNothing(String action) {
    return '$action: nothing to change';
  }

  @override
  String noteHullAtEditorCap(int count) {
    return 'Maximum hull unknown without ftl.dat: hull set to the editor cap $count';
  }

  @override
  String get noteSystemsAtEditorCap =>
      'A system has no limit in the ship blueprint: it went to the cap of unmodded FTL';

  @override
  String get noteRoomsNotDecoded =>
      'Rooms not decoded (no ship layout): fires and breaches unknown';

  @override
  String get paletteHint => 'Search fields, actions and areas';

  @override
  String get paletteNoMatch => 'No match';

  @override
  String get paletteArea => 'Area';

  @override
  String get paletteAction => 'Action';

  @override
  String get paletteResource => 'Resource';

  @override
  String get paletteSkill => 'Crew · Skill';

  @override
  String get paletteHealth => 'Crew · Health';

  @override
  String get paletteSystem => 'System';

  @override
  String get paletteAugment => 'Augment';

  @override
  String get paletteFire => 'Floorplan · Fire';

  @override
  String get paletteBreach => 'Floorplan · Breach';

  @override
  String get paletteOxygen => 'Floorplan · Oxygen';

  @override
  String get paletteBeacon => 'Map · Beacon';

  @override
  String profileUnlockLayout(String ship, String layout) {
    return '$ship: unlock Type-$layout';
  }

  @override
  String profileLockLayout(String ship, String layout) {
    return '$ship: lock Type-$layout';
  }

  @override
  String profileLockRemoves(String achievements) {
    return 'Removes: $achievements';
  }

  @override
  String get profileLayoutBRule =>
      'Type-B has no flag of its own: FTL unlocks it from two of the ship\'s three achievements. Unlocking it adds the missing ones on Easy, locking it removes them until one is left.';

  @override
  String profileLayoutClick(String state) {
    return '$state. Click to change.';
  }

  @override
  String get profileAddAchievement => 'Add achievement';

  @override
  String profileAchievementAdd(String name, String difficulty) {
    return 'Add $name on $difficulty';
  }

  @override
  String profileAchievementRemove(String name) {
    return 'Remove $name';
  }

  @override
  String profileAchievementSet(String name, String difficulty) {
    return '$name on $difficulty';
  }

  @override
  String get profileAchievementRemoveButton => 'Remove';

  @override
  String get profileAchievementRemoved => 'Removed';

  @override
  String get profileAchievementDifficulty => 'Difficulty';

  @override
  String get equipInstall => 'Install';

  @override
  String equipSystemDamage(String name) {
    return '$name: broken bars';
  }

  @override
  String equipSystemIon(String name) {
    return '$name: ion lock';
  }

  @override
  String equipDamaged(int count) {
    return '$count broken';
  }

  @override
  String equipIonTip(int count) {
    return 'Ion lock $count of 5. Click: one more, after 5 none. Right-click: a number.';
  }

  @override
  String equipZoltanShield(String name) {
    return '$name: Zoltan shield';
  }

  @override
  String equipZoltanTip(int count, int max) {
    return 'Zoltan shield $count of $max. Click a bar: that many, the top bar one less. Right-click: a number.';
  }

  @override
  String equipSystemInstalled(String name) {
    return '$name installed';
  }

  @override
  String get equipBlockNoGameData =>
      'Needs ftl.dat: the ship blueprint names the rooms for systems.';

  @override
  String get equipBlockNoRoom => 'The ship has no room for this system.';

  @override
  String get equipBlockNotInFormat =>
      'This save format has no place for this system.';

  @override
  String get equipBlockArtillery =>
      'FTL gives artillery only to the ships that start with it.';

  @override
  String get equipBlockSystemLimit =>
      'The ship holds 8 systems, the most FTL sells. Remove one first.';

  @override
  String get equipBlockSubsystemLimit =>
      'The ship holds 4 subsystems, the most FTL sells. Remove one first.';

  @override
  String get equipBlockStartingSystem =>
      'The ship starts with this system. FTL builds it again when it loads the save.';

  @override
  String get equipBlockSwapBack =>
      'The ship starts with a Medbay or Clone Bay. Install the other one to swap them.';

  @override
  String get equipBlockItemsInSlots =>
      'Weapons or drones are in its slots. Remove them first.';

  @override
  String get equipBlockNotDecoded =>
      'A part of the save this changes is not decoded (ftl.dat).';

  @override
  String get equipBlockNoStation =>
      'On an enemy ship FTL puts the station of this system on a random square.';

  @override
  String get shipPlayer => 'Your ship';

  @override
  String get shipEnemy => 'Enemy ship';

  @override
  String shipEnemyField(String field) {
    return 'Enemy ship: $field';
  }

  @override
  String get shipRename => 'Rename ship';

  @override
  String get fieldShipName => 'Ship name';

  @override
  String get fieldShipType => 'Ship type';

  @override
  String get fieldDifficulty => 'Difficulty';

  @override
  String shipTypeLayout(String name, String layout) {
    return '$name · Type $layout';
  }

  @override
  String get shipChangeType => 'Change ship type';

  @override
  String get shipChangeDifficulty => 'Change difficulty';

  @override
  String get shipTypeNote =>
      'A type fits the save only with the same rooms and doors: FTL reads the stored rooms, doors and crew positions on the floor of the new type. No two hangar ships share them.';

  @override
  String get shipDifficultyNote =>
      'FTL applies a new difficulty from the next load on: rewards, enemy ships, door health, the score.';

  @override
  String get shipSectorFixed =>
      'The sector follows the sector tree. Jump to another sector on the Map page, in the sector tree.';

  @override
  String get typeBlockOtherLayout => 'Other rooms and doors';

  @override
  String typeBlockSystemRoom(String system) {
    return 'The $system system sits in other rooms';
  }

  @override
  String typeBlockStartSystem(String system) {
    return 'Starts with the $system system, which this ship lacks';
  }

  @override
  String get typeBlockWeaponSlots => 'Too few weapon slots';

  @override
  String get typeBlockDroneSlots => 'Too few drone slots';

  @override
  String get typeBlockUnknownShip => 'Not in ftl.dat';

  @override
  String get typeBlockNoGameData => 'Needs ftl.dat';

  @override
  String get typeBlockNotDecoded => 'The rooms of the ship are not decoded';

  @override
  String get typeBlockEnemyShip => 'Only the player ship changes its type';

  @override
  String get difficultyBlockFlagship =>
      'The rebel flagship is on the map: FTL rebuilds its beacons by difficulty, so the stored ones would not match.';

  @override
  String get difficultyBlockNotDecoded =>
      'The sector map is not decoded, so the editor cannot check the difficulty.';

  @override
  String equipReplaces(String name) {
    return 'Replaces the $name';
  }

  @override
  String get expertWarningTitle => 'Raw edits can break the save';

  @override
  String get expertWarning =>
      'These are the fields of the file as the editor decodes them, edited without the rules of the other pages. A value FTL does not expect can crash the game or end the run. Save still checks that the file reads back and writes nothing else.';

  @override
  String get expertSearch => 'Search path or value';

  @override
  String get expertNoFile => 'Open a saved game to see its fields.';

  @override
  String get expertNoMatch => 'No field matches.';

  @override
  String expertShown(int shown, int total) {
    return '$shown of $total fields';
  }

  @override
  String get expertUnknownTag => 'unknown';

  @override
  String get expertUnknownHint =>
      'Nobody knows yet what these fields mean. Change one only to test a guess, and keep the backup.';

  @override
  String expertUndecoded(int count) {
    return 'Undecoded bytes ($count)';
  }

  @override
  String get expertUndecodedHint =>
      'The editor does not decode these bytes. It writes them back unchanged; they cannot be edited.';

  @override
  String get expertAllDecoded =>
      'The editor decodes the whole file: no byte is left undecoded.';

  @override
  String fieldRaw(String path) {
    return 'Raw: $path';
  }

  @override
  String fieldStateVar(String id) {
    return 'State var: $id';
  }

  @override
  String mapBeacon(int n) {
    return 'Beacon $n';
  }

  @override
  String mapBeaconField(int n, String field) {
    return 'Beacon $n: $field';
  }

  @override
  String get mapVisited => 'Visited';

  @override
  String get mapNotVisited => 'Not visited';

  @override
  String get mapUnvisit => 'Mark unvisited';

  @override
  String get mapUnvisitHint => 'Its event comes back when you arrive.';

  @override
  String get mapVisitKept => 'Your ship is here; FTL keeps this visit.';

  @override
  String get mapSeen => 'Revealed';

  @override
  String get mapHidden => 'Hidden';

  @override
  String get mapReveal => 'Reveal';

  @override
  String get mapHide => 'Hide';

  @override
  String get mapShipHere => 'Ship';

  @override
  String get mapStore => 'Store';

  @override
  String get mapQuest => 'Quest';

  @override
  String get mapEnemy => 'Ship event';

  @override
  String get mapOvertaken => 'Overtaken by the rebel fleet';

  @override
  String get mapForSale => 'For sale';

  @override
  String get mapSoldOut => 'Sold out';

  @override
  String get mapRestock => 'Restock';

  @override
  String get mapSellOut => 'Mark sold';

  @override
  String mapStoreItemField(int n, String item) {
    return 'Beacon $n store: $item';
  }

  @override
  String get mapFleet => 'Rebel fleet';

  @override
  String get mapFleetField => 'Rebel fleet position';

  @override
  String mapFleetOvertaken(int count, int total) {
    return '$count of $total beacons overtaken';
  }

  @override
  String get mapFleetBack => 'Back one jump';

  @override
  String get mapFleetStart => 'Sector start';

  @override
  String get mapFleetLastStand =>
      'In the Last Stand the fleet follows the flagship; FTL ignores its circle.';

  @override
  String get mapFleetUnknown =>
      'The beacon positions are unknown, so the fleet cannot be moved.';

  @override
  String get mapLayoutNative =>
      'This game uses the random numbers of its OS; the beacon positions are unknown.';

  @override
  String get mapLayoutMismatch =>
      'The layout seed builds another beacon count than the file holds; the positions are unknown.';

  @override
  String get mapNone => 'This save holds no sector map.';

  @override
  String get mapClose => 'Close';

  @override
  String get crewAdd => 'Add crew member';

  @override
  String get crewAddFull => 'FTL takes no more crew on this ship';

  @override
  String get crewNew => 'New';

  @override
  String get crewRemove => 'Remove from the ship';

  @override
  String get crewKeep => 'Keep aboard';

  @override
  String get crewRemoveLast => 'The last crew member of a side stays';

  @override
  String get crewRemoved => 'Removed';

  @override
  String get crewLook => 'Look';

  @override
  String get crewChangeLook => 'Change look';

  @override
  String get crewLookNote =>
      'The looks of FTL\'s character creator for this race.';

  @override
  String get crewMale => 'Male';

  @override
  String get crewFemale => 'Female';

  @override
  String get fieldCrewSex => 'Sex';

  @override
  String fieldCrewColor(int layer) {
    return 'Color layer $layer';
  }

  @override
  String crewColor(int number) {
    return 'Color $number';
  }

  @override
  String get fieldCrewAdd => 'New crew members';

  @override
  String get fieldCrewRemove => 'Remove';

  @override
  String get crewAboard => 'Aboard';

  @override
  String get mapFleetForward => 'Forward one jump';

  @override
  String get mapShipField => 'Ship position';

  @override
  String get mapMoveShip => 'Move ship here';

  @override
  String get mapMoveHint =>
      'Arrives like a return: no enemy, no event, no hazard.';

  @override
  String get mapMoveBlockLastStand =>
      'In the Last Stand the ship follows the flagship\'s path.';

  @override
  String get mapMoveBlockCrew =>
      'A crew member is aboard the other ship; a jump would leave them there.';

  @override
  String get mapMoveBlockFleet =>
      'The rebel fleet holds this beacon; FTL keeps its fleet event, which needs a ship.';

  @override
  String get mapMoveBlockQuest =>
      'A quest waits here; FTL would play its event again without its ship.';

  @override
  String get mapMoveBlockUnknown =>
      'The ship\'s situation is not decoded; it needs ftl.dat.';

  @override
  String get mapStoreAdd => 'Make a store';

  @override
  String get mapStoreRemove => 'Remove the store';

  @override
  String get mapStoreAddHint =>
      'FTL loads a stored stock at any beacon; a new store sells weapons, drones and augments.';

  @override
  String get mapKindsNote =>
      'Quest, distress, nebula and other events are not in the save: FTL builds them from the event lists of ftl.dat.';

  @override
  String get mapSectorsNote =>
      'Other sectors: open the sector tree to jump there.';

  @override
  String get mapSwapItem => 'Swap item';

  @override
  String get mapNoChoices => 'No other items of this kind without ftl.dat.';

  @override
  String get mapBeacons => 'Beacons';

  @override
  String get sectorTree => 'Sector tree';

  @override
  String get sectorCivilian => 'Civilian';

  @override
  String get sectorHostile => 'Hostile';

  @override
  String get sectorNebula => 'Nebula';

  @override
  String get sectorJumpField => 'Sector jump';

  @override
  String get sectorJumpHere => 'Jump here';

  @override
  String get sectorJumpHint =>
      'Click a sector to jump there: the ship arrives at its start beacon.';

  @override
  String get sectorJumpArrival =>
      'The ship arrives at the start beacon of a new map of this sector, as after a jump: the rebel fleet starts over, the beacons and stores of this sector are new.';

  @override
  String get sectorShipHere => 'Your ship is in this sector.';

  @override
  String get sectorJumpPendingNote =>
      'A sector jump is pending: this is the new sector. Save the jump before you edit its beacons.';

  @override
  String get sectorJumpLastStand =>
      'FTL builds the Last Stand with the rebel flagship, which the editor does not place.';

  @override
  String get sectorJumpQuests =>
      'Quests wait for the next sector; FTL places them on its beacons with its own random numbers.';

  @override
  String get sectorJumpHidden =>
      'The ship is in the hidden Crystal sector, which is not in the tree.';

  @override
  String get sectorTreeNative =>
      'This game uses the random numbers of its OS; the sector tree is unknown.';

  @override
  String get sectorTreeNoGameData =>
      'The sector tree needs the sector lists of ftl.dat.';

  @override
  String get sectorTreeMismatch =>
      'The tree seed builds another sector count than the file holds; the sector tree is unknown.';
}
