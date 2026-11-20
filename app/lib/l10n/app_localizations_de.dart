// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for German (`de`).
class AppLocalizationsDe extends AppLocalizations {
  AppLocalizationsDe([String locale = 'de']) : super(locale);

  @override
  String get appTitle => 'FTL Save Editor';

  @override
  String get open => 'Öffnen';

  @override
  String get save => 'Speichern';

  @override
  String sector(int n) {
    return 'Sektor $n';
  }

  @override
  String get difficultyEasy => 'Leicht';

  @override
  String get difficultyNormal => 'Normal';

  @override
  String get difficultyHard => 'Schwer';

  @override
  String get resources => 'Ressourcen';

  @override
  String get fieldHull => 'Hülle';

  @override
  String get fieldFuel => 'Treibstoff';

  @override
  String get fieldDroneParts => 'Drohnenteile';

  @override
  String get fieldMissiles => 'Raketen';

  @override
  String get fieldScrap => 'Scrap';

  @override
  String get notANumber => 'Keine Zahl';

  @override
  String outOfRange(int min, int max) {
    return 'Erlaubt: $min bis $max';
  }

  @override
  String saved(String backup) {
    return 'Gespeichert. Backup: $backup';
  }

  @override
  String get closeFtlHint =>
      'Schließe FTL vor dem Speichern: Das Spiel schreibt diese Datei beim Beenden.';

  @override
  String error(String message) {
    return 'Fehler: $message';
  }

  @override
  String errorNoSave(String path) {
    return 'Kein Spielstand unter $path. FTL löscht continue.sav, wenn ein Run endet. Starte oder setze einen Run fort und beende dann FTL, damit die Datei entsteht.';
  }

  @override
  String errorChangedOnDisk(String path) {
    return '$path hat sich seit dem Öffnen geändert (läuft FTL?). Nichts wurde geschrieben. Schließe FTL, lade neu und wiederhole die Änderung.';
  }

  @override
  String errorDecode(String path, String detail) {
    return '$path lässt sich nicht lesen. Detail: $detail';
  }

  @override
  String errorVerify(String detail) {
    return 'Die geänderte Datei ließe sich nicht korrekt zurücklesen. Nichts wurde geschrieben. Detail: $detail';
  }

  @override
  String errorEdit(String detail) {
    return 'Ungültiger Wert. Detail: $detail';
  }

  @override
  String errorIo(String path, String detail) {
    return 'Kein Zugriff auf $path. Detail: $detail';
  }

  @override
  String errorNoHome(String detail) {
    return 'Kein Home-Ordner gefunden. Detail: $detail';
  }

  @override
  String get ftlSaves => 'FTL-Spielstände';

  @override
  String get allFiles => 'Alle Dateien';

  @override
  String get chooseFile => 'Spielstand wählen';

  @override
  String get reloadFile => 'Spielstand neu von der Platte laden';

  @override
  String get discardEditsTitle => 'Änderungen verwerfen?';

  @override
  String get discardEditsBody =>
      'Neu laden liest die Datei noch einmal. Die nicht gespeicherten Änderungen gehen verloren.';

  @override
  String get discardEdits => 'Verwerfen und neu laden';

  @override
  String get noFileOpen => 'Kein Spielstand geöffnet';

  @override
  String get advancedEdition => 'Advanced Edition';

  @override
  String setToMax(int max) {
    return 'Auf Maximum setzen ($max)';
  }

  @override
  String get galleryTitle => 'UI-Varianten';

  @override
  String get galleryDark => 'Dunkles Design';

  @override
  String get galleryLanguage => 'Sprache wechseln';

  @override
  String get areaShip => 'Schiff';

  @override
  String get areaCrew => 'Crew';

  @override
  String get areaEquipment => 'Ausrüstung';

  @override
  String get areaMap => 'Karte';

  @override
  String get areaProfile => 'Profil';

  @override
  String get areaExpert => 'Experte';

  @override
  String get expertDump => 'Rohdaten';

  @override
  String get expertStateVars => 'Zustandsvariablen';

  @override
  String get expertUnknown => 'Unbekannte Felder';

  @override
  String get statusFtlRunning => 'FTL läuft';

  @override
  String get statusFtlClosed => 'FTL ist geschlossen';

  @override
  String get statusSteamCloud => 'Steam Cloud an';

  @override
  String get skillPilot => 'Steuern';

  @override
  String get skillEngines => 'Antrieb';

  @override
  String get skillShields => 'Schilde';

  @override
  String get skillWeapons => 'Waffen';

  @override
  String get skillRepair => 'Reparatur';

  @override
  String get skillCombat => 'Kampf';

  @override
  String get raceHuman => 'Mensch';

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
  String get crewTrainAll => 'Voll trainieren';

  @override
  String get crewHeal => 'Heilen';

  @override
  String get raceCrystal => 'Kristall';

  @override
  String get raceLanius => 'Lanius';

  @override
  String get raceGhost => 'Geist';

  @override
  String get raceDrone => 'Abwehrdrohne';

  @override
  String get fieldCrewHealth => 'Gesundheit';

  @override
  String get fieldCrewName => 'Name';

  @override
  String get fieldCrewRace => 'Rasse';

  @override
  String get crewRename => 'Umbenennen';

  @override
  String get crewNameRule => '1 bis 32 Zeichen';

  @override
  String get crewChangeRace => 'Rasse ändern';

  @override
  String get crewRaceNote =>
      'Eine neue Rasse behält den Anteil der Gesundheit und die Meisterstufe jeder Fähigkeit.';

  @override
  String crewField(String name, String field) {
    return '$name: $field';
  }

  @override
  String crewSkillTooltip(String skill, int points, int max, int level) {
    return '$skill: $points von $max, Stufe $level von 2';
  }

  @override
  String get crewSkillHint =>
      'Klick: nächste Meisterstufe. Langes Drücken oder Rechtsklick: genaue Punkte.';

  @override
  String get crewNone => 'Keine Crew an Bord.';

  @override
  String statusLastBackup(String time) {
    return 'Letztes Backup: $time';
  }

  @override
  String get apply => 'Übernehmen';

  @override
  String get cancel => 'Abbrechen';

  @override
  String resourceTooltip(String label, int min, int max) {
    return '$label, $min bis $max. Zum Bearbeiten klicken.';
  }

  @override
  String resourceEdited(String label, int inFile) {
    return '$label: geändert, in der Datei $inFile';
  }

  @override
  String get laterMap =>
      'Die Karte kommt mit der Kartenansicht: Sektoren, Beacons und Läden.';

  @override
  String get statusFtlUnknown => 'FTL: unbekannt';

  @override
  String get statusSteamCloudOff => 'Steam Cloud aus';

  @override
  String get statusSteamCloudUnknown => 'Steam Cloud: unbekannt';

  @override
  String get statusSteamCloudHint =>
      'Steam Cloud kann die geänderte Datei beim Start von FTL durch die eigene Kopie ersetzen.';

  @override
  String get statusNotChecked => 'Öffne eine Datei, um das zu prüfen.';

  @override
  String get statusNoBackup => 'Kein Backup in dieser Sitzung';

  @override
  String get ftlDatFound => 'Spieldaten gefunden';

  @override
  String get ftlDatFoundHint =>
      'Der Editor zeigt Bilder von Schiffen, Crew und Ausrüstung aus dieser ftl.dat deiner FTL-Installation. Verwenden?';

  @override
  String get ftlDatConfirm => 'Diese ftl.dat verwenden';

  @override
  String get ftlDatChoose => 'ftl.dat auswählen';

  @override
  String get ftlDatNotFound => 'Keine ftl.dat gefunden';

  @override
  String get ftlDatNotFoundHint =>
      'Der Editor läuft im reduzierten Modus ohne Spielbilder. Wähle die ftl.dat, ihren Ordner oder FTL.app.';

  @override
  String get ftlDatFiles => 'FTL-Spieldaten';

  @override
  String get ftlDatApp => 'FTL-App';

  @override
  String get reducedMode => 'Reduzierter Modus: keine ftl.dat';

  @override
  String errorFtlDatNotFound(String detail) {
    return 'Keine ftl.dat gefunden. Detail: $detail';
  }

  @override
  String errorConfig(String path, String detail) {
    return 'Die Editor-Einstellungen in $path sind nicht nutzbar. Detail: $detail';
  }

  @override
  String errorUnknownImageId(String detail) {
    return 'Die Spieldaten haben dieses Bild nicht. Detail: $detail';
  }

  @override
  String get errorFtlDatNotOpen =>
      'Keine ftl.dat ist geöffnet. Der Editor läuft im reduzierten Modus.';

  @override
  String errorEntryMissing(String detail) {
    return 'Die Spieldaten haben diese Datei nicht. Detail: $detail';
  }

  @override
  String errorFtlDatUnreadable(String path, String detail) {
    return 'Die Spieldaten $path sind nicht lesbar. Detail: $detail';
  }

  @override
  String get galleryLook => 'FTL-Look';

  @override
  String get lookNoFtlDat =>
      'Keine ftl.dat gefunden: Der FTL-Look liest Schriften und Rahmen aus dem lokalen Spiel.';

  @override
  String lookUnreadable(String detail) {
    return 'Der FTL-Look lässt sich nicht aus ftl.dat lesen: $detail';
  }

  @override
  String get equipReactor => 'Reaktor';

  @override
  String get equipSystems => 'Systeme';

  @override
  String get equipWeapons => 'Waffen';

  @override
  String get equipDrones => 'Drohnen';

  @override
  String get equipAugments => 'Augments';

  @override
  String equipSlots(int used, int all) {
    return '$used von $all Plätzen';
  }

  @override
  String equipPower(int power) {
    return 'Energie $power';
  }

  @override
  String get equipNotInstalled => 'Nicht eingebaut';

  @override
  String get equipAddAugment => 'Augment hinzufügen';

  @override
  String get equipEmptySlot => 'Freier Platz';

  @override
  String get equipRemove => 'Entfernen';

  @override
  String get equipCargo => 'Frachtraum';

  @override
  String get equipAddWeapon => 'Waffe hinzufügen';

  @override
  String get equipAddDrone => 'Drohne hinzufügen';

  @override
  String get equipAddCargo => 'In den Frachtraum';

  @override
  String get equipOn => 'an';

  @override
  String get equipPowerOn => 'Einschalten';

  @override
  String get equipPowerOff => 'Ausschalten';

  @override
  String get equipToCargo => 'In den Frachtraum legen';

  @override
  String get equipToSlot => 'In einen Platz einbauen';

  @override
  String get equipCargoUnknown =>
      'Der Frachtraum braucht die Sektorkarte nach dem Schiff und damit das Schiffslayout aus ftl.dat.';

  @override
  String get equipItemsLocked =>
      'Waffen und Drohnen brauchen ftl.dat und die Zustände nach dem Schiff in der Nähe.';

  @override
  String equipPowered(String name) {
    return '$name: eingeschaltet';
  }

  @override
  String equipInCargo(String name) {
    return '$name im Frachtraum';
  }

  @override
  String get equipRowHint =>
      'Klick auf ein Segment setzt die Stufe, Rechtsklick beschädigt oder repariert es.';

  @override
  String get equipNotDecoded =>
      'Waffen, Drohnen und Augments brauchen das Schiffslayout aus ftl.dat.';

  @override
  String get equipFilter => 'Nach Namen filtern';

  @override
  String get statusReducedMode => 'Reduzierter Modus';

  @override
  String get statusReducedModeHint =>
      'Keine ftl.dat geöffnet: Der Editor zeigt Symbole statt Spielbildern.';

  @override
  String get statusGameData => 'Spieldaten: ftl.dat';

  @override
  String get statusChooseFile =>
      'Klicken, um einen anderen Spielstand zu öffnen.';

  @override
  String get statusChooseGameData =>
      'Klicken, um eine andere ftl.dat zu wählen.';

  @override
  String statusProcessListUnavailable(String detail) {
    return 'Der Editor kann die Prozessliste nicht lesen. Ob FTL läuft, ist deshalb unbekannt. Beende FTL vor dem Speichern. Detail: $detail';
  }

  @override
  String get ftlRunningTitle => 'FTL läuft';

  @override
  String get ftlRunningBody =>
      'FTL läuft und überschreibt diese Datei beim Beenden. Beende zuerst FTL und speichere dann.';

  @override
  String get saveAnyway => 'Trotzdem speichern';

  @override
  String get changesTitle => 'Diese Änderungen schreiben';

  @override
  String get changesHint =>
      'Prüfe jede Änderung, bevor der Editor die Datei schreibt.';

  @override
  String get undo => 'Rückgängig';

  @override
  String get redo => 'Wiederherstellen';

  @override
  String get writeFile => 'Datei schreiben';

  @override
  String backupsTitle(String file) {
    return 'Backups von $file';
  }

  @override
  String get backupsEmpty => 'Noch keine Backups dieser Datei.';

  @override
  String get backupsShow => 'Backups dieser Datei anzeigen';

  @override
  String get restore => 'Wiederherstellen';

  @override
  String get restoreTitle => 'Dieses Backup wiederherstellen?';

  @override
  String restoreBody(String time) {
    return 'Der Editor schreibt das Backup von $time über die geöffnete Datei. Er sichert die aktuelle Datei vorher. Nicht gespeicherte Änderungen gehen verloren.';
  }

  @override
  String restored(String backup) {
    return 'Backup wiederhergestellt. Backup der ersetzten Datei: $backup';
  }

  @override
  String get close => 'Schließen';

  @override
  String get profileChoose => 'Profil wählen';

  @override
  String get profileNoFile => 'Kein Profil geöffnet';

  @override
  String get profileShips => 'Spielerschiffe';

  @override
  String profileAchievements(int n) {
    return 'Achievements ($n)';
  }

  @override
  String get profileStats => 'Statistik';

  @override
  String get profileTopScores => 'Bestenliste';

  @override
  String get profileNoScores => 'Noch keine Punkte';

  @override
  String get profileUnlockAll => 'Alle Schiffe freischalten';

  @override
  String profileUnlockAdds(String achievements) {
    return 'Ergänzt auf Leicht: $achievements';
  }

  @override
  String get profileAllUnlocked =>
      'Alle Layouts aller Schiffe sind schon freigeschaltet.';

  @override
  String profileLayoutUnlocked(String layout) {
    return 'Typ $layout freigeschaltet';
  }

  @override
  String profileLayoutLocked(String layout) {
    return 'Typ $layout gesperrt';
  }

  @override
  String get profileLayoutNone => 'Dieses Schiff hat keinen Typ C';

  @override
  String get profileCloseFtl =>
      'Schließe FTL vor dem Speichern: Das Spiel schreibt das Profil beim Beenden.';

  @override
  String get profileVictory => 'Sieg';

  @override
  String get statBest => 'Bestwert';

  @override
  String get statTotal => 'Gesamt';

  @override
  String get statShipsDefeated => 'Schiffe besiegt';

  @override
  String get statBeaconsExplored => 'Beacons erkundet';

  @override
  String get statScrapCollected => 'Schrott gesammelt';

  @override
  String get statCrewHired => 'Crew angeheuert';

  @override
  String get statGamesPlayed => 'Spiele gespielt';

  @override
  String get statVictories => 'Siege';

  @override
  String errorNoProfile(String path) {
    return 'Kein Profil unter $path. FTL schreibt ae_prof.sav, wenn es zum ersten Mal beendet wird. Starte FTL einmal und beende es.';
  }

  @override
  String get profileBackups => 'Backups';

  @override
  String get floorTitle => 'Grundriss';

  @override
  String get floorHint =>
      'Ein Klick auf ein Systemsymbol zeigt seine Stufen, Feuer und Hüllenbrüche. Ein Klick auf ein Feld legt oder löscht ein Feuer, setzt oder dichtet einen Hüllenbruch ab, ändert den Sauerstoff des Raums oder wählt ein Crewmitglied dort.';

  @override
  String floorRoom(int room) {
    return 'Raum $room';
  }

  @override
  String get floorFire => 'Feuer';

  @override
  String get floorPutOut => 'Löschen';

  @override
  String get floorBreach => 'Hüllenbruch';

  @override
  String get floorSeal => 'Abdichten';

  @override
  String get squareLightFire => 'Feuer legen';

  @override
  String get squarePutOutFire => 'Feuer löschen';

  @override
  String get squareAddBreach => 'Hüllenbruch setzen';

  @override
  String get squareSealBreach => 'Hüllenbruch abdichten';

  @override
  String get floorOxygen => 'Sauerstoff';

  @override
  String floorOxygenValue(int percent) {
    return 'Sauerstoff $percent %';
  }

  @override
  String get floorOxygenSet => 'Ändern';

  @override
  String fieldOxygen(String name) {
    return 'Sauerstoff: $name';
  }

  @override
  String get floorClose => 'Schließen';

  @override
  String fieldFire(String name) {
    return 'Feuer: $name';
  }

  @override
  String fieldBreach(String name) {
    return 'Hüllenbruch: $name';
  }

  @override
  String get actionMaxEverything => 'Alles auf Maximum';

  @override
  String get actionCrewFullyTrained => 'Crew voll ausbilden';

  @override
  String get actionRepairShip => 'Schiff reparieren';

  @override
  String get actionHealCrew => 'Crew heilen';

  @override
  String actionNothing(String action) {
    return '$action: nichts zu ändern';
  }

  @override
  String noteHullAtEditorCap(int count) {
    return 'Maximale Hülle ohne ftl.dat unbekannt: Hülle auf die Grenze des Editors $count gesetzt';
  }

  @override
  String get noteSystemsAtEditorCap =>
      'Ein System hat keine Grenze im Schiffsbauplan: es ging auf die Grenze des unveränderten FTL';

  @override
  String get noteRoomsNotDecoded =>
      'Räume nicht dekodiert (kein Schiffslayout): Feuer und Brüche unbekannt';

  @override
  String get paletteHint => 'Felder, Aktionen und Bereiche suchen';

  @override
  String get paletteNoMatch => 'Kein Treffer';

  @override
  String get paletteArea => 'Bereich';

  @override
  String get paletteAction => 'Aktion';

  @override
  String get paletteResource => 'Ressource';

  @override
  String get paletteSkill => 'Crew · Fähigkeit';

  @override
  String get paletteHealth => 'Crew · Gesundheit';

  @override
  String get paletteSystem => 'System';

  @override
  String get paletteAugment => 'Erweiterung';

  @override
  String get paletteFire => 'Grundriss · Feuer';

  @override
  String get paletteBreach => 'Grundriss · Hüllenbruch';

  @override
  String get paletteOxygen => 'Grundriss · Sauerstoff';

  @override
  String get paletteBeacon => 'Karte · Leuchtfeuer';

  @override
  String profileUnlockLayout(String ship, String layout) {
    return '$ship: Typ $layout freischalten';
  }

  @override
  String profileLockLayout(String ship, String layout) {
    return '$ship: Typ $layout sperren';
  }

  @override
  String profileLockRemoves(String achievements) {
    return 'Entfernt: $achievements';
  }

  @override
  String get profileLayoutBRule =>
      'Typ B hat kein eigenes Flag: FTL schaltet ihn mit zwei der drei Achievements des Schiffs frei. Freischalten ergänzt die fehlenden auf Leicht, Sperren entfernt sie bis auf eines.';

  @override
  String profileLayoutClick(String state) {
    return '$state. Klicken zum Ändern.';
  }

  @override
  String get profileAddAchievement => 'Achievement hinzufügen';

  @override
  String profileAchievementAdd(String name, String difficulty) {
    return '$name auf $difficulty hinzufügen';
  }

  @override
  String profileAchievementRemove(String name) {
    return '$name entfernen';
  }

  @override
  String profileAchievementSet(String name, String difficulty) {
    return '$name auf $difficulty';
  }

  @override
  String get profileAchievementRemoveButton => 'Entfernen';

  @override
  String get profileAchievementRemoved => 'Entfernt';

  @override
  String get profileAchievementDifficulty => 'Schwierigkeit';

  @override
  String get equipInstall => 'Einbauen';

  @override
  String equipSystemDamage(String name) {
    return '$name: beschädigte Balken';
  }

  @override
  String equipSystemIon(String name) {
    return '$name: Ionensperre';
  }

  @override
  String equipDamaged(int count) {
    return '$count beschädigt';
  }

  @override
  String equipIonTip(int count) {
    return 'Ionensperre $count von 5. Klick: eine mehr, nach 5 keine. Rechtsklick: eine Zahl.';
  }

  @override
  String equipZoltanShield(String name) {
    return '$name: Zoltan-Schild';
  }

  @override
  String equipZoltanTip(int count, int max) {
    return 'Zoltan-Schild $count von $max. Klick auf einen Balken: so viele, der oberste einen weniger. Rechtsklick: eine Zahl.';
  }

  @override
  String equipSystemInstalled(String name) {
    return '$name eingebaut';
  }

  @override
  String get equipBlockNoGameData =>
      'Braucht ftl.dat: der Bauplan des Schiffs nennt die Räume für Systeme.';

  @override
  String get equipBlockNoRoom =>
      'Das Schiff hat keinen Raum für dieses System.';

  @override
  String get equipBlockNotInFormat =>
      'Dieses Spielstandformat hat keinen Platz für dieses System.';

  @override
  String get equipBlockArtillery =>
      'FTL gibt Artillerie nur den Schiffen, die mit ihr starten.';

  @override
  String get equipBlockSystemLimit =>
      'Das Schiff hat 8 Systeme, mehr verkauft FTL nicht. Baue zuerst eines aus.';

  @override
  String get equipBlockSubsystemLimit =>
      'Das Schiff hat 4 Subsysteme, mehr verkauft FTL nicht. Baue zuerst eines aus.';

  @override
  String get equipBlockStartingSystem =>
      'Das Schiff startet mit diesem System. FTL baut es beim Laden wieder ein.';

  @override
  String get equipBlockSwapBack =>
      'Das Schiff startet mit Krankenstation oder Klonstation. Baue die andere ein, um sie zu tauschen.';

  @override
  String get equipBlockItemsInSlots =>
      'Seine Plätze halten Waffen oder Drohnen. Entferne sie zuerst.';

  @override
  String get equipBlockNotDecoded =>
      'Ein Teil des Spielstands, den das ändert, ist nicht dekodiert (ftl.dat).';

  @override
  String get equipBlockNoStation =>
      'Auf einem Gegnerschiff setzt FTL die Station dieses Systems auf ein zufälliges Feld.';

  @override
  String get shipPlayer => 'Dein Schiff';

  @override
  String get shipEnemy => 'Gegnerschiff';

  @override
  String shipEnemyField(String field) {
    return 'Gegnerschiff: $field';
  }

  @override
  String get shipRename => 'Schiff umbenennen';

  @override
  String get fieldShipName => 'Schiffsname';

  @override
  String get fieldShipType => 'Schiffstyp';

  @override
  String get fieldDifficulty => 'Schwierigkeit';

  @override
  String shipTypeLayout(String name, String layout) {
    return '$name · Typ $layout';
  }

  @override
  String get shipChangeType => 'Schiffstyp ändern';

  @override
  String get shipChangeDifficulty => 'Schwierigkeit ändern';

  @override
  String get shipTypeNote =>
      'Ein Typ passt nur mit denselben Räumen und Türen zum Spielstand: FTL liest die gespeicherten Räume, Türen und Crew-Positionen auf dem Grundriss des neuen Typs. Keine zwei Hangar-Schiffe teilen sie.';

  @override
  String get shipDifficultyNote =>
      'FTL wendet eine neue Schwierigkeit ab dem nächsten Laden an: Belohnungen, Gegnerschiffe, Türstärke, Punktzahl.';

  @override
  String get shipSectorFixed =>
      'Der Sektor folgt dem Sektorbaum. Springe auf der Kartenseite im Sektorbaum in einen anderen Sektor.';

  @override
  String get typeBlockOtherLayout => 'Andere Räume und Türen';

  @override
  String typeBlockSystemRoom(String system) {
    return 'Das System $system liegt in anderen Räumen';
  }

  @override
  String typeBlockStartSystem(String system) {
    return 'Startet mit dem System $system, das diesem Schiff fehlt';
  }

  @override
  String get typeBlockWeaponSlots => 'Zu wenige Waffenplätze';

  @override
  String get typeBlockDroneSlots => 'Zu wenige Drohnenplätze';

  @override
  String get typeBlockUnknownShip => 'Nicht in ftl.dat';

  @override
  String get typeBlockNoGameData => 'Braucht ftl.dat';

  @override
  String get typeBlockNotDecoded =>
      'Die Räume des Schiffs sind nicht dekodiert';

  @override
  String get typeBlockEnemyShip => 'Nur das Spielerschiff wechselt seinen Typ';

  @override
  String get difficultyBlockFlagship =>
      'Das Rebellen-Flaggschiff ist auf der Karte: FTL baut seine Baken nach der Schwierigkeit neu, die gespeicherten passten dann nicht.';

  @override
  String get difficultyBlockNotDecoded =>
      'Die Sektorkarte ist nicht dekodiert, also kann der Editor die Schwierigkeit nicht prüfen.';

  @override
  String equipReplaces(String name) {
    return 'Ersetzt die $name';
  }

  @override
  String get expertWarningTitle =>
      'Rohe Änderungen können den Spielstand beschädigen';

  @override
  String get expertWarning =>
      'Das sind die Felder der Datei, wie der Editor sie dekodiert, bearbeitet ohne die Regeln der anderen Seiten. Ein Wert, den FTL nicht erwartet, kann das Spiel abstürzen lassen oder den Lauf beenden. Speichern prüft weiter, dass sich die Datei zurücklesen lässt, und schreibt sonst nichts.';

  @override
  String get expertSearch => 'Pfad oder Wert suchen';

  @override
  String get expertNoFile =>
      'Öffne einen Spielstand, um seine Felder zu sehen.';

  @override
  String get expertNoMatch => 'Kein Feld passt.';

  @override
  String expertShown(int shown, int total) {
    return '$shown von $total Feldern';
  }

  @override
  String get expertUnknownTag => 'unbekannt';

  @override
  String get expertUnknownHint =>
      'Niemand weiß bisher, was diese Felder bedeuten. Ändere eines nur, um eine Vermutung zu prüfen, und behalte das Backup.';

  @override
  String expertUndecoded(int count) {
    return 'Nicht dekodierte Bytes ($count)';
  }

  @override
  String get expertUndecodedHint =>
      'Der Editor dekodiert diese Bytes nicht. Er schreibt sie unverändert zurück; sie lassen sich nicht bearbeiten.';

  @override
  String get expertAllDecoded =>
      'Der Editor dekodiert die ganze Datei: Kein Byte bleibt undekodiert.';

  @override
  String fieldRaw(String path) {
    return 'Roh: $path';
  }

  @override
  String fieldStateVar(String id) {
    return 'Zustandsvariable: $id';
  }

  @override
  String mapBeacon(int n) {
    return 'Leuchtfeuer $n';
  }

  @override
  String mapBeaconField(int n, String field) {
    return 'Leuchtfeuer $n: $field';
  }

  @override
  String get mapVisited => 'Besucht';

  @override
  String get mapNotVisited => 'Nicht besucht';

  @override
  String get mapUnvisit => 'Als unbesucht markieren';

  @override
  String get mapUnvisitHint => 'Sein Ereignis kommt zurück, wenn du ankommst.';

  @override
  String get mapVisitKept => 'Dein Schiff ist hier; FTL behält diesen Besuch.';

  @override
  String get mapSeen => 'Aufgedeckt';

  @override
  String get mapHidden => 'Verborgen';

  @override
  String get mapReveal => 'Aufdecken';

  @override
  String get mapHide => 'Verbergen';

  @override
  String get mapShipHere => 'Schiff';

  @override
  String get mapStore => 'Laden';

  @override
  String get mapQuest => 'Auftrag';

  @override
  String get mapEnemy => 'Schiffsereignis';

  @override
  String get mapOvertaken => 'Von der Rebellenflotte eingeholt';

  @override
  String get mapForSale => 'Zu verkaufen';

  @override
  String get mapSoldOut => 'Ausverkauft';

  @override
  String get mapRestock => 'Auffüllen';

  @override
  String get mapSellOut => 'Als verkauft markieren';

  @override
  String mapStoreItemField(int n, String item) {
    return 'Laden von Leuchtfeuer $n: $item';
  }

  @override
  String get mapFleet => 'Rebellenflotte';

  @override
  String get mapFleetField => 'Position der Rebellenflotte';

  @override
  String mapFleetOvertaken(int count, int total) {
    return '$count von $total Leuchtfeuern eingeholt';
  }

  @override
  String get mapFleetBack => 'Einen Sprung zurück';

  @override
  String get mapFleetStart => 'Sektorstart';

  @override
  String get mapFleetLastStand =>
      'Im letzten Gefecht folgt die Flotte dem Flaggschiff; FTL ignoriert ihren Kreis.';

  @override
  String get mapFleetUnknown =>
      'Die Positionen der Leuchtfeuer sind unbekannt, daher lässt sich die Flotte nicht bewegen.';

  @override
  String get mapLayoutNative =>
      'Dieser Spielstand nutzt die Zufallszahlen seines Betriebssystems; die Positionen der Leuchtfeuer sind unbekannt.';

  @override
  String get mapLayoutMismatch =>
      'Der Layout-Seed baut eine andere Zahl Leuchtfeuer, als die Datei hat; die Positionen sind unbekannt.';

  @override
  String get mapNone => 'Dieser Spielstand hat keine Sektorkarte.';

  @override
  String get mapClose => 'Schließen';

  @override
  String get crewAdd => 'Crewmitglied hinzufügen';

  @override
  String get crewAddFull =>
      'FTL nimmt auf diesem Schiff keine weitere Crew auf';

  @override
  String get crewNew => 'Neu';

  @override
  String get crewRemove => 'Vom Schiff entfernen';

  @override
  String get crewKeep => 'An Bord behalten';

  @override
  String get crewRemoveLast => 'Das letzte Crewmitglied einer Seite bleibt';

  @override
  String get crewRemoved => 'Entfernt';

  @override
  String get crewLook => 'Aussehen';

  @override
  String get crewChangeLook => 'Aussehen ändern';

  @override
  String get crewLookNote =>
      'Die Varianten des Charakter-Editors von FTL für diese Rasse.';

  @override
  String get crewMale => 'Männlich';

  @override
  String get crewFemale => 'Weiblich';

  @override
  String get fieldCrewSex => 'Geschlecht';

  @override
  String fieldCrewColor(int layer) {
    return 'Farbschicht $layer';
  }

  @override
  String crewColor(int number) {
    return 'Farbe $number';
  }

  @override
  String get fieldCrewAdd => 'Neue Crewmitglieder';

  @override
  String get fieldCrewRemove => 'Entfernen';

  @override
  String get crewAboard => 'An Bord';

  @override
  String get mapFleetForward => 'Einen Sprung vor';

  @override
  String get mapShipField => 'Position des Schiffs';

  @override
  String get mapMoveShip => 'Schiff hierher';

  @override
  String get mapMoveHint =>
      'Ankunft wie bei einer Rückkehr: kein Gegner, kein Ereignis, keine Gefahr.';

  @override
  String get mapMoveBlockLastStand =>
      'Im letzten Gefecht folgt das Schiff dem Weg des Flaggschiffs.';

  @override
  String get mapMoveBlockCrew =>
      'Ein Crewmitglied ist auf dem anderen Schiff; ein Sprung ließe es dort.';

  @override
  String get mapMoveBlockFleet =>
      'Die Rebellenflotte hält dieses Leuchtfeuer; FTL behält ihr Flottenereignis, das ein Schiff braucht.';

  @override
  String get mapMoveBlockQuest =>
      'Hier wartet ein Auftrag; FTL spielte sein Ereignis ohne sein Schiff erneut.';

  @override
  String get mapMoveBlockUnknown =>
      'Die Lage des Schiffs ist nicht dekodiert; dafür braucht es ftl.dat.';

  @override
  String get mapStoreAdd => 'Laden einrichten';

  @override
  String get mapStoreRemove => 'Laden entfernen';

  @override
  String get mapStoreAddHint =>
      'FTL lädt einen gespeicherten Laden an jedem Leuchtfeuer; ein neuer Laden verkauft Waffen, Drohnen und Erweiterungen.';

  @override
  String get mapKindsNote =>
      'Auftrag, Notruf, Nebel und andere Ereignisse stehen nicht im Spielstand: FTL baut sie aus den Ereignislisten von ftl.dat.';

  @override
  String get mapSectorsNote =>
      'Andere Sektoren: Öffne den Sektorbaum, um dorthin zu springen.';

  @override
  String get mapSwapItem => 'Gegenstand tauschen';

  @override
  String get mapNoChoices =>
      'Ohne ftl.dat keine anderen Gegenstände dieser Art.';

  @override
  String get mapBeacons => 'Baken';

  @override
  String get sectorTree => 'Sektorbaum';

  @override
  String get sectorCivilian => 'Zivil';

  @override
  String get sectorHostile => 'Feindlich';

  @override
  String get sectorNebula => 'Nebel';

  @override
  String get sectorJumpField => 'Sektorsprung';

  @override
  String get sectorJumpHere => 'Hierher springen';

  @override
  String get sectorJumpHint =>
      'Klicke auf einen Sektor, um dorthin zu springen: Das Schiff kommt an seiner Startbake an.';

  @override
  String get sectorJumpArrival =>
      'Das Schiff kommt wie nach einem Sprung an der Startbake einer neuen Karte dieses Sektors an: Die Rebellenflotte beginnt von vorn, Baken und Läden des Sektors sind neu.';

  @override
  String get sectorShipHere => 'Dein Schiff ist in diesem Sektor.';

  @override
  String get sectorJumpPendingNote =>
      'Ein Sektorsprung ist vorgemerkt: Das ist der neue Sektor. Speichere den Sprung, bevor du seine Baken bearbeitest.';

  @override
  String get sectorJumpLastStand =>
      'FTL baut das letzte Gefecht mit dem Rebellen-Flaggschiff, das der Editor nicht platziert.';

  @override
  String get sectorJumpQuests =>
      'Aufträge warten auf den nächsten Sektor; FTL verteilt sie mit eigenen Zufallszahlen auf seine Baken.';

  @override
  String get sectorJumpHidden =>
      'Das Schiff ist im versteckten Kristallsektor, der nicht im Baum liegt.';

  @override
  String get sectorTreeNative =>
      'Dieses Spiel nutzt die Zufallszahlen seines Betriebssystems; der Sektorbaum ist unbekannt.';

  @override
  String get sectorTreeNoGameData =>
      'Der Sektorbaum braucht die Sektorlisten aus ftl.dat.';

  @override
  String get sectorTreeMismatch =>
      'Der Baum-Seed baut eine andere Sektorzahl, als die Datei enthält; der Sektorbaum ist unbekannt.';
}
