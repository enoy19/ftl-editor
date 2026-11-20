import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:intl/intl.dart' as intl;

import 'app_localizations_de.dart';
import 'app_localizations_en.dart';

// ignore_for_file: type=lint

/// Callers can lookup localized strings with an instance of AppLocalizations
/// returned by `AppLocalizations.of(context)`.
///
/// Applications need to include `AppLocalizations.delegate()` in their app's
/// `localizationDelegates` list, and the locales they support in the app's
/// `supportedLocales` list. For example:
///
/// ```dart
/// import 'l10n/app_localizations.dart';
///
/// return MaterialApp(
///   localizationsDelegates: AppLocalizations.localizationsDelegates,
///   supportedLocales: AppLocalizations.supportedLocales,
///   home: MyApplicationHome(),
/// );
/// ```
///
/// ## Update pubspec.yaml
///
/// Please make sure to update your pubspec.yaml to include the following
/// packages:
///
/// ```yaml
/// dependencies:
///   # Internationalization support.
///   flutter_localizations:
///     sdk: flutter
///   intl: any # Use the pinned version from flutter_localizations
///
///   # Rest of dependencies
/// ```
///
/// ## iOS Applications
///
/// iOS applications define key application metadata, including supported
/// locales, in an Info.plist file that is built into the application bundle.
/// To configure the locales supported by your app, you’ll need to edit this
/// file.
///
/// First, open your project’s ios/Runner.xcworkspace Xcode workspace file.
/// Then, in the Project Navigator, open the Info.plist file under the Runner
/// project’s Runner folder.
///
/// Next, select the Information Property List item, select Add Item from the
/// Editor menu, then select Localizations from the pop-up menu.
///
/// Select and expand the newly-created Localizations item then, for each
/// locale your application supports, add a new item and select the locale
/// you wish to add from the pop-up menu in the Value field. This list should
/// be consistent with the languages listed in the AppLocalizations.supportedLocales
/// property.
abstract class AppLocalizations {
  AppLocalizations(String locale)
    : localeName = intl.Intl.canonicalizedLocale(locale.toString());

  final String localeName;

  static AppLocalizations? of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations);
  }

  static const LocalizationsDelegate<AppLocalizations> delegate =
      _AppLocalizationsDelegate();

  /// A list of this localizations delegate along with the default localizations
  /// delegates.
  ///
  /// Returns a list of localizations delegates containing this delegate along with
  /// GlobalMaterialLocalizations.delegate, GlobalCupertinoLocalizations.delegate,
  /// and GlobalWidgetsLocalizations.delegate.
  ///
  /// Additional delegates can be added by appending to this list in
  /// MaterialApp. This list does not have to be used at all if a custom list
  /// of delegates is preferred or required.
  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates =
      <LocalizationsDelegate<dynamic>>[
        delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
      ];

  /// A list of this localizations delegate's supported locales.
  static const List<Locale> supportedLocales = <Locale>[
    Locale('de'),
    Locale('en'),
  ];

  /// No description provided for @appTitle.
  ///
  /// In en, this message translates to:
  /// **'FTL Save Editor'**
  String get appTitle;

  /// No description provided for @open.
  ///
  /// In en, this message translates to:
  /// **'Open'**
  String get open;

  /// No description provided for @save.
  ///
  /// In en, this message translates to:
  /// **'Save'**
  String get save;

  /// No description provided for @sector.
  ///
  /// In en, this message translates to:
  /// **'Sector {n}'**
  String sector(int n);

  /// No description provided for @difficultyEasy.
  ///
  /// In en, this message translates to:
  /// **'Easy'**
  String get difficultyEasy;

  /// No description provided for @difficultyNormal.
  ///
  /// In en, this message translates to:
  /// **'Normal'**
  String get difficultyNormal;

  /// No description provided for @difficultyHard.
  ///
  /// In en, this message translates to:
  /// **'Hard'**
  String get difficultyHard;

  /// No description provided for @resources.
  ///
  /// In en, this message translates to:
  /// **'Resources'**
  String get resources;

  /// No description provided for @fieldHull.
  ///
  /// In en, this message translates to:
  /// **'Hull'**
  String get fieldHull;

  /// No description provided for @fieldFuel.
  ///
  /// In en, this message translates to:
  /// **'Fuel'**
  String get fieldFuel;

  /// No description provided for @fieldDroneParts.
  ///
  /// In en, this message translates to:
  /// **'Drone parts'**
  String get fieldDroneParts;

  /// No description provided for @fieldMissiles.
  ///
  /// In en, this message translates to:
  /// **'Missiles'**
  String get fieldMissiles;

  /// No description provided for @fieldScrap.
  ///
  /// In en, this message translates to:
  /// **'Scrap'**
  String get fieldScrap;

  /// No description provided for @notANumber.
  ///
  /// In en, this message translates to:
  /// **'Not a number'**
  String get notANumber;

  /// No description provided for @outOfRange.
  ///
  /// In en, this message translates to:
  /// **'Allowed: {min} to {max}'**
  String outOfRange(int min, int max);

  /// No description provided for @saved.
  ///
  /// In en, this message translates to:
  /// **'Saved. Backup: {backup}'**
  String saved(String backup);

  /// No description provided for @closeFtlHint.
  ///
  /// In en, this message translates to:
  /// **'Close FTL before you save: the game writes this file when it quits.'**
  String get closeFtlHint;

  /// No description provided for @error.
  ///
  /// In en, this message translates to:
  /// **'Error: {message}'**
  String error(String message);

  /// No description provided for @errorNoSave.
  ///
  /// In en, this message translates to:
  /// **'No saved game at {path}. FTL deletes continue.sav when a run ends. Start or continue a run, then quit FTL to write it.'**
  String errorNoSave(String path);

  /// No description provided for @errorChangedOnDisk.
  ///
  /// In en, this message translates to:
  /// **'{path} changed since it was opened (is FTL running?). Nothing was written. Close FTL, reload and repeat the edit.'**
  String errorChangedOnDisk(String path);

  /// No description provided for @errorDecode.
  ///
  /// In en, this message translates to:
  /// **'Cannot read {path}. Detail: {detail}'**
  String errorDecode(String path, String detail);

  /// No description provided for @errorVerify.
  ///
  /// In en, this message translates to:
  /// **'The edited file would not read back correctly. Nothing was written. Detail: {detail}'**
  String errorVerify(String detail);

  /// No description provided for @errorEdit.
  ///
  /// In en, this message translates to:
  /// **'Invalid value. Detail: {detail}'**
  String errorEdit(String detail);

  /// No description provided for @errorIo.
  ///
  /// In en, this message translates to:
  /// **'Cannot access {path}. Detail: {detail}'**
  String errorIo(String path, String detail);

  /// No description provided for @errorNoHome.
  ///
  /// In en, this message translates to:
  /// **'No home folder found. Detail: {detail}'**
  String errorNoHome(String detail);

  /// No description provided for @ftlSaves.
  ///
  /// In en, this message translates to:
  /// **'FTL saved games'**
  String get ftlSaves;

  /// No description provided for @allFiles.
  ///
  /// In en, this message translates to:
  /// **'All files'**
  String get allFiles;

  /// No description provided for @chooseFile.
  ///
  /// In en, this message translates to:
  /// **'Choose saved game'**
  String get chooseFile;

  /// No description provided for @reloadFile.
  ///
  /// In en, this message translates to:
  /// **'Reload the saved game from disk'**
  String get reloadFile;

  /// No description provided for @discardEditsTitle.
  ///
  /// In en, this message translates to:
  /// **'Discard changes?'**
  String get discardEditsTitle;

  /// No description provided for @discardEditsBody.
  ///
  /// In en, this message translates to:
  /// **'Reloading reads the file again. The changes not yet saved are lost.'**
  String get discardEditsBody;

  /// No description provided for @discardEdits.
  ///
  /// In en, this message translates to:
  /// **'Discard and reload'**
  String get discardEdits;

  /// No description provided for @noFileOpen.
  ///
  /// In en, this message translates to:
  /// **'No saved game open'**
  String get noFileOpen;

  /// No description provided for @advancedEdition.
  ///
  /// In en, this message translates to:
  /// **'Advanced Edition'**
  String get advancedEdition;

  /// No description provided for @setToMax.
  ///
  /// In en, this message translates to:
  /// **'Set to maximum ({max})'**
  String setToMax(int max);

  /// No description provided for @galleryTitle.
  ///
  /// In en, this message translates to:
  /// **'UI variants'**
  String get galleryTitle;

  /// No description provided for @galleryDark.
  ///
  /// In en, this message translates to:
  /// **'Dark theme'**
  String get galleryDark;

  /// No description provided for @galleryLanguage.
  ///
  /// In en, this message translates to:
  /// **'Switch language'**
  String get galleryLanguage;

  /// No description provided for @areaShip.
  ///
  /// In en, this message translates to:
  /// **'Ship'**
  String get areaShip;

  /// No description provided for @areaCrew.
  ///
  /// In en, this message translates to:
  /// **'Crew'**
  String get areaCrew;

  /// No description provided for @areaEquipment.
  ///
  /// In en, this message translates to:
  /// **'Equipment'**
  String get areaEquipment;

  /// No description provided for @areaMap.
  ///
  /// In en, this message translates to:
  /// **'Map'**
  String get areaMap;

  /// No description provided for @areaProfile.
  ///
  /// In en, this message translates to:
  /// **'Profile'**
  String get areaProfile;

  /// No description provided for @areaExpert.
  ///
  /// In en, this message translates to:
  /// **'Expert'**
  String get areaExpert;

  /// No description provided for @expertDump.
  ///
  /// In en, this message translates to:
  /// **'Raw dump'**
  String get expertDump;

  /// No description provided for @expertStateVars.
  ///
  /// In en, this message translates to:
  /// **'State vars'**
  String get expertStateVars;

  /// No description provided for @expertUnknown.
  ///
  /// In en, this message translates to:
  /// **'Unknown fields'**
  String get expertUnknown;

  /// No description provided for @statusFtlRunning.
  ///
  /// In en, this message translates to:
  /// **'FTL is running'**
  String get statusFtlRunning;

  /// No description provided for @statusFtlClosed.
  ///
  /// In en, this message translates to:
  /// **'FTL is closed'**
  String get statusFtlClosed;

  /// No description provided for @statusSteamCloud.
  ///
  /// In en, this message translates to:
  /// **'Steam Cloud on'**
  String get statusSteamCloud;

  /// No description provided for @skillPilot.
  ///
  /// In en, this message translates to:
  /// **'Pilot'**
  String get skillPilot;

  /// No description provided for @skillEngines.
  ///
  /// In en, this message translates to:
  /// **'Engines'**
  String get skillEngines;

  /// No description provided for @skillShields.
  ///
  /// In en, this message translates to:
  /// **'Shields'**
  String get skillShields;

  /// No description provided for @skillWeapons.
  ///
  /// In en, this message translates to:
  /// **'Weapons'**
  String get skillWeapons;

  /// No description provided for @skillRepair.
  ///
  /// In en, this message translates to:
  /// **'Repair'**
  String get skillRepair;

  /// No description provided for @skillCombat.
  ///
  /// In en, this message translates to:
  /// **'Combat'**
  String get skillCombat;

  /// No description provided for @raceHuman.
  ///
  /// In en, this message translates to:
  /// **'Human'**
  String get raceHuman;

  /// No description provided for @raceEngi.
  ///
  /// In en, this message translates to:
  /// **'Engi'**
  String get raceEngi;

  /// No description provided for @raceMantis.
  ///
  /// In en, this message translates to:
  /// **'Mantis'**
  String get raceMantis;

  /// No description provided for @raceRock.
  ///
  /// In en, this message translates to:
  /// **'Rock'**
  String get raceRock;

  /// No description provided for @raceZoltan.
  ///
  /// In en, this message translates to:
  /// **'Zoltan'**
  String get raceZoltan;

  /// No description provided for @raceSlug.
  ///
  /// In en, this message translates to:
  /// **'Slug'**
  String get raceSlug;

  /// No description provided for @crewTrainAll.
  ///
  /// In en, this message translates to:
  /// **'Train fully'**
  String get crewTrainAll;

  /// No description provided for @crewHeal.
  ///
  /// In en, this message translates to:
  /// **'Heal'**
  String get crewHeal;

  /// No description provided for @raceCrystal.
  ///
  /// In en, this message translates to:
  /// **'Crystal'**
  String get raceCrystal;

  /// No description provided for @raceLanius.
  ///
  /// In en, this message translates to:
  /// **'Lanius'**
  String get raceLanius;

  /// No description provided for @raceGhost.
  ///
  /// In en, this message translates to:
  /// **'Ghost'**
  String get raceGhost;

  /// No description provided for @raceDrone.
  ///
  /// In en, this message translates to:
  /// **'Anti-personnel drone'**
  String get raceDrone;

  /// No description provided for @fieldCrewHealth.
  ///
  /// In en, this message translates to:
  /// **'Health'**
  String get fieldCrewHealth;

  /// No description provided for @fieldCrewName.
  ///
  /// In en, this message translates to:
  /// **'Name'**
  String get fieldCrewName;

  /// No description provided for @fieldCrewRace.
  ///
  /// In en, this message translates to:
  /// **'Race'**
  String get fieldCrewRace;

  /// No description provided for @crewRename.
  ///
  /// In en, this message translates to:
  /// **'Rename'**
  String get crewRename;

  /// No description provided for @crewNameRule.
  ///
  /// In en, this message translates to:
  /// **'1 to 32 characters'**
  String get crewNameRule;

  /// No description provided for @crewChangeRace.
  ///
  /// In en, this message translates to:
  /// **'Change race'**
  String get crewChangeRace;

  /// No description provided for @crewRaceNote.
  ///
  /// In en, this message translates to:
  /// **'A new race keeps the share of health and the mastery level of each skill.'**
  String get crewRaceNote;

  /// No description provided for @crewField.
  ///
  /// In en, this message translates to:
  /// **'{name}: {field}'**
  String crewField(String name, String field);

  /// No description provided for @crewSkillTooltip.
  ///
  /// In en, this message translates to:
  /// **'{skill}: {points} of {max}, level {level} of 2'**
  String crewSkillTooltip(String skill, int points, int max, int level);

  /// No description provided for @crewSkillHint.
  ///
  /// In en, this message translates to:
  /// **'Click: next mastery level. Long press or right click: exact points.'**
  String get crewSkillHint;

  /// No description provided for @crewNone.
  ///
  /// In en, this message translates to:
  /// **'No crew aboard.'**
  String get crewNone;

  /// No description provided for @statusLastBackup.
  ///
  /// In en, this message translates to:
  /// **'Last backup: {time}'**
  String statusLastBackup(String time);

  /// No description provided for @apply.
  ///
  /// In en, this message translates to:
  /// **'Apply'**
  String get apply;

  /// No description provided for @cancel.
  ///
  /// In en, this message translates to:
  /// **'Cancel'**
  String get cancel;

  /// No description provided for @resourceTooltip.
  ///
  /// In en, this message translates to:
  /// **'{label}, {min} to {max}. Click to edit.'**
  String resourceTooltip(String label, int min, int max);

  /// No description provided for @resourceEdited.
  ///
  /// In en, this message translates to:
  /// **'{label}: changed, {inFile} in the file'**
  String resourceEdited(String label, int inFile);

  /// No description provided for @laterMap.
  ///
  /// In en, this message translates to:
  /// **'The map comes with the map view: sectors, beacons and stores.'**
  String get laterMap;

  /// No description provided for @statusFtlUnknown.
  ///
  /// In en, this message translates to:
  /// **'FTL: unknown'**
  String get statusFtlUnknown;

  /// No description provided for @statusSteamCloudOff.
  ///
  /// In en, this message translates to:
  /// **'Steam Cloud off'**
  String get statusSteamCloudOff;

  /// No description provided for @statusSteamCloudUnknown.
  ///
  /// In en, this message translates to:
  /// **'Steam Cloud: unknown'**
  String get statusSteamCloudUnknown;

  /// No description provided for @statusSteamCloudHint.
  ///
  /// In en, this message translates to:
  /// **'Steam Cloud can replace the edited file with its own copy when FTL starts.'**
  String get statusSteamCloudHint;

  /// No description provided for @statusNotChecked.
  ///
  /// In en, this message translates to:
  /// **'Open a file to check this.'**
  String get statusNotChecked;

  /// No description provided for @statusNoBackup.
  ///
  /// In en, this message translates to:
  /// **'No backup in this session'**
  String get statusNoBackup;

  /// No description provided for @ftlDatFound.
  ///
  /// In en, this message translates to:
  /// **'Game data found'**
  String get ftlDatFound;

  /// No description provided for @ftlDatFoundHint.
  ///
  /// In en, this message translates to:
  /// **'The editor shows ship, crew and item images from this ftl.dat of your FTL installation. Use it?'**
  String get ftlDatFoundHint;

  /// No description provided for @ftlDatConfirm.
  ///
  /// In en, this message translates to:
  /// **'Use this ftl.dat'**
  String get ftlDatConfirm;

  /// No description provided for @ftlDatChoose.
  ///
  /// In en, this message translates to:
  /// **'Choose ftl.dat'**
  String get ftlDatChoose;

  /// No description provided for @ftlDatNotFound.
  ///
  /// In en, this message translates to:
  /// **'No ftl.dat found'**
  String get ftlDatNotFound;

  /// No description provided for @ftlDatNotFoundHint.
  ///
  /// In en, this message translates to:
  /// **'The editor runs in reduced mode without game images. Choose ftl.dat, its folder or FTL.app.'**
  String get ftlDatNotFoundHint;

  /// No description provided for @ftlDatFiles.
  ///
  /// In en, this message translates to:
  /// **'FTL game data'**
  String get ftlDatFiles;

  /// No description provided for @ftlDatApp.
  ///
  /// In en, this message translates to:
  /// **'FTL app'**
  String get ftlDatApp;

  /// No description provided for @reducedMode.
  ///
  /// In en, this message translates to:
  /// **'Reduced mode: no ftl.dat'**
  String get reducedMode;

  /// No description provided for @errorFtlDatNotFound.
  ///
  /// In en, this message translates to:
  /// **'No ftl.dat found. Detail: {detail}'**
  String errorFtlDatNotFound(String detail);

  /// No description provided for @errorConfig.
  ///
  /// In en, this message translates to:
  /// **'Cannot use the editor settings in {path}. Detail: {detail}'**
  String errorConfig(String path, String detail);

  /// No description provided for @errorUnknownImageId.
  ///
  /// In en, this message translates to:
  /// **'The game data has no such image. Detail: {detail}'**
  String errorUnknownImageId(String detail);

  /// No description provided for @errorFtlDatNotOpen.
  ///
  /// In en, this message translates to:
  /// **'No ftl.dat is open. The editor runs in reduced mode.'**
  String get errorFtlDatNotOpen;

  /// No description provided for @errorEntryMissing.
  ///
  /// In en, this message translates to:
  /// **'The game data has no such file. Detail: {detail}'**
  String errorEntryMissing(String detail);

  /// No description provided for @errorFtlDatUnreadable.
  ///
  /// In en, this message translates to:
  /// **'Cannot read the game data {path}. Detail: {detail}'**
  String errorFtlDatUnreadable(String path, String detail);

  /// No description provided for @galleryLook.
  ///
  /// In en, this message translates to:
  /// **'FTL look'**
  String get galleryLook;

  /// No description provided for @lookNoFtlDat.
  ///
  /// In en, this message translates to:
  /// **'No ftl.dat found: the FTL look reads its fonts and frames from the local game.'**
  String get lookNoFtlDat;

  /// No description provided for @lookUnreadable.
  ///
  /// In en, this message translates to:
  /// **'The FTL look cannot be read from ftl.dat: {detail}'**
  String lookUnreadable(String detail);

  /// No description provided for @equipReactor.
  ///
  /// In en, this message translates to:
  /// **'Reactor'**
  String get equipReactor;

  /// No description provided for @equipSystems.
  ///
  /// In en, this message translates to:
  /// **'Systems'**
  String get equipSystems;

  /// No description provided for @equipWeapons.
  ///
  /// In en, this message translates to:
  /// **'Weapons'**
  String get equipWeapons;

  /// No description provided for @equipDrones.
  ///
  /// In en, this message translates to:
  /// **'Drones'**
  String get equipDrones;

  /// No description provided for @equipAugments.
  ///
  /// In en, this message translates to:
  /// **'Augments'**
  String get equipAugments;

  /// No description provided for @equipSlots.
  ///
  /// In en, this message translates to:
  /// **'{used} of {all} slots'**
  String equipSlots(int used, int all);

  /// No description provided for @equipPower.
  ///
  /// In en, this message translates to:
  /// **'Power {power}'**
  String equipPower(int power);

  /// No description provided for @equipNotInstalled.
  ///
  /// In en, this message translates to:
  /// **'Not installed'**
  String get equipNotInstalled;

  /// No description provided for @equipAddAugment.
  ///
  /// In en, this message translates to:
  /// **'Add augment'**
  String get equipAddAugment;

  /// A weapon, drone, cargo or augment slot that holds nothing.
  ///
  /// In en, this message translates to:
  /// **'Empty slot'**
  String get equipEmptySlot;

  /// No description provided for @equipRemove.
  ///
  /// In en, this message translates to:
  /// **'Remove'**
  String get equipRemove;

  /// No description provided for @equipCargo.
  ///
  /// In en, this message translates to:
  /// **'Cargo'**
  String get equipCargo;

  /// No description provided for @equipAddWeapon.
  ///
  /// In en, this message translates to:
  /// **'Add weapon'**
  String get equipAddWeapon;

  /// No description provided for @equipAddDrone.
  ///
  /// In en, this message translates to:
  /// **'Add drone'**
  String get equipAddDrone;

  /// No description provided for @equipAddCargo.
  ///
  /// In en, this message translates to:
  /// **'Add to cargo'**
  String get equipAddCargo;

  /// No description provided for @equipOn.
  ///
  /// In en, this message translates to:
  /// **'on'**
  String get equipOn;

  /// No description provided for @equipPowerOn.
  ///
  /// In en, this message translates to:
  /// **'Power on'**
  String get equipPowerOn;

  /// No description provided for @equipPowerOff.
  ///
  /// In en, this message translates to:
  /// **'Power off'**
  String get equipPowerOff;

  /// No description provided for @equipToCargo.
  ///
  /// In en, this message translates to:
  /// **'Move to cargo'**
  String get equipToCargo;

  /// No description provided for @equipToSlot.
  ///
  /// In en, this message translates to:
  /// **'Move to a slot'**
  String get equipToSlot;

  /// No description provided for @equipCargoUnknown.
  ///
  /// In en, this message translates to:
  /// **'The cargo needs the sector map after the ship, which needs the ship layout from ftl.dat.'**
  String get equipCargoUnknown;

  /// No description provided for @equipItemsLocked.
  ///
  /// In en, this message translates to:
  /// **'Weapons and drones need ftl.dat and the states after the nearby ship.'**
  String get equipItemsLocked;

  /// No description provided for @equipPowered.
  ///
  /// In en, this message translates to:
  /// **'{name}: powered'**
  String equipPowered(String name);

  /// No description provided for @equipInCargo.
  ///
  /// In en, this message translates to:
  /// **'{name} in the cargo'**
  String equipInCargo(String name);

  /// No description provided for @equipRowHint.
  ///
  /// In en, this message translates to:
  /// **'Click a segment to set the level, right-click it to break or repair it.'**
  String get equipRowHint;

  /// No description provided for @equipNotDecoded.
  ///
  /// In en, this message translates to:
  /// **'Weapons, drones and augments need the ship layout from ftl.dat.'**
  String get equipNotDecoded;

  /// No description provided for @equipFilter.
  ///
  /// In en, this message translates to:
  /// **'Filter by name'**
  String get equipFilter;

  /// No description provided for @statusReducedMode.
  ///
  /// In en, this message translates to:
  /// **'Reduced mode'**
  String get statusReducedMode;

  /// No description provided for @statusReducedModeHint.
  ///
  /// In en, this message translates to:
  /// **'No ftl.dat is open: the editor shows icons instead of game images.'**
  String get statusReducedModeHint;

  /// No description provided for @statusGameData.
  ///
  /// In en, this message translates to:
  /// **'Game data: ftl.dat'**
  String get statusGameData;

  /// No description provided for @statusChooseFile.
  ///
  /// In en, this message translates to:
  /// **'Click to open another saved game.'**
  String get statusChooseFile;

  /// No description provided for @statusChooseGameData.
  ///
  /// In en, this message translates to:
  /// **'Click to choose another ftl.dat.'**
  String get statusChooseGameData;

  /// No description provided for @statusProcessListUnavailable.
  ///
  /// In en, this message translates to:
  /// **'The editor cannot read the process list, so a running FTL is unknown. Quit FTL before you save. Detail: {detail}'**
  String statusProcessListUnavailable(String detail);

  /// No description provided for @ftlRunningTitle.
  ///
  /// In en, this message translates to:
  /// **'FTL is running'**
  String get ftlRunningTitle;

  /// No description provided for @ftlRunningBody.
  ///
  /// In en, this message translates to:
  /// **'FTL is running and overwrites this file when it quits. Quit FTL first, then save.'**
  String get ftlRunningBody;

  /// No description provided for @saveAnyway.
  ///
  /// In en, this message translates to:
  /// **'Save anyway'**
  String get saveAnyway;

  /// No description provided for @changesTitle.
  ///
  /// In en, this message translates to:
  /// **'Changes to write'**
  String get changesTitle;

  /// No description provided for @changesHint.
  ///
  /// In en, this message translates to:
  /// **'Check each change before the editor writes the file.'**
  String get changesHint;

  /// No description provided for @undo.
  ///
  /// In en, this message translates to:
  /// **'Undo'**
  String get undo;

  /// No description provided for @redo.
  ///
  /// In en, this message translates to:
  /// **'Redo'**
  String get redo;

  /// No description provided for @writeFile.
  ///
  /// In en, this message translates to:
  /// **'Write file'**
  String get writeFile;

  /// No description provided for @backupsTitle.
  ///
  /// In en, this message translates to:
  /// **'Backups of {file}'**
  String backupsTitle(String file);

  /// No description provided for @backupsEmpty.
  ///
  /// In en, this message translates to:
  /// **'No backups of this file yet.'**
  String get backupsEmpty;

  /// No description provided for @backupsShow.
  ///
  /// In en, this message translates to:
  /// **'Show the backups of this file'**
  String get backupsShow;

  /// No description provided for @restore.
  ///
  /// In en, this message translates to:
  /// **'Restore'**
  String get restore;

  /// No description provided for @restoreTitle.
  ///
  /// In en, this message translates to:
  /// **'Restore this backup?'**
  String get restoreTitle;

  /// No description provided for @restoreBody.
  ///
  /// In en, this message translates to:
  /// **'The editor writes the backup from {time} over the open file. It backs up the current file first. Edits that are not saved are lost.'**
  String restoreBody(String time);

  /// No description provided for @restored.
  ///
  /// In en, this message translates to:
  /// **'Restored the backup. Backup of the replaced file: {backup}'**
  String restored(String backup);

  /// No description provided for @close.
  ///
  /// In en, this message translates to:
  /// **'Close'**
  String get close;

  /// No description provided for @profileChoose.
  ///
  /// In en, this message translates to:
  /// **'Choose profile'**
  String get profileChoose;

  /// No description provided for @profileNoFile.
  ///
  /// In en, this message translates to:
  /// **'No profile open'**
  String get profileNoFile;

  /// No description provided for @profileShips.
  ///
  /// In en, this message translates to:
  /// **'Player ships'**
  String get profileShips;

  /// No description provided for @profileAchievements.
  ///
  /// In en, this message translates to:
  /// **'Achievements ({n})'**
  String profileAchievements(int n);

  /// No description provided for @profileStats.
  ///
  /// In en, this message translates to:
  /// **'Stats'**
  String get profileStats;

  /// No description provided for @profileTopScores.
  ///
  /// In en, this message translates to:
  /// **'Top scores'**
  String get profileTopScores;

  /// No description provided for @profileNoScores.
  ///
  /// In en, this message translates to:
  /// **'No scores yet'**
  String get profileNoScores;

  /// No description provided for @profileUnlockAll.
  ///
  /// In en, this message translates to:
  /// **'Unlock all ships'**
  String get profileUnlockAll;

  /// No description provided for @profileUnlockAdds.
  ///
  /// In en, this message translates to:
  /// **'Adds on Easy: {achievements}'**
  String profileUnlockAdds(String achievements);

  /// No description provided for @profileAllUnlocked.
  ///
  /// In en, this message translates to:
  /// **'Every layout of every ship is already unlocked.'**
  String get profileAllUnlocked;

  /// No description provided for @profileLayoutUnlocked.
  ///
  /// In en, this message translates to:
  /// **'Type-{layout} unlocked'**
  String profileLayoutUnlocked(String layout);

  /// No description provided for @profileLayoutLocked.
  ///
  /// In en, this message translates to:
  /// **'Type-{layout} locked'**
  String profileLayoutLocked(String layout);

  /// No description provided for @profileLayoutNone.
  ///
  /// In en, this message translates to:
  /// **'This ship has no Type-C'**
  String get profileLayoutNone;

  /// No description provided for @profileCloseFtl.
  ///
  /// In en, this message translates to:
  /// **'Close FTL before you save: the game writes the profile when it quits.'**
  String get profileCloseFtl;

  /// No description provided for @profileVictory.
  ///
  /// In en, this message translates to:
  /// **'Victory'**
  String get profileVictory;

  /// No description provided for @statBest.
  ///
  /// In en, this message translates to:
  /// **'Best'**
  String get statBest;

  /// No description provided for @statTotal.
  ///
  /// In en, this message translates to:
  /// **'Total'**
  String get statTotal;

  /// No description provided for @statShipsDefeated.
  ///
  /// In en, this message translates to:
  /// **'Ships defeated'**
  String get statShipsDefeated;

  /// No description provided for @statBeaconsExplored.
  ///
  /// In en, this message translates to:
  /// **'Beacons explored'**
  String get statBeaconsExplored;

  /// No description provided for @statScrapCollected.
  ///
  /// In en, this message translates to:
  /// **'Scrap collected'**
  String get statScrapCollected;

  /// No description provided for @statCrewHired.
  ///
  /// In en, this message translates to:
  /// **'Crew hired'**
  String get statCrewHired;

  /// No description provided for @statGamesPlayed.
  ///
  /// In en, this message translates to:
  /// **'Games played'**
  String get statGamesPlayed;

  /// No description provided for @statVictories.
  ///
  /// In en, this message translates to:
  /// **'Victories'**
  String get statVictories;

  /// No description provided for @errorNoProfile.
  ///
  /// In en, this message translates to:
  /// **'No profile at {path}. FTL writes ae_prof.sav when it quits for the first time. Start FTL once and quit it.'**
  String errorNoProfile(String path);

  /// No description provided for @profileBackups.
  ///
  /// In en, this message translates to:
  /// **'Backups'**
  String get profileBackups;

  /// No description provided for @floorTitle.
  ///
  /// In en, this message translates to:
  /// **'Floorplan'**
  String get floorTitle;

  /// No description provided for @floorHint.
  ///
  /// In en, this message translates to:
  /// **'Click a system symbol for its levels, fire and breaches. Click a square to light or put out a fire, add or seal a breach, set the oxygen of its room, or pick a crew member standing there.'**
  String get floorHint;

  /// No description provided for @floorRoom.
  ///
  /// In en, this message translates to:
  /// **'Room {room}'**
  String floorRoom(int room);

  /// No description provided for @floorFire.
  ///
  /// In en, this message translates to:
  /// **'Fire'**
  String get floorFire;

  /// No description provided for @floorPutOut.
  ///
  /// In en, this message translates to:
  /// **'Put out'**
  String get floorPutOut;

  /// No description provided for @floorBreach.
  ///
  /// In en, this message translates to:
  /// **'Hull breach'**
  String get floorBreach;

  /// No description provided for @floorSeal.
  ///
  /// In en, this message translates to:
  /// **'Seal'**
  String get floorSeal;

  /// No description provided for @squareLightFire.
  ///
  /// In en, this message translates to:
  /// **'Light fire'**
  String get squareLightFire;

  /// No description provided for @squarePutOutFire.
  ///
  /// In en, this message translates to:
  /// **'Put out fire'**
  String get squarePutOutFire;

  /// No description provided for @squareAddBreach.
  ///
  /// In en, this message translates to:
  /// **'Add breach'**
  String get squareAddBreach;

  /// No description provided for @squareSealBreach.
  ///
  /// In en, this message translates to:
  /// **'Seal breach'**
  String get squareSealBreach;

  /// No description provided for @floorOxygen.
  ///
  /// In en, this message translates to:
  /// **'Oxygen'**
  String get floorOxygen;

  /// No description provided for @floorOxygenValue.
  ///
  /// In en, this message translates to:
  /// **'Oxygen {percent}%'**
  String floorOxygenValue(int percent);

  /// No description provided for @floorOxygenSet.
  ///
  /// In en, this message translates to:
  /// **'Set'**
  String get floorOxygenSet;

  /// No description provided for @fieldOxygen.
  ///
  /// In en, this message translates to:
  /// **'Oxygen: {name}'**
  String fieldOxygen(String name);

  /// No description provided for @floorClose.
  ///
  /// In en, this message translates to:
  /// **'Close'**
  String get floorClose;

  /// No description provided for @fieldFire.
  ///
  /// In en, this message translates to:
  /// **'Fire: {name}'**
  String fieldFire(String name);

  /// No description provided for @fieldBreach.
  ///
  /// In en, this message translates to:
  /// **'Hull breach: {name}'**
  String fieldBreach(String name);

  /// No description provided for @actionMaxEverything.
  ///
  /// In en, this message translates to:
  /// **'Max everything'**
  String get actionMaxEverything;

  /// No description provided for @actionCrewFullyTrained.
  ///
  /// In en, this message translates to:
  /// **'Train crew fully'**
  String get actionCrewFullyTrained;

  /// No description provided for @actionRepairShip.
  ///
  /// In en, this message translates to:
  /// **'Repair ship'**
  String get actionRepairShip;

  /// No description provided for @actionHealCrew.
  ///
  /// In en, this message translates to:
  /// **'Heal crew'**
  String get actionHealCrew;

  /// No description provided for @actionNothing.
  ///
  /// In en, this message translates to:
  /// **'{action}: nothing to change'**
  String actionNothing(String action);

  /// No description provided for @noteHullAtEditorCap.
  ///
  /// In en, this message translates to:
  /// **'Maximum hull unknown without ftl.dat: hull set to the editor cap {count}'**
  String noteHullAtEditorCap(int count);

  /// No description provided for @noteSystemsAtEditorCap.
  ///
  /// In en, this message translates to:
  /// **'A system has no limit in the ship blueprint: it went to the cap of unmodded FTL'**
  String get noteSystemsAtEditorCap;

  /// No description provided for @noteRoomsNotDecoded.
  ///
  /// In en, this message translates to:
  /// **'Rooms not decoded (no ship layout): fires and breaches unknown'**
  String get noteRoomsNotDecoded;

  /// No description provided for @paletteHint.
  ///
  /// In en, this message translates to:
  /// **'Search fields, actions and areas'**
  String get paletteHint;

  /// No description provided for @paletteNoMatch.
  ///
  /// In en, this message translates to:
  /// **'No match'**
  String get paletteNoMatch;

  /// No description provided for @paletteArea.
  ///
  /// In en, this message translates to:
  /// **'Area'**
  String get paletteArea;

  /// No description provided for @paletteAction.
  ///
  /// In en, this message translates to:
  /// **'Action'**
  String get paletteAction;

  /// No description provided for @paletteResource.
  ///
  /// In en, this message translates to:
  /// **'Resource'**
  String get paletteResource;

  /// No description provided for @paletteSkill.
  ///
  /// In en, this message translates to:
  /// **'Crew · Skill'**
  String get paletteSkill;

  /// No description provided for @paletteHealth.
  ///
  /// In en, this message translates to:
  /// **'Crew · Health'**
  String get paletteHealth;

  /// No description provided for @paletteSystem.
  ///
  /// In en, this message translates to:
  /// **'System'**
  String get paletteSystem;

  /// No description provided for @paletteAugment.
  ///
  /// In en, this message translates to:
  /// **'Augment'**
  String get paletteAugment;

  /// No description provided for @paletteFire.
  ///
  /// In en, this message translates to:
  /// **'Floorplan · Fire'**
  String get paletteFire;

  /// No description provided for @paletteBreach.
  ///
  /// In en, this message translates to:
  /// **'Floorplan · Breach'**
  String get paletteBreach;

  /// No description provided for @paletteOxygen.
  ///
  /// In en, this message translates to:
  /// **'Floorplan · Oxygen'**
  String get paletteOxygen;

  /// No description provided for @paletteBeacon.
  ///
  /// In en, this message translates to:
  /// **'Map · Beacon'**
  String get paletteBeacon;

  /// No description provided for @profileUnlockLayout.
  ///
  /// In en, this message translates to:
  /// **'{ship}: unlock Type-{layout}'**
  String profileUnlockLayout(String ship, String layout);

  /// No description provided for @profileLockLayout.
  ///
  /// In en, this message translates to:
  /// **'{ship}: lock Type-{layout}'**
  String profileLockLayout(String ship, String layout);

  /// No description provided for @profileLockRemoves.
  ///
  /// In en, this message translates to:
  /// **'Removes: {achievements}'**
  String profileLockRemoves(String achievements);

  /// No description provided for @profileLayoutBRule.
  ///
  /// In en, this message translates to:
  /// **'Type-B has no flag of its own: FTL unlocks it from two of the ship\'s three achievements. Unlocking it adds the missing ones on Easy, locking it removes them until one is left.'**
  String get profileLayoutBRule;

  /// No description provided for @profileLayoutClick.
  ///
  /// In en, this message translates to:
  /// **'{state}. Click to change.'**
  String profileLayoutClick(String state);

  /// No description provided for @profileAddAchievement.
  ///
  /// In en, this message translates to:
  /// **'Add achievement'**
  String get profileAddAchievement;

  /// No description provided for @profileAchievementAdd.
  ///
  /// In en, this message translates to:
  /// **'Add {name} on {difficulty}'**
  String profileAchievementAdd(String name, String difficulty);

  /// No description provided for @profileAchievementRemove.
  ///
  /// In en, this message translates to:
  /// **'Remove {name}'**
  String profileAchievementRemove(String name);

  /// No description provided for @profileAchievementSet.
  ///
  /// In en, this message translates to:
  /// **'{name} on {difficulty}'**
  String profileAchievementSet(String name, String difficulty);

  /// No description provided for @profileAchievementRemoveButton.
  ///
  /// In en, this message translates to:
  /// **'Remove'**
  String get profileAchievementRemoveButton;

  /// No description provided for @profileAchievementRemoved.
  ///
  /// In en, this message translates to:
  /// **'Removed'**
  String get profileAchievementRemoved;

  /// No description provided for @profileAchievementDifficulty.
  ///
  /// In en, this message translates to:
  /// **'Difficulty'**
  String get profileAchievementDifficulty;

  /// No description provided for @equipInstall.
  ///
  /// In en, this message translates to:
  /// **'Install'**
  String get equipInstall;

  /// No description provided for @equipSystemDamage.
  ///
  /// In en, this message translates to:
  /// **'{name}: broken bars'**
  String equipSystemDamage(String name);

  /// No description provided for @equipSystemIon.
  ///
  /// In en, this message translates to:
  /// **'{name}: ion lock'**
  String equipSystemIon(String name);

  /// No description provided for @equipDamaged.
  ///
  /// In en, this message translates to:
  /// **'{count} broken'**
  String equipDamaged(int count);

  /// No description provided for @equipIonTip.
  ///
  /// In en, this message translates to:
  /// **'Ion lock {count} of 5. Click: one more, after 5 none. Right-click: a number.'**
  String equipIonTip(int count);

  /// No description provided for @equipZoltanShield.
  ///
  /// In en, this message translates to:
  /// **'{name}: Zoltan shield'**
  String equipZoltanShield(String name);

  /// No description provided for @equipZoltanTip.
  ///
  /// In en, this message translates to:
  /// **'Zoltan shield {count} of {max}. Click a bar: that many, the top bar one less. Right-click: a number.'**
  String equipZoltanTip(int count, int max);

  /// No description provided for @equipSystemInstalled.
  ///
  /// In en, this message translates to:
  /// **'{name} installed'**
  String equipSystemInstalled(String name);

  /// No description provided for @equipBlockNoGameData.
  ///
  /// In en, this message translates to:
  /// **'Needs ftl.dat: the ship blueprint names the rooms for systems.'**
  String get equipBlockNoGameData;

  /// No description provided for @equipBlockNoRoom.
  ///
  /// In en, this message translates to:
  /// **'The ship has no room for this system.'**
  String get equipBlockNoRoom;

  /// No description provided for @equipBlockNotInFormat.
  ///
  /// In en, this message translates to:
  /// **'This save format has no place for this system.'**
  String get equipBlockNotInFormat;

  /// No description provided for @equipBlockArtillery.
  ///
  /// In en, this message translates to:
  /// **'FTL gives artillery only to the ships that start with it.'**
  String get equipBlockArtillery;

  /// No description provided for @equipBlockSystemLimit.
  ///
  /// In en, this message translates to:
  /// **'The ship holds 8 systems, the most FTL sells. Remove one first.'**
  String get equipBlockSystemLimit;

  /// No description provided for @equipBlockSubsystemLimit.
  ///
  /// In en, this message translates to:
  /// **'The ship holds 4 subsystems, the most FTL sells. Remove one first.'**
  String get equipBlockSubsystemLimit;

  /// No description provided for @equipBlockStartingSystem.
  ///
  /// In en, this message translates to:
  /// **'The ship starts with this system. FTL builds it again when it loads the save.'**
  String get equipBlockStartingSystem;

  /// No description provided for @equipBlockSwapBack.
  ///
  /// In en, this message translates to:
  /// **'The ship starts with a Medbay or Clone Bay. Install the other one to swap them.'**
  String get equipBlockSwapBack;

  /// No description provided for @equipBlockItemsInSlots.
  ///
  /// In en, this message translates to:
  /// **'Weapons or drones are in its slots. Remove them first.'**
  String get equipBlockItemsInSlots;

  /// No description provided for @equipBlockNotDecoded.
  ///
  /// In en, this message translates to:
  /// **'A part of the save this changes is not decoded (ftl.dat).'**
  String get equipBlockNotDecoded;

  /// No description provided for @equipBlockNoStation.
  ///
  /// In en, this message translates to:
  /// **'On an enemy ship FTL puts the station of this system on a random square.'**
  String get equipBlockNoStation;

  /// No description provided for @shipPlayer.
  ///
  /// In en, this message translates to:
  /// **'Your ship'**
  String get shipPlayer;

  /// No description provided for @shipEnemy.
  ///
  /// In en, this message translates to:
  /// **'Enemy ship'**
  String get shipEnemy;

  /// A field of the ship at the current beacon in the change sheet and the command palette.
  ///
  /// In en, this message translates to:
  /// **'Enemy ship: {field}'**
  String shipEnemyField(String field);

  /// No description provided for @shipRename.
  ///
  /// In en, this message translates to:
  /// **'Rename ship'**
  String get shipRename;

  /// No description provided for @fieldShipName.
  ///
  /// In en, this message translates to:
  /// **'Ship name'**
  String get fieldShipName;

  /// No description provided for @fieldShipType.
  ///
  /// In en, this message translates to:
  /// **'Ship type'**
  String get fieldShipType;

  /// No description provided for @fieldDifficulty.
  ///
  /// In en, this message translates to:
  /// **'Difficulty'**
  String get fieldDifficulty;

  /// No description provided for @shipTypeLayout.
  ///
  /// In en, this message translates to:
  /// **'{name} · Type {layout}'**
  String shipTypeLayout(String name, String layout);

  /// No description provided for @shipChangeType.
  ///
  /// In en, this message translates to:
  /// **'Change ship type'**
  String get shipChangeType;

  /// No description provided for @shipChangeDifficulty.
  ///
  /// In en, this message translates to:
  /// **'Change difficulty'**
  String get shipChangeDifficulty;

  /// No description provided for @shipTypeNote.
  ///
  /// In en, this message translates to:
  /// **'A type fits the save only with the same rooms and doors: FTL reads the stored rooms, doors and crew positions on the floor of the new type. No two hangar ships share them.'**
  String get shipTypeNote;

  /// No description provided for @shipDifficultyNote.
  ///
  /// In en, this message translates to:
  /// **'FTL applies a new difficulty from the next load on: rewards, enemy ships, door health, the score.'**
  String get shipDifficultyNote;

  /// No description provided for @shipSectorFixed.
  ///
  /// In en, this message translates to:
  /// **'The sector follows the sector tree. Jump to another sector on the Map page, in the sector tree.'**
  String get shipSectorFixed;

  /// No description provided for @typeBlockOtherLayout.
  ///
  /// In en, this message translates to:
  /// **'Other rooms and doors'**
  String get typeBlockOtherLayout;

  /// No description provided for @typeBlockSystemRoom.
  ///
  /// In en, this message translates to:
  /// **'The {system} system sits in other rooms'**
  String typeBlockSystemRoom(String system);

  /// No description provided for @typeBlockStartSystem.
  ///
  /// In en, this message translates to:
  /// **'Starts with the {system} system, which this ship lacks'**
  String typeBlockStartSystem(String system);

  /// No description provided for @typeBlockWeaponSlots.
  ///
  /// In en, this message translates to:
  /// **'Too few weapon slots'**
  String get typeBlockWeaponSlots;

  /// No description provided for @typeBlockDroneSlots.
  ///
  /// In en, this message translates to:
  /// **'Too few drone slots'**
  String get typeBlockDroneSlots;

  /// No description provided for @typeBlockUnknownShip.
  ///
  /// In en, this message translates to:
  /// **'Not in ftl.dat'**
  String get typeBlockUnknownShip;

  /// No description provided for @typeBlockNoGameData.
  ///
  /// In en, this message translates to:
  /// **'Needs ftl.dat'**
  String get typeBlockNoGameData;

  /// No description provided for @typeBlockNotDecoded.
  ///
  /// In en, this message translates to:
  /// **'The rooms of the ship are not decoded'**
  String get typeBlockNotDecoded;

  /// No description provided for @typeBlockEnemyShip.
  ///
  /// In en, this message translates to:
  /// **'Only the player ship changes its type'**
  String get typeBlockEnemyShip;

  /// No description provided for @difficultyBlockFlagship.
  ///
  /// In en, this message translates to:
  /// **'The rebel flagship is on the map: FTL rebuilds its beacons by difficulty, so the stored ones would not match.'**
  String get difficultyBlockFlagship;

  /// No description provided for @difficultyBlockNotDecoded.
  ///
  /// In en, this message translates to:
  /// **'The sector map is not decoded, so the editor cannot check the difficulty.'**
  String get difficultyBlockNotDecoded;

  /// No description provided for @equipReplaces.
  ///
  /// In en, this message translates to:
  /// **'Replaces the {name}'**
  String equipReplaces(String name);

  /// No description provided for @expertWarningTitle.
  ///
  /// In en, this message translates to:
  /// **'Raw edits can break the save'**
  String get expertWarningTitle;

  /// No description provided for @expertWarning.
  ///
  /// In en, this message translates to:
  /// **'These are the fields of the file as the editor decodes them, edited without the rules of the other pages. A value FTL does not expect can crash the game or end the run. Save still checks that the file reads back and writes nothing else.'**
  String get expertWarning;

  /// No description provided for @expertSearch.
  ///
  /// In en, this message translates to:
  /// **'Search path or value'**
  String get expertSearch;

  /// No description provided for @expertNoFile.
  ///
  /// In en, this message translates to:
  /// **'Open a saved game to see its fields.'**
  String get expertNoFile;

  /// No description provided for @expertNoMatch.
  ///
  /// In en, this message translates to:
  /// **'No field matches.'**
  String get expertNoMatch;

  /// No description provided for @expertShown.
  ///
  /// In en, this message translates to:
  /// **'{shown} of {total} fields'**
  String expertShown(int shown, int total);

  /// No description provided for @expertUnknownTag.
  ///
  /// In en, this message translates to:
  /// **'unknown'**
  String get expertUnknownTag;

  /// No description provided for @expertUnknownHint.
  ///
  /// In en, this message translates to:
  /// **'Nobody knows yet what these fields mean. Change one only to test a guess, and keep the backup.'**
  String get expertUnknownHint;

  /// No description provided for @expertUndecoded.
  ///
  /// In en, this message translates to:
  /// **'Undecoded bytes ({count})'**
  String expertUndecoded(int count);

  /// No description provided for @expertUndecodedHint.
  ///
  /// In en, this message translates to:
  /// **'The editor does not decode these bytes. It writes them back unchanged; they cannot be edited.'**
  String get expertUndecodedHint;

  /// No description provided for @expertAllDecoded.
  ///
  /// In en, this message translates to:
  /// **'The editor decodes the whole file: no byte is left undecoded.'**
  String get expertAllDecoded;

  /// No description provided for @fieldRaw.
  ///
  /// In en, this message translates to:
  /// **'Raw: {path}'**
  String fieldRaw(String path);

  /// No description provided for @fieldStateVar.
  ///
  /// In en, this message translates to:
  /// **'State var: {id}'**
  String fieldStateVar(String id);

  /// No description provided for @mapBeacon.
  ///
  /// In en, this message translates to:
  /// **'Beacon {n}'**
  String mapBeacon(int n);

  /// No description provided for @mapBeaconField.
  ///
  /// In en, this message translates to:
  /// **'Beacon {n}: {field}'**
  String mapBeaconField(int n, String field);

  /// No description provided for @mapVisited.
  ///
  /// In en, this message translates to:
  /// **'Visited'**
  String get mapVisited;

  /// No description provided for @mapNotVisited.
  ///
  /// In en, this message translates to:
  /// **'Not visited'**
  String get mapNotVisited;

  /// No description provided for @mapUnvisit.
  ///
  /// In en, this message translates to:
  /// **'Mark unvisited'**
  String get mapUnvisit;

  /// No description provided for @mapUnvisitHint.
  ///
  /// In en, this message translates to:
  /// **'Its event comes back when you arrive.'**
  String get mapUnvisitHint;

  /// No description provided for @mapVisitKept.
  ///
  /// In en, this message translates to:
  /// **'Your ship is here; FTL keeps this visit.'**
  String get mapVisitKept;

  /// No description provided for @mapSeen.
  ///
  /// In en, this message translates to:
  /// **'Revealed'**
  String get mapSeen;

  /// No description provided for @mapHidden.
  ///
  /// In en, this message translates to:
  /// **'Hidden'**
  String get mapHidden;

  /// No description provided for @mapReveal.
  ///
  /// In en, this message translates to:
  /// **'Reveal'**
  String get mapReveal;

  /// No description provided for @mapHide.
  ///
  /// In en, this message translates to:
  /// **'Hide'**
  String get mapHide;

  /// No description provided for @mapShipHere.
  ///
  /// In en, this message translates to:
  /// **'Ship'**
  String get mapShipHere;

  /// No description provided for @mapStore.
  ///
  /// In en, this message translates to:
  /// **'Store'**
  String get mapStore;

  /// No description provided for @mapQuest.
  ///
  /// In en, this message translates to:
  /// **'Quest'**
  String get mapQuest;

  /// No description provided for @mapEnemy.
  ///
  /// In en, this message translates to:
  /// **'Ship event'**
  String get mapEnemy;

  /// No description provided for @mapOvertaken.
  ///
  /// In en, this message translates to:
  /// **'Overtaken by the rebel fleet'**
  String get mapOvertaken;

  /// No description provided for @mapForSale.
  ///
  /// In en, this message translates to:
  /// **'For sale'**
  String get mapForSale;

  /// No description provided for @mapSoldOut.
  ///
  /// In en, this message translates to:
  /// **'Sold out'**
  String get mapSoldOut;

  /// No description provided for @mapRestock.
  ///
  /// In en, this message translates to:
  /// **'Restock'**
  String get mapRestock;

  /// No description provided for @mapSellOut.
  ///
  /// In en, this message translates to:
  /// **'Mark sold'**
  String get mapSellOut;

  /// No description provided for @mapStoreItemField.
  ///
  /// In en, this message translates to:
  /// **'Beacon {n} store: {item}'**
  String mapStoreItemField(int n, String item);

  /// No description provided for @mapFleet.
  ///
  /// In en, this message translates to:
  /// **'Rebel fleet'**
  String get mapFleet;

  /// No description provided for @mapFleetField.
  ///
  /// In en, this message translates to:
  /// **'Rebel fleet position'**
  String get mapFleetField;

  /// No description provided for @mapFleetOvertaken.
  ///
  /// In en, this message translates to:
  /// **'{count} of {total} beacons overtaken'**
  String mapFleetOvertaken(int count, int total);

  /// No description provided for @mapFleetBack.
  ///
  /// In en, this message translates to:
  /// **'Back one jump'**
  String get mapFleetBack;

  /// No description provided for @mapFleetStart.
  ///
  /// In en, this message translates to:
  /// **'Sector start'**
  String get mapFleetStart;

  /// No description provided for @mapFleetLastStand.
  ///
  /// In en, this message translates to:
  /// **'In the Last Stand the fleet follows the flagship; FTL ignores its circle.'**
  String get mapFleetLastStand;

  /// No description provided for @mapFleetUnknown.
  ///
  /// In en, this message translates to:
  /// **'The beacon positions are unknown, so the fleet cannot be moved.'**
  String get mapFleetUnknown;

  /// No description provided for @mapLayoutNative.
  ///
  /// In en, this message translates to:
  /// **'This game uses the random numbers of its OS; the beacon positions are unknown.'**
  String get mapLayoutNative;

  /// No description provided for @mapLayoutMismatch.
  ///
  /// In en, this message translates to:
  /// **'The layout seed builds another beacon count than the file holds; the positions are unknown.'**
  String get mapLayoutMismatch;

  /// No description provided for @mapNone.
  ///
  /// In en, this message translates to:
  /// **'This save holds no sector map.'**
  String get mapNone;

  /// No description provided for @mapClose.
  ///
  /// In en, this message translates to:
  /// **'Close'**
  String get mapClose;

  /// No description provided for @crewAdd.
  ///
  /// In en, this message translates to:
  /// **'Add crew member'**
  String get crewAdd;

  /// No description provided for @crewAddFull.
  ///
  /// In en, this message translates to:
  /// **'FTL takes no more crew on this ship'**
  String get crewAddFull;

  /// No description provided for @crewNew.
  ///
  /// In en, this message translates to:
  /// **'New'**
  String get crewNew;

  /// No description provided for @crewRemove.
  ///
  /// In en, this message translates to:
  /// **'Remove from the ship'**
  String get crewRemove;

  /// No description provided for @crewKeep.
  ///
  /// In en, this message translates to:
  /// **'Keep aboard'**
  String get crewKeep;

  /// No description provided for @crewRemoveLast.
  ///
  /// In en, this message translates to:
  /// **'The last crew member of a side stays'**
  String get crewRemoveLast;

  /// No description provided for @crewRemoved.
  ///
  /// In en, this message translates to:
  /// **'Removed'**
  String get crewRemoved;

  /// No description provided for @crewLook.
  ///
  /// In en, this message translates to:
  /// **'Look'**
  String get crewLook;

  /// No description provided for @crewChangeLook.
  ///
  /// In en, this message translates to:
  /// **'Change look'**
  String get crewChangeLook;

  /// No description provided for @crewLookNote.
  ///
  /// In en, this message translates to:
  /// **'The looks of FTL\'s character creator for this race.'**
  String get crewLookNote;

  /// No description provided for @crewMale.
  ///
  /// In en, this message translates to:
  /// **'Male'**
  String get crewMale;

  /// No description provided for @crewFemale.
  ///
  /// In en, this message translates to:
  /// **'Female'**
  String get crewFemale;

  /// No description provided for @fieldCrewSex.
  ///
  /// In en, this message translates to:
  /// **'Sex'**
  String get fieldCrewSex;

  /// No description provided for @fieldCrewColor.
  ///
  /// In en, this message translates to:
  /// **'Color layer {layer}'**
  String fieldCrewColor(int layer);

  /// No description provided for @crewColor.
  ///
  /// In en, this message translates to:
  /// **'Color {number}'**
  String crewColor(int number);

  /// No description provided for @fieldCrewAdd.
  ///
  /// In en, this message translates to:
  /// **'New crew members'**
  String get fieldCrewAdd;

  /// No description provided for @fieldCrewRemove.
  ///
  /// In en, this message translates to:
  /// **'Remove'**
  String get fieldCrewRemove;

  /// No description provided for @crewAboard.
  ///
  /// In en, this message translates to:
  /// **'Aboard'**
  String get crewAboard;

  /// No description provided for @mapFleetForward.
  ///
  /// In en, this message translates to:
  /// **'Forward one jump'**
  String get mapFleetForward;

  /// No description provided for @mapShipField.
  ///
  /// In en, this message translates to:
  /// **'Ship position'**
  String get mapShipField;

  /// No description provided for @mapMoveShip.
  ///
  /// In en, this message translates to:
  /// **'Move ship here'**
  String get mapMoveShip;

  /// No description provided for @mapMoveHint.
  ///
  /// In en, this message translates to:
  /// **'Arrives like a return: no enemy, no event, no hazard.'**
  String get mapMoveHint;

  /// No description provided for @mapMoveBlockLastStand.
  ///
  /// In en, this message translates to:
  /// **'In the Last Stand the ship follows the flagship\'s path.'**
  String get mapMoveBlockLastStand;

  /// No description provided for @mapMoveBlockCrew.
  ///
  /// In en, this message translates to:
  /// **'A crew member is aboard the other ship; a jump would leave them there.'**
  String get mapMoveBlockCrew;

  /// No description provided for @mapMoveBlockFleet.
  ///
  /// In en, this message translates to:
  /// **'The rebel fleet holds this beacon; FTL keeps its fleet event, which needs a ship.'**
  String get mapMoveBlockFleet;

  /// No description provided for @mapMoveBlockQuest.
  ///
  /// In en, this message translates to:
  /// **'A quest waits here; FTL would play its event again without its ship.'**
  String get mapMoveBlockQuest;

  /// No description provided for @mapMoveBlockUnknown.
  ///
  /// In en, this message translates to:
  /// **'The ship\'s situation is not decoded; it needs ftl.dat.'**
  String get mapMoveBlockUnknown;

  /// No description provided for @mapStoreAdd.
  ///
  /// In en, this message translates to:
  /// **'Make a store'**
  String get mapStoreAdd;

  /// No description provided for @mapStoreRemove.
  ///
  /// In en, this message translates to:
  /// **'Remove the store'**
  String get mapStoreRemove;

  /// No description provided for @mapStoreAddHint.
  ///
  /// In en, this message translates to:
  /// **'FTL loads a stored stock at any beacon; a new store sells weapons, drones and augments.'**
  String get mapStoreAddHint;

  /// No description provided for @mapKindsNote.
  ///
  /// In en, this message translates to:
  /// **'Quest, distress, nebula and other events are not in the save: FTL builds them from the event lists of ftl.dat.'**
  String get mapKindsNote;

  /// No description provided for @mapSectorsNote.
  ///
  /// In en, this message translates to:
  /// **'Other sectors: open the sector tree to jump there.'**
  String get mapSectorsNote;

  /// No description provided for @mapSwapItem.
  ///
  /// In en, this message translates to:
  /// **'Swap item'**
  String get mapSwapItem;

  /// No description provided for @mapNoChoices.
  ///
  /// In en, this message translates to:
  /// **'No other items of this kind without ftl.dat.'**
  String get mapNoChoices;

  /// No description provided for @mapBeacons.
  ///
  /// In en, this message translates to:
  /// **'Beacons'**
  String get mapBeacons;

  /// No description provided for @sectorTree.
  ///
  /// In en, this message translates to:
  /// **'Sector tree'**
  String get sectorTree;

  /// No description provided for @sectorCivilian.
  ///
  /// In en, this message translates to:
  /// **'Civilian'**
  String get sectorCivilian;

  /// No description provided for @sectorHostile.
  ///
  /// In en, this message translates to:
  /// **'Hostile'**
  String get sectorHostile;

  /// No description provided for @sectorNebula.
  ///
  /// In en, this message translates to:
  /// **'Nebula'**
  String get sectorNebula;

  /// No description provided for @sectorJumpField.
  ///
  /// In en, this message translates to:
  /// **'Sector jump'**
  String get sectorJumpField;

  /// No description provided for @sectorJumpHere.
  ///
  /// In en, this message translates to:
  /// **'Jump here'**
  String get sectorJumpHere;

  /// No description provided for @sectorJumpHint.
  ///
  /// In en, this message translates to:
  /// **'Click a sector to jump there: the ship arrives at its start beacon.'**
  String get sectorJumpHint;

  /// No description provided for @sectorJumpArrival.
  ///
  /// In en, this message translates to:
  /// **'The ship arrives at the start beacon of a new map of this sector, as after a jump: the rebel fleet starts over, the beacons and stores of this sector are new.'**
  String get sectorJumpArrival;

  /// No description provided for @sectorShipHere.
  ///
  /// In en, this message translates to:
  /// **'Your ship is in this sector.'**
  String get sectorShipHere;

  /// No description provided for @sectorJumpPendingNote.
  ///
  /// In en, this message translates to:
  /// **'A sector jump is pending: this is the new sector. Save the jump before you edit its beacons.'**
  String get sectorJumpPendingNote;

  /// No description provided for @sectorJumpLastStand.
  ///
  /// In en, this message translates to:
  /// **'FTL builds the Last Stand with the rebel flagship, which the editor does not place.'**
  String get sectorJumpLastStand;

  /// No description provided for @sectorJumpQuests.
  ///
  /// In en, this message translates to:
  /// **'Quests wait for the next sector; FTL places them on its beacons with its own random numbers.'**
  String get sectorJumpQuests;

  /// No description provided for @sectorJumpHidden.
  ///
  /// In en, this message translates to:
  /// **'The ship is in the hidden Crystal sector, which is not in the tree.'**
  String get sectorJumpHidden;

  /// No description provided for @sectorTreeNative.
  ///
  /// In en, this message translates to:
  /// **'This game uses the random numbers of its OS; the sector tree is unknown.'**
  String get sectorTreeNative;

  /// No description provided for @sectorTreeNoGameData.
  ///
  /// In en, this message translates to:
  /// **'The sector tree needs the sector lists of ftl.dat.'**
  String get sectorTreeNoGameData;

  /// No description provided for @sectorTreeMismatch.
  ///
  /// In en, this message translates to:
  /// **'The tree seed builds another sector count than the file holds; the sector tree is unknown.'**
  String get sectorTreeMismatch;
}

class _AppLocalizationsDelegate
    extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  Future<AppLocalizations> load(Locale locale) {
    return SynchronousFuture<AppLocalizations>(lookupAppLocalizations(locale));
  }

  @override
  bool isSupported(Locale locale) =>
      <String>['de', 'en'].contains(locale.languageCode);

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}

AppLocalizations lookupAppLocalizations(Locale locale) {
  // Lookup logic when only language code is specified.
  switch (locale.languageCode) {
    case 'de':
      return AppLocalizationsDe();
    case 'en':
      return AppLocalizationsEn();
  }

  throw FlutterError(
    'AppLocalizations.delegate failed to load unsupported locale "$locale". This is likely '
    'an issue with the localizations generation tool. Please file an issue '
    'on GitHub with a reproducible sample app and the gen-l10n configuration '
    'that was used.',
  );
}
