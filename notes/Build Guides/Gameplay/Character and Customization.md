---
type: build-guide
section: Gameplay
updated: 2026-10-12
---

# Character and Customization

Part of [[Build Guides]] › Gameplay. Related: [[Items and Inventory]], [[Combat and Abilities]].

<!-- NARRATIVE START -->
## How it works

**How to read this.** Confidence words (`confirmed`, `likely`, `speculative`) are the native-map labels for a function. Source tags: *client* = read from the shipped executable in a shard or in this pass (pseudocode, raw bytes, RTTI, strings); *client Lua* = the client's own UI script bytecode, losslessly decompiled in the lab (not redistributed); *client resource* = a data file from the shipped client's `Resources` folder (every count below was recomputed from the files in this pass); *client locale* = the shipped text table; *video* = a reviewed frame record (5,760 records, 656 videos; no player names or chat are recorded); *wiki* = the fan wiki; *vault* = a project research note or a native run of our Rust server, used here only for facts it derived from the client or from our own runs. Precedence is client, then video, then wiki. *(R)* marks reconstructed policy that is ours, not original behaviour. No original server code exists, so every statement about what the server did is a hypothesis built from what the client requires. The community emulators' creator tables, packet writers, profile tables and database rows are not used anywhere in this guide. OSFR-held data (packet captures and server JSON) is not used. Where a native run depended on a community-supplied file (the creator scene, the populated starting-set table, the character-select movie override) the dependency is named in Evidence and in Open questions. Request opcodes and sub-opcode numbers below are the ones the client's own constructors and dispatchers show. Related: [[Items and Inventory]] (equipment, style cards, item definitions), [[Combat and Abilities]] (ability sets, stats), [[Quests and Objectives]] and [[Rewards and Collections]] (where stars come from), [[Login and Account]] and [[Gateway and Transport]] (the login channel and the world tunnel), [[User Data and Storage]], [[Text and Localization]], [[Scaleform and Flash UI]], [[Scripting (Lua)]], [[Actors and NPCs]], [[Content]] and the job pages ([[Adventurer]], [[Archer]], [[Blacksmith]], [[Brawler]], [[Card Duelist]], [[Chef]], [[Demolition Derby Driver]], [[Fisherman]], [[Kart Driver]], [[Medic]], [[Miner]], [[Ninja]], [[Pet Trainer]], [[Postman]], [[Soccer Star]], [[Warrior]], [[Wizard]]).

### Overview

Nine player-facing pieces share this guide. They share one character record but are separate screens and separate packets.

1. **Character select** (a Scaleform list, the movie `characterSelectV2`): one row per character, a final "Add a Character" row, a delete control and a Go button.
2. **Character creation**: a race-and-gender screen with four mannequins, a name screen, and the nine-category appearance creator, ending in a confirmation prompt.
3. **Appearance in the world**: how the look chosen at creation, and later changes from style cards, is drawn on your own character and on other players.
4. **Jobs** (the client calls them profiles): the job browser, the "Jobs and Equipment" screen and the quick job switch beside the portrait.
5. **Job levels and stars**: a rank from 1 to 20 per job, a percent bar, and the stars that fill it.
6. **Level-up notices**: a banner and a sound when a job gains a rank.
7. **The About Me panel** ("My Details"): name, wallet, jobs gained, collections, quests, age, play time, membership, VIP and per-job stats.
8. **Titles**: a selectable title shown with the player's name.
9. **Inspect**: another player's look, jobs and gear.

What the player sees:

- **Character select.** One name per row, the last row "Add a Character" (client locale 436106). When the account's maximum character count is not above one, the Add row shows the system message "Get more Characters! Become A Member!" (client locale 435820) instead of opening the creator (client Lua `CharacterSelect:OnUserEvent`). Delete asks for confirmation with a warning that "deleting this character will delete all items that character owns, including items you purchased from the Station Cash Store" and that it "cannot be undone" (client locale 435948; a second storefront variant is 434777).
- **Race and gender.** Four mannequins stand in a dressing-room scene: Human Male, Human Female, Pixie Male, Pixie Female (client resource `CharacterCreateCharacterTypes.txt`; client locale 430557 to 430560). The client data names two races, `Human` (id 1) and `Pixie` (id 2), but the model files of the second are called `fairy_*` (client resource `RaceDefinitions.txt`, `Models.txt`).
- **Name.** Two ways in: a name wheel built from glossary parts (a first name, plus a last name made of a prefix and a suffix) with a Random button, or a custom name. A custom name "will need to be approved" and a random name "will appear in game until then" (client locale 435950, 430554). Rejections have their own texts: already in use (432106), questionable content (432107), not valid (432108), first or last name too short (432111, 432112), too long (432097, 432096), wrong total length (432109) and a server-side failure (432104) (client locale through `CodeStringMappings.txt`). Video shows the same flow in a browser-based creator: "Darn, that name is taken! pick another:", "Your name will need to be approved", then "Temporary Name / Needs Approval / All done! Let's go into Free Realms!" (video, one recording, 5 frames).
- **Appearance creator.** Nine category buttons: Face Type, Skin Tone, Hair Style, Hair Color, Eye Color, Facial Hair (Human males only), Wings (Pixies only), Clothes (labelled Outfits in one video) and Face Paint, plus Random, Back and Next (client resource `CharacterCreateCategories.txt`; client locale 434046 to 434053, 156816). Video has 11 creation frames in 3 recordings, 5 of them with the category strip ("Face Type: Seafarer, Skin Tone: Gold, Hair Style: Shaggy, Hair Color: Dark Abyss, Eye Color: Sea Stone, Facial Hair: Chin, Clothes: Blue"). The creator starts on a random pick in every category (client Lua `CharacterCreate:Randomize`, once per race and gender choice).
- **Confirmation.** "Are you finished editing this Character? Your character will appear in the game like it appears now." (client locale 430551), then "Entering Free Realms using the character" (430553) and a loading screen.
- **Jobs.** The job browser lists the character's jobs with name, description, badge, rank, a percent bar and, for members-only jobs, a lock; a job in trial shows a trial hint. Selecting one switches job. "Select a Job below to switch jobs." and "Click on the Job icon next to your character portrait at the top left-hand corner of the screen to switch jobs before entering a battle or duel." show where it lives and that it is meant to be done before a fight (client locale 3933813042, 2994175518; the long ids are the locale file's own).
- **Levels and stars.** "As you play minigames and battle opponents, you get better at your jobs and level up! Leveling up a job lets you wear new outfits and get new quests." (client locale 4024725827). "You earn stars by playing a minigame like combat or cooking ... Stars only count for the job you were using when they were earned." (client locale 2583709755). The client says "You can get every job in the game! Anyone can play up to level 4, but every job goes up to level 20! Use the Marketplace to purchase any job you want, or pay for a membership to unlock all of them!" (348625404) and, for new players, "you can try any job up to at least level five" (2065313732). The top rank is 20 and the bar is forced to 100 percent at rank 20 (client Lua `JobBrowser:Populate`, `LevelProgressBar:Populate`). All 17 job pages give the same cap (max level 20) and the wiki gives 4 as the limit for most jobs without a membership (wiki, Jobs). Video job readings: 3,534 frames carry a numeric job level, 1,450 of them 20, none above 20 except three frames the reviewer flagged as unreadable small digits (video).
- **Advanced jobs.** The client intro text calls Ninja, Wizard and Warrior "Advanced Jobs ... that you can earn by being a master Archer, Medic, or Brawler" (client locale 1073904650). Job-board notices say "Requires: Level 20 Archer" for Ninja, "Level 20 Brawler" for Warrior and "Level 20 Medic" for Wizard (client locale 1592607723, 79965512, 1780716543). The wiki gives no prerequisite; whether the live game enforced the gate is not proven (job pages Ninja, Warrior, Wizard).
- **Level-up banner.** "<Job> Level Up!" with the job's badge art. Video shows it in 37 frames of 25 recordings, 35 for Adventurer and 2 for Kart Driver.
- **About Me.** Name, coins, wallet, "job levels gained", collections completed, quests completed, creation date and age, play time, membership, gender, race, VIP rank and progress, and a per-job stat list (client Lua `AboutPlayer:Populate`, `PlayerInfo_DS`). Labels seen include "Age:" (438998) and "Gender:" (401367).
- **Titles.** A drop-down of owned titles, one selected, shown as a prefix, a suffix or an alias in place of the name (client Lua `AboutPlayer`; client `PlayerTitleTemplateExpand`). In the client Lua the title list is requested only when the China-environment flag is set, so titles may have been a regional feature (see Open questions).
- **Unlocking jobs.** Most jobs unlock by speaking to a trainer (job pages, wiki and client text); a minigame object in the world can start the unlock quest of a job you do not have; the job browser can start a job's unlock quest by profile id (wiki; client Lua `JobBrowser:StartJobQuest`).

### How the client implements it

#### Main classes and objects

| Class or object | Role | Evidence |
|---|---|---|
| `ClientAccountManager` (state machine `Run` `00715ec0`, state names `00705610`, constructor `0071a530`) | The pre-game driver: server list, account info, character list, creator, name check, store, delete, login. Holds the cached character list (`+0x22290`, nodes of about `0x62b8` bytes linked at `+0x62b0`), the pending creation record (`+0x5387c`), the selected character (`+0x53970`) and the state dword (`+0x22270`). | confirmed (`00715ec0`, `00705610`, `0070c8b0`) |
| `ClientAccountInterface` (`0071cec0`) with `ClientAccountDataManager` (`00725180`) | The creator's data side: loads the six `CharacterCreate` tables, keeps the nine selected item ids at `+0x9f7c..+0x9f9c`, resolves them to definitions (`ChangeItem` `0071f030`) and builds the category buttons (`BuildCategoryItems` `0071fd60`). | confirmed (`0071f030`, `0071fd60`, `00724d80`) |
| `CharacterCreationManager` (`009836e0`, bootstrap `0097c960..00982e70`) | The second creator-side manager: character types, customization categories and items, the name wheel, the preset scanner and the data sources `BaseClient.StartingSets`, `.CharacterCustomizationCategory`, `.CharacterCreateCustomization<Suffix>`, `.CharacterNameCategories`, `.CharacterPresets`, `.AccountInfo` and `.ServerList`. A `ServerQueryInterface` container is named by RTTI. | confirmed (`0097f690`, `00980370`, `00980540`, `00980740`, `009829f0`) |
| `ControllerDressingRoom` (`00985790..00985ed0`) | The controller of the creator scene the mannequins stand in. | confirmed (RTTI) |
| Lua objects `CharacterCreateType`, `CharacterCreate`, `CharacterNameScreen`, `CharacterSelect`, `CharacterCreatePopups`, `CharacterNameHandler` | The presentation layer of the creator and select screens. | client Lua |
| Native Lua module `CharacterCreateUi` (registrar `00c13e70`, table `0182bb88`, 20 entries) | `CreationFinished`, `CreationAborted`, `CreationChangeCategory`, `CreationChangeItem`, `CreationRequestItemData`, `CreationNameAccepted`, `CreationNameCancelled`, `CreationNameKeyboardOpen`, `CreationRandomize`, `CreationGetState`, `CreationSetTraceState`, `CreationSelectCurrentCharacter`, `CreationGetPlayerRaceAndGender`, `CreationSelectMannequinByRaceAndGender` and six `CreationMenuButton*` entries. | table read in this pass (`00c13b00..00c14000`) |
| `ProfileDataSource`, `ProfileFilteredDataSource` (`00c90630`, `00c90790`; `BaseClient.Profiles`, `BaseClient.Profiles.Filtered`) | The job list the browser binds to: 16 columns (see Data and formats). | confirmed (`00c90630`, column names `00c8f2b0`) |
| `ClientProfileData` map (50 buckets, 0x58-byte nodes, key at `+0x50`; record reader `00a3a2b0`, loader `00a3a580`, packet parse `00a3a900`) | The job definitions the server sends once (reference data 44/3). | confirmed (`00a3a900`), likely (readers) |
| `ClientPcProfile` nodes in the player data (list head at `ClientPcData+0x49c`, next link at dword `0x38`) and `ClientPcProfileManager` (`00a197e0`) | The character's own per-job state: rank, percent, stars, ability sets, equipment. | confirmed (handler `009392c0` cases 14 and 18; manager constructor) |
| `PlayerCustomization` (constructor `00bd9e20`) over `HashListMap<int,PlayerCustomizationData,10,-1>` | The live appearance store of a character: nine customization slots with committed and pending values, ten name strings and a map of data rows. | confirmed constructor; apply logic likely |
| `ProxiedCharacter`, `ProxiedCharacterAttachmentGroup`, `PlayerSwitchProfileAttachmentGroup`, `NewHatHairAttachmentGroup`, `ProxiedCharacterAttachmentGroupManager` | The world representation of a player and the groups that load its models. | confirmed (RTTI; S0256, S0259, S0573) |
| `PlayerSwitchProfileEvent` (`00bfb4d0`) and `ProcessProfileSwap` (`00bfb620`) | The local event and the required-slot check run when a profile swap's attachment group becomes ready. | confirmed |
| `CharacterDataSource` (`00bfdf20`; `BaseClient.PlayerInfo`) | The About Me data: 28 named fields. | confirmed (`00bfca90`, `00bfcfe0`) |
| `PlayerTitleDataSource` (`00cc5930`; `BaseClient.PlayerTitles`) over `HashListMap<int,PlayerTitleData,32,-1>` | The title list, six columns. | confirmed |
| `InspectDataSource`, `InspectItemsDataSource`, `InspectProfilesDataSource` (`BaseClient.Inspect`, `.Inspect.Items`, `.Inspect.Jobs`) | The inspect panel tables. | confirmed |
| `CharacterBase` (constructor `01042bd0`, property slots `01042ed0..01044470`) and `CharacterVariables` (`01041c30`) | A very wide property object: hundreds of virtual getter and setter slots over `+0x3d0..+0x5f4`, every fifth slot returning the literal "entryTag", plus a string-keyed variable map. The slots carry no feature meaning by themselves. | confirmed (S0569, S0570, S0571) |

Of the 829 mapped rows, 374 come from S0569 and S0570 alone (the `CharacterBase` accessors), and many more are generic containers, effects, movement and mount code in shards that merely touch a character class. They add nothing to the feature. The useful rows are the account manager, the two creator managers, the customization store, the profile and title code and the data sources.

#### The pre-game state machine

`ClientAccountManager::Run` (`00715ec0`) loops while the state dword is below `0xfd` and switches on it. The 53 state names are printed by `00705610`. Reading the transitions in the pseudocode gives this flow (client; the conditions inside some states were not traced):

| Phase | States | What happens |
|---|---|---|
| Account | 1 Init, 2 PendingServerList, 3 ReceivedServerList, 4 PendingAccountInfo | The server list is requested and only entries that are online and unlocked are kept (`00711ca0`). Account info (`00719330`) copies three values: a member flag, the maximum character count and an admin flag. The character list reply (`00719550`) stores a has-characters flag and moves to state 5 or 7. |
| Branch | 5 ReceivedAccountInfo_ExistingCharacter, 7 ReceivedAccountInfo_NoCharacter, 8 ReceivedNoAccountInfo, 6 ..._WaitForKeyPress | 5 and 7 both go to SetupZone and request the creator resources (`0071efb0` asks for `./Resources/CharacterCreate/`). State 5 also builds one mannequin per saved character from its stored appearance and selects the first. State 6 waits about 60 frames for a key chord that starts a delete, otherwise launches with the character. |
| Scene | 11 SetupZone, 12 StartUi | Waits for the dressing-room zone, builds the four creator mannequins (one per race and gender pair, `CreatePlayer` `007140c0`), then starts the UI: with saved characters it goes to 14 ShowExistingPlayer_Init, otherwise to 17 SelectRaceGender_FirstTime. |
| Select | 14 ShowExistingPlayer_Init, 15 ShowExistingPlayer, 16 ConfirmCharacterDelete, 50 WaitingForDelete | The select screen. From 15 the interface flags choose Go (to 43 Login), delete (to 16, then 50) or add (to 17). |
| Race and gender | 17 SelectRaceGender_FirstTime, 18 SelectRaceGender_Init, 19 SelectRaceGender | The type screen. Accepting goes to 20; Back returns to 14 when characters exist. |
| Name | 20 to 26 SelectUsername_*, 27 to 35 CheckNames_* | The name wheel or custom entry. A name is sent for checking (`PacketCheckNameRequest`, see Network) and the state waits (30). A good preset, custom or plain result goes to 36 CustomizeItems_Init; a bad name shows a message and returns to the name screen. |
| Customize | 36 CustomizeItems_Init, 37 CustomizeItems, 38 FinishedEditing | The nine-category creator. Next raises the confirmation popup; accepting moves to 39, declining returns to 36. |
| Store | 39 StoreNewCharacter, 40 NewCharacterSaving, 41 NewCharacterSaved, 42 NewCharacterSaveFailed | State 39 copies the nine chosen ids from `ClientAccountInterface+0x9f7c`, resolves each to a definition and the clothing set to its three items, fills the pending record and sends the create request (`00705a10` to `00956590`). The reply (`007100c0`) either stores the new entity key on the new list node and enters 41, or routes back as described below. |
| Login | 43 Login, 44 WaitingForLogin, 45 WaitingForLoginFailed, 46 WaitingForLoginSucceeded, 47 Run, 255 StartGame | State 43 sends the character login request with the selected entity key (`0070a980`). The reply (`00711220`) carries a gateway address, ticket and key; success goes to 46, then 47, and starts the game. |
| Exits | 48 CreateAborted, 49 LoginRefused, 51 WaitForErrorMessage, 52 DoNothing, 253 ExitGame, 254 BackToLauncher | CreateAborted returns to the name screen (20). |

The order in the native client is therefore race and gender, then name, then appearance, then confirm (client; state transitions). Video of the browser-based creator shows appearance first and name second, so the two creators are different flows (video, two recordings).

The Lua side agrees. `CharacterCreate:StartInGameFlow(a)` shows `CharacterCreateType` when the argument is 0 (no characters) and `CharacterSelect` otherwise. `CharacterNameScreen` Next calls `CreationNameAccepted(first, lastPrefix, last)` and `CreationMenuButtonAccept`. `CharacterCreate` Next sets the creation-warning flag and calls `CreationFinished`; the popup's OK calls `CreationMenuButtonAccept`; Back and Cancel call `CreationAborted` and `CreationMenuButtonCancel` (client Lua).

#### Creation: how the appearance choice is built

1. **Four mannequins.** For each race (1 Human, 2 Pixie) and gender (1 male, 2 female) `CreatePlayer(race, gender, ...)` (`007140c0`) looks up a **starting set** whose race (`+0xac`) and gender (`+0xb0`) match, then the starting set's **clothing set**, then that set's feet, legs and chest item definitions. A failed lookup is logged to `ClientAccountManager.log` with the missing definition ids and the mannequin is not built. The starting-set table in the supplied client has a header and no rows, so every call fails with "startingSetId=-1" (client: the loader `00723e90` parses whatever rows the file has; vault: Character Creation Audit). A separate, populated table, `CharacterCreatePresets.txt` (4 rows), is what the type screen's mannequin selection scans (`SelectCharacterDummyByRaceGender` `00711a60`).
2. **Choosing.** Each category button lists the option rows whose race and gender match the current mannequin (`CharacterCreate:IsUsed`). Selecting an option calls `CreationChangeItem(itemId, categoryId)`. The native `ChangeItem` (`0071f030`) stores the item id in one of nine slots in this order: 1 skin tone, 2 hair style, 3 hair color, 4 eye color, 5 facial hair, 6 face paint, 7 wings, 8 clothes, 9 head or face type. It resolves the id through the mapping table, then the definition, builds the asset name (templates with `<race>`, `<gender>` and `<hairtype>` placeholders, `0070ca40`) and applies it to the mannequin. A failed lookup logs a category-specific error and does not crash.
3. **Face.** `CharacterCreate:SetFace` selects mapping ids 6 (Human male), 9 (Human female), 12 (Pixie male) and 15 (Pixie female) in slot 9, the Caucasian head of that pair, before the player touches anything (client Lua; ids checked against the mapping table).
4. **Randomize.** The first time a race and gender is chosen the Lua picks a random row in every used category and applies it, then keeps the choices so Back and Next do not lose them. Changing race or gender clears them (client Lua).
5. **Names.** The name screen holds the first-name list for the chosen gender and the prefix and suffix lists from the glossary tables (`SetMaleFirstNameData` and its siblings), a Random button that fills all three and a custom-name mode that first shows the approval notice. `CreationNameAccepted` joins first name, prefix and suffix with the format `%s %s%s` (`00c14000`).
6. **Store.** State 39 gathers the nine chosen ids and fills the pending record. The client's own log lines name its fields (`007100c0`): `FirstName`, `LastName`, `TemporaryFirstName`, `TemporaryLastName`, then `Model`, `Head`, `Hair`, `ModelCustomization`, `FacePaint`, `SkinTone`, `ItemHands`, `ItemFeet`, `ItemLegs`, `ItemChest`, `ItemHead`, `ItemShoulders`, `EyeColor`, `HairColor` and `Location`. The state sets `Location` to 1.
7. **Reply.** The create reply (`007100c0`) has a result and the new entity key. Result 1 is success. Results 4, 5 and 7 (name taken, temporary name taken, invalid name) go back to the name screen with a message; result 6 (too many characters on the account) goes back to the select screen; results 2, 3 and anything else show an "unable to create" message and go back to the appearance step. The names the client logs are `FailureUnableToCreateCharacter` (2), `FailureUnableToProcessRequest` (3), `FailureNameAlreadyTaken` (4), `FailureTemporaryNameAlreadyTaken` (5), `FailureTooManyCharactersOnAccount` (6) and `FailureInvalidName` (7). The numeric error tables that would carry message text are header-only in the supplied client, so the client's own switch is the vocabulary.

#### Appearance: from a choice to a drawn character

1. **A choice is an item definition.** Every creator option row points (through `ITEM_ID`) at a row of `CharacterCreateItemMappings.txt`, whose `TYPE` says what kind of thing it is and whose `ITEM_ID` names an item definition. Skin tones, hair styles, hair colors, eye colors, facial hair, face paint, wings and face types resolve to **type 12** definitions (the style-card type; factory `00724a80` builds `ItemDefinitionStyleCard`). Clothes resolve to a **clothing set** of three ordinary type 1 definitions (feet, legs, chest). In a type 12 definition `PARAM1` is a customization slot code and `PARAM2` is an index into a small lookup file (`HairMappings.txt`, `HeadMappings.txt`, `SkinToneMappings.txt`, `FacePaintMappings.txt`, `ModelCustomizationMappings.txt`), each loaded into an `IdToStringLookup` by `00724d80` and read by `00725a70` (client resource, recomputed in this pass).
2. **The slot codes line up with the live store.** For skin tone, hair, hair color, eye color, facial hair, face paint and wings the definitions' `PARAM1` takes 1, 2, 3, 4, 5, 6 and 7. The live customization store (`PlayerCustomization_ApplyChangeRecord` `00bd8e00`) switches on a record id 0 to 8: id 0 applies a head attachment (equipment slot kind `CustomizationHead`), 2 a hair attachment together with the hair tint (`CustomizationHair`), 3 sets the hair tint, 4 the eye tint, 5 and 7 apply a model customization (`CustomizationModel`: facial hair and wings), 8 sets a numeric body value, and ids 1 and 6 take a string and go to other calls. The pending-commit code gives string parameters to slots 0, 1, 2, 5 and 6, which fits skin-tone and face-paint texture aliases (R for the 1 and 6 assignment). The equipment-slot table carries the three kinds as slots 18 to 20 (client resource `EquipmentSlotDefinitions.txt`). Face types are the exception: the twelve standard face-type definitions use `PARAM1` 9 (the creator's own slot number for the head) while the four rows that name a "China" head use `PARAM1` 0 with category 141 (client resource).
3. **The store.** `PlayerCustomization` keeps eight committed slot values at `+0x44..+0x60` and seven pending values at `+0xb4..+0xd0`, all starting at -1, and ten name strings. `00bd8b20` commits pending values into committed and reports what changed; `00bd8a00` reverts. A change record is `{Id, StringParam, Param, ItemId}` addressed by a 64-bit character guid and a preview flag. For the local player a preview commits nothing and a real change writes the committed slot and stores `{Param, ItemId}` in the id-keyed map; a preview for a remote character is ignored (`00bd8e00`). `Slots_CommitChangedApply` (`00bd9560`) applies each changed slot when the pending set is accepted.
4. **Two ways records arrive.** `PlayerUpdatePacketCustomizationChange` (family 35, sub-opcode 39, constructor `0091d5f0`) carries a list of change records for one character; each goes through the function above and, unless the packet is a preview, a composite effect plays on the actor. `PlayerUpdatePacketCustomizationData` (35, sub 65, `0091d4d0`) replaces the whole stored set (`00bd9730`: remove all rows, copy each incoming row, copy committed values into pending, reset the string slots) (client).
5. **Hair and hats.** Hair models are chosen by template, `<race>_<gender>_hair_<style>_<hairtype>.adr`. `<race>` becomes `human` or `fairy`, `<gender>` the descriptor `m` or `f`, and `<hairtype>` is chosen by `NewHatHairAttachmentGroup` (constructor `01051490`, selector at `+0xa0`). With no match the builder falls back to `<race>_<gender>_hair_eyebrows_bald.adr` (`0070ca40`, `01051580`). Equipped headgear can therefore swap the hair variant. Which hat sets which `<hairtype>` is not recovered.
6. **Tints and textures.** Hair and eye colour are tint ids (`HairTint`, `EyeTint` properties, `00958110`, `00958130`; the creator's colour rows carry the tint in `ICON_TINT`). Face paint and skin tone are texture aliases (`skintone1` to `skintone8`, `skull`, `risingsun`, ...). Clothes carry their tints in the definitions.
7. **Remote players.** A player update (family 35) spawns or updates a `ProxiedCharacter`; its attachments go through `ProxiedCharacterAttachmentGroup`. Temporary appearance is sub-opcode 14 (`PlayerUpdatePacketUpdateTemporaryAppearance`, handled by `ProxiedCharacter::SwitchToTemporaryAppearance` `0096bd90`) and its removal 15; scale is 13, head size 30 (`HeadInflationScale`), a rename 19, a tint-alias change 52, a title 40. Equip changes are 35/6 and 35/7 ([[Items and Inventory]]).
8. **Slot checks.** Every attachment group runs `ValidateSlots` and `CheckDuplicateSlots` (log text "multiple items in equipment slot"); a group that cannot load logs through `BaseAttachmentGroup::FailedGroupLoad` ("AGR file name: %s") and the character shows without it (client S0152, S0380).

#### Character select

The select screen is the Scaleform movie `characterSelectV2` (32,799 bytes) driven by `CharacterSelect` (client Lua). `Populate` reads the `BaseClient.AccountInfo` data source and calls the movie's `addCharacter(index, guidText, nameText, "character")` once per character, then `addCharacter(300, 0, "Add a Character", "add")`. The data source's columns are IsMember, MaxCharacters, IsAdminAccount, HeadshotUrl, CharacterGuid and CharacterName. Events from the movie: `SelectCharacter` (the mannequin changes, `CreationSelectCurrentCharacter(guid)`), `goCharacter` (accept), `deleteCharacter` (selects it and raises the deletion warning) and `addCharacter` (see Overview).

The mannequin shown is built from the saved creation attachments (feet, legs and chest of the creation clothing set, hair and face paint), not from the job gear the character last wore (vault, Native Character Selection Labels).

The record the client reads for each row (`OnRequestCharacterList` `00719550`, from its log format) holds: guid, name, model id, head, hair, model customization, face paint, skin tone, eye tint id and hair tint id, plus a list of attachments (`CharacterAttachmentData`, helpers `00712110`, `00712340`, `00712440`). On the login channel each entity travels as a key, a server id, a status and a base64 payload (`ReadEntityRecord` `00fa4370`).

#### Jobs: the profile model

- **Definitions arrive once.** Reference data 44/3 (`PlayClient::DispatchReferenceData` `00928d00` calls `ReferenceDataPacketClientProfileData_Parse` `00a3a900`) rebuilds a 50-bucket map: a 32-bit row count, then per row a key and a record of 11 `i32`, two single-byte flags and a counted list of rows of two `u32` (`00a3a2b0`, `008fc8c0`). After it the client refreshes `BaseClient.Profiles` and `.Profiles.Filtered`. The reader does not print field names; the data source columns suggest them (see Data and formats). The client defines no job: names, descriptions, badges, types and members flags all come from the server. The client does ship the **job ids** it filters items by (`ItemFilterOptions.txt`) and the **rank requirement table** (`Requirements.txt`).
- **Per-character job state.** Each job the character has is a `ClientPcProfile` node in the self-data body (message id 12, read by `00944fe0`). It carries rank, percent, stars and the ability sets. The node is then updated by 38/14, 38/18 and 38/21. The active profile key is read from the player object (`+0x504`, `+0x508`, the same two words the client echoes in `ActivateProfileFailed`).
- **The job browser.** `JobBrowser:Populate` (client Lua) walks `BaseClient.Profiles` and calls the movie's `AddClass(id, type, name, description, availablePoints, rank, exp, icon, badge, button, membersOnly, hint, trial, trialExpired)`. When a profile type is passed (as on the minigame start screen), `ValidateJobForStartScreen` hides a job whose trial has expired unless the player is a member.
- **Selecting a job.** The movie's `SelectJob` event calls `JobScreen.SelectJob(id)` only if `Ui.GetActiveProfileId()` differs from the id (client Lua). `JobScreen` is a native Lua module (`00c0a760`, five entries: `PopulateJobList`, `SelectJob`, `PopulateJobInfo`, `PurchaseAbility`, `GetJobCount`). `SelectJob` (`00c0a630`) calls the client object's virtual slot 36, `PlayClient::ChangeProfile` (`00938e10`), which builds and sends `CommandPacketSetProfile` (command family 26, sub-opcode 13, one `i32` profile id), then `CommandPacketRequestRewardPreviewUpdate` (26, sub 37), then raises the Lua event `ActivityPortal:OnProfileChange`. The equipment screen has its own `EquipScreen.SelectJob` (`00c10b20`). A second sender, `PlayClient::SetProfileByItemDefinitionId` (`0092ef50`), sends 26/36 with an item definition id, so an item can name a job to switch into; no caller of it was found.
- **The reply.** The server answers with `ClientUpdatePacketActivateProfile` (family 38, sub-opcode 21). The client reads it as two `i16` header values, a length-prefixed profile record, a list of `CharacterAttachmentData` and three more `i32` (`0091b080`, run by `00925550`). The handler (`009392c0`, case 0x15) builds a `PlayerSwitchProfileAttachmentGroup` for the local player (`00965f90`), runs the equip-screen completion hook (`00bdb400`) and refreshes the profile data sources and the group list. When the group is ready, `ProcessProfileSwap` (`00bfb620`) checks every **required equipment slot** (Chest and Legs in the shipped slot table) for an item. If one is missing it logs "Player %s got a profile with no item for the required slot %d!" and marks the group failed, so the new outfit does not appear. Otherwise it plays the `ap_profile_swap` animation (looked up by name, with a default duration if the animation is missing) and the swap completes through `ProxiedCharacter::OnProfileSwapComplete` (`00963410`, which logs `JobHandler:OnProfileSwapComplete`) (client).
- **Failure.** `PlayClient::SendActivateProfileFailed` (`0092f260`, virtual slot 41) sends command 26/26 with the two profile words `+0x504` and `+0x508`. No direct caller was found. The reply meaning is not recovered (client).
- **Trials.** `PacketClientTrialProfileUpsell` (top-level opcode 145; body `i16`, `i32`, `i32`) makes the client drop a pending request object, format the keys `TrialJobUpsellTextId_%d` and `TrialJobUpsellBundleTextId_%d` and call `HandlerJob:OnJobTrialExpired` (client `009471a0` case 0x91, `008d61c0`). The data source has `Trial` and `TrialExpired` columns.
- **Gates.** `Requirements.txt` has 24 `CheckRankLastProfileByType` rows (a profile type and a rank, with an operator code 4 or 6), `CheckCurrentProfile` rows (including profile ids 120 and 121), `SourceVipRank`, `CheckRenamePending`, `CheckNudge` and `AlwaysFails`. `ClientRequirementCheckLastPlayedProfileOfTypeRank::Evaluate` (`00ce7980`) finds the character's profile of the stored type and compares its rank. The types seen are 2, 3, 9, 10 and 12 (the vault reads them as combat, Postman, Miner, Chef and Blacksmith jobs, R); types 3, 9, 10 and 12 use ranks 5, 9, 13 and 17, and type 2 uses 5, 8, 9, 10, 12, 13, 16 and 17 (client resource; the meaning of operators 4 and 6 is inferred, R). These are item and slot gates; the client has no job-unlock table.
- **Ids.** The job ids the client filters items by are Adventurer 1, Ninja 2, Postman 4, Medic 11, Wizard 12, Miner 14, Blacksmith 16, Warrior 32, Archer 35, Brawler 43, Chef 45, Kart Driver 48, Demo Derby Driver 49, Soccer Star 52, Card Duelist 120, Pet Trainer 121 and Fisherman 137 (client resource `ItemFilterOptions.txt`). Seventeen ids match the seventeen job pages.

#### Levels, stars and the level-up notice

- **No curve in the client.** The client has no experience table and no star-to-rank table. It stores a rank and a percent per job and draws them; the server decides both. The rank is capped at 20 in the UI (client Lua), the equipment-slot table gates the Star Rune slot at rank 10 and the Moon Rune at rank 20 (client resource), and the wiki gives the same cap with a members-only limit of 4 for most jobs.
- **Updates.** `ClientUpdatePacketUpdateProfileExperience` (38/14) carries six `i32` after the header: a profile id and five values (`008d86e0`). The handler finds the matching profile node by id. If the second value differs from the node's rank and is not 1, it sends an activity-list refresh request (`008a2330`, service 167, sub 4). It then stores the second value in the node's rank (dword 5), the third in dword 6 (the percent), and the fourth, fifth and sixth in dwords 7, 9 and 10 (star counts; R). Finally it calls `PushJobSkillPoints` (`00900a20`, which calls `HandlerJobBrowser:SetJobSkillPoints`) and refreshes the profile data sources. `ClientUpdatePacketUpdateProfileRank` (38/18) carries a profile id and a rank (`008d8910`). When the rank is above the stored one the client calls `NotificationHandler:JobLevelUp(jobType, jobName, iconId, jobLevel, badgeImageSet)` with the profile name resolved through the locale, the node's dword `0xb` as the icon and dword `0xc` as the badge set, then refreshes the sources and the group. A rank that merely differs triggers the same activity-list refresh (client `009392c0`, cases 14 and 18).
- **Order matters.** Because 38/18 compares the new rank with the stored one and 38/14 overwrites the stored one, the rank message must come first or no banner shows. The reconstructed Rust slice already sends 38/18 before 38/14 (vault, `src/profiles.rs`).
- **The banner.** `JobLevelUp` shows only while the main HUD is up (otherwise it queues in `LevelUpNotificationQueue` and flushes when the HUD returns), raises the first-time event `JobsLeveling` from rank 2, and picks the banner movie and sound **by the profile's badge image set id**: 10 `levelup_fighter`, 11 `levelup_service`, 12 `levelup_combat_caster`, 13 `levelup_combat_support`, 19 `levelup_explorer`, 20 `levelup_treasure`, 32 `levelup_combat_ranged`, 1783 `levelup_tcg`, 1785 `levelup_pettrainer`, 1999 `levelup_crafter`, 46 `levelup_cardriver`, 4842 `levelup_global_generic`, 4843 `levelup_fishingjob`. A badge id not in that list shows no banner. The sound table has entries only for ranks 1, 4, 8, 12 and 16 (client Lua `NotificationHandler:JobLevelUp`).
- **Stars.** Stars are the experience currency: earned by minigames, quests, collections and in-world combat, and counted only for the job in use (client locale 2583709755; wiki, Stars). Combat jobs also spend stars on ability ranks (36/7, [[Combat and Abilities]]). Some collections give wildcard stars that level any job (wiki). Non-members can bank stars and coins in a deferred bank shown on the XP button (vault, Reward Toast and Deferred Bank Findings; [[Rewards and Collections]]).
- **Other uses of the rank.** Items gate on `MIN_PROFILE_RANK`; combat tiers 3 to 5 need levels 5, 9 and 13 (wiki, Combat); an ability slot has a level requirement (`00c43cd0`, [[Combat and Abilities]]); the level is drawn in group rows (`GroupHandler:SetJobLevelExp`).

#### About Me, titles and inspect

- **About Me.** `AboutPlayer:Populate` (client Lua) fills the panel from `BaseClient.PlayerInfo` (`CharacterDataSource`, 28 columns: Name, Guid, NumCoins, Wallet, JobLevelsGained, CollectionsCompleted, QuestsCompleted, CreationDate, Age, PlayTime, IsMember, Gender, Race, VipRank, VipProgress, Viptitle, CoinCount, SCCount, PlayerIsMale, PlayerBirthday, Signature, Interests, QQAccount, MsnAccount, AboutMe, PlayerAge, City, ExtraDataPopulated) and the stat list from `BaseClient.PlayerStats` (a stat id and a value per row, with icon and name looked up in the stat-definition table). The social fields are used only in the China environment (client Lua `Ui.IsChinaEnvironment()` checks).
- **Titles.** The title packets are family 152 (`BasePlayerTitlePacket`, opcode `0x98`). Received: 1 add one title, 2 remove one title, 3 replace the list (`HandlePlayerTitlePacket` `00b4a3a0`). Sent: 4 request-select with a title id (`Ui.SelectPlayerTitle` through `SelectPlayerTitle` `00c18c80` and `SendPlayerTitleRequestSelect` `00b4a880`) and 5 refresh request (`00b4a910`). Sub-opcode 7 is a title update with a list of ints, built for social posting. A title record is four `u32` after the header. The data source columns are Title Id, Title Type, String, Is Selected, Preview Name and Icon Id. The type decides the format: 1 prefix, 2 alias (first name, title, last name) and 3 suffix, using the keys `PlayerTitleFormatPrefix`, `PlayerTitleFormatAlias` and `PlayerTitleFormatSuffix` (ids 3916, 3917 and 3919 in `CodeStringMappings.txt`), and the title text is the entry `Global.Text.<id>` (`0097c390`). The player update 35/40 (`PlayerUpdatePacketPlayerTitle`) tells the zone which title id a player shows; when it names the local player the client stores it as the selected title (`00b49a10`). The locale holds three templates, `[*title*] [*fullName*]`, `[*fullName*][*title*]` and `[*firstName*] [*title*] [*lastName*]`; which belongs to which key is by arrangement, not read (R). Titles named in client text include "Marksman" (Archer questline), "Arcane Master" (Wizard questline), "The Sniper Extraordinaire" (Archer level 20) and "Epic Enchanter" (Wizard level 20) (client locale; job pages).
- **Inspect.** `BaseClient.Inspect` has 30 columns (0 to 0x1d) for the target's identity, stats, jobs and profile text; `BaseClient.Inspect.Jobs` has five (id, rank, name, icon, badge); `BaseClient.Inspect.Items` reuses the item columns plus `IsInMarketplace`. The inspect family is 149: sub 1 start inspect (constructed in the base dispatcher), sub 2 stop inspect (sent by `StopInspect` `009347d0`) and sub 4 `WorldInspectReplyPacket` (received). `PlayerInspect:Show` opens the panel (`008c6030`). The request that begins an inspect was not traced.
- **Rename.** A paid rename uses `CharacterNameHandler` and the `NameChange.*` data sources (`isCheckPending`, `isRenamePending`, `Name`, `Status`, `StatusString`, `CanChangeName`); the zone learns the new name through 35/19 (`PlayerUpdatePacketRenamePlayer`, which also updates the About Me source). The request side belongs to [[Login and Account]] (`net.namechange`).

### Data and formats

**Client resource files for creation** (counts are data rows, recomputed from the supplied client's `Resources` folder in this pass):

| File | Contents | Source |
|---|---|---|
| `CharacterCreateCharacterTypes.txt` | 4 rows `CHARACTER_TYPE_ID^CHARACTER_TYPE_NAME_STRING_ID^RACE^GENDER`: 1 Pixie Female (430560), 2 Human Female (430558), 3 Human Male (430557), 4 Pixie Male (430559). They bind one-to-one to the four `IS_VALID_FOR_PC` rows of `Models.txt`: model 1 `human_m.adr`, 2 `fairy_m.adr`, 60 `human_f.adr`, 61 `fairy_f.adr`. | client resource; parser `00980370` |
| `CharacterCreateCategories.txt`, `CharacterCreateCustomizationCategories.txt` | 9 rows each, `CATEGORY_ID^CATEGORY_TITLE_ID^DATA_SOURCE_NAME_SUFFIX^RACE^GENDER`. First file: 1 FaceType, 2 SkinTone, 3 HairStyle, 4 HairColor, 5 EyeColor, 6 WingsPixie, 7 FaceHair, 8 Clothes, 9 FacePaint. Second file (the one the Lua binds, `CharacterCustomizationCategory_DS`): 1 SkinTone, 2 HairStyle, 3 HairColor, 4 EyeColor, 5 FaceHair, 6 FacePaint, 7 WingsPixie, 8 Clothes, 9 FaceType. `FaceHair` is Human/Male only, `WingsPixie` Pixie/All, everything else All/All. The second file's ids 1 to 9 are exactly the nine slot numbers of `ChangeItem`, which is why it is the one the Lua binds; no loader was traced for the first file. | client resource; parser `00980540` |
| `CharacterCreateCustomization<Suffix>.txt` (9 files) | 253 option rows, `GUID^ICON_ID^ICON_TINT^ITEM_ID^RACE^GENDER^NAME^DESCRIPTION`, GUIDs 1000001 and up per file. `ITEM_ID` is a `CharacterCreateItemMappings` id. Per race and gender: FaceType 4, SkinTone 8, HairStyle 8, HairColor 10, EyeColor 10, Clothes 9, FacePaint 12 (female) or 7 (male), Wings 6 (Pixie only) and FaceHair 7 (Human male only). Totals per type: Pixie Female 67, Human Female 61, Human Male 63, Pixie Male 62. Each file becomes a data source `BaseClient.CharacterCreateCustomization<Suffix>` (`0097f570`, `009809f0`). | client resource (recomputed) |
| `CharacterCreate/CharacterCreateItemMappings.txt` | 272 rows (68 per race and gender), `ID^TYPE^RACE^GENDER^DESCRIPTION^ITEM_ID`; race 1 Human, 2 Pixie, gender 1 male, 2 female. Types: 0 skin tone (24 rows), 1 hair style (62), 2 hair color (40), 3 eye color (40), 4 facial hair (8), 5 face paint (34), 6 wings (12), 7 clothes (36), 8 face type (16). `ITEM_ID` is an item definition id (types 0 to 6 and 8) or a clothing-set id (type 7). 223 rows are reachable from the creator tables and 49 are not (type 0: 16, type 1: 30, types 3, 4, 5: one each). | client resource; loader `00723860` |
| `CharacterCreate/ItemDefinitions.txt` | 302 starter definitions in the 51-column definition schema ([[Items and Inventory]]): 203 of type 12 (appearance cards) and 99 of type 1 (clothing). 42 rows carry `CATEGORY_ID` 141: the six standard skin tones (ids 77484 to 77489), 30 hair styles named `...FRC...` (78037 to 78066), four "China" head rows (78086 to 78089) and two extra skin tones (78090, 78091). Mapping rows 2 to 5 (Human male skin tones 3 to 6) name definitions 78092 to 78095 that are not in this file. | client resource; loader `00724b80`, factory `00724a80` |
| `CharacterCreate/CharacterCreateItemClothingSets.txt` | 36 rows, `ID^DESCRIPTION^DESCRIPTION_ID^ICON_ID^FEET_ITEM_ID^LEGS_ITEM_ID^CHEST_ITEM_ID`, nine sets per race and gender: Human male Collar, Hoodie and Tee Shirt in three colourways each; Pixie female Casual, Party and Sporty in three each; Pixie male Athletic, Fitted and Skater in three each; Human female nine distinct sets. Each set names three definitions of slots 5, 4 and 3. | client resource; loader `00723c80` |
| `CharacterCreate/CharacterCreateItemStartingSets.txt` | **Header only, no rows.** Columns `ID^DESCRIPTION^RACE^GENDER^MODEL_ID^SKIN_TONE_ID^HAIR_STYLE_ID^HAIR_COLOR_ID^EYE_COLOR_ID^FACIAL_HAIR_ID^FACE_PAINT_ID^WINGS_ID^CLOTHING_SET_ID^FACE_TYPE_ID`. The parser (`007257d0`) puts race at `+0xac`, gender at `+0xb0` and the nine ids from `+0xb8`. `CharacterCreateItemTypes.txt` is also header only. | client resource; loaders `00723e90`, `00723a70` |
| `CharacterCreate/CharacterCreateNames.txt` | 1,318 rows: two format rows (`character.name.format` `[*char_firstname*] [*char_lastname*]`, `character.lastname.format` `[*char_lastname_prefix*][*char_lastname_suffix*]`), 425 female and 415 male first names, 234 last-name prefixes and 242 suffixes. | client resource; loader `007236c0` |
| `CharacterNames.txt` | 1,335 rows in the same glossary shape (435 female and 418 male first names, 480 last-name parts); not identical to the file above. The name wheel of `CharacterCreationManager` reads this one (`009829f0`). | client resource |
| `HairMappings.txt`, `HeadMappings.txt`, `SkinToneMappings.txt`, `FacePaintMappings.txt`, `ModelCustomizationMappings.txt` | Lookup files, 83, 8, 8, 51 and 45 rows, `ID^FILENAME` or `ID^TEXTURE_ALIAS`. Hair: `<race>_m_hair_afro_<hairtype>.adr` and so on. Head: 1 to 3 `<race>_<gender>_head_caucasian/african/asian.adr`, 4 `<race>_<gender>_head_china.adr`, 5 to 8 fixed China heads. Skin: `skintone1` to `skintone8`. Face paint: `none`, `skull`, `risingsun`, `moons`, `lightning`, `goth`, `bridge`, `bigstar`, `butterflys`, `colorstars`, `flames` and seasonal entries. Model customizations: `human_m_facialhair_beard/chinwhiskers/goatee/mustache/sideburns/soulpatch.adr` and `fairy_f_wings_*.adr`. | client resource; `IdToStringLookup` (`00725a70`) |
| `CharacterCreatePresets.txt` | 4 rows `ID^DESCRIPTION^MODEL_ID^ITEM_GROUP_ID^GENDER^HEAD_ID^HAIR_ID^HAIR_TINT_ID^EYE_TINT_ID^SKIN_TONE_ID^FACE_ID^FACIAL_HAIR_ID^RACIAL_CUSTOMIZATION_ID^...`: starting presets for model 1 (hair tint 134, eye tint 196, skin 1), 60 (150, 197, 217), 2 (189, 207, 221) and 61 (135, 123, 227). | client resource; loader `010524a0` |
| `Models.txt`, `RaceDefinitions.txt`, `GenderDefinitions.txt`, `Mannequins.txt` | 3,743 model rows (4 valid for players), 65 race rows (Human 1 and Pixie 2 are the player races; their `DESCRIPTOR` in `Models.txt` is `human` and `fairy`), 2 gender rows (1 `m`, 2 `f`), 2 mannequin rows. `RaceSets.txt` and `RaceSetIds.txt` are header only. | client resource; loaders `007291c0`, `0072eba0` |
| `ErrorCodesCharCreate.txt`, `ErrorCodesCharDelete.txt`, `ErrorCodesCheckName.txt`, `ErrorCodesLoginResult.txt` | Header only (36 or 37 bytes). The client falls back on its own switch and on the `Char_*` keys of `CodeStringMappings.txt` (for example `Char_Name_Taken_PS3` 432106, `Char_Server_Failure_PS3` 432104, `Char_Unable_Log_Server_PS3` 432497). | client resource |
| `DiplomacyRanks.txt`, `DiplomacyTitles.txt` | Header only: `TITLE_ID^REQUIRED_RANK^REQUIRED_RANK_COUNT^TITLE_STRING_ID^ICON_ID`. No loader was traced for them in this pass. | client resource |
| `EquipmentSlotDefinitions.txt` (rows used here) | Slots 3 Chest and 4 Legs are required; 14 Star Rune needs rank 10 and 15 Moon Rune rank 20; 17 WholeBodyCostume carries requirement 241; 18 `CustomizationHead`, 19 `CustomizationHair` and 20 `CustomizationModel` are the three appearance kinds. | client resource |
| `Requirements.txt`, `RequirementGroups.txt`, `RequirementSets.txt` | 43, 39 and 34 rows; the 24 `CheckRankLastProfileByType` rows are described under Jobs. | client resource |
| `ItemFilterOptions.txt` | The job ids (see Jobs). | client resource |

**The creation request record** (client `00954bd0` serializes it; `007100c0` logs the fields by name). The record is the pending creation structure of `ClientAccountManager` starting at `+0x5387c`. Four string objects of 0x24 bytes each, then 21 signed 32-bit integers in this order:

| Order | Field | Source of the value |
|---|---|---|
| strings 1 to 4 | `FirstName`, `LastName`, `TemporaryFirstName`, `TemporaryLastName` | the name screen; the temporary pair is what shows until a custom name is approved (client, R for the exact split) |
| 1 | `Model` | the race-and-gender model id (1, 2, 60 or 61) |
| 2 to 6 | `Head`, `Hair`, `ModelCustomization`, `FacePaint`, `SkinTone` | taken from the resolved appearance definitions (`def+0x218`, read here as `PARAM2`, R): head mapping id, hair mapping id, model-customization id, face-paint id, skin tone 1 to 8 |
| 7 to 12 | `ItemHands`, `ItemFeet`, `ItemLegs`, `ItemChest`, `ItemHead`, `ItemShoulders` | clothing item definition ids from the chosen clothing set (feet, legs, chest); the other three are not set by the creator (R) |
| 13 to 18 | six integers at record offsets `+0xc0..+0xd4` | not logged and not named by any code read here |
| 19, 20 | `EyeColor`, `HairColor` | tint ids |
| 21 | `Location` | set to 1 by the store state |

The first eight integers are appended four bytes at a time and the remaining thirteen through a second helper (`009548c0`); that is the client's own proof of the count. The vault's native proof lists the same shape ("four strings and 21 signed little-endian integers for appearance, clothing, tints and location") and a saved test character whose parsed fields were model 1, a Caucasian head, a shaggy hair style, hair colour 10, eye colour 51, skin tone 8, skull face paint, no model customization and three clothing items (feet, legs, chest) (vault, Character Creation Audit, Native Creator Delivery Proof).

**Reference data 44/3: job definitions.** Per row: a key, then eleven `i32` (at record offsets `+0x00` to `+0x20` and `+0x28`, `+0x2c`), two bytes (`+0x24`, `+0x25`) and a counted list of two-`u32` rows (client `00a3a2b0`). The `BaseClient.Profiles` columns, in order, are: `Id`, `Name`, `Description`, `Type`, `Rank`, `StarsAvailable`, `Icon`, `BadgeImageSet`, `ButtonImageSet`, `AbilityBgImageSet`, `MembersOnly`, `StarsEarned`, `RankPercent`, `Hint`, `Trial`, `TrialExpired` (client Lua `Profile_DS`, column names `00c8f2b0`). The definition supplies the first ten and the hint, the character supplies rank, stars and percent, and the trial flags are combined at run time. Which record field is which column was not read (R). In `JobHandler` the `Type` column is also used to special-case a slot for type 2 (combat) jobs (client Lua).

**Per-profile character state.** Rank, percent and stars-available are three consecutive `i32` in the self body's profile block, in that order, in the order our Rust slice writes them (R; vault, `src/profiles.rs`, not independently checked against any original payload); the client's profile node keeps rank at dword 5, percent at 6, and further counters at 7, 9 and 10 (client, 38/14 handler).

**Packet shapes.**

| Packet | Shape |
|---|---|
| 38/14 UpdateProfileExperience | `u16 38`, `u16 14`, `i32 profile`, `i32 rank`, `i32 percent`, three more `i32` (zero in our slice) |
| 38/18 UpdateProfileRank | `u16 38`, `u16 18`, `i32 profile`, `i32 rank` |
| 38/21 ActivateProfile | `u16 38`, `u16 21`, `u32 length` + profile record, a list of attachment records, three `i32` (client reader `0091b080`) |
| 38/20 DeleteProfile | `u16 38`, `u16 20`, one `i32` profile id (client reader `008e3fc0`); removes the profile from the character's list |
| 35/39 CustomizationChange | header, target guid, preview byte, list of change records |
| 35/65 CustomizationData | header, list of change records for the owner |
| change record | `i32 Id` (0 to 8), string `StringParam`, `i32 Param`, `i32 ItemId` (node fields: `Id` at `+0`, string at `+4`, `Param` at `+0x14`, `ItemId` at `+0x18`) |
| 152 title packets | header `i16` plus `i32` (the sub-opcode); add: four `u32` (id, kind, text id, icon); remove: one `u32` (the key); update-all: a counted list of entries; select request: one `u32` title id |

The byte layout of the profile record, the attachment records and the change-record string field is taken from the client's readers only (`0091b080` and the helpers named above). No outside serializer is used to confirm it, so the exact string and length encoding of those three records is verified only by the client accepting our native-run output (see Evidence).

**The seventeen jobs.** Ids from the client's item filter; class, members flag and cap from the job pages (page flags mix client and wiki; see Open questions).

| Id | Job | Class | Members (job page) | Unlock hint (client or wiki) |
|---|---|---|---|---|
| 1 | Adventurer | freestyle | no | the old tutorial gave it automatically; which job a new character starts on is our own server policy (R), and the original starting set is unknown |
| 2 | Ninja | combat | yes | a trainer in Lakeshore; client text also needs level 20 Archer |
| 4 | Postman | freestyle | yes | an introduction quest from postboxes |
| 11 | Medic | combat | yes | a nurse in Sanctuary; client also names a doctor |
| 12 | Wizard | combat | yes | a wizard in the Robgoblin Camp; client text also needs level 20 Medic |
| 14 | Miner | crafting | no | trainers; free to level 20 per client text 2640708938 |
| 16 | Blacksmith | crafting | yes | a smith in Snowhill; client text 2640708938 lists it as free to 20 |
| 32 | Warrior | combat | yes | a drill sergeant in Snowhill; client text also needs level 20 Brawler |
| 35 | Archer | combat | no | a trainer at the Greenwood Camp |
| 43 | Brawler | combat | no | three trainers, one per starting area |
| 45 | Chef | crafting | yes | trainers |
| 48 | Kart Driver | freestyle | yes | trainers and minigame objects |
| 49 | Demolition Derby Driver | freestyle | no | trainers and minigame objects |
| 52 | Soccer Star | freestyle | no | trainers and minigame objects |
| 120 | Card Duelist | freestyle | no | card shops and objects |
| 121 | Pet Trainer | freestyle | yes | removed from new players (wiki, December 2009) |
| 137 | Fisherman | freestyle | no | fishing objects |

Every row has maximum level 20 on its job page. The client's `Requirements.txt` uses 120 and 121 as current-profile checks (client resource). Which of these jobs the original server listed on a new account is unknown (an earlier vault count of 16 profiles came from a recorded server payload and is not used); the starting set is our own policy (R).

**Locale ids used by the creator and select screens** (client locale): 430550 name in use, 430551 finished editing, 430552 Customize Character, 430553 Entering Free Realms, 430554 custom name approval, 430555 "To play Free Realms, you need to create your character", 432096 to 432112 name length messages, 434046 to 434053 category titles, 435820 more characters, 435948 deletion warning, 435950 custom-name notice, 436106 Add a Character.

### Network

The login channel and the world tunnel are covered in [[Login and Account]] and [[Gateway and Transport]]; the table lists what the character feature builds and reads. Directions are proven when a sender builds the packet (C>S) or a dispatcher parses it (S>C).

**Login channel (before the game starts).** The client's `ExternalLoginApi` builds these requests with a leading opcode dword; replies arrive by name and fill events (`00fa97d0`).

| Opcode or name | Direction | Body | Handler or sender |
|---|---|---|---|
| 0xd ServerListRequest | C>S | none | `00fa91e0` |
| 0xb CharacterSelectInfoRequest | C>S | none | `00fa91a0` |
| CharacterSelectInfoReply | S>C | a list of entities: key (64-bit), server id, status, payload (base64 text) | `00fa7730`, record reader `00fa4370`; `OnRequestCharacterList` `00719550` |
| 5 CharacterCreateRequest | C>S | the creation record above, server id | `00fa9db0`, `00956590`, record writer `00954bd0` |
| CharacterCreateReply | S>C | status, 64-bit entity key | `00fa5ce0`; `007100c0` (statuses 1 to 7) |
| 7 CharacterLoginRequest | C>S | the entity key pair and the server | `00fa9eb0`, `0070a980` |
| CharacterLoginReply | S>C | status, entity key, server, gateway address, ticket, crypto key; statuses 1 success, 2 account in use, 6 unable to load the character, 7 unable to assign to a gateway; any other status exits the game | `00fa5d80`; `00711220` |
| 9 CharacterDeleteRequest | C>S | the entity key pair | `00fa9f80`, `00708660` |
| CharacterDeleteReply | S>C | entity key, status, payload | `00fa5e50`; `007198b0` |
| 210 PacketCheckNameRequest | C>S | five strings: first name, last name, name definition, last-prefix definition, last-suffix definition (vault; client constructor `00953950`) | `009564a0`, `00956230`, tunnel `00fa9fe0` |
| 211 PacketCheckNameReply | S>C | first name, last name, an `i32` result, a suggestion list (rows logged as Name, LastPrefix, LastSuffix) | `009566c0`, `00956100`; `OnRequestCheckName` `00710e20` |

Check-name results the client understands (client `00710e20`, text from `CodeStringMappings.txt` and the locale): 1 available, 2 name taken, 3 profane, 5 incorrect total length, 6 server failure, 7 and 8 first or last too short, 9 and 10 first or last too long, -1 and 4 invalid, anything else "unable to log in to the server". There is no case for 11. Our own piece-050 note treats the length thresholds in the messages as the client's statement of the rule (first and last at least 2, at most 14 characters, 15 combined for custom names) and records that the server side is unverified (vault).

**World channel (after login).**

| Fam/sub | Name (client RTTI) | Dir | Body | Handler or sender |
|---|---|---|---|---|
| message id 12 | SendSelfToClient | S>C | whole player body, includes the profile list, appearance and inventory | `009471a0` case 12, `00944fe0` |
| 44/3 | ReferenceDataPacketClientProfileData | S>C | counted job definitions | `00928d00`, `00a3a900` |
| 26/13 | CommandPacketSetProfile | C>S | `i32` profile id | `00938e10`, ctor `008a29f0` |
| 26/37 | CommandPacketRequestRewardPreviewUpdate | C>S | none | `008b6cb0`, sent with every profile change |
| 26/36 | CommandPacketSetProfileByItemDefinitionId | C>S | `i32` item definition id | `0092ef50`, ctor `008a2b20` |
| 26/26 | CommandPacketActivateProfileFailed | C>S | two `i32` (the player's `+0x504`, `+0x508`) | `0092f260`, ctor `008a2ca0` |
| 38/21 | ClientUpdatePacketActivateProfile | S>C | see Data and formats | `009392c0` case 0x15 |
| 38/14 | UpdateProfileExperience | S>C | see Data and formats | `009392c0` case 0xe, ctor `0089de90` |
| 38/18 | UpdateProfileRank | S>C | see Data and formats | `009392c0` case 0x12, ctor `0089e350` |
| 38/20 | DeleteProfile | S>C | one `i32` profile id | `009392c0` case 0x14, ctor `0089e220` |
| 38/15, 38/22, 38/24 | profile ability sets and abilities | S>C | not decoded here | [[Combat and Abilities]] |
| 38/19 | CoinCount | S>C | one `i32` | [[Items and Inventory]] |
| 35/39 | PlayerUpdatePacketCustomizationChange | S>C | target guid, preview flag, change records | `0092f460` case 0x27, ctor `0091d5f0` |
| 35/65 | PlayerUpdatePacketCustomizationData | S>C | change records (replace all) | `0092f460` case 0x41, ctor `0091d4d0` |
| 35/13, 35/30 | UpdateScale, HeadInflationScale | S>C | target guid pair and one float | `0092f460` cases 0xd, 0x1e (readers `008d71b0`, `008d6900`) |
| 35/14, 35/15 | UpdateTemporaryAppearance, RemoveTemporaryAppearance | S>C | not decoded here | `0092f460` cases 0xe, 0xf; `0096bd90` |
| 35/19 | RenamePlayer | S>C | guid and new name | `0092f460` case 0x13, ctor `008dcc10` |
| 35/40 | PlayerTitle | S>C | guid and title id | `0092f460` case 0x28, ctor `008b50a0` |
| 35/52, 35/46, 35/48 | UpdateTintAlias, CustomizeNpc, RemoveNpcCustomization | S>C | not decoded here | `0092f460` |
| 42/10, 42/11, 42/12 | UseStyleCard, PreviewStyleCard, UseStyleCardByItemRecord | C>S | one item value | [[Items and Inventory]] |
| 152/1, 152/2, 152/3 | PlayerTitleAdd, Remove, UpdateAll | S>C | see Data and formats | `00b4a3a0` |
| 152/4, 152/5 | PlayerTitleRequestSelect, RefreshRequest | C>S | `u32` title id; none | `00b4a880`, `00b4a910` |
| 152/7 | PlayerTitleSNSUpdate | C>S | list of ints | `00bb85d0` |
| 145 | PacketClientTrialProfileUpsell | S>C | `i16`, `i32`, `i32` | `009471a0` case 0x91 |
| 149/1, 149/2, 149/4 | Start, Stop, WorldInspectReply | start and reply S>C, stop C>S | not decoded here | `009471a0`, `009347d0`, ctor `008b7790` |
| 167/4 | activity list refresh | C>S | none read | `008a2330`, sent when a profile's rank changes |
| 156/2, 156/3 | PortraitDataRequest, PlayerImageData | C>S, S>C | a 64-bit character id; header, id, flag, image bytes (headshots used by group and account rows) | `00bd3ff0`, `00bd4d30` |

### What the Rust server must do

Items marked (R) are reconstructed policy, not original behaviour.

1. **Run the login service the client expects.** Answer the server list, the select-info request, create, login and delete with the vocabularies in Network. The create reply carries a status and the new 64-bit entity key; the login reply carries a status, the gateway address, a ticket and a key. Use the client's own statuses: create 1 success, 2 and 3 failure, 4 name taken, 5 temporary name taken, 6 too many characters, 7 invalid name; login 1 success, 2 account in use, 6 unable to load the character, 7 unable to assign a gateway (any other status makes the client exit). Delete removes the character and everything it owns (the client says so on screen), then the client rebuilds its list from a new select-info reply (R for that follow-up).
2. **Send the character slot count.** `MaxCharacters` in the account info decides whether "Add a Character" opens the creator (more than one) or shows the membership message (one). The original value is unrecovered; choose a bound and keep it a setting (R).
3. **Answer the name check.** Reply to 210 with 211: availability, the status codes in Network, and a suggestion list when the name is taken. Enforce the thresholds the client messages state (first and last at least 2 and at most 14 characters, 15 combined for custom names) and a profanity filter of our own (R; the server side of these rules is unverified). Support the temporary-name path: a custom name is stored as pending approval while the temporary pair is what the world shows (client strings; the approval workflow itself is R).
4. **Ship a populated starting-set table.** The client cannot draw its four creator mannequins without rows in `CharacterCreateItemStartingSets.txt`; the supplied file is empty. Write our own four rows (one per race and gender) from valid combinations in the creator tables, using the column order in Data and formats, and install them with the client resources. Do not copy a community table (R).
5. **Own the creator rules.** The offered options are the client's tables. Accept a create request only when every submitted id resolves through option row, mapping row and definition for the submitted race and gender, the clothing set belongs to that race and gender, and the textures for the choice exist in our asset set (R; whether the original validated is unknown). Skin tones 7 and 8 have no texture for the Human male Caucasian head in the delivered packs, and skin 8 made a native run fail at the first profile load; the twelve rows that select skin 7, skin 8 or a "China" head are the ones to withhold until their assets are proven (vault; client resource).
6. **Persist the creation record.** Keep the 21 integers and four strings as sent, plus the entity key, so the select-info payload, the self body and the creation attachments can all be rebuilt. The select preview shows the creation attachments, not current job gear, so keep them separately from equipment (vault).
7. **Build the first login.** The self body carries the character's name and appearance, one profile block per job the character owns (rank, percent, stars available), the active profile key and the inventory. Send the job definitions (44/3) before the self body so the data source has names, badges and flags. The first job and the initial unlock set are content decisions; the Rust slice starts on Adventurer (id 1) and unlocks Brawler (43) when its quest line starts (R; vault).
8. **Define every job row.** Each definition needs a name and description id, a type (the combat jobs appear to use 2; the client's requirement table also uses 3, 9, 10 and 12 for the crafting and gathering jobs, R), an icon, a badge image set, a button set, an ability background set, members flag, trial flag and a hint text id (client data source columns). The **badge image set must be one of the thirteen the level-up code knows** (10, 11, 12, 13, 19, 20, 32, 1783, 1785, 1999, 46, 4842, 4843) or no banner shows (client Lua). Our job pages supply names, classes and level gates.
9. **Switch jobs.** On 26/13 check the profile exists and is unlocked, membership and trial state (R), and that the character may switch now (the client text says "before entering a battle or duel", so refuse during a fight; R). Reply 38/21 with the profile block, the attachment list for **that job's equipment including an item in every required slot** (Chest and Legs) or the client logs the missing slot and the outfit never appears, then the three trailing integers (the meaning of the three values is not recovered; our slice currently sends 3001, 4005 and 0, which have no evidence here and are R to be re-chosen and tested against the client). Persist the active profile before replying. Send 26/37's reply if a reward preview is defined; otherwise ignore it (R). Tell the zone about the new look (35/7 style, R) so other players see the change.
10. **Own the XP curve.** The client has none. Decide stars per rank for ranks 1 to 20, how each activity awards stars to the job in use, the wildcard stars of collections and the members-only deferred bank. The Rust slice uses 100 stars per rank, rank = 1 + stars/100 capped at 20, as a placeholder (R). Send ranks above 20 never; at rank 20 send percent 0 (the client shows 100).
11. **Send progression in the right order.** For a gain that raises the rank send 38/18 (profile, new rank) **first** and then 38/14 (profile, rank, percent, star counters); in the other order the stored rank already equals the new one and no banner shows. Each rank change also makes the client request an activity-list refresh (167/4), which the activity service must answer ([[Matchmaking and Lobby]]). Send 38/14 alone for percent-only gains. Keep the three counters the client stores in dwords 7, 9 and 10 explicit once their meanings are settled (R: stars available and stars earned).
12. **Apply members rules.** The client text says anyone can play to level 4 and a new player can try any job to at least level 5, jobs can be bought individually, and a membership unlocks all (client locale 348625404, 2065313732). Send `PacketClientTrialProfileUpsell` (145) when a trial ends, with the text and bundle ids the client formats into keys, and decide caps per job; the free-to-20 jobs named in the Sunstone Valley text are Brawler, Miner and Blacksmith (client locale 2640708938) (R for the combination).
13. **Advanced job gates.** Ninja, Warrior and Wizard may require level 20 in Archer, Brawler and Medic respectively (client text). Enforce it as content policy; the wiki and the live game may not have (R).
14. **Appearance after creation.** Send 35/65 with the full customization set when a character spawns or its set changes wholesale, and 35/39 for incremental changes with the preview flag clear for stored changes and set for a try-on. A style-card use (42/10) must validate ownership and race and gender, persist the new slot value and answer with the change record. Records use the slot ids 0 to 8 of Data and formats. A preview never persists (client: remote previews are ignored).
15. **Temporary appearance, scale and head size.** Send 35/14, 35/15, 35/13 and 35/30 where a feature needs them (costume foods, pets' effects); none of their bodies is decoded here.
16. **Titles.** If titles ship, send 152/3 with the list on login (and 152/1, 152/2 for changes), answer 152/4 by validating ownership, persisting the choice and broadcasting 35/40 to the zone, and answer 152/5 with a fresh list. Each title has a kind (1 prefix, 2 alias, 3 suffix), a text id for `Global.Text.<id>` and an icon. The client asks for the list itself only when its China flag is set (client Lua), so a normal build must push it (R).
17. **Renames.** Use 35/19 to update the name in the zone and the About Me source; keep the temporary-name state until approval.
18. **About Me and inspect.** Fill `PlayerInfo` (28 columns), the stat list and the inspect source; answer 149 start and stop for inspect. The social fields are for the China build only.
19. **Locale.** Every name, hint, description, category title and error message id must exist in the locale we ship or the client prints a not-found marker.
20. **Account safety.** The client is a request source only. Re-check on the server that the entity key in a login or delete request belongs to the account, that the profile in a switch is the character's, and that every submitted creation id is allowed.

### Evidence

- **Client.** 829 accepted functions across 46 shards (S0143 to S0722): 540 confirmed, 181 likely, 66 speculative, 42 unknown; subsystems `game.character` 414, `game.charactercreate` 171, `world.character` 134, `ui.equip_screen` 34, `game.customization` 30, `ui.inspect` 26, `ui.charactercreate` 20. The table is mostly plumbing: 374 rows are the `CharacterBase` accessors of S0569 and S0570, and shards S0143, S0146, S0167 to S0173, S0184, S0219, S0222, S0225, S0254 to S0260, S0332 and S0722 are mostly containers, effects, movement, mounts, graphics or packet constructors for other systems. The feature itself is in S0145 to S0152 (account manager, creator definitions), S0262 and S0263 (`CharacterCreationManager`), S0344 (titles), S0372 to S0374 (portraits, customization store), S0381 and S0382 (profile swap, `CharacterDataSource`), S0384 (Lua modules), S0406 and S0414 (profile and inspect data sources) and S0573 (presets, hat and hair groups). Extra reads in this pass: the account state machine (`00715ec0` and its state names), the create, login and delete callbacks (`007100c0`, `00711220`, `007198b0`), `CreatePlayer`, `ChangeItem`, the creation record writer (`00954bd0`), the dispatcher cases for 38/14, 38/18, 38/20, 38/21 and 35/13, 14, 15, 19, 27, 28, 30, 41 (`009392c0`, `0092f460`), the title handler and data source, `ProcessProfileSwap`, `PlayerCustomization_ApplyChangeRecord` and its commit and revert helpers, `ChangeProfile`, `JobScreen`, the profile reference-data loader and the `CharacterCreateUi` and `EquipScreen` module tables.
- **Client Lua and resources.** `ScriptsBase.lua` for `CharacterCreateType`, `CharacterCreate`, `CharacterNameScreen`, `CharacterSelect`, `JobBrowser`, `JobHandler`, `LevelProgressBar`, `NotificationHandler:JobLevelUp`, `AboutPlayer`, `PlayerInfo_DS` and `Profile_DS`; the creation and customization tables, lookup files, presets, models, race and gender files, requirement, filter and equipment-slot tables named in Data and formats, all recounted from the files; the locale for every string quoted.
- **Video.** 11 creation frames in 3 recordings; 37 level-up frames in 25 recordings (35 Adventurer, 2 Kart Driver); 3,772 frames in 449 recordings with a job reading, 3,534 with a numeric level (1,450 at level 20). All are curated reads, not counts of every frame.
- **Wiki and job pages.** The Jobs and Stars wiki pages and the 17 job pages supply the job classes, caps, members flags and unlock routes. They are secondary; no numbers from them are used as rules except where labelled.
- **Vault.** Used for client-derived facts and our own native runs: Character Creation Audit (the empty starting-set table and `CreatePlayer` failures, the missing skin-8 head texture), Creator Asset Recovery (the dressing-room scene files), Native Creator Delivery Proof (creation saved and the self body delivered to and accepted by the client; its byte comparison with a community serializer is not used), Native Character Selection Labels (the GUID label defect and the single-operand override), 044 (the client-side constructors of 26/13, 26/26, 26/36, which were re-read in the binary here), 049 (253 creator rows, 12 withheld rows; recounted here), 050 (check-name opcodes, result switch, length messages; the client parts were re-read here), 095 (the client's multi-character handling) and 032 to 043 (rank ceiling 20, rank gates). Not used for facts: Piece 031 (its job catalogue rests on a recorded server payload and the community profile table), the recorded-payload and community-serializer comparisons inside the Native Creator Delivery Proof and Piece 095 notes, the community packet writers and database rows those notes cross-check, the community launcher's resource files and manifests, the community profile table, and any FreeRealms-Legacy or OpenSourceFreeRealms material. The 3001, 4005 values and the community custom creator scene are named where they appear.
- **What is verified and what is reconstructed.** Verified in the original client by our native runs: the creator renders and a manual creation submits through the login service (opcode 5 with server id 0) and is saved; the client accepts the delivered self body for the saved identity; a second test identity (skin tone 3) reaches World Ready and persists across restart; the character-select screen shows the saved name and the Add a Character label only after the local movie override, and Go restores the same identity; selecting Brawler (26/13) produces 38/21 profile blocks for profiles 1 and 43 that the client accepts; XP grants emit 38/18 and 38/14 and the whole gameplay state, including class, level and XP, survives reload (project notes: reviewed native runs 103605 and 111125). The first identity failed at first profile load because the skin-8 head texture is not available (client log, 404). Reconstructed and not accepted natively: the 100-star rank curve, unlock timing, the starter kits, the refusal policy for a bad switch, the check-name exchange (no native 210/211 capture decoded), custom names and temporary-name approval, delete, a second character, the whole title system, rename, style-card changes in the world, inspect, About Me and the on-screen level-up banner. The Rust login service still holds a single character (vault, Piece 095).

### Open questions and contradictions

- **No original server code.** Every server-side rule above is inferred. The vault's native runs prove the client accepts the reconstructed messages, not that the original sent them. No outside serializer is used as a byte oracle; client acceptance is the only check, so exact field encodings the client does not read strictly remain unverified.
- **Starting sets.** The supplied `CharacterCreateItemStartingSets.txt` has no rows, and `CreatePlayer` needs them. Whether the original client shipped rows, or the server supplied the starting sets through `BaseClient.StartingSets`, is not decided. The native runs that created characters used a populated community file in a disposable client; its content is not recorded or used here.
- **The creator scene.** Nine dressing-room terrain tiles are absent from all supplied packs. The native runs that reached the creator used 19 community custom files in `Client/custom`; they do not match the original manifest rows, so the original dressing room is not reproduced (vault, Creator Asset Recovery).
- **Select-screen label.** The supplied movie renders the GUID in each row because its row renderer uses `_id` where the data path passes the name in a second field; the original Lua passes both correctly. Whether an unmodified original movie differed, or the original data path put the name in the id field, is unknown. Our private fix changes one constant-pool operand in a copy, loaded from `custom`; the original file is unchanged.
- **Creator order.** The native client asks for the name before the appearance, and the browser-based creator in video does the reverse. `CharacterCreate:GoToName` exists in the Lua but the state machine does not use it. Which order the original game used for the in-client creator is only known from the state machine.
- **Twelve extra option rows.** The supplied tables offer 253 rows; twelve select skin tone 7, skin tone 8 or a "China" face type, and the head texture for skin 8 on the Human male Caucasian head is missing. The other eleven were not individually proven missing. Video shows "Face Type: Seafarer" selected for a Human male, but two Human male rows carry that name (an ordinary head and the "China" head), so the video does not show which was offered. Four mapping rows (2 to 5, Human male skin tones 3 to 6) name definitions 78092 to 78095 that are not in the starter definitions file and no option row reaches them.
- **A data conflict.** The Human female eye colour "Exotic Gem" (row 1000013) points at mapping 195, which is a Human male mapping, and its definition 77605 is tagged boys-only. The "China" face-type definitions for Pixie female and Human female (78086 and 78089) carry the same boys-only tag. Whether the gender tags matter to the client was not traced.
- **Appearance record fields.** Six integers of the creation record (offsets `+0xc0` to `+0xd4`) are not named in any client log or reader. Which of `ItemHands`, `ItemHead` and `ItemShoulders` the creator ever sets is not shown; the vault's saved test character carries three clothing items. `Head`, `Hair`, `ModelCustomization`, `FacePaint` and `SkinTone` are read here as `PARAM2` of the definitions; that offset (`+0x218`) is inferred from the accessor layout.
- **Customization slot ids.** The mapping of record ids 1 and 6 to skin tone and face paint, and the meaning of id 8, rest on code shape (string parameters and the neighbouring cases), not on a name. The `PARAM1` of the standard face-type definitions is 9 while the live store has no id 9.
- **Hair variants.** `<hairtype>` is chosen by `NewHatHairAttachmentGroup`; the values and which hats select them are not recovered.
- **Profile record.** The 44/3 record has eleven integers, two flags and a list of integer pairs, but no reader names a field. The mapping to `Profile_DS` columns is by the data source's order, not read. The `Hint` column is a locale id; how the browser shows a job the character has not unlocked is not shown (the Rust slice refuses the switch on the server).
- **38/21 tail.** The three trailing integers are read by the client but not named. The Rust slice currently sends 3001, 4005 and 0, values inherited from an earlier outside reference that is not used here. The client does not confirm their meaning (an animation, a composite effect and one more are plausible, R), so the correct server values are unknown.
- **38/14 counters.** Dwords 7, 9 and 10 of a profile node take the last three values; the data-source columns `StarsAvailable` and `StarsEarned` suggest what they are. The client handler calls `HandlerJobBrowser:SetJobSkillPoints` right after, which fits stars available, but this is not proven.
- **Login failures.** For statuses 2, 6 and 7 the pseudocode passes state 46 as the state to continue to after the error popup, which is unexpected for a failure; the intent is unresolved (client `00711220`, `0070fdc0`).
- **Activate-failed and by-item switching.** 26/26 and 26/36 have senders in the client but no caller was found; their triggers and the server's replies are unknown.
- **XP curve, caps and members limits.** None of them is in the client. The client says anyone can play to level 4 and new players can try any job to at least level 5; the wiki says 4 for most jobs; the Sunstone Valley text says Brawler, Miner and Blacksmith go to level 20 free, yet the Blacksmith page flags it members only. Which jobs were capped at what, and when each limit changed, is not settled. The members flag per job also differs between sources: the wiki list marks Warrior, Wizard, Chef, Kart Driver, Pet Trainer and Postman, while the job pages also flag Medic, Ninja and Blacksmith from client upgrade prompts ("Want to keep playing as a Medic? Upgrade to unlock more fun!"). The client's own `MembersOnly` column is server data.
- **Advanced job gates.** Level 20 in Archer, Brawler or Medic is a client-text claim (job-board notices and trainer lines); the wiki does not mention it and the live game may not have enforced it.
- **Titles.** The title list is requested from the Lua only when the China flag is set, so whether titles were shown in other regions, how titles were earned, the text of each id and which template belongs to which kind are unknown. `DiplomacyTitles.txt` has the columns for a title unlocked by a rank and a count of ranks but no rows and no traced loader. 152/7 is a social-posting update that points at a post-launch regional build ([[Social (Friends, Guilds, Chat)]]).
- **Check-name details.** The suggestion row layout is not decoded; the numeric error tables are header only; there is no case for result 11; the profanity list and the uniqueness scope (per account or per realm) are unknown.
- **Delete and multiple characters.** The client supports both (a list, an Add row, delete with a warning, a slot count) and the vault's login service does not. No native run covers either.
- **Two name tables.** `CharacterNames.txt` and `CharacterCreate/CharacterCreateNames.txt` overlap but differ by a few dozen rows; which one the shipped build treated as authoritative for a given screen is not shown.
- **Inspect.** The request that starts an inspect and the bodies of 149/1 and 149/4 were not traced.
- **Duplicated or mislabelled functions.** `00705a10`, `00705a30` and `00705a60` are thin forwarders; `00963d80`, `00963da0` and many `ProxiedCharacter` helpers are speculative field setters; `00bf1ee0` and `009580d0` have a conflicting label (the adjustor adds 0x54, not 0x154); `00960060` and `0095e890` are movement queue helpers that landed here by class name only.

### Key functions

| Address | Role |
|---|---|
| `00715ec0`, `00705610`, `0070c8b0` | Account state machine, state names, set-state |
| `00719330`, `00719550`, `00711ca0` | Account info, character list and server list callbacks |
| `007140c0`, `0071efb0`, `00711a60` | `CreatePlayer` (mannequins), creator resource request, dummy selection by race and gender |
| `0071f030`, `0071fd60`, `0071daa0`, `0071d030`, `0071c150` | Creator item change, category items, category buttons, item data, slot handler |
| `00724d80`, `00724b80`, `007236c0`, `00723860`, `00723a70`, `00723c80`, `00723e90`, `00725a70` | Definition loaders (items, names, mappings, item types, clothing sets, starting sets) and the id-to-string lookup |
| `0097c960..00982e70`, `00980370`, `00980540`, `00980740`, `009829f0`, `009809f0` | `CharacterCreationManager` bootstrap and table parsers |
| `010524a0`, `01051490`, `0070ca40`, `01051580` | Presets loader, hat and hair group, hair path builder |
| `00c13e70`, `00c13b00..00c14000` | `CharacterCreateUi` module and its entries |
| `007100c0`, `00711220`, `007198b0`, `00710e20` | Create, login, delete and check-name replies |
| `00954bd0`, `00956590`, `00fa9db0`, `00fa9eb0`, `00fa9f80`, `00fa91a0`, `00fa91e0`, `00fa97d0` | Creation record writer, create sender, login request builders, reply dispatcher |
| `009564a0`, `00956230`, `009566c0`, `00956100`, `00953950` | Check-name request, send, reply dispatch and parse, constructor |
| `00c90630`, `00c8f2b0`, `00c8ff30`, `00a3a900`, `00a3a2b0`, `00928d00` | Profile data sources, column names, filtered refresh, 44/3 parse, reference-data dispatcher |
| `00c0a760`, `00c0a630`, `00938e10`, `008a29f0`, `0092ef50`, `0092f260` | `JobScreen` module, select job, `ChangeProfile`, `SetProfile` constructor, by-item sender, activate-failed sender |
| `009392c0` | Family 38 handler: 38/14, 38/18, 38/20, 38/21 and others |
| `0091b080`, `00925550`, `00965f90`, `00bfb620`, `00963410`, `00bdb400` | Activate-profile reader, switch group creation, required-slot check, swap complete, equip-screen finish |
| `00900a20`, `008d86e0`, `008d8910`, `008a2330` | Skill-point push, 38/14 and 38/18 readers, activity refresh |
| `0092f460` | Family 35 player-update dispatcher: 13, 14, 15, 19, 27, 28, 30, 41 |
| `00bd9e20`, `00bd8e00`, `00bd8b20`, `00bd8a00`, `00bd9560`, `00bd9730`, `00bd9850` | Customization store: constructor, apply record, commit, revert, commit-and-apply, replace, destructor |
| `0072db70`, `00958110`, `00958130`, `0096bd90` | Slot kind names, hair and eye tint setters, temporary appearance |
| `00b4a3a0`, `00b4a880`, `00b4a910`, `00c18c80`, `00cc5930`, `0097c390`, `00b49a10` | Title handler, select and refresh senders, Lua entry, data source, template expand, set selected |
| `00bfdf20`, `00bfca90`, `00bfcfe0`, `00cbfd20`, `00cc0760`, `008c6030`, `009347d0` | About Me and inspect sources, inspect show and stop |
| `009471a0`, `008d61c0` | Base dispatcher: self body (12), trial upsell (145), inspect (149), titles (152) |
| `00ce7980`, `00ce78f0` | Rank and current-profile requirement checks |
| `01042bd0`, `01041c30` | `CharacterBase` and `CharacterVariables` constructors |
<!-- NARRATIVE END -->

## Mapped functions (generated)

829 accepted functions: 540 confirmed, 181 likely, 66 speculative, 42 unknown. Network-linked (`wire`): 3.

Subsystems: `game.character` (414), `game.charactercreate` (171), `world.character` (134), `ui.equip_screen` (34), `game.customization` (30), `ui.inspect` (26), `ui.charactercreate` (20)

### `game.character`

**CharacterBase** (361)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0103ede0` | CharacterBase::TailJumpSub178_Slot2 | confirmed | Virtual slot 9 of CharacterBase: adds 0x178 to ECX and tail-jumps (jmp eax) to vtable slot 2 (+8) of the sub-object at this+0x178; no frame and no return of it… |
| `0103ee30` | CharacterBase::PredField90LeVirtual67 | confirmed | Virtual slot 32 of CharacterBase: calls vtable slot 67 (+0x10c) of *this, then returns 1 in EAX when the int it points to is <= the dword at this+0x90, else 0. |
| `0103ee90` | CharacterBase::SetField90AndNotify | confirmed | Virtual slot 17 of CharacterBase: stores the first stack argument at this+0x90, calls vtable slot 34 (+0x88) with ECX = this and no arguments, and returns with… |
| `0103eeb0` | CharacterBase::PredVirtual87LeField98 | confirmed | Virtual slot 33 of CharacterBase: calls vtable slot 87 (+0x15c) of *this and returns 1 in EAX when the int it points to is <= the dword at this+0x98, else 0. |
| `0103f020` | CharacterBase::SetByteF0True | confirmed | Virtual slot 34 of CharacterBase: stores 1 in the byte at this+0xf0 (mov byte [ecx+0xf0],1; ret). |
| `0103f030` | CharacterBase::GetFloatF8 | confirmed | Virtual slot 42 of CharacterBase: loads the float at this+0xf8 onto the x87 stack and returns it (fld dword [ecx+0xf8]; ret). |
| `0103fb70` | CharacterBase::GetDwordOfSlot23Object_14 | confirmed | Virtual slot 22 of CharacterBase: calls vtable slot 23 (+0x5c) of *this; when the pointer it returns is non-null, returns the dword at that object's +0x14, oth… |
| `0103fb90` | CharacterBase::GetDwordOfSlot23Object_18 | confirmed | Virtual slot 21 of CharacterBase: calls vtable slot 23 (+0x5c) of *this; when the pointer it returns is non-null, returns the dword at that object's +0x18, oth… |
| `0103fbc0` | CharacterBase::SetLocationChecked | confirmed | Virtual slot 16 of CharacterBase: copies the four floats at the argument into this+0xb0..+0xbc when x, y and z each lie strictly inside the global bounds DAT_0… |
| `01040630` | CharacterBase::LookupEntryForwardTo38 | confirmed | Virtual slot 53 of CharacterBase: looks up the entry through vtable slot 51 (+0xcc) with the argument; when found, calls 0072d5d0 with that entry's +0x38 dword… |
| `01040bd0` | CharacterBase::AssignSerialIfUnset | confirmed | Virtual slot 40 of CharacterBase: when the node param_2 has a zero dword at +8, sets it to one more than the largest +8 serial in the list at this+0x1d0 (links… |
| `01040da0` | CharacterBase::RemoveByKeyAndNotify | confirmed | Virtual slot 45 of CharacterBase: walks the list at this+0x1e4 (links +0x1f0) for the node whose first dword equals the argument; when found, detaches it with … |
| `01040ea0` | CharacterBase::Thunk_Adjust1cc_ToImpl008c0f90 | confirmed | Virtual slot 39 of CharacterBase: adjustor thunk (add ecx,0x1cc; jmp 008c0f90) that moves this by +0x1cc and jumps to 008c0f90; no frame of its own. |
| `010416a0` | CharacterBase::Thunk_Adjust110_ToImpl01041360 | confirmed | Virtual slot 56 of CharacterBase: adjustor thunk (add ecx,0x110; jmp 01041360) that moves this by +0x110 and jumps to 01041360 (HashList16RemoveTailAndFree). |
| `010419a0` | CharacterBase::ForwardEntryTo34 | confirmed | Virtual slot 52 of CharacterBase: looks up the entry through vtable slot 51 (+0xcc); when found, calls 006f9e60 with the entry's +0x34 sub-object and returns 1… |
| `01042bd0` | CharacterBase::CharacterBase | confirmed | __thiscall(this, int arg): installs CharacterBase::vftable at +0, stores arg at +0x18 and sets the byte at +0x175 to (arg == 1); installs SoeUtil::IString<char… |
| `01042ed0` | CharacterBase::CopyByteFromArg_174 | confirmed | Stores the low byte of its one stdcall argument into this+0x174 (mov al,[esp+4]; mov [ecx+0x174],al; ret 4). No return value. |
| `01042ee0` | CharacterBase::GetDword_18 | confirmed | Returns the dword at this+0x18 in EAX (mov eax,[ecx+0x18]; ret). No arguments. |
| `01042ef0` | CharacterBase::Field18Equals1 | confirmed | Returns 1 in EAX when the dword at this+0x18 equals 1, else 0 (xor eax,eax; cmp [ecx+0x18],1; sete al; ret). |
| `01042f00` | CharacterBase::Field18Equals2 | confirmed | Returns 1 in EAX when the dword at this+0x18 equals 2, else 0 (xor eax,eax; cmp [ecx+0x18],2; sete al; ret). |
| `01042f30` | CharacterBase::NotVirtualSlot28Result | confirmed | Calls the virtual at byte offset 0x70 (vtable slot 28) of *this with its one stdcall argument, then returns 1 in EAX when the returned byte is 0 and 0 otherwis… |
| `01042f50` | CharacterBase::GetByte_f0 | confirmed | Returns the byte at this+0xf0 in AL (mov al,[ecx+0xf0]; ret). No arguments. |
| `01042f70` | CharacterBase::FieldAddr_3d0 | confirmed | Returns the address this+0x3d0 in EAX without touching memory (lea eax,[ecx+0x3d0]; ret). |
| `01042f80` | CharacterBase::CopyDwordFromPtr_3d0 | confirmed | Copies the dword at *arg (one stdcall argument, a pointer) into this+0x3d0 (mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x3d0],edx; ret 4). |
| `01042f90` | CharacterBase::FieldAddr_3d4 | confirmed | Returns the address this+0x3d4 in EAX without touching memory (lea eax,[ecx+0x3d4]; ret). |
| `01042fa0` | CharacterBase::CopyDwordFromPtr_3d4 | confirmed | Copies the dword at *arg (one stdcall argument, a pointer) into this+0x3d4 (ret 4). |
| `01042fb0` | CharacterBase::EntryTagKeyString_s64 | confirmed | Returns the address of the literal string entryTag (mov eax,0x0187f160; ret). No arguments and no memory writes. |
| `01042fc0` | CharacterBase::FieldAddr_3d8 | confirmed | Returns the address this+0x3d8 in EAX without touching memory (lea eax,[ecx+0x3d8]; ret). |
| `01042fd0` | CharacterBase::CopyDwordFromPtr_3d8 | confirmed | Copies the dword at *arg (one stdcall argument, a pointer) into this+0x3d8 (ret 4). |
| `01042fe0` | CharacterBase::FieldAddr_3dc | confirmed | Returns the address this+0x3dc in EAX without touching memory (lea eax,[ecx+0x3dc]; ret). |
| `01042ff0` | CharacterBase::CopyDwordFromPtr_3dc | confirmed | Copies the dword at *arg (one stdcall argument, a pointer) into this+0x3dc (ret 4). |
| `01043000` | CharacterBase::EntryTagKeyString_s69 | confirmed | Returns the address of the literal string entryTag (mov eax,0x0187f160; ret). No arguments and no memory writes. |
| `01043010` | CharacterBase::FieldAddr_3e0 | confirmed | Returns the address this+0x3e0 in EAX without touching memory (lea eax,[ecx+0x3e0]; ret). |
| `01043020` | CharacterBase::CopyFloatFromPtr_3e0 | confirmed | Copies the 32-bit float at *arg into this+0x3e0 (mov eax,[esp+4]; fld dword [eax]; fstp dword [ecx+0x3e0]; ret 4). |
| `01043030` | CharacterBase::FieldAddr_3e4 | confirmed | Returns the address this+0x3e4 in EAX without touching memory (lea eax,[ecx+0x3e4]; ret). |
| `01043040` | CharacterBase::CopyFloatFromPtr_3e4 | confirmed | Copies the 32-bit float at *arg into this+0x3e4 (x87 fld/fstp; ret 4). |
| `01043050` | CharacterBase::EntryTagKeyString_s74 | confirmed | Returns the address of the literal string entryTag (mov eax,0x0187f160; ret). No arguments and no memory writes. |
| `01043060` | CharacterBase::FieldAddr_3e8 | confirmed | Returns the address this+0x3e8 in EAX without touching memory (lea eax,[ecx+0x3e8]; ret). |
| `01043070` | CharacterBase::CopyFloatFromPtr_3e8 | confirmed | Copies the 32-bit float at *arg into this+0x3e8 (x87 fld/fstp; ret 4). |
| `01043080` | CharacterBase::FieldAddr_3ec | confirmed | Returns the address this+0x3ec in EAX without touching memory (lea eax,[ecx+0x3ec]; ret). |
| `01043090` | CharacterBase::CopyFloatFromPtr_3ec | confirmed | Copies the 32-bit float at *arg into this+0x3ec (x87 fld/fstp; ret 4). |
| `010430a0` | CharacterBase::EntryTagKeyString_s79 | confirmed | Returns the address of the literal string entryTag (mov eax,0x0187f160; ret). No arguments and no memory writes. |
| `010430b0` | CharacterBase::FieldAddr_3f0 | confirmed | Returns the address this+0x3f0 in EAX without touching memory (lea eax,[ecx+0x3f0]; ret). |
| `010430c0` | CharacterBase::CopyDwordFromPtr_3f0 | confirmed | Copies the dword at *arg into this+0x3f0 (mov edx,[eax]; mov [ecx+0x3f0],edx; ret 4). |
| `010430d0` | CharacterBase::FieldAddr_3f4 | confirmed | Returns the address this+0x3f4 in EAX without touching memory (lea eax,[ecx+0x3f4]; ret). |
| `010430e0` | CharacterBase::CopyDwordFromPtr_3f4 | confirmed | Copies the dword at *arg into this+0x3f4 (ret 4). |
| `010430f0` | CharacterBase::EntryTagKeyString_s84 | confirmed | Returns the address of the literal string entryTag (mov eax,0x0187f160; ret). No arguments and no memory writes. |
| `01043100` | CharacterBase::FieldAddr_3f8 | confirmed | Returns the address this+0x3f8 in EAX without touching memory (lea eax,[ecx+0x3f8]; ret). |
| `01043110` | CharacterBase::CopyDwordFromPtr_3f8 | confirmed | Copies the dword at *arg into this+0x3f8 (ret 4). |
| `01043120` | CharacterBase::FieldAddr_3fc | confirmed | Returns the address this+0x3fc in EAX without touching memory (lea eax,[ecx+0x3fc]; ret). |
| `01043130` | CharacterBase::CopyDwordFromPtr_3fc | confirmed | Copies the dword at *arg into this+0x3fc (ret 4). |
| `01043140` | CharacterBase::EntryTagKeyString_s89 | confirmed | Returns the address of the literal string entryTag (mov eax,0x0187f160; ret). No arguments and no memory writes. |
| `01043150` | CharacterBase::FieldAddr_400 | confirmed | Returns the address this+0x400 in EAX without touching memory (lea eax,[ecx+0x400]; ret). |
| `01043160` | CharacterBase::CopyDwordFromPtr_400 | confirmed | Copies the dword at *arg into this+0x400 (ret 4). |
| `01043170` | CharacterBase::FieldAddr_404 | confirmed | Returns the address this+0x404 in EAX without touching memory (lea eax,[ecx+0x404]; ret). |
| `01043180` | CharacterBase::CopyDwordFromPtr_404 | confirmed | Copies the dword at *arg into this+0x404 (ret 4). |
| `01043190` | CharacterBase::EntryTagKeyString_s94 | confirmed | Returns the address of the literal string entryTag (mov eax,0x0187f160; ret). No arguments and no memory writes. |
| `010431a0` | CharacterBase::FieldAddr_408 | confirmed | Returns the address this+0x408 in EAX without touching memory (lea eax,[ecx+0x408]; ret). |
| `010431b0` | CharacterBase::CopyDwordFromPtr_408 | confirmed | Copies the dword at *arg into this+0x408 (ret 4). |
| `010431c0` | CharacterBase::FieldAddr_40c | confirmed | Returns the address this+0x40c in EAX without touching memory (lea eax,[ecx+0x40c]; ret). |
| `010431d0` | CharacterBase::CopyDwordFromPtr_40c | confirmed | Copies the dword at *arg into this+0x40c (ret 4). |
| `010431e0` | CharacterBase::EntryTagKeyString_s99 | confirmed | Returns the address of the literal string entryTag (mov eax,0x0187f160; ret). No arguments and no memory writes. |
| `010431f0` | CharacterBase::FieldAddr_410 | confirmed | Returns the address this+0x410 in EAX without touching memory (lea eax,[ecx+0x410]; ret). |
| `01043200` | CharacterBase::CopyDwordFromPtr_410 | confirmed | Copies the dword at *arg into this+0x410 (ret 4). |
| `01043210` | CharacterBase::FieldAddr_414 | confirmed | Returns the address this+0x414 in EAX without touching memory (lea eax,[ecx+0x414]; ret). |
| `01043220` | CharacterBase::CopyDwordFromPtr_414 | confirmed | Copies the dword at *arg into this+0x414 (ret 4). |
| `01043230` | CharacterBase::EntryTagKeyString_s104 | confirmed | Returns the address of the literal string entryTag (mov eax,0x0187f160; ret). No arguments and no memory writes. |
| `01043240` | CharacterBase::FieldAddr_418 | confirmed | Returns the address this+0x418 in EAX without touching memory (lea eax,[ecx+0x418]; ret). |
| `01043250` | CharacterBase::CopyDwordFromPtr_418 | confirmed | Copies the dword at *arg into this+0x418 (ret 4). |
| `01043260` | CharacterBase::FieldAddr_41c | confirmed | Returns the address this+0x41c in EAX without touching memory (lea eax,[ecx+0x41c]; ret). |
| `01043270` | CharacterBase::CopyDwordFromPtr_41c | confirmed | Copies the dword at *arg into this+0x41c (ret 4). |
| `01043280` | CharacterBase::EntryTagKeyString_s109 | confirmed | Returns the address of the literal string entryTag (mov eax,0x0187f160; ret). No arguments and no memory writes. |
| `01043290` | CharacterBase::FieldAddr_420 | confirmed | Returns the address this+0x420 in EAX without touching memory (lea eax,[ecx+0x420]; ret). |
| `010432a0` | CharacterBase::CopyFloatFromPtr_420 | confirmed | Copies the 32-bit float at *arg into this+0x420 (x87 fld/fstp; ret 4). |
| `010432b0` | CharacterBase::FieldAddr_424 | confirmed | Returns the address this+0x424 in EAX without touching memory (lea eax,[ecx+0x424]; ret). |
| `010432c0` | CharacterBase::CopyFloatFromPtr_424 | confirmed | Copies the 32-bit float at *arg into this+0x424 (x87 fld/fstp; ret 4). |
| `010432d0` | CharacterBase::EntryTagKeyString_s114 | confirmed | Returns the address of the literal string entryTag (mov eax,0x0187f160; ret). No arguments and no memory writes. |
| `010432e0` | CharacterBase::FieldAddr_428 | confirmed | Returns the address this+0x428 in EAX without touching memory (lea eax,[ecx+0x428]; ret). |
| `010432f0` | CharacterBase::CopyDwordFromPtr_428 | confirmed | Copies the dword at *arg into this+0x428 (ret 4). |
| `01043300` | CharacterBase::FieldAddr_42c | confirmed | Returns the address this+0x42c in EAX without touching memory (lea eax,[ecx+0x42c]; ret). |
| `01043310` | CharacterBase::CopyDwordFromPtr_42c | confirmed | Copies the dword at *arg into this+0x42c (ret 4). |
| `01043320` | CharacterBase::EntryTagKeyString_s119 | confirmed | Returns the address of the literal string entryTag (mov eax,0x0187f160; ret). No arguments and no memory writes. |
| `01043330` | CharacterBase::FieldAddr_430 | confirmed | Returns the address this+0x430 in EAX without touching memory (lea eax,[ecx+0x430]; ret). |
| `01043340` | CharacterBase::CopyFloatFromPtr_430 | confirmed | Copies the 32-bit float at *arg into this+0x430 (x87 fld/fstp; ret 4). |
| `01043350` | CharacterBase::FieldAddr_434 | confirmed | Returns the address this+0x434 in EAX without touching memory (lea eax,[ecx+0x434]; ret). |
| `01043360` | CharacterBase::CopyFloatFromPtr_434 | confirmed | Copies the 32-bit float at *arg into this+0x434 (x87 fld/fstp; ret 4). |
| `01043370` | CharacterBase::EntryTagKeyString_s124 | confirmed | Returns the address of the literal string entryTag (mov eax,0x0187f160; ret). No arguments and no memory writes. |
| `01043380` | CharacterBase::FieldAddr_438 | confirmed | Returns the address this+0x438 in EAX without touching memory (lea eax,[ecx+0x438]; ret). |
| `01043390` | CharacterBase::CopyDwordFromPtr_438 | confirmed | Copies the dword at *arg into this+0x438 (ret 4). |
| `010433a0` | CharacterBase::FieldAddr_43c | confirmed | Returns the address this+0x43c in EAX without touching memory (lea eax,[ecx+0x43c]; ret). |
| `010433b0` | CharacterBase::CopyDwordFromPtr_43c | confirmed | Copies the dword at *arg into this+0x43c (ret 4). |
| `010433c0` | CharacterBase::EntryTagKeyString_s129 | confirmed | Returns the address of the literal string entryTag (mov eax,0x0187f160; ret). No arguments and no memory writes. |
| `010433d0` | CharacterBase::FieldAddr_440 | confirmed | Returns the address this+0x440 in EAX without touching memory (lea eax,[ecx+0x440]; ret). |
| `010433e0` | CharacterBase::CopyDwordFromPtr_440 | confirmed | Copies the dword at *arg into this+0x440 (ret 4). |
| `010433f0` | CharacterBase::FieldAddr_444 | confirmed | Returns the address this+0x444 in EAX without touching memory (lea eax,[ecx+0x444]; ret). |
| `01043400` | CharacterBase::CopyDwordFromPtr_444 | confirmed | Copies the dword at *arg into this+0x444 (ret 4). |
| `01043410` | CharacterBase::EntryTagKeyString_s134 | confirmed | Returns the address of the literal string entryTag (mov eax,0x0187f160; ret). No arguments and no memory writes. |
| `01043420` | CharacterBase::FieldAddr_448 | confirmed | Returns the address this+0x448 in EAX without touching memory (lea eax,[ecx+0x448]; ret). |
| `01043430` | CharacterBase::CopyDwordFromPtr_448 | confirmed | Copies the dword at *arg into this+0x448 (ret 4). |
| `01043440` | CharacterBase::FieldAddr_44c | confirmed | Returns the address this+0x44c in EAX without touching memory (lea eax,[ecx+0x44c]; ret). |
| `01043450` | CharacterBase::CopyDwordFromPtr_44c | confirmed | Copies the dword at *arg into this+0x44c (ret 4). |
| `01043460` | CharacterBase::EntryTagKeyString_s139 | confirmed | Returns the address of the literal string entryTag (mov eax,0x0187f160; ret). No arguments and no memory writes. |
| `01043470` | CharacterBase::FieldAddr_450 | confirmed | Returns the address this+0x450 in EAX without touching memory (lea eax,[ecx+0x450]; ret). |
| `01043480` | CharacterBase::CopyDwordFromPtr_450 | confirmed | Copies the dword at *arg into this+0x450 (ret 4). |
| `01043490` | CharacterBase::FieldAddr_454 | confirmed | Returns the address this+0x454 in EAX without touching memory (lea eax,[ecx+0x454]; ret). |
| `010434a0` | CharacterBase::CopyDwordFromPtr_454 | confirmed | Copies the dword at *arg into this+0x454 (ret 4). |
| `010434b0` | CharacterBase::EntryTagKeyString_s144 | confirmed | Returns the address of the literal string entryTag (mov eax,0x0187f160; ret). No arguments and no memory writes. |
| `010434c0` | CharacterBase::FieldAddr_458 | confirmed | Returns the address this+0x458 in EAX without touching memory (lea eax,[ecx+0x458]; ret). |
| `010434d0` | CharacterBase::CopyDwordFromPtr_458 | confirmed | Copies the dword at *arg into this+0x458 (ret 4). |
| `010434e0` | CharacterBase::FieldAddr_45c | confirmed | Returns the address this+0x45c in EAX without touching memory (lea eax,[ecx+0x45c]; ret). |
| `010434f0` | CharacterBase::CopyDwordFromPtr_45c | confirmed | Copies the dword at *arg into this+0x45c (ret 4). |
| `01043500` | CharacterBase::EntryTagKeyString_s149 | confirmed | Returns the address of the literal string entryTag (mov eax,0x0187f160; ret). No arguments and no memory writes. |
| `01043510` | CharacterBase::FieldAddr_460 | confirmed | Returns the address this+0x460 in EAX without touching memory (lea eax,[ecx+0x460]; ret). |
| `01043520` | CharacterBase::CopyDwordFromPtr_460 | confirmed | Copies the dword at *arg into this+0x460 (ret 4). |
| `01043530` | CharacterBase::FieldAddr_464 | confirmed | Returns the address this+0x464 in EAX without touching memory (lea eax,[ecx+0x464]; ret). |
| `01043540` | CharacterBase::CopyDwordFromPtr_464 | confirmed | Copies the dword at *arg into this+0x464 (ret 4). |
| `01043550` | CharacterBase::EntryTagKeyString_s154 | confirmed | Returns the address of the literal string entryTag (mov eax,0x0187f160; ret). No arguments and no memory writes. |
| `01043560` | CharacterBase::FieldAddr_468 | confirmed | Returns the address this+0x468 in EAX without touching memory (lea eax,[ecx+0x468]; ret). |
| `01043570` | CharacterBase::CopyDwordFromPtr_468 | confirmed | Copies the dword at *arg into this+0x468 (ret 4). |
| `01043580` | CharacterBase::FieldAddr_46c | confirmed | Returns the address this+0x46c in EAX without touching memory (lea eax,[ecx+0x46c]; ret). |
| `01043590` | CharacterBase::CopyDwordFromPtr_46c | confirmed | Copies the dword at *arg into this+0x46c (ret 4). |
| `010435a0` | CharacterBase::EntryTagKeyString_s159 | confirmed | Returns the address of the literal string entryTag (mov eax,0x0187f160; ret). No arguments and no memory writes. |
| `010435b0` | CharacterBase::FieldAddr_470 | confirmed | Returns the address this+0x470 in EAX without touching memory (lea eax,[ecx+0x470]; ret). |
| `010435c0` | CharacterBase::CopyFloatFromPtr_470 | confirmed | Copies the 32-bit float at *arg into this+0x470 (x87 fld/fstp; ret 4). |
| `010435d0` | CharacterBase::FieldAddr_474 | confirmed | Returns the address this+0x474 in EAX without touching memory (lea eax,[ecx+0x474]; ret). |
| `010435e0` | CharacterBase::CopyFloatFromPtr_474 | confirmed | Copies the 32-bit float at *arg into this+0x474 (x87 fld/fstp; ret 4). |
| `010435f0` | CharacterBase::EntryTagKeyString_s164 | confirmed | Returns the address of the literal string entryTag (mov eax,0x0187f160; ret). No arguments and no memory writes. |
| `01043600` | CharacterBase::FieldAddr_478 | confirmed | Returns the address this+0x478 in EAX without touching memory (lea eax,[ecx+0x478]; ret). |
| `01043610` | CharacterBase::CopyDwordFromPtr_478 | confirmed | Copies the dword at *arg into this+0x478 (ret 4). |
| `01043620` | CharacterBase::FieldAddr_47c | confirmed | Returns the address this+0x47c in EAX without touching memory (lea eax,[ecx+0x47c]; ret). |
| `01043630` | CharacterBase::CopyDwordFromPtr_47c | confirmed | Copies the dword at *arg into this+0x47c (ret 4). |
| `01043640` | CharacterBase::EntryTagKeyString_s169 | confirmed | Returns the address of the literal string entryTag (mov eax,0x0187f160; ret). No arguments and no memory writes. |
| `01043650` | CharacterBase::FieldAddr_480 | confirmed | Returns the address this+0x480 in EAX without touching memory (lea eax,[ecx+0x480]; ret). |
| `01043660` | CharacterBase::CopyFloatFromPtr_480 | confirmed | Copies the 32-bit float at *arg into this+0x480 (x87 fld/fstp; ret 4). |
| `01043670` | CharacterBase::FieldAddr_484 | confirmed | Returns the address this+0x484 in EAX without touching memory (lea eax,[ecx+0x484]; ret). |
| `01043680` | CharacterBase::CopyFloatFromPtr_484 | confirmed | Copies the 32-bit float at *arg into this+0x484 (x87 fld/fstp; ret 4). |
| `01043690` | CharacterBase::EntryTagKeyString_s174 | confirmed | Returns the address of the literal string entryTag (mov eax,0x0187f160; ret). No arguments and no memory writes. |
| `010436a0` | CharacterBase::FieldAddr_488 | confirmed | Returns the address this+0x488 in EAX without touching memory (lea eax,[ecx+0x488]; ret). |
| `010436b0` | CharacterBase::CopyDwordFromPtr_488 | confirmed | Copies the dword at *arg into this+0x488 (ret 4). |
| `010436c0` | CharacterBase::FieldAddr_048c | confirmed | Thiscall, no stack args. Bytes 8d 81 8c 04 00 00 c3 = lea eax,[ecx+0x48c]; ret. Returns the address this+0x48c, not the value stored there. |
| `010436d0` | CharacterBase::CopyDwordFromPtr_048c | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 8c 04 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x48c],edx; ret 4. Co… |
| `010436e0` | CharacterBase::EntryTagKeyString_s179 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `010436f0` | CharacterBase::FieldAddr_0490 | confirmed | Thiscall, no stack args. Bytes 8d 81 90 04 00 00 c3 = lea eax,[ecx+0x490]; ret. Returns the address this+0x490, not the value stored there. |
| `01043700` | CharacterBase::CopyDwordFromPtr_0490 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 90 04 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x490],edx; ret 4. Co… |
| `01043710` | CharacterBase::FieldAddr_0494 | confirmed | Thiscall, no stack args. Bytes 8d 81 94 04 00 00 c3 = lea eax,[ecx+0x494]; ret. Returns the address this+0x494, not the value stored there. |
| `01043720` | CharacterBase::CopyDwordFromPtr_0494 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 94 04 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x494],edx; ret 4. Co… |
| `01043730` | CharacterBase::EntryTagKeyString_s184 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01043740` | CharacterBase::FieldAddr_0498 | confirmed | Thiscall, no stack args. Bytes 8d 81 98 04 00 00 c3 = lea eax,[ecx+0x498]; ret. Returns the address this+0x498, not the value stored there. |
| `01043750` | CharacterBase::CopyDwordFromPtr_0498 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 98 04 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x498],edx; ret 4. Co… |
| `01043760` | CharacterBase::FieldAddr_049c | confirmed | Thiscall, no stack args. Bytes 8d 81 9c 04 00 00 c3 = lea eax,[ecx+0x49c]; ret. Returns the address this+0x49c, not the value stored there. |
| `01043770` | CharacterBase::CopyDwordFromPtr_049c | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 9c 04 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x49c],edx; ret 4. Co… |
| `01043780` | CharacterBase::EntryTagKeyString_s189 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01043790` | CharacterBase::FieldAddr_04a0 | confirmed | Thiscall, no stack args. Bytes 8d 81 a0 04 00 00 c3 = lea eax,[ecx+0x4a0]; ret. Returns the address this+0x4a0, not the value stored there. |
| `010437a0` | CharacterBase::CopyDwordFromPtr_04a0 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 a0 04 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x4a0],edx; ret 4. Co… |
| `010437b0` | CharacterBase::FieldAddr_04a4 | confirmed | Thiscall, no stack args. Bytes 8d 81 a4 04 00 00 c3 = lea eax,[ecx+0x4a4]; ret. Returns the address this+0x4a4, not the value stored there. |
| `010437c0` | CharacterBase::CopyDwordFromPtr_04a4 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 a4 04 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x4a4],edx; ret 4. Co… |
| `010437d0` | CharacterBase::EntryTagKeyString_s194 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `010437e0` | CharacterBase::FieldAddr_04a8 | confirmed | Thiscall, no stack args. Bytes 8d 81 a8 04 00 00 c3 = lea eax,[ecx+0x4a8]; ret. Returns the address this+0x4a8, not the value stored there. |
| `010437f0` | CharacterBase::CopyDwordFromPtr_04a8 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 a8 04 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x4a8],edx; ret 4. Co… |
| `01043800` | CharacterBase::FieldAddr_04ac | confirmed | Thiscall, no stack args. Bytes 8d 81 ac 04 00 00 c3 = lea eax,[ecx+0x4ac]; ret. Returns the address this+0x4ac, not the value stored there. |
| `01043810` | CharacterBase::CopyDwordFromPtr_04ac | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 ac 04 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x4ac],edx; ret 4. Co… |
| `01043820` | CharacterBase::EntryTagKeyString_s199 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01043830` | CharacterBase::FieldAddr_04b0 | confirmed | Thiscall, no stack args. Bytes 8d 81 b0 04 00 00 c3 = lea eax,[ecx+0x4b0]; ret. Returns the address this+0x4b0, not the value stored there. |
| `01043840` | CharacterBase::CopyDwordFromPtr_04b0 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 b0 04 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x4b0],edx; ret 4. Co… |
| `01043850` | CharacterBase::FieldAddr_04b4 | confirmed | Thiscall, no stack args. Bytes 8d 81 b4 04 00 00 c3 = lea eax,[ecx+0x4b4]; ret. Returns the address this+0x4b4, not the value stored there. |
| `01043860` | CharacterBase::CopyDwordFromPtr_04b4 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 b4 04 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x4b4],edx; ret 4. Co… |
| `01043870` | CharacterBase::EntryTagKeyString_s204 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01043880` | CharacterBase::FieldAddr_04b8 | confirmed | Thiscall, no stack args. Bytes 8d 81 b8 04 00 00 c3 = lea eax,[ecx+0x4b8]; ret. Returns the address this+0x4b8, not the value stored there. |
| `01043890` | CharacterBase::CopyDwordFromPtr_04b8 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 b8 04 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x4b8],edx; ret 4. Co… |
| `010438a0` | CharacterBase::FieldAddr_04bc | confirmed | Thiscall, no stack args. Bytes 8d 81 bc 04 00 00 c3 = lea eax,[ecx+0x4bc]; ret. Returns the address this+0x4bc, not the value stored there. |
| `010438b0` | CharacterBase::CopyDwordFromPtr_04bc | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 bc 04 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x4bc],edx; ret 4. Co… |
| `010438c0` | CharacterBase::EntryTagKeyString_s209 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `010438d0` | CharacterBase::FieldAddr_04c0 | confirmed | Thiscall, no stack args. Bytes 8d 81 c0 04 00 00 c3 = lea eax,[ecx+0x4c0]; ret. Returns the address this+0x4c0, not the value stored there. |
| `010438e0` | CharacterBase::CopyDwordFromPtr_04c0 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 c0 04 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x4c0],edx; ret 4. Co… |
| `010438f0` | CharacterBase::FieldAddr_04c4 | confirmed | Thiscall, no stack args. Bytes 8d 81 c4 04 00 00 c3 = lea eax,[ecx+0x4c4]; ret. Returns the address this+0x4c4, not the value stored there. |
| `01043900` | CharacterBase::CopyDwordFromPtr_04c4 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 c4 04 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x4c4],edx; ret 4. Co… |
| `01043910` | CharacterBase::EntryTagKeyString_s214 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01043920` | CharacterBase::FieldAddr_04c8 | confirmed | Thiscall, no stack args. Bytes 8d 81 c8 04 00 00 c3 = lea eax,[ecx+0x4c8]; ret. Returns the address this+0x4c8, not the value stored there. |
| `01043930` | CharacterBase::CopyDwordFromPtr_04c8 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 c8 04 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x4c8],edx; ret 4. Co… |
| `01043940` | CharacterBase::FieldAddr_04cc | confirmed | Thiscall, no stack args. Bytes 8d 81 cc 04 00 00 c3 = lea eax,[ecx+0x4cc]; ret. Returns the address this+0x4cc, not the value stored there. |
| `01043950` | CharacterBase::CopyDwordFromPtr_04cc | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 cc 04 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x4cc],edx; ret 4. Co… |
| `01043960` | CharacterBase::EntryTagKeyString_s219 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01043970` | CharacterBase::FieldAddr_04d0 | confirmed | Thiscall, no stack args. Bytes 8d 81 d0 04 00 00 c3 = lea eax,[ecx+0x4d0]; ret. Returns the address this+0x4d0, not the value stored there. |
| `01043980` | CharacterBase::CopyDwordFromPtr_04d0 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 d0 04 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x4d0],edx; ret 4. Co… |
| `01043990` | CharacterBase::FieldAddr_04d4 | confirmed | Thiscall, no stack args. Bytes 8d 81 d4 04 00 00 c3 = lea eax,[ecx+0x4d4]; ret. Returns the address this+0x4d4, not the value stored there. |
| `010439a0` | CharacterBase::CopyDwordFromPtr_04d4 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 d4 04 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x4d4],edx; ret 4. Co… |
| `010439b0` | CharacterBase::EntryTagKeyString_s224 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `010439c0` | CharacterBase::FieldAddr_04d8 | confirmed | Thiscall, no stack args. Bytes 8d 81 d8 04 00 00 c3 = lea eax,[ecx+0x4d8]; ret. Returns the address this+0x4d8, not the value stored there. |
| `010439d0` | CharacterBase::CopyDwordFromPtr_04d8 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 d8 04 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x4d8],edx; ret 4. Co… |
| `010439e0` | CharacterBase::FieldAddr_04dc | confirmed | Thiscall, no stack args. Bytes 8d 81 dc 04 00 00 c3 = lea eax,[ecx+0x4dc]; ret. Returns the address this+0x4dc, not the value stored there. |
| `010439f0` | CharacterBase::CopyDwordFromPtr_04dc | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 dc 04 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x4dc],edx; ret 4. Co… |
| `01043a00` | CharacterBase::EntryTagKeyString_s229 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01043a10` | CharacterBase::FieldAddr_04e0 | confirmed | Thiscall, no stack args. Bytes 8d 81 e0 04 00 00 c3 = lea eax,[ecx+0x4e0]; ret. Returns the address this+0x4e0, not the value stored there. |
| `01043a20` | CharacterBase::SetFloatFromPtr_04e0 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 d9 00 d9 99 e0 04 00 00 c2 04 00 = mov eax,[esp+4]; fld dword [eax]; fstp dword [ecx+0x4e0]; ret … |
| `01043a30` | CharacterBase::FieldAddr_04e4 | confirmed | Thiscall, no stack args. Bytes 8d 81 e4 04 00 00 c3 = lea eax,[ecx+0x4e4]; ret. Returns the address this+0x4e4, not the value stored there. |
| `01043a40` | CharacterBase::SetFloatFromPtr_04e4 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 d9 00 d9 99 e4 04 00 00 c2 04 00 = mov eax,[esp+4]; fld dword [eax]; fstp dword [ecx+0x4e4]; ret … |
| `01043a50` | CharacterBase::EntryTagKeyString_s234 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01043a60` | CharacterBase::FieldAddr_04e8 | confirmed | Thiscall, no stack args. Bytes 8d 81 e8 04 00 00 c3 = lea eax,[ecx+0x4e8]; ret. Returns the address this+0x4e8, not the value stored there. |
| `01043a70` | CharacterBase::CopyDwordFromPtr_04e8 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 e8 04 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x4e8],edx; ret 4. Co… |
| `01043a80` | CharacterBase::FieldAddr_04ec | confirmed | Thiscall, no stack args. Bytes 8d 81 ec 04 00 00 c3 = lea eax,[ecx+0x4ec]; ret. Returns the address this+0x4ec, not the value stored there. |
| `01043a90` | CharacterBase::CopyDwordFromPtr_04ec | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 ec 04 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x4ec],edx; ret 4. Co… |
| `01043aa0` | CharacterBase::EntryTagKeyString_s239 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01043ab0` | CharacterBase::FieldAddr_04f0 | confirmed | Thiscall, no stack args. Bytes 8d 81 f0 04 00 00 c3 = lea eax,[ecx+0x4f0]; ret. Returns the address this+0x4f0, not the value stored there. |
| `01043ac0` | CharacterBase::CopyDwordFromPtr_04f0 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 f0 04 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x4f0],edx; ret 4. Co… |
| `01043ad0` | CharacterBase::FieldAddr_04f4 | confirmed | Thiscall, no stack args. Bytes 8d 81 f4 04 00 00 c3 = lea eax,[ecx+0x4f4]; ret. Returns the address this+0x4f4, not the value stored there. |
| `01043ae0` | CharacterBase::CopyDwordFromPtr_04f4 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 f4 04 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x4f4],edx; ret 4. Co… |
| `01043af0` | CharacterBase::EntryTagKeyString_s244 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01043b00` | CharacterBase::FieldAddr_04f8 | confirmed | Thiscall, no stack args. Bytes 8d 81 f8 04 00 00 c3 = lea eax,[ecx+0x4f8]; ret. Returns the address this+0x4f8, not the value stored there. |
| `01043b10` | CharacterBase::CopyDwordFromPtr_04f8 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 f8 04 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x4f8],edx; ret 4. Co… |
| `01043b20` | CharacterBase::FieldAddr_04fc | confirmed | Thiscall, no stack args. Bytes 8d 81 fc 04 00 00 c3 = lea eax,[ecx+0x4fc]; ret. Returns the address this+0x4fc, not the value stored there. |
| `01043b30` | CharacterBase::CopyDwordFromPtr_04fc | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 fc 04 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x4fc],edx; ret 4. Co… |
| `01043b40` | CharacterBase::EntryTagKeyString_s249 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01043b50` | CharacterBase::FieldAddr_0500 | confirmed | Thiscall, no stack args. Bytes 8d 81 00 05 00 00 c3 = lea eax,[ecx+0x500]; ret. Returns the address this+0x500, not the value stored there. |
| `01043b60` | CharacterBase::CopyDwordFromPtr_0500 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 00 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x500],edx; ret 4. Co… |
| `01043b70` | CharacterBase::FieldAddr_0504 | confirmed | Thiscall, no stack args. Bytes 8d 81 04 05 00 00 c3 = lea eax,[ecx+0x504]; ret. Returns the address this+0x504, not the value stored there. |
| `01043b80` | CharacterBase::CopyDwordFromPtr_0504 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 04 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x504],edx; ret 4. Co… |
| `01043b90` | CharacterBase::EntryTagKeyString_s254 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01043ba0` | CharacterBase::FieldAddr_0508 | confirmed | Thiscall, no stack args. Bytes 8d 81 08 05 00 00 c3 = lea eax,[ecx+0x508]; ret. Returns the address this+0x508, not the value stored there. |
| `01043bb0` | CharacterBase::CopyDwordFromPtr_0508 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 08 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x508],edx; ret 4. Co… |
| `01043bc0` | CharacterBase::FieldAddr_050c | confirmed | Thiscall, no stack args. Bytes 8d 81 0c 05 00 00 c3 = lea eax,[ecx+0x50c]; ret. Returns the address this+0x50c, not the value stored there. |
| `01043bd0` | CharacterBase::CopyDwordFromPtr_050c | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 0c 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x50c],edx; ret 4. Co… |
| `01043be0` | CharacterBase::EntryTagKeyString_s259 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01043bf0` | CharacterBase::FieldAddr_0510 | confirmed | Thiscall, no stack args. Bytes 8d 81 10 05 00 00 c3 = lea eax,[ecx+0x510]; ret. Returns the address this+0x510, not the value stored there. |
| `01043c00` | CharacterBase::CopyDwordFromPtr_0510 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 10 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x510],edx; ret 4. Co… |
| `01043c10` | CharacterBase::FieldAddr_0514 | confirmed | Thiscall, no stack args. Bytes 8d 81 14 05 00 00 c3 = lea eax,[ecx+0x514]; ret. Returns the address this+0x514, not the value stored there. |
| `01043c20` | CharacterBase::CopyDwordFromPtr_0514 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 14 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x514],edx; ret 4. Co… |
| `01043c30` | CharacterBase::EntryTagKeyString_s264 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01043c40` | CharacterBase::FieldAddr_0518 | confirmed | Thiscall, no stack args. Bytes 8d 81 18 05 00 00 c3 = lea eax,[ecx+0x518]; ret. Returns the address this+0x518, not the value stored there. |
| `01043c50` | CharacterBase::CopyDwordFromPtr_0518 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 18 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x518],edx; ret 4. Co… |
| `01043c60` | CharacterBase::FieldAddr_051c | confirmed | Thiscall, no stack args. Bytes 8d 81 1c 05 00 00 c3 = lea eax,[ecx+0x51c]; ret. Returns the address this+0x51c, not the value stored there. |
| `01043c70` | CharacterBase::CopyDwordFromPtr_051c | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 1c 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x51c],edx; ret 4. Co… |
| `01043c80` | CharacterBase::EntryTagKeyString_s269 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01043c90` | CharacterBase::FieldAddr_0520 | confirmed | Thiscall, no stack args. Bytes 8d 81 20 05 00 00 c3 = lea eax,[ecx+0x520]; ret. Returns the address this+0x520, not the value stored there. |
| `01043ca0` | CharacterBase::CopyDwordFromPtr_0520 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 20 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x520],edx; ret 4. Co… |
| `01043cb0` | CharacterBase::FieldAddr_0524 | confirmed | Thiscall, no stack args. Bytes 8d 81 24 05 00 00 c3 = lea eax,[ecx+0x524]; ret. Returns the address this+0x524, not the value stored there. |
| `01043cc0` | CharacterBase::CopyDwordFromPtr_0524 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 24 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x524],edx; ret 4. Co… |
| `01043cd0` | CharacterBase::EntryTagKeyString_s274 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01043ce0` | CharacterBase::FieldAddr_0528 | confirmed | Thiscall, no stack args. Bytes 8d 81 28 05 00 00 c3 = lea eax,[ecx+0x528]; ret. Returns the address this+0x528, not the value stored there. |
| `01043cf0` | CharacterBase::CopyDwordFromPtr_0528 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 28 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x528],edx; ret 4. Co… |
| `01043d00` | CharacterBase::FieldAddr_052c | confirmed | Thiscall, no stack args. Bytes 8d 81 2c 05 00 00 c3 = lea eax,[ecx+0x52c]; ret. Returns the address this+0x52c, not the value stored there. |
| `01043d10` | CharacterBase::CopyDwordFromPtr_052c | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 2c 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x52c],edx; ret 4. Co… |
| `01043d20` | CharacterBase::EntryTagKeyString_s279 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01043d30` | CharacterBase::FieldAddr_0530 | confirmed | Thiscall, no stack args. Bytes 8d 81 30 05 00 00 c3 = lea eax,[ecx+0x530]; ret. Returns the address this+0x530, not the value stored there. |
| `01043d40` | CharacterBase::CopyDwordFromPtr_0530 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 30 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x530],edx; ret 4. Co… |
| `01043d50` | CharacterBase::FieldAddr_0534 | confirmed | Thiscall, no stack args. Bytes 8d 81 34 05 00 00 c3 = lea eax,[ecx+0x534]; ret. Returns the address this+0x534, not the value stored there. |
| `01043d60` | CharacterBase::CopyDwordFromPtr_0534 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 34 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x534],edx; ret 4. Co… |
| `01043d70` | CharacterBase::EntryTagKeyString_s284 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01043d80` | CharacterBase::FieldAddr_0538 | confirmed | Thiscall, no stack args. Bytes 8d 81 38 05 00 00 c3 = lea eax,[ecx+0x538]; ret. Returns the address this+0x538, not the value stored there. |
| `01043d90` | CharacterBase::CopyDwordFromPtr_0538 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 38 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x538],edx; ret 4. Co… |
| `01043da0` | CharacterBase::FieldAddr_053c | confirmed | Thiscall, no stack args. Bytes 8d 81 3c 05 00 00 c3 = lea eax,[ecx+0x53c]; ret. Returns the address this+0x53c, not the value stored there. |
| `01043db0` | CharacterBase::CopyDwordFromPtr_053c | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 3c 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x53c],edx; ret 4. Co… |
| `01043dc0` | CharacterBase::EntryTagKeyString_s289 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01043dd0` | CharacterBase::FieldAddr_0540 | confirmed | Thiscall, no stack args. Bytes 8d 81 40 05 00 00 c3 = lea eax,[ecx+0x540]; ret. Returns the address this+0x540, not the value stored there. |
| `01043de0` | CharacterBase::CopyDwordFromPtr_0540 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 40 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x540],edx; ret 4. Co… |
| `01043df0` | CharacterBase::FieldAddr_0544 | confirmed | Thiscall, no stack args. Bytes 8d 81 44 05 00 00 c3 = lea eax,[ecx+0x544]; ret. Returns the address this+0x544, not the value stored there. |
| `01043e00` | CharacterBase::CopyDwordFromPtr_0544 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 44 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x544],edx; ret 4. Co… |
| `01043e10` | CharacterBase::EntryTagKeyString_s294 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01043e20` | CharacterBase::FieldAddr_0548 | confirmed | Thiscall, no stack args. Bytes 8d 81 48 05 00 00 c3 = lea eax,[ecx+0x548]; ret. Returns the address this+0x548, not the value stored there. |
| `01043e30` | CharacterBase::CopyDwordFromPtr_0548 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 48 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x548],edx; ret 4. Co… |
| `01043e40` | CharacterBase::FieldAddr_054c | confirmed | Thiscall, no stack args. Bytes 8d 81 4c 05 00 00 c3 = lea eax,[ecx+0x54c]; ret. Returns the address this+0x54c, not the value stored there. |
| `01043e50` | CharacterBase::CopyDwordFromPtr_054c | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 4c 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x54c],edx; ret 4. Co… |
| `01043e60` | CharacterBase::EntryTagKeyString_s299 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01043e70` | CharacterBase::FieldAddr_0550 | confirmed | Thiscall, no stack args. Bytes 8d 81 50 05 00 00 c3 = lea eax,[ecx+0x550]; ret. Returns the address this+0x550, not the value stored there. |
| `01043e80` | CharacterBase::CopyDwordFromPtr_0550 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 50 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x550],edx; ret 4. Co… |
| `01043e90` | CharacterBase::FieldAddr_0554 | confirmed | Thiscall, no stack args. Bytes 8d 81 54 05 00 00 c3 = lea eax,[ecx+0x554]; ret. Returns the address this+0x554, not the value stored there. |
| `01043ea0` | CharacterBase::CopyDwordFromPtr_0554 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 54 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x554],edx; ret 4. Co… |
| `01043eb0` | CharacterBase::EntryTagKeyString_s304 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01043ec0` | CharacterBase::FieldAddr_0558 | confirmed | Thiscall, no stack args. Bytes 8d 81 58 05 00 00 c3 = lea eax,[ecx+0x558]; ret. Returns the address this+0x558, not the value stored there. |
| `01043ed0` | CharacterBase::CopyDwordFromPtr_0558 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 58 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x558],edx; ret 4. Co… |
| `01043ee0` | CharacterBase::FieldAddr_055c | confirmed | Thiscall, no stack args. Bytes 8d 81 5c 05 00 00 c3 = lea eax,[ecx+0x55c]; ret. Returns the address this+0x55c, not the value stored there. |
| `01043ef0` | CharacterBase::CopyDwordFromPtr_055c | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 5c 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x55c],edx; ret 4. Co… |
| `01043f00` | CharacterBase::EntryTagKeyString_s309 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01043f10` | CharacterBase::FieldAddr_0560 | confirmed | Thiscall, no stack args. Bytes 8d 81 60 05 00 00 c3 = lea eax,[ecx+0x560]; ret. Returns the address this+0x560, not the value stored there. |
| `01043f20` | CharacterBase::CopyDwordFromPtr_0560 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 60 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x560],edx; ret 4. Co… |
| `01043f30` | CharacterBase::FieldAddr_0564 | confirmed | Thiscall, no stack args. Bytes 8d 81 64 05 00 00 c3 = lea eax,[ecx+0x564]; ret. Returns the address this+0x564, not the value stored there. |
| `01043f40` | CharacterBase::CopyDwordFromPtr_0564 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 64 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x564],edx; ret 4. Co… |
| `01043f50` | CharacterBase::EntryTagKeyString_s314 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01043f60` | CharacterBase::FieldAddr_0568 | confirmed | Thiscall, no stack args. Bytes 8d 81 68 05 00 00 c3 = lea eax,[ecx+0x568]; ret. Returns the address this+0x568, not the value stored there. |
| `01043f70` | CharacterBase::CopyDwordFromPtr_0568 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 68 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x568],edx; ret 4. Co… |
| `01043f80` | CharacterBase::FieldAddr_056c | confirmed | Thiscall, no stack args. Bytes 8d 81 6c 05 00 00 c3 = lea eax,[ecx+0x56c]; ret. Returns the address this+0x56c, not the value stored there. |
| `01043f90` | CharacterBase::CopyDwordFromPtr_056c | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 6c 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x56c],edx; ret 4. Co… |
| `01043fa0` | CharacterBase::EntryTagKeyString_s319 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01043fb0` | CharacterBase::FieldAddr_0570 | confirmed | Thiscall, no stack args. Bytes 8d 81 70 05 00 00 c3 = lea eax,[ecx+0x570]; ret. Returns the address this+0x570, not the value stored there. |
| `01043fc0` | CharacterBase::CopyDwordFromPtr_0570 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 70 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x570],edx; ret 4. Co… |
| `01043fd0` | CharacterBase::FieldAddr_0574 | confirmed | Thiscall, no stack args. Bytes 8d 81 74 05 00 00 c3 = lea eax,[ecx+0x574]; ret. Returns the address this+0x574, not the value stored there. |
| `01043fe0` | CharacterBase::CopyDwordFromPtr_0574 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 74 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x574],edx; ret 4. Co… |
| `01043ff0` | CharacterBase::EntryTagKeyString_s324 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01044000` | CharacterBase::FieldAddr_0578 | confirmed | Thiscall, no stack args. Bytes 8d 81 78 05 00 00 c3 = lea eax,[ecx+0x578]; ret. Returns the address this+0x578, not the value stored there. |
| `01044010` | CharacterBase::SetFloatFromPtr_0578 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 d9 00 d9 99 78 05 00 00 c2 04 00 = mov eax,[esp+4]; fld dword [eax]; fstp dword [ecx+0x578]; ret … |
| `01044020` | CharacterBase::FieldAddr_057c | confirmed | Thiscall, no stack args. Bytes 8d 81 7c 05 00 00 c3 = lea eax,[ecx+0x57c]; ret. Returns the address this+0x57c, not the value stored there. |
| `01044030` | CharacterBase::SetFloatFromPtr_057c | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 d9 00 d9 99 7c 05 00 00 c2 04 00 = mov eax,[esp+4]; fld dword [eax]; fstp dword [ecx+0x57c]; ret … |
| `01044040` | CharacterBase::EntryTagKeyString_s329 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01044050` | CharacterBase::FieldAddr_0580 | confirmed | Thiscall, no stack args. Bytes 8d 81 80 05 00 00 c3 = lea eax,[ecx+0x580]; ret. Returns the address this+0x580, not the value stored there. |
| `01044060` | CharacterBase::CopyDwordFromPtr_0580 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 80 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x580],edx; ret 4. Co… |
| `01044070` | CharacterBase::FieldAddr_0584 | confirmed | Thiscall, no stack args. Bytes 8d 81 84 05 00 00 c3 = lea eax,[ecx+0x584]; ret. Returns the address this+0x584, not the value stored there. |
| `01044080` | CharacterBase::CopyDwordFromPtr_0584 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 84 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x584],edx; ret 4. Co… |
| `01044090` | CharacterBase::EntryTagKeyString_s334 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `010440a0` | CharacterBase::FieldAddr_0588 | confirmed | Thiscall, no stack args. Bytes 8d 81 88 05 00 00 c3 = lea eax,[ecx+0x588]; ret. Returns the address this+0x588, not the value stored there. |
| `010440b0` | CharacterBase::CopyDwordFromPtr_0588 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 88 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x588],edx; ret 4. Co… |
| `010440c0` | CharacterBase::FieldAddr_058c | confirmed | Thiscall, no stack args. Bytes 8d 81 8c 05 00 00 c3 = lea eax,[ecx+0x58c]; ret. Returns the address this+0x58c, not the value stored there. |
| `010440d0` | CharacterBase::CopyDwordFromPtr_058c | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 8c 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x58c],edx; ret 4. Co… |
| `010440e0` | CharacterBase::EntryTagKeyString_s339 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `010440f0` | CharacterBase::FieldAddr_0590 | confirmed | Thiscall, no stack args. Bytes 8d 81 90 05 00 00 c3 = lea eax,[ecx+0x590]; ret. Returns the address this+0x590, not the value stored there. |
| `01044100` | CharacterBase::SetFloatFromPtr_0590 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 d9 00 d9 99 90 05 00 00 c2 04 00 = mov eax,[esp+4]; fld dword [eax]; fstp dword [ecx+0x590]; ret … |
| `01044110` | CharacterBase::FieldAddr_0594 | confirmed | Thiscall, no stack args. Bytes 8d 81 94 05 00 00 c3 = lea eax,[ecx+0x594]; ret. Returns the address this+0x594, not the value stored there. |
| `01044120` | CharacterBase::SetFloatFromPtr_0594 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 d9 00 d9 99 94 05 00 00 c2 04 00 = mov eax,[esp+4]; fld dword [eax]; fstp dword [ecx+0x594]; ret … |
| `01044130` | CharacterBase::EntryTagKeyString_s344 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01044140` | CharacterBase::FieldAddr_0598 | confirmed | Thiscall, no stack args. Bytes 8d 81 98 05 00 00 c3 = lea eax,[ecx+0x598]; ret. Returns the address this+0x598, not the value stored there. |
| `01044150` | CharacterBase::SetFloatFromPtr_0598 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 d9 00 d9 99 98 05 00 00 c2 04 00 = mov eax,[esp+4]; fld dword [eax]; fstp dword [ecx+0x598]; ret … |
| `01044160` | CharacterBase::FieldAddr_059c | confirmed | Thiscall, no stack args. Bytes 8d 81 9c 05 00 00 c3 = lea eax,[ecx+0x59c]; ret. Returns the address this+0x59c, not the value stored there. |
| `01044170` | CharacterBase::SetFloatFromPtr_059c | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 d9 00 d9 99 9c 05 00 00 c2 04 00 = mov eax,[esp+4]; fld dword [eax]; fstp dword [ecx+0x59c]; ret … |
| `01044180` | CharacterBase::EntryTagKeyString_s349 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01044190` | CharacterBase::FieldAddr_05a0 | confirmed | Thiscall, no stack args. Bytes 8d 81 a0 05 00 00 c3 = lea eax,[ecx+0x5a0]; ret. Returns the address this+0x5a0, not the value stored there. |
| `010441a0` | CharacterBase::CopyDwordFromPtr_05a0 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 a0 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x5a0],edx; ret 4. Co… |
| `010441b0` | CharacterBase::FieldAddr_05a4 | confirmed | Thiscall, no stack args. Bytes 8d 81 a4 05 00 00 c3 = lea eax,[ecx+0x5a4]; ret. Returns the address this+0x5a4, not the value stored there. |
| `010441c0` | CharacterBase::CopyDwordFromPtr_05a4 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 a4 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x5a4],edx; ret 4. Co… |
| `010441d0` | CharacterBase::EntryTagKeyString_s354 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `010441e0` | CharacterBase::FieldAddr_05a8 | confirmed | Thiscall, no stack args. Bytes 8d 81 a8 05 00 00 c3 = lea eax,[ecx+0x5a8]; ret. Returns the address this+0x5a8, not the value stored there. |
| `010441f0` | CharacterBase::SetFloatFromPtr_05a8 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 d9 00 d9 99 a8 05 00 00 c2 04 00 = mov eax,[esp+4]; fld dword [eax]; fstp dword [ecx+0x5a8]; ret … |
| `01044200` | CharacterBase::FieldAddr_05ac | confirmed | Thiscall, no stack args. Bytes 8d 81 ac 05 00 00 c3 = lea eax,[ecx+0x5ac]; ret. Returns the address this+0x5ac, not the value stored there. |
| `01044210` | CharacterBase::SetFloatFromPtr_05ac | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 d9 00 d9 99 ac 05 00 00 c2 04 00 = mov eax,[esp+4]; fld dword [eax]; fstp dword [ecx+0x5ac]; ret … |
| `01044220` | CharacterBase::EntryTagKeyString_s359 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01044230` | CharacterBase::FieldAddr_05b0 | confirmed | Thiscall, no stack args. Bytes 8d 81 b0 05 00 00 c3 = lea eax,[ecx+0x5b0]; ret. Returns the address this+0x5b0, not the value stored there. |
| `01044240` | CharacterBase::SetFloatFromPtr_05b0 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 d9 00 d9 99 b0 05 00 00 c2 04 00 = mov eax,[esp+4]; fld dword [eax]; fstp dword [ecx+0x5b0]; ret … |
| `01044250` | CharacterBase::FieldAddr_05b4 | confirmed | Thiscall, no stack args. Bytes 8d 81 b4 05 00 00 c3 = lea eax,[ecx+0x5b4]; ret. Returns the address this+0x5b4, not the value stored there. |
| `01044260` | CharacterBase::SetFloatFromPtr_05b4 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 d9 00 d9 99 b4 05 00 00 c2 04 00 = mov eax,[esp+4]; fld dword [eax]; fstp dword [ecx+0x5b4]; ret … |
| `01044270` | CharacterBase::EntryTagKeyString_s364 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01044280` | CharacterBase::FieldAddr_05b8 | confirmed | Thiscall, no stack args. Bytes 8d 81 b8 05 00 00 c3 = lea eax,[ecx+0x5b8]; ret. Returns the address this+0x5b8, not the value stored there. |
| `01044290` | CharacterBase::SetFloatFromPtr_05b8 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 d9 00 d9 99 b8 05 00 00 c2 04 00 = mov eax,[esp+4]; fld dword [eax]; fstp dword [ecx+0x5b8]; ret … |
| `010442a0` | CharacterBase::FieldAddr_05bc | confirmed | Thiscall, no stack args. Bytes 8d 81 bc 05 00 00 c3 = lea eax,[ecx+0x5bc]; ret. Returns the address this+0x5bc, not the value stored there. |
| `010442b0` | CharacterBase::SetFloatFromPtr_05bc | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 d9 00 d9 99 bc 05 00 00 c2 04 00 = mov eax,[esp+4]; fld dword [eax]; fstp dword [ecx+0x5bc]; ret … |
| `010442c0` | CharacterBase::EntryTagKeyString_s369 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `010442d0` | CharacterBase::FieldAddr_05c0 | confirmed | Thiscall, no stack args. Bytes 8d 81 c0 05 00 00 c3 = lea eax,[ecx+0x5c0]; ret. Returns the address this+0x5c0, not the value stored there. |
| `010442e0` | CharacterBase::SetFloatFromPtr_05c0 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 d9 00 d9 99 c0 05 00 00 c2 04 00 = mov eax,[esp+4]; fld dword [eax]; fstp dword [ecx+0x5c0]; ret … |
| `010442f0` | CharacterBase::FieldAddr_05c4 | confirmed | Thiscall, no stack args. Bytes 8d 81 c4 05 00 00 c3 = lea eax,[ecx+0x5c4]; ret. Returns the address this+0x5c4, not the value stored there. |
| `01044300` | CharacterBase::SetFloatFromPtr_05c4 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 d9 00 d9 99 c4 05 00 00 c2 04 00 = mov eax,[esp+4]; fld dword [eax]; fstp dword [ecx+0x5c4]; ret … |
| `01044310` | CharacterBase::EntryTagKeyString_s374 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01044320` | CharacterBase::FieldAddr_05c8 | confirmed | Thiscall, no stack args. Bytes 8d 81 c8 05 00 00 c3 = lea eax,[ecx+0x5c8]; ret. Returns the address this+0x5c8, not the value stored there. |
| `01044330` | CharacterBase::SetFloatFromPtr_05c8 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 d9 00 d9 99 c8 05 00 00 c2 04 00 = mov eax,[esp+4]; fld dword [eax]; fstp dword [ecx+0x5c8]; ret … |
| `01044340` | CharacterBase::FieldAddr_05cc | confirmed | Thiscall, no stack args. Bytes 8d 81 cc 05 00 00 c3 = lea eax,[ecx+0x5cc]; ret. Returns the address this+0x5cc, not the value stored there. |
| `01044350` | CharacterBase::SetFloatFromPtr_05cc | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 d9 00 d9 99 cc 05 00 00 c2 04 00 = mov eax,[esp+4]; fld dword [eax]; fstp dword [ecx+0x5cc]; ret … |
| `01044360` | CharacterBase::EntryTagKeyString_s379 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01044370` | CharacterBase::FieldAddr_05d0 | confirmed | Thiscall, no stack args. Bytes 8d 81 d0 05 00 00 c3 = lea eax,[ecx+0x5d0]; ret. Returns the address this+0x5d0, not the value stored there. |
| `01044380` | CharacterBase::CopyDwordFromPtr_05d0 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 d0 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x5d0],edx; ret 4. Co… |
| `01044390` | CharacterBase::FieldAddr_05d4 | confirmed | Thiscall, no stack args. Bytes 8d 81 d4 05 00 00 c3 = lea eax,[ecx+0x5d4]; ret. Returns the address this+0x5d4, not the value stored there. |
| `010443a0` | CharacterBase::CopyDwordFromPtr_05d4 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 d4 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x5d4],edx; ret 4. Co… |
| `010443b0` | CharacterBase::EntryTagKeyString_s384 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `010443c0` | CharacterBase::FieldAddr_05d8 | confirmed | Thiscall, no stack args. Bytes 8d 81 d8 05 00 00 c3 = lea eax,[ecx+0x5d8]; ret. Returns the address this+0x5d8, not the value stored there. |
| `010443d0` | CharacterBase::CopyDwordFromPtr_05d8 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 d8 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x5d8],edx; ret 4. Co… |
| `010443e0` | CharacterBase::FieldAddr_05dc | confirmed | Thiscall, no stack args. Bytes 8d 81 dc 05 00 00 c3 = lea eax,[ecx+0x5dc]; ret. Returns the address this+0x5dc, not the value stored there. |
| `010443f0` | CharacterBase::CopyDwordFromPtr_05dc | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 dc 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x5dc],edx; ret 4. Co… |
| `01044400` | CharacterBase::EntryTagKeyString_s389 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01044410` | CharacterBase::FieldAddr_05e0 | confirmed | Thiscall, no stack args. Bytes 8d 81 e0 05 00 00 c3 = lea eax,[ecx+0x5e0]; ret. Returns the address this+0x5e0, not the value stored there. |
| `01044420` | CharacterBase::CopyDwordFromPtr_05e0 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 e0 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x5e0],edx; ret 4. Co… |
| `01044430` | CharacterBase::FieldAddr_05e4 | confirmed | Thiscall, no stack args. Bytes 8d 81 e4 05 00 00 c3 = lea eax,[ecx+0x5e4]; ret. Returns the address this+0x5e4, not the value stored there. |
| `01044440` | CharacterBase::CopyDwordFromPtr_05e4 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 8b 10 89 91 e4 05 00 00 c2 04 00 = mov eax,[esp+4]; mov edx,[eax]; mov [ecx+0x5e4],edx; ret 4. Co… |
| `01044450` | CharacterBase::EntryTagKeyString_s394 | confirmed | No arguments. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns the address of the literal string "entryTag" and writes no memory. |
| `01044460` | CharacterBase::FieldAddr_05e8 | confirmed | Thiscall, no stack args. Bytes 8d 81 e8 05 00 00 c3 = lea eax,[ecx+0x5e8]; ret. Returns the address this+0x5e8, not the value stored there. |
| `01044470` | CharacterBase::SetFloatFromPtr_05e8 | confirmed | Thiscall, one stack arg (pointer p), ret 4. Bytes 8b 44 24 04 d9 00 d9 99 e8 05 00 00 c2 04 00 = mov eax,[esp+4]; fld dword [eax]; fstp dword [ecx+0x5e8]; ret … |
| `0103e6e0` | CharacterBase::ZeroFields_3d0_to_5f4 | likely | __fastcall with ECX = this: stores zero into the 138 dwords from this+0x3d0 through this+0x5f4 (6-byte stores, some through an xmm0 zero) and returns nothing. |
| `0103edb0` | — | unknown | Virtual slot 8 of CharacterBase: pushes the second stack argument and calls vtable slot 1 (+4) of the sub-object at this+0x178 (ECX = that sub-object), then ca… |

**CharacterDataSource** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bfcfe0` | CharacterDataSource::GetAboutMeValue | confirmed | Switches on the requested property, reads player statistics or durations, formats localized labels, and writes the result through the supplied output interface. |
| `00bfdd20` | CharacterDataSource::DtorBody | confirmed | Restores three CharacterDataSource vtable pointers, releases seven owned IString buffers by decrementing their refcounts, then destroys the embedded GuiUserDat… |
| `00bfdf20` | CharacterDataSource::Ctor | confirmed | Stores the passed owner pointer, installs three CharacterDataSource vtables, clears a flag, and initializes seven empty IString members. |
| `00bfe070` | CharacterDataSource::ScalarDeletingDtor | confirmed | Calls 00bfdd20 to destroy the object and calls _free(this) only when flag bit 0 is set; otherwise returns this. |

**CharacterVariables** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b2c80` | CharacterVariables::CharacterVariables | confirmed | Installs CharacterVariables::vftable, zeroes fields +4, +8, +0xc, +0x10 and the 0x40-byte block starting at +0x14 via _memset, and zeroes +4 last. Returns this. |
| `008b2cf0` | CharacterVariables::~CharacterVariables | confirmed | Stores CharacterVariables::vftable into *this and returns; it frees nothing and is called from six MSVC unwind funclets (0158da84, 0158db44, 01597440, 01597778… |
| `008b2d40` | CharacterVariables::ScalarDeletingDestructor | confirmed | Stores CharacterVariables::vftable into *this and, when bit 0 of the flag argument is set, calls operator delete on this; returns this. |
| `01040660` | CharacterVariables::ForEachEntryCallSlot1 | confirmed | Virtual slot 3 of CharacterVariables: walks the list at param_2+8 (links at +0x20) and, for each node, calls vtable slot 1 (+4) of *this with the node's +4 and… |
| `01040f50` | CharacterVariables::LookupKeyOrDefault | confirmed | Virtual slot 2 of CharacterVariables: hashes the C-string key with 00705bc0, walks the 16-entry bucket chain at this+0x14 (hash at entry+0x18, link +0x14), com… |
| `01041c30` | CharacterVariables::SetKeyValue | confirmed | Virtual slot 1 of CharacterVariables: hashes the C-string key with 00705bc0 and scans the 16-entry bucket chain at this+0x14 (hash at entry+0x18, link +0x14, s… |

**PlayerSwitchProfileEvent** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bfb4d0` | PlayerSwitchProfileEvent::PlayerSwitchProfileEvent | confirmed | Calls the base-event initializer, installs PlayerSwitchProfileEvent::vftable, clears the byte at object offset 0x50, and returns this. |
| `00bfb550` | PlayerSwitchProfileEvent::~PlayerSwitchProfileEvent | confirmed | Stores the PlayerSwitchProfileEvent vtable, changes it to PlayerEvent, then calls 007e7980 with ECX=this for base cleanup; it does not free the object. |
| `00bfb5c0` | PlayerSwitchProfileEvent::GetFlag50 | confirmed | Returns the byte stored at this+0x50. |
| `00bfb5e0` | PlayerSwitchProfileEvent::SetFlag50WhenBit0 | confirmed | If param_3 bit 0 is set, writes 1 to this+0x50; always returns 0. |
| `00bfb600` | PlayerSwitchProfileEvent::DeleteWhenFlagSet | confirmed | Calls the event destructor body, frees the object only when param_2 bit 0 is set, and returns the original pointer otherwise. |

**ProxiedCharacter** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00963410` | ProxiedCharacter::OnProfileSwapComplete | confirmed | Sets byte +0x18d4, clears +0x18d5 when set, and for the owning character optionally logs JobHandler:OnProfileSwapComplete; if the profile predicate and other s… |

**(no class)** (37)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00712340` | CharacterAttachmentRecord_DtorBody | confirmed | Stores SoeUtil::List<CharacterAttachmentData,-1>::vftable at [this+0x14c]; unlinks and releases each node of the list at [this+0x150] through 0070e670 until em… |
| `008c6d30` | LocalPlayerLoadFailed | confirmed | Sets dword this+0xc8 to 0. Calls FUN_008b8ff0(4) and, when it returns nonzero, calls vslot 9 (+0x24) of the object at this+0xec. Then calls vslot 16 (+0x40) of… |
| `00bfb620` | ProcessProfileSwap | confirmed | Walks the selected profile's required-slot entries and returns after reporting a missing item. On success it updates profile state and counters, constructs pro… |
| `0102b8a0` | LoadCharacterStatDefinitions | confirmed | Builds a path by assigning param_1 into a local IString (00402ce0) and appending 'CharacterStatDefinitions.txt' (00402770); reads the file through 00fbf4a0, th… |
| `0102c930` | LoadCharacterStatSetMappings | confirmed | Thiscall (ECX = 10-bucket table, param_2 = path prefix): builds prefix + 'CharacterStatSetMappings.txt' and reads it through 00fbf4a0; for each line, parses a … |
| `01041770` | FormatCharacterIdString | confirmed | __thiscall(this, out): formats the 64-bit value at this+0x10/+0x14 as 'Character (%llu)' into the output object through 00402fd0 and returns the dword at out+4. |
| `010428d0` | CtorSub_10c_1d4 | confirmed | __fastcall with ECX = this: zeroes the leading fields (+0..+0x20 region), clears 0x80 bytes at +0x28 (memset via 00d836a0), installs SoeUtil::IString<char> at … |
| `01042ae0` | DtorBody_10c_1d4 | confirmed | __fastcall with ECX = this: destroys the sub-object of 010428d0 (calls 01040fe0, 010412e0 and 01041360 on it), installs the SoeUtil::List<EffectTag,-1> vtable … |
| `00712110` | CharacterAttachmentRecord_Ctor | likely | Zeroes [this], [this+4], [this+0x3c], [this+0x144], [this+0x148], [this+0x150], [this+0x154] and [this+0x158]; calls 00703540 (StringFixed<32> constructor) on … |
| `00712440` | StringListRecord_CopyAssign | likely | Copy-assigns from the record passed on the stack: copies dwords [this], [this+4], [this+0x3c], [this+0x144] and [this+0x148]; assigns the six IString members a… |
| `00713b90` | RecordAndString_Ctor | likely | Calls 00712110 (CharacterAttachment record constructor) with ECX=this, constructs a StringFixed<32> at this+0x160 through 00703540, and returns this. |
| `00713bf0` | RecordAndString_DtorBody | likely | Calls 00703350 on this+0x160 (member cleanup), then calls 00712340 (CharacterAttachment record destructor body) on this; returns void. |
| `00713c90` | ListNodeWithRecord_DtorBody | likely | Calls 00712f40 on this+0x1a0 (FriendDetails list destructor body), then calls 00713bf0 on this+8 (record-plus-string cleanup); returns void. |
| `00713e10` | ListNodeWithRecord_Ctor | likely | Zeroes the byte at [this] and the dword at [this+4]; calls 00713b90 on this+8 (record-plus-string constructor); calls 007124f0 on this+0x1a0 (FriendDetails lis… |
| `00713ed0` | Wrapper_00713e10 | likely | Calls 00713e10 with ECX=this and returns this (the list-node constructor). |
| `00713f20` | ListNodePushFront | likely | Allocates a node through vtable slot 2 of the list object at [ECX] (+8), constructs it through 00713ed0, then links it at the front: node[+0x62b0] = old front,… |
| `00713fc0` | ListNodePushBack | likely | Allocates a node through vtable slot 2 of the list object at [ECX] (+8), constructs it through 00713ed0, then links it at the back: node[+0x62b4] = old back, n… |
| `00714060` | ListNodeWithRecord_ScalarDtor | likely | Calls 00713c90 with ECX=this, then calls _free(this) when bit 0 of the stack byte is set; returns this. |
| `00728a40` | LoadRaceSetIdsFile | likely | thiscall (this = param_1) with one argument (param_2, a format argument). Formats '%sRaceSetIds.txt' with param_2 into a local IString, opens it with FUN_00fbd… |
| `00960960` | ClearStateAndUpdateWaterRelatedObject | likely | Repeatedly clears the dwords at this+0x4d0 and +0x4d4, masks bits at this+0x143, conditionally calls FUN_00b60420, updates a related object through FUN_009d08f… |
| `00b49a10` | SetTitleIdAndNotify | likely | If param_2 differs from the value at this+0x98, stores param_2 there and invokes the callback at the vtable pointer held by this+0x9c, slot +0x10; otherwise re… |
| `00b49a50` | Thunk_00cc55d0 | likely | Loads the pointer at this+0x9c into ECX, then jumps to 00cc55d0; this is an adjustor thunk and contains no callback body. |
| `013a7d20` | PossessivePronounFromEnum | likely | One pointer argument (the out std::string, returned in EAX). Calls 013a36e0 (CachedEnumFromKey3E, speculative) and assigns the out string: value 1 gives his, v… |
| `013a7dc0` | ObjectPronounFromEnum | likely | Same shape as 013a7d20: calls 013a36e0 and assigns the out string him for value 1, the 3-character PTR_LAB_018dedd8 text for value 2, and them otherwise. |
| `00713e80` | Wrapper_00713c90 | speculative | Calls 00713c90 with ECX=this and no stack arguments; returns void. |
| `009616c0` | CanRunCharacterAction | speculative | Starts with a true result, conditionally checks FUN_009ad300 when the context virtual and related state are present, then requires a second virtual check, FUN_… |
| `009653c0` | ApplyHeadingAndRefreshOnce | speculative | Thiscall. Returns at once when the byte at this+0x1b0 is already 1. Otherwise it sets that byte, takes fpatan(param_4[0], param_4[2]) as a float, copies param_… |
| `00965700` | SetTargetAndFlushQueue | speculative | Thiscall with a pointer argument. When param_2 is non-null it sets this+0x504 = param_2+0x268, calls 007744f0(this,1), sets param_2 byte +0x226 from bit 6 of t… |
| `0103ee50` | PercentOf90ByVirtual67 | speculative | __fastcall with ECX = this: calls vtable slot 67 (+0x10c) of *this; when the int it returns is 0 yields 0, otherwise yields (this+0x90 * 100) divided by that i… |
| `0103eed0` | PercentOf98ByVirtual87 | speculative | __fastcall with ECX = this: calls vtable slot 87 (+0x15c); yields 0 when the int it returns is below 1, otherwise (this+0x98 * 100) divided by that int from a … |
| `0103ef80` | PercentOf9cByVirtual530 | speculative | __fastcall with ECX = this: calls vtable slot 332 (+0x530); yields 0 when the int it returns is below 1, otherwise (this+0x9c * 100) divided by that int from a… |
| `01041600` | EntryInitRecord | speculative | __fastcall with ECX = this: under an SEH frame calls 00703540 with ECX = this, then calls it again with ECX = this+0x34 after the guard state is set; then sets… |
| `010419d0` | DrainListNodesCall01360 | speculative | __fastcall with ECX = this: walks the list whose head is this+0x1e4 (next link +0x1f0) and calls 01041360 once per entry with ECX = entry+0x184 (the container … |
| `007288a0` | — | unknown | cdecl with one argument (param_1, a format argument). Formats '%sRaceDefinitions.txt' with param_1 into a local IString (FUN_00402fd0), opens that file with FU… |
| `00728fb0` | — | unknown | cdecl with one argument (param_1, a format argument). Formats '%sRaceSets.txt' with param_1 into a local IString, opens it with FUN_00fbda90(name, 0x5e) and wh… |
| `010417a0` | — | unknown | __thiscall(this, key, flagByte, a, b, modeByte, flag2Byte): looks up the entry through vtable slot 51 (+0xcc). When none exists it allocates 0x98 bytes, initia… |
| `01041940` | — | unknown | __thiscall(this, key): finds the entry through vtable slot 51 (+0xcc); when found, removes it from the 16-entry bucket (010400b0) and the list (01040040) of th… |

### `game.charactercreate`

**BaseDefinitionManager** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0072eff0` | BaseDefinitionManager::DtorBody | confirmed | Under an EH frame calls 0072ee90 (HashListMap5 destructor body) on this, then stores BaseDefinitionManager::vftable at *this. Returns nothing. |

**CCItemClothingSetsDefinition** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00721a50` | CCItemClothingSetsDefinition_ScalarDeletingDtor | confirmed | Installs CCItemClothingSetsDefinition::vftable and calls _free on the object when bit 0 of the flag byte is set. |
| `00725710` | CCItemClothingSetsDefinition_ParseFields | likely | Tokenizes the row into +0x14, skips one delimited token, then stores five numeric columns at +0x18, +0x1c, +0x20, +0x24 and +0x28. |

**CCItemMappingsDefinition** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00721990` | CCItemMappingsDefinition_ScalarDeletingDtor | confirmed | Installs CCItemMappingsDefinition::vftable and calls _free on the object when bit 0 of the flag byte is set. |
| `007255e0` | CCItemMappingsDefinition_ParseFields | likely | Tokenizes five numeric columns from the line into +0x14, +0x18, +0x1c, +0x20, then skips a delimited token and parses a final numeric column into +0x24. |

**CCItemStartingSetsDefinition** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00723200` | CCItemStartingSetsDefinition::CCItemStartingSetsDefinition | confirmed | Installs CCItemStartingSetsDefinition::vftable, clears +4..+0x10, stores param_2 at +0x14, constructs the embedded StringFixed<128> at +0x18 through 00402950 a… |
| `007232c0` | CCItemStartingSetsDefinition::~CCItemStartingSetsDefinition | confirmed | Installs CCItemStartingSetsDefinition::vftable and destroys the embedded IStringFixed<char,128> at +0x18 through 00401fd0. |
| `00723320` | CCItemStartingSetsDefinition_ScalarDeletingDtor | likely | Calls the starting-set destructor body 007232c0 for this and calls _free when bit 0 of the flag byte is set. |
| `007257d0` | CCItemStartingSetsDefinition_ParseFields | likely | Tokenizes the first column into +0x14, skips one delimited token, then stores twelve consecutive numeric columns at +0xac, +0xb0, +0xb4, +0xb8, +0xbc, +0xc0, +… |

**CCItemTypesDefinition** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007219f0` | CCItemTypesDefinition_ScalarDeletingDtor | confirmed | Installs CCItemTypesDefinition::vftable and calls _free on the object when bit 0 of the flag byte is set. |
| `00725690` | CCItemTypesDefinition_ParseFields | likely | Tokenizes the first numeric column into +0x14, skips a delimited token with 006fb560 and stores the next numeric column at +0x18. |

**CCNamesDefinition** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00722c60` | CCNamesDefinition::~CCNamesDefinition | confirmed | Installs CCNamesDefinition::vftable, then rewrites the vtable of the embedded IString at +0x14 and releases that string buffer when it is owned (capacity field… |
| `00722cf0` | CCNamesDefinition::CCNamesDefinition | confirmed | Installs CCNamesDefinition::vftable, clears +4..+0x10, constructs the embedded IString at +0x14 (vtable plus static empty buffer), releases any previous buffer… |
| `00725540` | CCNamesDefinition_ParseFields | confirmed | Scans one delimited [:...:] token from the row with 006fb560, formats it into the embedded string at +0x14 as [:%s:], skips the following delimited token and t… |
| `00722cd0` | CCNamesDefinition_ScalarDeletingDtor | likely | Calls the CCNamesDefinition destructor body 00722c60 for this and calls _free when bit 0 of the flag byte is set; returns this. |
| `00723660` | CCNamesDefinition_ScalarDeletingDtor_B | likely | Under an SEH frame calls the CCNamesDefinition destructor body 00722c60 for this and calls _free when bit 0 of the flag byte is set. |

**CharacterCreationManager** (34)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0097c960` | CharacterCreationManager_CreateStartingSetsDataSource | confirmed | Allocates 600 (0x258) bytes with the instrumented operator new, constructs a CharacterCreationStartingSetsDataSource over the block (callee 00c2acb0, which ins… |
| `0097f690` | CharacterCreationManager_CreateCustomizationCategoryDataSou… | confirmed | Allocates 0x268 bytes, constructs the category data source with FUN_00c2b130 (which names it 'BaseClient.CharacterCustomizationCategory'), stores it at this+0x… |
| `0097f790` | CharacterCreationManager_CreateNameDataSource | confirmed | Allocates 0x268 bytes, constructs a name data source with FUN_00c2b5f0 from its three arguments (category/name key strings), stores the object through the firs… |
| `0097f960` | CharacterCreationManager_CreateServerListDataSource | confirmed | Allocates 0x260 bytes, constructs the server list data source with FUN_00c2a020 (name 'BaseClient.ServerList') over the manager pointer at this+0x37c, stores i… |
| `0097fa70` | CharacterCreationManager_CreateAccountInfoDataSource | confirmed | Allocates 0x264 bytes, constructs the account info data source with FUN_00c2a760 (name 'BaseClient.AccountInfo') passing the manager itself, stores it at this+… |
| `0097fb80` | CharacterCreationManager_CreatePresetDataSource | confirmed | Builds the preset scanner (FUN_010528b0) into this+0x338, points it at 'Resources\' through its vtable slot 1, then allocates 600 bytes, constructs the Charact… |
| `00980370` | CharacterCreationManager_ParseCharacterTypes | confirmed | Parses the CharacterTypes data source: logs 'CharacterTypes:', opens the reader with token '^' (FUN_00fbda90), for each non-'#' line splits typeId, nameStringI… |
| `00980540` | CharacterCreationManager_ParseCustomizationCategories | confirmed | Parses the CustomizationCategories data source: logs 'CustomizationCategories:', opens the reader (token '^'), for each non-'#' row splits categoryId, category… |
| `00980740` | CharacterCreationManager_ParseCustomizationCategoryData | confirmed | Parses the CustomizationCategoryData data source: logs 'CustomizationCategoryData:', splits each non-'#' row into two ints, a third int resolved through FUN_00… |
| `009809f0` | CharacterCreationManager_CreateCustomizationDataSources | confirmed | Walks every node of the customization-category list (head this+0x214, successor +0x7c); per category looks up the node in the 9-bucket table at +0x220 by the s… |
| `00980c50` | CharacterCreationManager_CreateNameCategoryDataSources | confirmed | Creates the 'BaseClient.CharacterNameCategories' data source with FUN_00c2be20 (0x268 bytes) into this+0x2dc, registers and logs 'Created Name Category Data So… |
| `009829f0` | CharacterCreationManager_ParseCharacterNames | confirmed | Parses the CharacterNames data source: logs 'CharacterNames:', opens the reader with token '^', skips comment and 'character.'-prefixed lines, and for each rem… |
| `00982f10` | CharacterCreationManager::LoadCharacterCreationData | confirmed | Loads the whole character-creation data set from a directory prefix argument: builds '<prefix>CharacterCreateCharacterTypes.txt' with a 2048-char StringFixed a… |
| `00983500` | CharacterCreationManager::~CharacterCreationManager | confirmed | Destructor body of the CharacterCreationManager object under SEH frame LAB_01599c12: calls 0097d9c0, 0097d900, 0097d8b0, 0097d860, 00981080, 00981020, 0097d6c0… |
| `009836e0` | CharacterCreationManager::CharacterCreationManager | confirmed | Constructor: under SEH frame LAB_01599cca calls 00981900, sets [this+0x208]=0, calls 00982450, constructs the CustomizationData HashList base (00983340), sets … |
| `0097d670` | CharacterCreationManager_ReleaseMember208 | likely | Releases the tracked object stored in the manager member at this+0x208: calls its vtable slot 0x28 (key accessor) with 0, removes that key from the global mana… |
| `0097d6c0` | CharacterCreationManager_ReleaseMember27c | likely | Releases the tracked object stored in the manager member at this+0x27c: calls its vtable slot 0x28 (key accessor) with 0, removes that key from the global mana… |
| `0097d850` | CharacterCreationManager_GetNameWheelNodeCount | likely | Field getter: returns the dword at this+0x2c8 (the count field of the 4-bucket hash list embedded at +0x2b8, whose bucket array starts at +0x2cc). |
| `0097d860` | CharacterCreationManager_ReleaseMember360 | likely | Releases the tracked object stored in the manager member at this+0x360: calls its vtable slot 0x28 (key accessor) with 0, removes that key from the global mana… |
| `0097d8b0` | CharacterCreationManager_ReleaseMember364 | likely | Releases the tracked object stored in the manager member at this+0x364: calls its vtable slot 0x28 (key accessor) with 0, removes that key from the global mana… |
| `0097d900` | CharacterCreationManager_ReleasePresetMembers | likely | Releases both character-preset members: first the datasource reference at +0x33c through the standard tracked-object release (FUN_0081ad80 + refcount), then ca… |
| `0097d970` | CharacterCreationManager_GetPresetScannerField14 | likely | Getter: when the preset scanner object at this+0x338 exists, returns the dword at its +0x14 field; otherwise returns 0. |
| `0097d9c0` | CharacterCreationManager_ReleaseMember368 | likely | Releases the tracked object stored in the manager member at this+0x368: calls its vtable slot 0x28 (key accessor) with 0, removes that key from the global mana… |
| `0097e4f0` | HashList4_FindNodeByString | likely | String-keyed lookup in the 4-bucket container embedded at +0x2b8: hashes the string pointer at *(param_2+4) with FUN_00705b00, writes the hash to *param_3, wal… |
| `0097e590` | CharacterCreationManager_GetNameCategoryCount | likely | Looks up the name-wheel category for the name structure passed in (its string at +4) via 0097e4f0 and returns the dword at node+0x10, which is the count field … |
| `0097f570` | CharacterCreationManager_FormatCategorySuffix | likely | Looks up the category node by key param_2 in the 9-bucket table at this+0x220 (node hash +0x84, chain +0x88) and formats the identifier string into the caller'… |
| `0097f5d0` | CharacterCreationManager_BuildCategoryDataSourceName | likely | Builds the full data-source name for a category id: constructs a StringFixed<128>, fills it with 0097f570 (the 'CharacterCreateCustomization<suffix>' stem), th… |
| `0097f8a0` | CharacterCreationManager_GetNameCategoryNameAt | likely | Walks the name-wheel category list head at this+0x2c0 through the +0x448 successor links param_2 times, then copies the reached node's name (the StringFixed bu… |
| `0097f8e0` | CharacterCreationManager_GetNameInCategory | likely | Finds the category node for the supplied name string (0097e4f0), then walks the node's embedded 256-bucket name list head at node+8 through +0x34 links param_2… |
| `00981020` | CharacterCreationManager_ClearCharacterCreateDataSources | likely | Iterates the data-source list at this+0x288 and releases every tracked object (virtual key call, FUN_0081ad80 removal, refcount decrement, deleting destructor)… |
| `00981080` | CharacterCreationManager_ClearNameDataSources | likely | Releases every tracked object in the list at this+0x2e4, drains the CharacterNamesDataSourceElement list with 00980f80, then releases the name-category data so… |
| `00982820` | HashList4_InsertNewCategoryNode | likely | Insert for the 4-bucket category container at +0x2b8: allocates a node through vtable slot 1, constructs it with 00982700 (key from the call argument, stored a… |
| `009828e0` | HashList4_RemoveCategoryNode | likely | Removes category node param_2 from the 4-bucket container: unlinks it from its bucket chain (hash +0x450 & 3, chain +0x454), unlinks the list links (node+0x44c… |
| `0097d990` | CharacterCreationManager_GetPresetListEntry | speculative | Walks the linked list reached through *(this+0x338)+0xc, following node+0x40 links param_2 times, and returns the reached node (0 when the list or the index ru… |

**CharacterCreationPresetDefinitionManager** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010524a0` | CharacterCreationPresetDefinitionManager::LoadPresetsFile | confirmed | Thiscall (this = manager; the preset hash map is at this+4). Builds the path from the argument and CharacterCreatePresets.txt, reads the file through 00fbf4a0,… |
| `010528b0` | CharacterCreationPresetDefinitionManager::Ctor | confirmed | Fastcall ctor (this in ECX) with an EH frame. Stores BaseDefinitionManager::vftable at *this, then stores CharacterCreationPresetDefinitionManager::vftable, th… |
| `01052910` | CharacterCreationPresetDefinitionManager_DtorBody | confirmed | Fastcall (this in ECX) with an EH frame. Calls 010527a0 on this+4 (destroys the preset hash list in place), then stores BaseDefinitionManager::vftable at *this… |
| `01052970` | CharacterCreationPresetDefinitionManager_ScalarDeletingDtor | confirmed | Virtual slot [0] of CharacterCreationPresetDefinitionManager::vftable (thiscall; flag byte on the stack). Calls 01052910, then if bit 0 of the flag is set call… |

**ClientAccountDataManager** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00725180` | ClientAccountDataManager::ClientAccountDataManager | confirmed | Constructs the ItemDefinitionManager base chain through 007218e0, installs the two ClientAccountDataManager vtables at +0/+4, clears +8/+0xc/+0x10/+0x14, memse… |
| `00725470` | ClientAccountDataManager::~ClientAccountDataManager | confirmed | Installs the ClientAccountDataManager vtables at +0/+4, calls ClearDefinitions (00725030) to release all definition data, then drains the embedded List<CCNames… |
| `00725030` | ClientAccountDataManager_ClearDefinitions | likely | Drains every definition container: item definitions through 007227c0, the CharacterCreateNames list by repeated 00724150, then the four character-create lists … |
| `00725520` | ClientAccountDataManager_ScalarDeletingDtor | likely | Calls the ClientAccountDataManager destructor body 00725470 for this and frees through FUN_00401c00 when bit 0 of the flag byte is set. |

**ClientAccountInterface** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0071b350` | ReplaceRefCountedMemberA260 | confirmed | Releases old object at offset 0xa260, stores the new pointer, increments its reference count, and returns one. |
| `0071cb20` | ClientAccountInterface::Destructor | confirmed | Installs interface vtable, destroys two inline parameter arrays and ClientAccountDataManager, then tears down AssetRequester state. |
| `0071cec0` | ClientAccountInterface::Constructor | confirmed | Installs AssetRequester and ClientAccountInterface vtables, constructs ClientAccountDataManager and two inline arrays, stores inputs, and initializes selection… |
| `0071d010` | ClientAccountInterface::ScalarDeletingDtor | confirmed | Calls interface destructor and frees object when deleting flag bit is set. |

**ClientAccountManager** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00705610` | AccountManagerStateToString | confirmed | Maps account-manager state values to the corresponding state-name strings, including server-list and character-creation states. |
| `0070ea10` | ClientAccountManager::OnRequestFriendsList | confirmed | Logs result and friend fields, copies entries into a linked list, then sets state 0xff. |
| `0070f490` | ClientAccountManager::Initialize | confirmed | Loads config and loading-screen state, initializes fields, loads four web-service error tables, and returns false if a table fails. |
| `007100c0` | ClientAccountManager::OnRequestCharacterCreate | confirmed | Logs result and appearance fields, maps failures to messages/states, and on success stores returned GUID then sets state 0x29. |
| `00710e20` | ClientAccountManager::OnRequestCheckName | confirmed | Logs names/suggestions, maps result codes to locale keys, and sets state 0x20 or 0x21 on available-name result. |
| `00711220` | ClientAccountManager::OnRequestCharacterLogin | confirmed | Logs result, GUID, server, address, ticket and key; failure displays popup, success stores credentials and sets state 0x2e. |
| `00711ca0` | ClientAccountManager::OnRequestServerList | confirmed | Logs servers and online/locked flags, copies only online unlocked entries, marks availability, and enters state 3. |
| `007140c0` | ClientAccountManager::CreatePlayer | confirmed | Looks up starting-set and clothing/model-customization mappings for the supplied creation values, logs unresolved definition IDs, fills a temporary character r… |
| `0071a530` | ClientAccountManager::Constructor | confirmed | Initializes account data, character-detail list, arrays, strings, error maps, session fields, and configuration; logs when session id is zero. |
| `0071a8f0` | ClientAccountManager::Destructor | confirmed | Releases manager interfaces, strings, maps, arrays, character lists, data managers, and asset requester state. |
| `007073a0` | ClientAccountManagerListNodeAtIndex | likely | Walks the indexed list at manager offset 0x22290 using node link offset 0x62b0 and returns the node or null. |
| `00705a60` | InvokeAccountManagerService | speculative | Checks two flags, manager state, and the input value, calls an interface virtual, and logs through the account-manager logger. |

**GenderDefinitionManager** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0072eba0` | GenderDefinitionManager::LoadGenderDefinitions | confirmed | Assigns the path argument into a local IString and appends 'GenderDefinitions.txt', opens it through 00fbf4a0, then for each line not starting with '#' reads t… |
| `0072ef90` | GenderDefinitionManager::GenderDefinitionManager | confirmed | Stores BaseDefinitionManager::vftable and then GenderDefinitionManager::vftable into *this, then calls 0072ef00 (HashList5_Ctor) on this. Returns this. |
| `0072f050` | GenderDefinitionManager::ScalarDeletingDtor | confirmed | Calls 0072eff0 on this, then frees this through 00401c00 with alignment 0 when bit 0 of param_2 is set. Returns this. |

**IdToStringLookup** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00725f40` | IdToStringLookup::IdToStringLookup | confirmed | Installs BaseDefinitionManager::vftable then IdToStringLookup::vftable, clears +4/+8, memsets the 400-byte (100-bucket) table at +0xc, constructs the embedded … |
| `00725ff0` | IdToStringLookup::~IdToStringLookup | confirmed | Installs IdToStringLookup::vftable, calls RemoveAll (00725d90) to free every entry, destroys the embedded IStringNoShare through 00722bf0 and finishes with the… |

**(no class)** (92)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0070cc80` | LogAcceptedCreationName | confirmed | Converts name text, splits first and last components, measures wide and UTF-8 lengths, then formats a diagnostic. |
| `0070d0c0` | LogNameCheckInput | confirmed | Stores a flag, formats first and last names with lengths, optionally logs, then releases owned strings. |
| `0070f170` | LoadCreateCharacterErrors | confirmed | Logs progress and loads CreateCharacterErrors into manager offset +0x53da0. |
| `0070f1b0` | LoadCheckNameErrors | confirmed | Logs progress and loads CheckNameErrors into manager offset +0x53df4. |
| `0070f1f0` | LoadLoginResultErrors | confirmed | Logs progress and loads LoginResultErrors into manager offset +0x53e48. |
| `0070f230` | LoadCharacterDeleteErrors | confirmed | Logs progress and loads CharacterDeleteErrors into manager offset +0x53e9c. |
| `00711a60` | SelectCharacterDummyByRaceGender | confirmed | Only runs in state 0x13. Scans preset records, skips records whose first byte is nonzero, and matches race at object+0x6e4 plus gender at +0x6e8. If the matchi… |
| `0071c150` | HandleCustomizationSlot | confirmed | Switches on customization slots 1 through 9, performs item and model lookups and string formatting, and updates customization or preview state. |
| `00721a70` | NewCCItemMappingsDefinition | confirmed | Allocates 0x28 bytes through FUN_00d833a6, installs CCItemMappingsDefinition::vftable, zeroes fields +4..+0x10, stores the passed dword at +0x14 and zeroes +0x… |
| `00721af0` | NewCCItemTypesDefinition | confirmed | Allocates 0x1c bytes through FUN_00d833a6, installs CCItemTypesDefinition::vftable, zeroes +4..+0x10, stores the passed dword at +0x14 and zeroes +0x18; return… |
| `00721b70` | NewCCItemClothingSetsDefinition | confirmed | Allocates 0x2c bytes through FUN_00d833a6, installs CCItemClothingSetsDefinition::vftable, zeroes +4..+0x10, stores the passed dword at +0x14 and zeroes +0x18.… |
| `007236c0` | LoadCharacterCreateNames | confirmed | Builds the path prefix from param_1, appends CharacterCreateNames.txt, opens it through the text-file reader (00fbda90/00fbdaf0), loops lines via 00fbe130/00fb… |
| `00723860` | LoadCharacterCreateItemMappings | confirmed | Opens CharacterCreateItemMappings.txt through the text-file reader, per non-comment line tokenizes with 007065d0, allocates a CCItemMappingsDefinition with 007… |
| `00723a70` | LoadCharacterCreateItemTypes | confirmed | Opens CharacterCreateItemTypes.txt, per non-comment line allocates a CCItemTypesDefinition with 00721af0 using the first column, fills fields with 00725690 and… |
| `00723c80` | LoadCharacterCreateItemClothingSets | confirmed | Opens CharacterCreateItemClothingSets.txt, per non-comment line allocates a CCItemClothingSetsDefinition with 00721b70 using the first column, fills fields wit… |
| `00723e90` | LoadCharacterCreateItemStartingSets | confirmed | Opens CharacterCreateItemStartingSets.txt, per non-comment line allocates a CCItemStartingSetsDefinition with 00723340, fills the twelve columns at +0xac..+0xd… |
| `00724d80` | LoadAllDefinitionFiles | confirmed | Runs all six definition loaders in order (ItemDefinitions.txt, CharacterCreateNames, ItemMappings, ItemTypes, ItemClothingSets, ItemStartingSets) with the give… |
| `00725110` | ListCCNames_Destruct | confirmed | Installs SoeUtil::List<CCNamesDefinition,-1>::vftable and drains the list by repeatedly calling CCNamesList_Remove (00724150) on the head at +4. |
| `00725140` | ListCCNames_ScalarDeletingDtor | confirmed | Installs SoeUtil::List<CCNamesDefinition,-1>::vftable, drains the list through 00724150 and calls _free on the container when bit 0 of the flag byte is set. |
| `0097eeb0` | CustomizationCategoryNode_CtorBase | confirmed | Base constructor of the customization-category node: installs SoeUtil::IString<char>::vftable at +4 with the shared empty-string buffer (&DAT_01be52dc) and zer… |
| `0070ca40` | BuildHairDescriptorPath | likely | For param_3==0, searches the input string at param_1+4 for "_hair_" using the two-argument case-insensitive helper; a qualifying earlier match selects the bald… |
| `0070cc70` | SetAccountManagerState30 | likely | Calls 0070c8b0 with state 0x30 and secondary value 0. |
| `0070cfa0` | SetTemporaryPlayerName | likely | Assigns three strings, then logs a name field using the fixed label Temp. Player Name. |
| `0070cff0` | SetAccountManagerState12 | likely | Calls 0070c8b0 with state 0x12 and secondary value 0. |
| `0070d000` | EnterAccountState4 | likely | If state differs from 4 and callback field is empty, sets state 4, calls 00952740, and stores the callback argument. |
| `0070d040` | EnterAccountState2 | likely | If state differs from 2 and callback field is empty, sets state 2, calls 00952750, and stores callback context. |
| `0070e7c0` | LoadWebServiceErrorTable | likely | Reads delimited rows and skips lines beginning with #. Parses the first token as an error code, parses a second token into temporary storage but does not use i… |
| `0070e9a0` | HasChildLinks | likely | Returns 0 if the child structure at +0x2227c is absent or both child links are null. If either link exists, it conditionally sets state 9 and calls 00952760 on… |
| `0070f270` | SelectCharacterDummyIndex | likely | Stores the supplied index at +0x53a20, constructs a temporary ClientLuaParam event, emits Select and HUD:PlaySound, extends the selection array when needed, in… |
| `00721c70` | HashList50_FindKeyAt9c78 | likely | Indexes the 50-bucket table at this+0x9c78 with param_2 modulo 0x32 and walks the chain at node+4 comparing the key at node+8; returns the matching node or 0. |
| `00721cb0` | HashList10_FindKeyAt9d50 | likely | Indexes the 10-bucket table at this+0x9d50 with param_2 modulo 10 and walks the chain at node+4 comparing the key at node+8; returns the matching node or 0. |
| `00721cf0` | HashList50_FindKeyAt9d88 | likely | Indexes the 50-bucket table at this+0x9d88 with param_2 modulo 0x32 and walks the chain at node+4 comparing the key at node+8; returns the matching node or 0. |
| `00721d30` | HashList50_FindKeyAt9e60 | likely | Indexes the 50-bucket table at this+0x9e60 with param_2 modulo 0x32 and walks the chain at node+4 comparing the key at node+8; returns the matching node or 0. |
| `00721d70` | CCNamesList_GetByIndex | likely | Follows the +0x28 link chain from the list head at this+0x9c5c param_2 times and returns that node, or 0 when the chain ends first. |
| `00722850` | HashList50_RemoveAll_A | likely | Walks the node list at this+4: for each node removes it from its 50-bucket hash chain (bucket key at node+8 modulo 0x32 over table this+0x10), fixes the doubly… |
| `007228f0` | HashList10_RemoveAll | likely | Same removal walk as 00722850 but with a 10-bucket hash (key modulo 10) over the table at this+0x10. |
| `00722990` | HashList50_RemoveAll_B | likely | Same removal walk as 00722850 with a 50-bucket hash over the table at this+0x10; separate instantiation for another row type. |
| `00722a30` | HashList50_RemoveAll_C | likely | Same removal walk as 00722850 with a 50-bucket hash over the table at this+0x10; third instantiation of the template. |
| `00722e40` | ConstructCCNamesDefinition | likely | Under an SEH frame calls the CCNamesDefinition constructor body 00722cf0 for the incoming object pointer and returns that pointer. |
| `00722ff0` | CCNamesDefinitionDtorCall | likely | Under an SEH frame forwards its incoming this pointer to the CCNamesDefinition destructor body 00722c60 and returns; no other work. |
| `00723340` | NewCCItemStartingSetsDefinition | likely | Allocates 0xdc bytes through FUN_00d833a6 and constructs a CCItemStartingSetsDefinition through 00723200, returning 0 when the allocation fails. |
| `00723410` | CCNamesList_Append | likely | Allocates a node through the container vtable slot +8, constructs a CCNamesDefinition in it via 00722e40, links the node at the list tail (list head +4, tail +… |
| `00724150` | CCNamesList_Remove | likely | Unlinks node param_2 from the CCNames list (head this+4, tail this+8, node prev/next at +0x28/+0x2c), decrements the count at +0xc and then destroys the row th… |
| `00724d60` | CCNamesList_RemoveAll | likely | While the list head at this+4 is non-null, repeatedly calls CCNamesList_Remove (00724150) on it, emptying the CharacterCreateNames list. |
| `00725930` | HashList100_Construct | likely | Zeroes +0 and +4 and memsets 400 bytes (100 dwords) of bucket table at +8. |
| `00725970` | HashList100_Unlink | likely | Hashes the node key at +0x18 modulo 100, walks the chain at this+8 (link at node+0x14), unlinks the matching node, zeroes its +0x14/+0x18 fields, decrements th… |
| `00725a70` | IdHash_FindValue | likely | Hashes param_2 modulo 100, walks the chain at this+0xc (node next link +0x14, key +0x18) and returns the dword stored at node+8; when nothing matches it return… |
| `00725d90` | IdToStringLookup_RemoveAll | likely | Scans the 100 buckets at this+0xc; for every chained entry it unlinks it from its bucket, zeroes the +0x14/+0x18 links, decrements the count at +8, destroys th… |
| `0097d7b0` | ReleaseTrackedObjectPtr | likely | Same release sequence as the member helpers but takes a pointer to a RefPtr field: if *param_1 is non-null it calls vtable slot 0x28 (0), removes that key from… |
| `0097d800` | ReleaseTrackedObjectPtrCopy | likely | Byte-identical twin of 0097d7b0: releases the tracked object held at *param_1 (virtual key call, global-list removal via FUN_0081ad80, refcount decrement, opti… |
| `0097e450` | ReleaseTrackedObjectPtr_EH | likely | Pointer release helper referenced from two EH unwind funclets (destructor of a member): releases the tracked object at *param_1 exactly like 0097d7b0 (virtual … |
| `0097e4a0` | ReleaseTrackedObjectPtr_EH2 | likely | Second copy of the unwind-funclet release helper: drops the tracked object referenced at *param_1 (key call, list removal, refcount decrement, deleting destruc… |
| `0097e700` | ReleaseTrackedObjectPtr_SEH | likely | SEH-framed copy of the pointer release helper: when *param_1 is non-null it calls vtable slot 0x28 (0), removes that key from the global tracked-object list (F… |
| `0097e780` | ReleaseTrackedObjectPtr_SEH2 | likely | Second SEH-framed copy of the release helper (same body as 0097e700: virtual key call, FUN_0081ad80 removal, refcount decrement, deleting destructor, pointer z… |
| `0097e8b0` | NameElementList_InsertNewTail | likely | Allocates a node through the container's vtable slot 2 (AllocNode), zeroes the node's first dword, links it at the tail ([2] of the list header: node+8 <- old … |
| `0097e940` | NodeCtor_StringFixed32_Key3c | likely | Constructs a 0x44-byte node carrying an SoeUtil StringFixed<32> (callee 00703540 constructs it) and stores the integer key from *param_2 at node+0x3c; returns … |
| `0097e9a0` | TwoStringFixed32_Dtor | likely | Destructor body that destroys two StringFixed<32> members (calls the IStringFixed<32> destructor FUN_00703350 twice, under an SEH frame); referenced from four … |
| `0097ea10` | CustomizationCategoryNode_Dtor | likely | Destructor body of the 0x8c-byte customization-category node: destroys the two StringFixed<32> members (FUN_00703350 x2), then releases the shared SoeUtil::ISt… |
| `0097eab0` | CustomizationDataNode_Dtor | likely | Destructor body of the customization-data node: releases the two shared IString members at +0x7c and +0x8c (refcount decrement at buffer-4, vtable slot 2 free … |
| `0097ecc0` | RefPtrWrapper_ScalarDeletingDtor | likely | Scalar deleting destructor: runs the release body 0097e700 on this and, when bit 0 of the flag is set, returns the object to _free. |
| `0097ece0` | RefPtrWrapper_ScalarDeletingDtor2 | likely | Scalar deleting destructor: runs 0097e780 on this and frees the object with _free when the delete flag bit is set. |
| `0097ee40` | NodeCtor_TwoStringFixed32 | likely | Node constructor: stores 0 at node+0 and then constructs two SoeUtil StringFixed<32> members through FUN_00703540 (two calls under one SEH frame); returns this. |
| `0097ef30` | CustomizationDataNode_Ctor | likely | Constructor of the customization-data node: zeroes fields 0..4 (field 3 set to -1), constructs two StringFixed<32> members via FUN_00703540 and installs two So… |
| `0097f020` | CharacterTypeNode_CtorWrapper | likely | Construction wrapper: calls the node constructor 0097ee40 on this (which zeroes the node and builds its two StringFixed<32> members) and returns this. |
| `0097f070` | CustomizationCategoryNode_CtorWithKey | likely | Constructs a customization-category node by calling the base constructor 0097eeb0 and then storing the key value *param_2 at node+0x84 (the hash/id used by the… |
| `0097f0d0` | CustomizationDataNode_CtorWrapper | likely | Construction wrapper calling 0097ef30 on this and returning this. |
| `0097f3f0` | ScalarDeletingDtor_TwoFixed32 | likely | Scalar deleting destructor: runs the two-StringFixed<32> destructor body 0097e9a0 on this and frees the object with _free when the flag bit 0 is set. |
| `0097f450` | CustomizationCategoryNode_ScalarDeletingDtor | likely | Scalar deleting destructor: runs the category-node destructor body 0097ea10 on this and frees the object with _free when the flag bit 0 is set. |
| `0097f4b0` | ScalarDeletingDtor_Fixed32 | likely | Scalar deleting destructor wrapping the single StringFixed<32> destructor FUN_00703350, followed by _free when the delete flag is set. |
| `0097f510` | CustomizationDataNode_ScalarDeletingDtor | likely | Scalar deleting destructor: runs the customization-data node destructor 0097eab0 and frees the object with _free when the delete flag is set. |
| `0097f940` | CharacterCreationManager_FormatFullDataSourceName | likely | Formats 'BaseClient.%s' into the output string using the dword at this+4 (the buffer pointer of a name data source object's IString). |
| `0097fd30` | List_CharacterType_InsertNewNode | likely | Allocates a 0x78-byte node through the container's vtable slot 2 (AllocNode), constructs it with 0097f020 and appends it at the tail of the List<CharacterType>… |
| `0097fdc0` | HashList9_InsertNewCategoryNode | likely | Allocates a node through the container's vtable slot 1 (AllocNode), constructs it with 0097f070 (which stores the key from param_2 at node+0x84), then links it… |
| `0097ffb0` | List_CustomizationData_InsertNewNode | likely | Allocates a node through the container's vtable slot 2, constructs it with 0097f0d0 and appends it at the tail of the List<CustomizationData> (node+0xa0 next, … |
| `009825c0` | CharacterNamesDataSource_Ctor | likely | Constructor of the character-name data source object: builds the embedded 256-bucket hash list with 00981780 and then constructs one SoeUtil StringFixed<32> me… |
| `00982620` | CharacterNamesDataSource_Dtor | likely | Destructor of the character-name data source object: destroys the StringFixed<32> member with FUN_00703350 and then the embedded hash list with 00981710. |
| `009826b0` | CharacterNamesDataSource_DtorThunk | likely | Destructor thunk: under the stack-cookie/SEH prologue it calls the character-name data-source destructor 00982620 and returns. |
| `00982700` | NameWheelCategoryNode_Ctor | likely | Constructor of the name-wheel category node (0x458 bytes): constructs the embedded name hash list and name string via 009825c0, then stores the key value *para… |
| `009827c0` | NameWheelCategoryNode_ScalarDeletingDtor | likely | Deleting destructor of the name-wheel category node: runs the node destructor 00982620 and frees the object with _free when the delete flag is set. |
| `0070d080` | EnterAccountState9 | speculative | If current state is not 9, sets state 9 and calls 00952760 with two arguments. |
| `007253a0` | GetField_14 | speculative | Returns the dword field at this+0x14. |
| `007253b0` | GetField_9c64 | speculative | Returns the dword field at this+0x9c64. |
| `007253c0` | GetField_9c5c | speculative | Returns the dword field at this+0x9c5c. |
| `007253d0` | GetField_9c74 | speculative | Returns the dword field at this+0x9c74. |
| `007253e0` | GetField_9c6c | speculative | Returns the dword field at this+0x9c6c. |
| `007253f0` | GetField_10 | speculative | Returns the dword field at this+0x10. |
| `00725400` | GetField_9d4c | speculative | Returns the dword field at this+0x9d4c. |
| `00725410` | GetField_9d84 | speculative | Returns the dword field at this+0x9d84. |
| `00725420` | GetField_9e5c | speculative | Returns the dword field at this+0x9e5c. |
| `00725430` | GetField_9e54 | speculative | Returns the dword field at this+0x9e54. |
| `00725440` | GetField_10_B | speculative | Returns the dword field at this+0x10; duplicate body of 007253f0. |
| `00725460` | GetField_28 | speculative | Returns the dword field at this+0x28. |

### `game.customization`

**NewHatHairAttachmentGroup** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0095fec0` | NewHatHairAttachmentGroup::~NewHatHairAttachmentGroup | confirmed | Restores the NewHatHairAttachmentGroup vtable, calls FUN_004022b0 on string subobjects at this+0x120 and this+0xa8, calls FUN_00709b80 on the subobject at this… |
| `0095ff80` | NewHatHairAttachmentGroup::ScalarDeletingDtor | confirmed | Calls the class destructor, then frees param_1 when deleting flag bit 0 is set. |

**ProxiedCharacterAttachmentGroup** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0095ff70` | ProxiedCharacterAttachmentGroup::GetByte9D | confirmed | Returns the byte stored at this+0x9d. |

**SoeUtil::HashList<PlayerCustomizationData,10,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bd9df0` | HashList_PlayerCustomizationData::ScalarDeletingDtor | confirmed | Virtual scalar deleting destructor (slot 0 of SoeUtil::HashList<PlayerCustomizationData,10,-1>::vftable): calls the destructor body 00bd9920 and, when the low … |

**SoeUtil::HashListMap<int,PlayerCustomizationData,10,-1>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bd9850` | HashListMap_PlayerCustomizationData::~HashListMap_PlayerCus… | confirmed | Stores the SoeUtil::HashListMap<int,PlayerCustomizationData,10,-1> vftable at object+0, then loops while the head pointer (+8) is non-null, calling 00bd8d70 on… |
| `00bd9880` | HashListMap_PlayerCustomizationData::ScalarDeletingDtor | confirmed | Virtual scalar deleting destructor (vtable slot 0 of SoeUtil::HashListMap<int,PlayerCustomizationData,10,-1>::vftable): stores the map vtable at object+0, then… |
| `00bd9920` | HashListMap_PlayerCustomizationData::~HashListMap_PlayerCus… | confirmed | Destructor body: stores the map vtable at object+0 and drains the node list by repeatedly calling 00bd8d70 on the head pointer at +8 until it is null; it never… |
| `00bd9d70` | HashListMap_PlayerCustomizationData::Ctor | confirmed | Constructor body: stores SoeUtil::HashListMap<int,PlayerCustomizationData,10,-1>::vftable at object+0, zeroes the pointer/size/count fields at +4..+0x10, zeroe… |
| `00bd5d00` | HashListMap10_FreeNode | likely | Virtual slot [2] of the PlayerCustomizationData 10-bucket map (read from the binary): frees its argument through 00401c00 (FreeMemory) with alignment 4; return… |
| `00bd5cc0` | HashListMap10_Slot1Helper | speculative | Virtual slot [1] of the PlayerCustomizationData 10-bucket map (read from the binary): calls 006fb9c0, then calls vtable slot [1] of the object returned by 006f… |

**(no class)** (20)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00780350` | CustomizationRecordDispatch_00780350 | confirmed | Thiscall with one argument (record pointer p2, ret 4). Returns 1 when this+0x4b8 is null; otherwise switches on the record's type dword *p2: type 0 returns 1; … |
| `007827a0` | CustomizationRecordDispatchAlloc_007827a0 | confirmed | Thiscall with two stack arguments (record pointer p2 whose first dword is the type, byte p3), returns 1 or 0. When this+0x4b8 is zero, the record type is non-z… |
| `0079e2f0` | LoadTintSetsXml | confirmed | Runs only while byte this+0xa0 is clear. Builds IString locals from TintSets.xml and calls library routines (00402770, 00d51ba0, 00fbf4a0, 00d549d0, 00d50ac0) … |
| `0072db70` | CustomizationSlotValueByKind | likely | Switches on param_2: 0 selects 'CustomizationHead', 2 'CustomizationHair', 5 or 7 'CustomizationModel', other values return 0. When param_1 is non-null and 007… |
| `00796850` | LoadTintValuesFromXml | likely | Thiscall (ECX = container, param_2 = config object whose +4 is the document name). Clears the tables with 00796650 and 00796330, opens the document through 00f… |
| `007e1c50` | BinXmlReadCustomizationSlotRef | likely | void, one stack arg (the BinXML reader), 434 bytes. Loops until 007df5e0 is true: tag 1 reads a flag byte (00f9f7a0), tag 2 reads a dword (00f9f8e0) kept as th… |
| `00958110` | SetHairTintProperty | likely | Stores param_2 at this+0x5f0 and calls 0072ce90 with the exact property name HairTint and the value. |
| `00958130` | SetEyeTintProperty | likely | Stores param_2 at this+0x5f4 and calls 0072ce90 with the exact property name EyeTint and the value. |
| `00958150` | GetEyeTint | likely | If the linked owner references this object and has a customization record, returns record field +0x35528; otherwise returns this+0x5f4. |
| `00958180` | GetHairTint | likely | If the linked owner references this object and has a customization record, returns record field +0x35524; otherwise returns this+0x5f0. |
| `00bd5980` | CommittedSlotMatch | likely | Thiscall (ECX = object with a 9-slot value block) with arg = record: selector at arg+0xf4 chooses the slot (0 at +0x50, 1 at +0x58, 2 at +0x54, 3 at +0x48, 4 a… |
| `00bd8a00` | PendingResetSlots | likely | For selector p2 (8, 3, 4, 0, 2, 1, 6, 5, 7, or -1 for all) copies the committed slot value (object+0x44..+0x60) into the pending slot (+0xb4..+0xd0). For slots… |
| `00bd8b20` | PendingCommitSlots | likely | For selector p2 (8, 3, 4, 0, 2, 1, 6, 5, or -1 for all) compares each pending slot (+0xb4..+0xd0) with its committed value (+0x44..+0x60). Where they differ it… |
| `00bd8e00` | PlayerCustomization_ApplyChangeRecord 🌐 | likely | Applies one PlayerCustomizationData-shaped record (param_2: Id at [0], StringParam IString at +4, Param at [5], ItemId at [6]) addressed by a 64-bit guid (para… |
| `00bd9560` | Slots_CommitChangedApply | likely | Builds a stack record shaped like PlayerCustomizationData (Id int, an IString member set to the static empty string DAT_01beaac0, and Param). For selector -1 i… |
| `00bd9730` | PlayerCustomization_ReplaceFromPacket 🌐 | likely | When its source argument is non-null: removes all nodes from its container via the tail (+0xc) using 00bd8d70. For each source record linked from source+0x10 (… |
| `00bd9e20` | PlayerCustomization_Ctor | likely | Constructor of the PlayerCustomization object: stores the caller argument at object+0, constructs the embedded HashListMap<int,PlayerCustomizationData> at +4 (… |
| `00bd5e90` | — | unknown | Hashes seven literal ItemGroup key names read through data pointers 01b65530..01b65548 (Hair, HairColor, FacialHair, SkinTone, EyeColor, FaceDesign, Species). … |
| `00bd9990` | — | unknown | Destructor body of the object built by 00bd9e20: for the seven refcounted pointers at +0x124..+0x140 (data sources) it calls virtual slot 0x28 then FUN_0081ad8… |
| `01051580` | — | unknown | Plain function (param_1 = hair record with a string at +4 and an int count at +8; param_2 = selector). Builds a local SoeUtil string via 010513d0. Selector 0: … |

### `ui.charactercreate`

**ClientAccountInterface** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0071d030` | ClientAccountInterface::BuildItemData | confirmed | Looks up customization data and dispatches SetItemData, SetItemIcon, SetItemLabel, and SetItemHighlight UI events with parameter arrays. |
| `0071f030` | ClientAccountInterface::ChangeItem | confirmed | Switch on slotId (param_3) 1..9 = skintone, hairstyle, haircolor, eyecolor, facialhair, facepaint, wings, clothes, head/face-type: stores itemId into this+0x9f… |
| `0071fd60` | ClientAccountInterface::BuildCategoryItems | confirmed | Walks categories 1..9; for each it fetches the current mapping/definition row from this+0x10, builds a fixed 7-parameter ClientLuaParam array through 0070b470/… |

**ClientAccountManager** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0070fdc0` | ClientAccountManager::ShowWebServiceErrorMessage | confirmed | Stores continuation state and logs it. If the supplied row is null, logs Unknown web service error and skips popup/state changes. Otherwise selects text or mes… |
| `0070ff40` | ClientAccountManager::ShowErrorMessage | confirmed | Stores continuation state, starts popup UI, fills message/display parameters, sets popup fields, then requests state 0x33. |

**(no class)** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0070fcd0` | DispatchCharacterCreateOffset | confirmed | When account connection exists, converts offset, prepares array, dispatches CharacterCreate:SetOffset, and calls 0071d030. |
| `0071afa0` | ShowCharacterCreateHide | confirmed | Dispatches CharacterCreate:Hide with two zero arguments. |
| `0071daa0` | BuildCharacterCreateCategoryButtons | confirmed | Iterates nine customization categories, gathers selected option label, count, id, and formatted data, then emits setup-button UI events. |
| `0071e770` | BuildCharacterNameData | confirmed | Iterates name rows, groups female and male first names plus last-name prefixes and suffixes, counts each group, and dispatches four data events. |
| `0071efe0` | ShowCharacterCreateScreen | confirmed | Clears byte this+0x9fac, dispatches CharacterCreate:Show, calls 0071b1b0, 0071daa0, and 0071d030(1,0,0), then stores 3 at this+8. |
| `00c13e70` | RegisterCharacterCreateUiLuaModule | confirmed | Registers the Lua module "CharacterCreateUi": FUN_00e39e40(param_1, "CharacterCreateUi", &table_0182bb88, 0) with 20 entries (CreationFinished, CreationAborted… |
| `00c14000` | CreationNameAccepted | confirmed | Lua C function bound as CharacterCreateUi.CreationNameAccepted: requires more than 2 arguments and a live creation manager (+8 non-zero); builds three SoeUtil:… |
| `0071efb0` | RequestCharacterCreateResourcesAndBuildNameData | likely | Calls the virtual method at this+0x10 vtable byte offset 4 with ./Resources/CharacterCreate/, sets byte this+0x9fad, calls 0071e770, then returns 1. It does no… |
| `00c13b40` | CreationChangeCategory | likely | Lua C function bound as CharacterCreateUi.CreationChangeCategory: requires exactly one argument, checks the character-create manager through the client singlet… |
| `00c13b90` | CreationChangeItem | likely | Lua C function bound as CharacterCreateUi.CreationChangeItem: requires exactly two arguments, checks the character-create manager through the client singleton … |
| `00c13bf0` | CreationRequestItemData | likely | Lua C function bound as CharacterCreateUi.CreationRequestItemData: requires exactly two arguments, checks the character-create manager through the client singl… |
| `00c13e90` | CreationSelectCurrentCharacter | speculative | Lua C function bound as CharacterCreateUi.CreationSelectCurrentCharacter: when the character-create manager exists (+8 non-zero), if at least one argument is p… |
| `00705870` | — | unknown | Reads two nested manager fields at offsets 0x6e4 and 0x6e8 and returns their boolean combination. |
| `00c13da0` | — | unknown | Lua C function bound as CharacterCreateUi.CreationGetPlayerRaceAndGender: when the character-create manager exists (+8 non-zero) and FUN_00705870() reports a s… |
| `00c13f00` | — | unknown | Lua C function bound as CharacterCreateUi.CreationGetState: returns 0 immediately when the character-create manager is missing (+8 zero); otherwise reads FUN_0… |

### `ui.equip_screen`

**EquipmentSlotsDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a737b0` | EquipmentSlotsDataSource::ColumnName | confirmed | Virtual slot [1]. Switch on the int argument: 0 returns Required, 1 Members Only, 2 Min Profile Rank, 3 Is Equippable, 4 Is Required, 5 Slot Name, any other va… |
| `00a739a0` | EquipmentSlotsDataSource::DtorBody | confirmed | Stores the EquipmentSlotsDataSource vtables at this, +0x30 and +0x34, then calls 0091bf60 (accepted label Dtor_GuiUserDataTableAt34) with ECX = this. Does not … |
| `00a73a10` | EquipmentSlotsDataSource::Ctor | confirmed | Calls the base constructor 00922890 with the name string BaseClient.EquipmentSlots, then installs the EquipmentSlotsDataSource vtables at +0, +0x30 and +0x34. … |
| `00a73a80` | EquipmentSlotsDataSource::ScalarDeletingDtor | confirmed | Virtual slot [0]. Calls the dtor body 00a739a0 on this, then calls _free on this when bit 0 of its stack argument is set; returns this (ret 4). |
| `00a73820` | — | unknown | Virtual slot [4] (key, column, out). Gets the global object pointer through 00769710 and its +4 sub-object, then the table owner from 00769fe0; returns 0 when … |

**(no class)** (29)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c10aa0` | RegisterEquipScreenLuaModule | confirmed | Registers the Lua module "EquipScreen": calls FUN_00e39e40(param_1, "EquipScreen", &table_at_0182b3d0, 0) and returns 0. The table maps 31 Lua names (Populate,… |
| `00990600` | SetVehicleEquipScreenState | likely | Walks the vehicle data-source list at this+0x364, updates each vehicle id, sets the slot filter, writes the supplied value at nested offset +0x260, then invoke… |
| `00a10750` | Slots_CommitChangedApplyThunk | likely | Eleven-byte adjustor thunk adds 0x354dc to ECX, then jumps to 00bd9560; it has no separate body. |
| `00c10bb0` | EquipScreenEquipItemGroup | likely | Lua C function (EquipScreen.EquipItemGroup): if arg1 is a number it parses it and calls the client method FUN_00bdbf90(value); returns 0. |
| `00c10bf0` | EquipScreenEquipItem | likely | Lua C function (EquipScreen.EquipItem): requires arg1 truthy and exactly 2 arguments (FUN_00e3a390 == 2); reads arg2 as an integer (FUN_00e3a7b0) and arg1 as a… |
| `00c10c50` | EquipScreenGetItemPreviewType | likely | Lua C function (EquipScreen.GetItemPreviewType): starts with result 0; when arg1 is truthy and there is exactly 1 argument, reads arg1 with FUN_00e3a7b0 and ma… |
| `00c10f10` | EquipScreenGetMaxItemsPurchasable | likely | Lua C function (EquipScreen.GetMaxItemsPurchasable): when arg1 is truthy and exactly one argument is present, calls FUN_00bdac10(arg1) and pushes the returned … |
| `00c11020` | EquipScreenPreviewStyleCard | likely | Lua C function (EquipScreen.PreviewStyleCard): if arg1 is a number it parses it and calls the client method FUN_00a10760(value); returns 0. |
| `00c11110` | EquipScreenSelectVehicle | likely | Lua C function (EquipScreen.SelectVehicle): if arg1 is a number it parses arg1 and arg2 (FUN_00e3a770 + FUN_00d83c90 each) and calls the client method FUN_00b8… |
| `00c111f0` | EquipScreenSetVehicleTypeFilter | likely | Lua C function (EquipScreen.SetVehicleTypeFilter): if arg1 is a number it parses it and calls the client method FUN_0098fac0(value); returns 0. |
| `00c11390` | EquipScreenGetMaxBundlesPurchasable | likely | Lua C function (EquipScreen.GetMaxBundlesPurchasable): when arg1 is truthy and there are 2 arguments it calls the client lookup FUN_00bdac80 with the second ar… |
| `00c11410` | EquipScreenGetBundlePreviewType | likely | Lua C function (EquipScreen.GetBundlePreviewType): when arg1 is truthy and there are 2 arguments it calls the client mapping FUN_00bdadb0 with the first argume… |
| `00c11500` | EquipScreenGetBundleEquippableState | likely | Lua C function (EquipScreen.GetBundleEquippableState): when arg1 is truthy and there are 2 arguments it calls FUN_00bdad00 with the second argument and pushes … |
| `00c11580` | EquipScreenCanPreviewBundle | likely | Lua C function (EquipScreen.CanPreviewBundle): when arg1 is truthy and there are 2 arguments it calls the client predicate FUN_00bdaf00 with the second argumen… |
| `00c11b10` | EquipScreenUseStyleCard 🌐 | likely | Lua C function (EquipScreen.UseStyleCard): requires arg1 to be numeric (FUN_00e3a5f0); reads it (FUN_00e3a770 + FUN_00d83c90), constructs an InventoryPacketUse… |
| `00c11cb0` | EquipScreenCheckImmediateActivationItem | likely | Lua C function (EquipScreen.CheckImmediateActivationItem): requires arg1 numeric, rounds it to an integer, constructs an InventoryPacketCheckImmediateActivatio… |
| `00c10b20` | EquipScreenSelectJob | speculative | Lua C function (EquipScreen.SelectJob): if arg1 is a number it parses it (FUN_00e3a770 + FUN_00d83c90) and calls the client method FUN_00bda7f0(value); returns… |
| `00c10b60` | EquipScreenEquipLightsaber | speculative | Lua C function (EquipScreen.EquipLightsaber): if arg1 is a number, reads arg1 and arg2 (FUN_00e3a770 + FUN_00d83c90 each) and calls the client method FUN_00bdb… |
| `00c10fd0` | EquipScreenUseStyleCardByItemId | speculative | Lua C function (EquipScreen.UseStyleCardByItemId, bound to 00c10fd0 in the EquipScreen table): if arg1 is a number it parses it (FUN_00e3a770 + FUN_00d83c90) a… |
| `00c11060` | EquipScreenClearPreviewStyleCard | speculative | Lua C function (EquipScreen.ClearPreviewStyleCard): with exactly one argument it parses arg1 and calls FUN_00a10750(value); with no argument it calls FUN_00a10… |
| `00c11160` | EquipScreenEquipVehicleItem | speculative | Lua C function (EquipScreen.EquipVehicleItem): requires arg1 truthy and exactly 1 argument; reads arg1 as a string (FUN_00e3a820) and calls the client method F… |
| `00c111b0` | EquipScreenRemoveVehicleItem | speculative | Lua C function (EquipScreen.RemoveVehicleItem): if arg1 is a number it parses it and calls the client method FUN_00b87b50(value); returns 0. |
| `00c11bc0` | EquipScreenUseImmediateActivationItem | speculative | Lua C function (EquipScreen.UseImmediateActivationItem): requires arg1 numeric, rounds it to an integer (lua_tonumber + ROUND), constructs an InventoryPacketUs… |
| `0098fa40` | — | unknown | Calls helper 00a6daa0 three times around virtual slot +0x44 calls through nested objects at this+0x378 and this+0x380, then calls VehicleListDataSource::SetAct… |
| `009905c0` | — | unknown | Walks the linked list rooted at this+0x350 through next pointer +4, assigning the supplied row id to each equip data source, then assigns it to the currently-e… |
| `00c10ac0` | — | unknown | Lua C function (EquipScreen table entry Populate): if arg1 is a number (FUN_00e3a5f0) it reads it with FUN_00e3a770 + FUN_00d83c90 and calls the client equipme… |
| `00c10d20` | — | unknown | Lua C function (EquipScreen.PreviewEquipItemById): when arg1 is truthy and there is at least one argument it builds a stack ItemRecord (vtable ItemRecord::vfta… |
| `00c10e30` | — | unknown | Lua C function (EquipScreen.GetItemEquippableState): when arg1 is truthy and there is exactly one argument, maps arg1 through FUN_00bdaae0 and pushes the resul… |
| `00c10ea0` | — | unknown | Lua C function (EquipScreen.CanPreviewItem): when arg1 is truthy and exactly one argument is present, calls the client predicate FUN_00bdae10(arg1) and pushes … |

### `ui.inspect`

**InspectDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cbfd20` | InspectDataSource::GetColumnName | confirmed | Maps integer selectors 0 through 0x1d to inspection column-name strings such as player identity, jobs, stats, profile and social fields, with an error fallback. |
| `00cc0140` | InspectDataSource::GetRowCount | confirmed | Reads the count field at inspect-object offset 0x174 and returns at least one, even when the stored count is below two. |
| `00cc0760` | InspectDataSource::GetValue | confirmed | Maps selectors 0 through 0x1d to the inspect target's identity, numeric stats, job values, strings and profile fields; computes age from the birthday string an… |
| `00cc0db0` | InspectDataSource::InspectDataSource | confirmed | Registers BaseClient.Inspect, installs the class's primary and secondary vtables, and stores the supplied inspect-object pointer. |
| `00cc0e90` | InspectDataSource::ScalarDeletingDtor | confirmed | Destroys the embedded GUI user-data holder and frees the object when the low bit of the flag argument is set; otherwise returns the object pointer. |

**InspectItemsDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cbff10` | InspectItemsDataSource::InspectItemsDataSource | confirmed | Registers BaseClient.Inspect.Items, installs the class's primary and secondary vtables, stores two arguments, and calls FUN_008356b0 with zero. |
| `00cc0020` | InspectItemsDataSource::GetColumnName | confirmed | Returns IsInMarketplace for selector 0x1b; for other selectors tail-jumps to FUN_00cb09d0 with the original selector. |
| `00cc0080` | InspectItemsDataSource::DestructorAdjustor | confirmed | Subtracts 0x254 from ECX and tail-jumps to FUN_00cc0170, adjusting the secondary-base this pointer for the shared scalar deleting destructor. |
| `00cc0170` | InspectItemsDataSource::ScalarDeletingDtor | confirmed | Calls FUN_00cb1b10 to clean up inspect-item data, frees the object when the low flag bit is set, and otherwise returns the object pointer. |
| `00cc02d0` | InspectItemsDataSource::GetValue | confirmed | Resolves an item from the inspect source and item-definition tables, then returns requested identifier, strings, flags, prices and definition-derived values fo… |

**InspectProfilesDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cbfeb0` | InspectProfilesDataSource::GetColumnName | confirmed | Maps profile selectors 0 through 4 to ID, rank, name, icon ID and badge-image ID, returning an error string for other values. |
| `00cc0160` | InspectProfilesDataSource::GetRowCount | confirmed | Returns the profile count stored at offset 0x184 in the object referenced at this+0x224. |
| `00cc0ef0` | InspectProfilesDataSource::InspectProfilesDataSource | confirmed | Registers BaseClient.Inspect.Jobs, installs the profile data-source vtables, and stores three supplied data pointers. |
| `00cc0fe0` | InspectProfilesDataSource::ScalarDeletingDtor | confirmed | Destroys the embedded GUI user-data holder and frees the object when the low bit of the flag argument is set; otherwise returns the object pointer. |
| `00cc01d0` | InspectProfilesDataSource::GetValue | likely | Bounds-checks a profile row, returns its ID or rank for columns zero and one, then resolves the profile record and returns its name/icon fields for columns two… |

**(no class)** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008c6030` | StateObj_ShowPlayerInspect | confirmed | Calls FUN_008b8ff0(0x16) and when it returns nonzero calls vslot 9 (+0x24) of the object at this+0xec. When that object's vslot 0 returns 0x16 it calls FUN_00a… |
| `00bb6740` | SetStringMember_0xf8 | likely | Setter for an embedded IString member: takes a SoeUtil::IString by value on the stack (16 bytes at [esp+0x14]) and copy-assigns it into the receiver at this+0x… |
| `00bb67d0` | SetStringMember_0x108 | likely | Setter for an embedded IString member: takes a SoeUtil::IString by value on the stack and copy-assigns it into the receiver at this+0x108 through FUN_006f9e60,… |
| `00bb6860` | SetStringMember_0x158 | likely | Setter for an embedded IString member: takes a SoeUtil::IString by value on the stack and copy-assigns it into the receiver at this+0x158 through FUN_006f9e60,… |
| `00bb68f0` | SetStringMember_0x118 | likely | Setter for an embedded IString member: takes a SoeUtil::IString by value on the stack and copy-assigns it into the receiver at this+0x118 through FUN_006f9e60,… |
| `00bb6980` | SetStringMember_0x128 | likely | Setter for an embedded IString member: takes a SoeUtil::IString by value on the stack and copy-assigns it into the receiver at this+0x128 through FUN_006f9e60,… |
| `00bb6a10` | SetStringMember_0x138 | likely | Setter for an embedded IString member: takes a SoeUtil::IString by value on the stack and copy-assigns it into the receiver at this+0x138 through FUN_006f9e60,… |
| `00bb6aa0` | SetStringMember_0x148 | likely | Setter for an embedded IString member: takes a SoeUtil::IString by value on the stack and copy-assigns it into the receiver at this+0x148 through FUN_006f9e60,… |
| `00cbffd0` | — | unknown | Creates an exception frame, calls FUN_00cb1b10 with the stack-cookie-derived argument, restores the exception list, and returns. |
| `00cc0e40` | — | unknown | Creates an exception frame, invokes FUN_0091bf60 with a stack-cookie-derived value, restores the exception list, and returns. |
| `00cc0f90` | — | unknown | Creates an exception frame, invokes FUN_0091bf60 with a stack-cookie-derived value, restores the exception list, and returns. |

### `world.character`

**CharacterBase** (18)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010444a0` | CharacterBase::EntryTagName | confirmed | No args. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns a pointer to the literal string "entryTag". |
| `010444f0` | CharacterBase::EntryTagName2 | confirmed | No args. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Same body as 010444a0: returns a pointer to the literal "entryTag". |
| `01044500` | CharacterBase::AdjustorThunk_016c | confirmed | Thunk. Bytes 81 c1 6c 01 00 00 e9 95 83 fd ff = add ecx,0x16c; jmp 0101c8a0. Adds 0x16c to this and jumps to 0101c8a0; it does no work of its own. |
| `010449e0` | CharacterBase::DtorBody | confirmed | Fastcall, ECX = this. Sets *this = CharacterBase::vftable, then repeatedly calls 008cace0 on the head at this+0x1d0 while it is non-null (ListRemoveNotify). Ca… |
| `01044b90` | CharacterBase::ScalarDeletingDtor | confirmed | Thiscall with a flag byte param_2 (bytes 56 8b f1 e8 .. f6 44 24 08 01 74 ..): saves this, calls the dtor body 010449e0, then if bit 0 of the flag is set calls… |
| `01044480` | CharacterBase::FieldAddr_05ec | likely | Thiscall, no stack args. Bytes 8d 81 ec 05 00 00 c3 = lea eax,[ecx+0x5ec]; ret. Returns the address this+0x5ec (not the value stored there). |
| `01044490` | CharacterBase::SetFloat_05ec | likely | Thiscall, one stack arg (pointer p). Bytes: mov eax,[esp+4]; fld dword [eax]; fstp dword [ecx+0x5ec]; ret 4. Copies the 4-byte float at *p into this+0x5ec (x87… |
| `010444b0` | CharacterBase::FieldAddr_05f0 | likely | Thiscall, no stack args. Bytes 8d 81 f0 05 00 00 c3 = lea eax,[ecx+0x5f0]; ret. Returns the address this+0x5f0. |
| `010444c0` | CharacterBase::SetFloatFromPtr_05f0 | likely | Thiscall, one stack arg (pointer p). Bytes: mov eax,[esp+4]; fld dword [eax]; fstp dword [ecx+0x5f0]; ret 4. Copies the 4-byte float at *p into this+0x5f0. |
| `010444d0` | CharacterBase::FieldAddr_05f4 | likely | Thiscall, no stack args. Bytes 8d 81 f4 05 00 00 c3 = lea eax,[ecx+0x5f4]; ret. Returns the address this+0x5f4. |
| `010444e0` | CharacterBase::SetFloat_05f4 | likely | Thiscall, one stack arg (pointer p). Bytes: mov eax,[esp+4]; fld dword [eax]; fstp dword [ecx+0x5f4]; ret 4. Copies the 4-byte float at *p into this+0x5f4. |
| `01044510` | CharacterBase::FieldAddr_016c | likely | Thiscall, no stack args. Bytes 8d 81 6c 01 00 00 c3 = lea eax,[ecx+0x16c]; ret. Returns the address this+0x16c. |
| `01044520` | CharacterBase::SetDwordPair_0010_0014 | likely | Thiscall, two stack args (ret 8). Bytes: mov eax,[esp+4]; mov edx,[esp+8]; mov [ecx+0x10],eax; mov [ecx+0x14],edx; ret 8. Stores the two dword arguments into t… |
| `01044540` | CharacterBase::GetDword_0114 | likely | Thiscall, no args. Bytes 8b 81 14 01 00 00 c3 = mov eax,[ecx+0x114]; ret. Returns the dword at this+0x114. |
| `01044710` | CharacterBase::FormatCounterPlus1Into34 | likely | Thiscall with one stack arg. Calls the virtual at this vtable+0xcc (slot 51) with param_2; if it returns non-null object p: calls 0072d5d0 on p+0x38, reads the… |
| `01044760` | CharacterBase::FormatCounterMinus1Into34 | likely | Thiscall with one stack arg. Same as 01044710 except the formatted value is (int read through 0072d5d0 on p+0x38) minus 1. Calls the virtual at this vtable+0xc… |
| `01044550` | CharacterBase::ReadDword_StackArg_0090 | speculative | One stack arg, ret 4. Bytes 8b 44 24 04 8b 80 90 00 00 00 c2 04 00 = mov eax,[esp+4]; mov eax,[eax+0x90]; ret 4. Returns the dword at arg+0x90 and does not rea… |
| `01044650` | — | unknown | Thiscall with seven stack params (Ghidra). Calls 00703540, then formats param_3 with the format string at 01770978 (bytes 25 64 00 = "%d") through 00402fd0 int… |

**ControllerPreviewPet** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b035c0` | ControllerPreviewPet::ControllerPreviewPet | confirmed | Constructor: runs the base constructor FUN_00950530(param_2,param_3,param_4), stores the globals DAT_0181be70, DAT_0179edb8 and DAT_0181cec0 at +0x168, +0x164 … |
| `00b036b0` | ControllerPreviewPet::ScalarDeletingDtor | confirmed | Scalar deleting destructor held in vtable slot 7, the slot this controller hierarchy uses for it: writes ControllerPreviewPet::vftable, calls the base destruct… |
| `00b03550` | ControllerPreviewPet::PreserveVec3AndCallBase | likely | Override of the shared ControllerRoot vtable slot 53 for the pet-preview controller: reads the three dwords at this+0x164, +0x168 and +0x16c into locals, calls… |

**ControllerRegular** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b00ac0` | — | unknown | Constructor. Runs the base constructor FUN_0094f6c0(param_2,param_3,param_4), installs ControllerRegular::vftable, stores param_5 at +0xe0 and the global DAT_0… |
| `00b01250` | — | unknown | Per-frame controller update (vtable slot 51). Fetches the local client object with FUN_0094cf60 and reads its transform and facing through virtual slots +0x1c … |
| `00b02880` | — | unknown | Input dispatcher (vtable slot 50). Walks the input event queue at param_1[0x32] (node link at +4) and switches on the event kind at node+0x18 and the key/actio… |

**NewHatHairAttachmentGroup** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01051490` | — | unknown | Thiscall ctor with 8 stack args. Calls 00731e30 with args 2, 3, 7, 8; stores NewHatHairAttachmentGroup::vftable at *this; zeroes the bytes at this+0x9c, this+0… |

**PlayerSwitchProfileAttachmentGroup** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00964ba0` | PlayerSwitchProfileAttachmentGroup::Constructor | confirmed | Adjusts nonzero param_4 by +0x54, calls FUN_00bf6d60 with the supplied constructor values, installs the PlayerSwitchProfileAttachmentGroup vtable, and stores t… |
| `00964c50` | PlayerSwitchProfileAttachmentGroup::GetAdjustedField174 | confirmed | Returns zero when this+0x174 is zero; otherwise returns this+0x174 minus 0x54. |
| `00964c70` | PlayerSwitchProfileAttachmentGroup::UpdateBaseAndNotify | confirmed | Calls FUN_0072fb60 with its argument, calls FUN_01051150, and when that returns nonzero calls FUN_00bfb620. |
| `00964ca0` | PlayerSwitchProfileAttachmentGroup::NotifyWhenReady | confirmed | Calls FUN_01051150 and, only when it returns nonzero, calls FUN_00bfb620. |
| `00964cc0` | PlayerSwitchProfileAttachmentGroup::FilterField174 | confirmed | Reads this+0x174; returns the value only when it is nonzero and not 0x54, otherwise returns zero. |
| `00964d50` | PlayerSwitchProfileAttachmentGroup::Destructor | confirmed | Restores the PlayerSwitchProfileAttachmentGroup vtable and calls FUN_00964b00 to destroy its attachment-group base. |
| `00964da0` | PlayerSwitchProfileAttachmentGroup::ScalarDeletingDestructor | confirmed | Restores the class vtable, calls FUN_00964b00 for base cleanup, and frees param_1 only when flag bit 0 is set. |
| `00964ce0` | PlayerSwitchProfileAttachmentGroup::CanApplyProfileChange | likely | Returns zero if FUN_01051150 is false; returns one when its stored node field is zero; otherwise checks a context field and hash-table lookups, then returns th… |

**ProxiedCharacter** (31)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0096fa70` | ProxiedCharacter::Ctor | confirmed | Calls ProxiedObject construction, installs ProxiedCharacter vtables, initializes identifiers, flags, transforms, targets, strings, collections, event lists, ma… |
| `00970770` | ProxiedCharacter::GetField200 | confirmed | Returns the address this+200. |
| `00970780` | ProxiedCharacter::SetField128 | confirmed | Stores param_2 at this+0x128 and returns. |
| `00970790` | ProxiedCharacter::Field200AdjustorThunk | confirmed | Subtracts 0x54 from ECX and tail-jumps to 00970770, which returns the adjusted object's address plus 200. |
| `009707a0` | ProxiedCharacter::ScalarDeletingDtorAdjustorThunk50 | confirmed | Subtracts 0x50 from ECX and tail-jumps to 009731e0, the scalar deleting destructor implementation. |
| `009707b0` | ProxiedCharacter::ScalarDeletingDtorAdjustorThunk54 | confirmed | Subtracts 0x54 from ECX and tail-jumps to 009731e0, the scalar deleting destructor implementation. |
| `009707c0` | ProxiedCharacter::IsFlagBit7ClearAtEC | confirmed | Reads the byte at this+0xec, inverts it, converts to unsigned, shifts right seven bits, and returns the resulting byte. |
| `009707d0` | ProxiedCharacter::GetFlagBit0At144 | confirmed | Reads the byte at this+0x144 and returns its low bit. |
| `009707e0` | ProxiedCharacter::AdjustorThunkTo00961550 | confirmed | Subtracts 0x54 from ECX and tail-jumps to 00961550. |
| `009707f0` | ProxiedCharacter::SetFlagBit0At144 | confirmed | Sets bit 0 at this+0x144 when param_2 is nonzero; otherwise clears that bit while preserving the other seven bits. |
| `00970810` | ProxiedCharacter::CreateStringFromField56C | confirmed | Initializes an output IString object with the shared empty string, then passes the field at this+0x56c to a string helper and returns the output object. |
| `00970880` | ProxiedCharacter::CreateStringFromField58C | confirmed | Initializes an output IString object with the shared empty string, then passes the field at this+0x58c to a string helper and returns the output object. |
| `00972570` | ProxiedCharacter::Dtor | confirmed | Restores ProxiedCharacter vtables, destroys subordinate state, calls RemoveRider, and frees the allocated mount-state block. |
| `009731e0` | ProxiedCharacter::ScalarDeletingDtor | confirmed | Thiscall with one flag argument (ret 4). Calls the ProxiedCharacter destructor body 00972570 on this, then calls _free (thunk 00d82463) on this when bit 0 of t… |
| `00973200` | ProxiedCharacter::Process | confirmed | Thiscall (this = param_1) with an int param_2 scaled to float by DAT_017ebf0c. Calls 0072cfe0 and then virtual slot [9] on this. Looks up the actor for the ter… |
| `009580f0` | ProxiedCharacter::LogAttachmentGroupError | likely | Calls 00704dd0 with the exact log name #AttachmentGroupErrors.log\|AttachmentGroupErrors.log, a global configuration pointer, and the supplied value. |
| `0096bd90` | ProxiedCharacter::SwitchToTemporaryAppearance | likely | Thiscall(param_1=P, param_2=request record). Returns when record+0x18 equals P[0x3cb]. Otherwise, when P[0x1ce]+0x959 is zero and the vtable call on P returns … |
| `009580d0` | ProxiedCharacter::DestructorAdjustor | speculative | Adds 0x154 to ECX and jumps to 00bf1ee0; decoded bytes show this is an adjustor thunk, not the destructor body. |
| `00958490` | ProxiedCharacter::Virtual29 | speculative | Calls vtable slot 0x70 with param_2 and destination this+0x610. |
| `00958630` | ProxiedCharacter::MarkFlagAt100 | speculative | Stores byte one at offset 0x100 of the object referenced by this+8. |
| `00959ab0` | ProxiedCharacter::GetPair24 | speculative | Copies two dwords from this+0x24 and this+0x28 to the output pair. |
| `0095a120` | ProxiedCharacter::HasActiveEffect | speculative | Walks an effect list and returns false if any active node's field +0x20 satisfies 0072a1c0; returns true when none do. |
| `0095a170` | ProxiedCharacter::SetEffectsEnabled | speculative | Walks active effect nodes and calls 0072ae80 on each node's field +0x20 with the inverse of param_2 as its second argument. |
| `0095a3c0` | ProxiedCharacter::SetContext504 | speculative | Calls the object at this+0x734 virtual slot 4 with this+0x504 as its argument. |
| `00961550` | ProxiedCharacter::DispatchCurrentAction | speculative | Reads this+0x1884; if it is zero and the relevant flag/value condition holds, calls vtable slot +0x24, otherwise dispatches the selected value through the obje… |
| `00966090` | ProxiedCharacter::VirtualSlot11NotifyChildren | speculative | Secondary-table virtual of ProxiedCharacter (adjustor -0x54 to the primary object). Calls 006f9e60(param_2); if the primary's vtable +0x24 predicate is non-zer… |
| `0096aaf0` | ProxiedCharacter::SetRecordValue_v30 | speculative | Virtual slot 30 of ProxiedCharacter::vftable. For index 0..17 it finds or creates the record as in 0096aa00, releases record+0x1d0 through 00969910 when set, a… |
| `0095a250` | — | unknown | Queries a name/string helper, resolves an object using this+0xb4, invokes a virtual callback with a resolved field, then conditionally calls a second interface… |
| `00960480` | — | unknown | Calls virtual slot 7, processes param_2 through two helpers, copies four words to this+0xf44, clears a field at +0x3f0, then empties a linked list. |
| `00960520` | — | unknown | For non-owner characters, forwards two vectors to virtual setters and normalizes direction through FUN_0095dae0; for the owner, copies position and direction i… |
| `00965c50` | — | unknown | Virtual slot 4 of ProxiedCharacter. Returns after clearing this+0x146 bit 0x40 when the sub-object at this+0x738 is null or its +0xc8 back-pointer is not this.… |

**ProxiedCharacterAttachmentGroup** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00964b00` | ProxiedCharacterAttachmentGroup::Destructor | confirmed | Restores the ProxiedCharacterAttachmentGroup vtable, installs the integer-list vtable for an embedded member, calls FUN_007798c0, then destroys the NewHatHairA… |
| `00964b70` | ProxiedCharacterAttachmentGroup::GetField174 | confirmed | Returns the dword at this+0x174. |
| `00964b80` | ProxiedCharacterAttachmentGroup::ScalarDeletingDestructor | confirmed | Calls FUN_00964b00 for class cleanup and frees param_1 only when flag bit 0 is set. |

**ProxiedCharacterAttachmentGroupManager** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bf0f70` | ProxiedCharacterAttachmentGroupManager::IsEmpty | confirmed | Returns true when the 32-bit field at object offset 0x118 is zero. |
| `00bf1320` | ProxiedCharacterAttachmentGroupManager::UpdateVisibility | confirmed | Updates a global visibility state from manager offset 0x104, then walks linked groups and disables the associated value for groups whose flag at offset 0x168 i… |
| `00bf1420` | ProxiedCharacterAttachmentGroupManager::Ctor | confirmed | Constructs a base object, installs the manager vtable, clears manager state, stores a parameter, and initializes the embedded attachment-group list and count. |
| `00bf14c0` | ProxiedCharacterAttachmentGroupManager::Dtor | confirmed | Restores the manager vtable, restores and clears the embedded list, then calls the base cleanup routine. |
| `00bf1530` | ProxiedCharacterAttachmentGroupManager::ProcessGroup | confirmed | Looks up attachment groups by GUID, invokes callbacks for active matching entries, resolves dual-wield slot definitions and logs missing definitions, then mark… |
| `00bf11e0` | ProxiedCharacterAttachmentGroupManager::RemoveGroup | likely | Rejects an ineligible group, optionally removes matching global attachments, optionally calls a manager reset slot, unlinks the matching node from the manager … |
| `00bf1290` | ProxiedCharacterAttachmentGroupManager::RemoveGroupsForId | likely | Finds the hash bucket for a group ID, skips entries with other group IDs, then removes matching entries while their secondary ID at offset 0xb4 equals the requ… |
| `00bf1820` | — | unknown | Thiscall, one stack arg (group), ret 4. Reads dword group+0x17c, looks it up in the 10-bucket table at GlobalGet_EC()+0x10 (key % 10) and returns when no node … |

**ProxiedCharacterAttachmentProcessor** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bf1c00` | ProxiedCharacterAttachmentProcessor::Ctor | confirmed | Fastcall (ECX=this): stores ProxiedCharacterAttachmentProcessor::vftable at +0 and zeroes the dword at +4. Nothing else. |
| `00bf1c60` | ProxiedCharacterAttachmentProcessor::DtorBody | confirmed | Fastcall (ECX=this): restores ProxiedCharacterAttachmentProcessor::vftable at +0 and returns. Frees nothing. |
| `00bf1c90` | ProxiedCharacterAttachmentProcessor::ScalarDeletingDtor | confirmed | Fastcall with one stack flag (ret 4): stores ProxiedCharacterAttachmentProcessor::vftable, then calls _free on this when flag bit 0 is set; returns this in EAX… |
| `00bf1c10` | ProxiedCharacterAttachmentProcessor::GetStaticDataAddress | speculative | No-argument virtual (mov eax, imm32; ret): returns the address of the static data pointer PTR_vftable_01b66f38. Binary vtable slot [3] of the processor. |

**SoeUtil::List<ProxiedCharacter::Destination,8>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009676d0` | DestinationList8DtorBody | confirmed | Fastcall (ECX = list object). Sets the List<Destination,8> vtable, calls 0095e530 (ListClear, speculative label) to empty the list, zeroes the dwords at +0x18 … |
| `00968250` | ListDestination8ScalarDeletingDtor | confirmed | Thiscall (ECX = object). Calls the list dtor body 009676d0 on the object, then when the stack flag byte has bit 0 set it frees the object with _free (through t… |

**(no class)** (53)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00963840` | QueuePositionAndDirectionUpdate | confirmed | When its state mask and movement-type guard pass, stores the four-word position at +0xf50, optionally adjusts it through FUN_010ba6e0, normalizes a nonzero dir… |
| `009710b0` | RemoveRider | confirmed | Looks up the stored rider identity, logs the RemoveRider record, updates related actor state through virtual/action calls, and frees the mount-state allocation. |
| `00972d30` | Mount | confirmed | Logs rider/mount names and IDs, returns early if mount state already exists, validates actor contexts and seat bounds, allocates and initializes mount state, i… |
| `0089f3f0` | TestBit0x80_6f8 | likely | Single-bit query: returns bit 7 of the dword at +0x6f8 of the object (shift right by 7, mask 1). No store and no call. |
| `0089f4c0` | TestBit0x8000_6f8 | likely | Single-bit query: returns bit 15 of the dword at +0x6f8 of the object (shift right by 0xf, mask 1). No store and no call. |
| `0089f4d0` | TestBit0x01_6fa | likely | Single-bit query: returns bit 0 of the byte at +0x6fa of the object (byte load, mask 1). No store and no call. |
| `00961630` | IsProfileContextCurrent | likely | Returns false when bit 7 at this+0x144 is set; otherwise compares two stored words with a related object's pair and may accept either of two further context ch… |
| `00962f30` | SetCharacterStateFlags | likely | Sets or clears bits 5 and 4 at byte +0x145, stores param_3 at this[0x1c2] unless bit 5 is clear (then stores -1), and when the selected object exists calls FUN… |
| `00962fb0` | ClearOwnedCharacterBuffers | likely | When +0x504 is nonzero, calls FUN_00960060 and releases it through the object at +0x734; then releases nonzero pointers at +0x514 and +0x510 through the same v… |
| `009630e0` | ProcessQueuedTriples | likely | Walks count at +0x7f4 over records of three dwords at +0x7f0 with 0xc-byte stride, passes each triple to FUN_00960170, then clears the count. |
| `00963790` | SetCharacterFlagAndNotifyModel | likely | When the requested value differs from bit 7 at +0x147, updates that bit and, if the virtual model lookup returns a value, calls FUN_0076c530 with the request. |
| `00963ef0` | ReleaseHandleAndSetModelFlag | likely | Invokes the object at +0x3fc with argument 1 when nonzero, clears +0x3fc, and calls FUN_00963040 with 1. |
| `00963fa0` | SetFlagAndRefreshRelatedObject | likely | When bit 4 at +0x147 differs from param_2, updates it, resolves either the object at this[0x142] or the primary virtual object, and calls FUN_00705200 if a tar… |
| `00966460` | ChooseColorStringAndApply | likely | Fastcall (ECX = this). If this[0x3ce] is set it calls the virtual +0x34 with that value and returns. Otherwise it picks an ARGB colour string: when the vtable … |
| `009667f0` | ReplaceMovementControllerWithRail | likely | Thiscall with three stack arguments. If this+0x3fc is non-null it calls the held object's vtable slot 0 with 1 (scalar deleting destructor shape), then zeroes … |
| `009668a0` | ReplaceMovementControllerWithRelativeRail | likely | Thiscall with six stack arguments. If this+0x3fc is non-null it calls the held object's vtable slot 0 with 1, then zeroes this+0x3fc, calls 00963040(1), alloca… |
| `00966970` | ReplaceMovementControllerWithSeek | likely | Fastcall (ECX = this). If this+0x3fc is non-null it calls the held object's vtable slot 0, then zeroes this+0x3fc, calls 00963040(), allocates 0x30 bytes with … |
| `0096f800` | ProcessCharacterAttachmentData | likely | Looks up a string through field +0x61e, initializes temporary CharacterAttachmentData and string lists, conditionally calls a virtual operation and 009653c0, s… |
| `00971250` | UpdateSeatAttachment | likely | Checks the indexed seat against the target model seat count, reads seat/model data, composes transforms, updates attachment/controller state, and cleans tempor… |
| `00b006d0` | CapsuleSweepMove3Iters | likely | Capsule sweep move step. Copies the start position (param_2), the 4-float up axis (param_3) and the displacement (param_4) into locals, then loops up to 3 time… |
| `00b01040` | MoveInSubStepsAndRecord | likely | Takes a start position (param_4), an up axis (param_5), a total displacement (*param_6) and a sub-step count param_3, divides the displacement by the count and… |
| `00960060` | MoveDestinationNodesToArray | speculative | Walks nodes from this+0x74c, appends each node's three selected words to a 0xc-byte array, removes each node through FUN_0095e740, and advances using its saved… |
| `00963130` | ClearCharacterStateAndReleaseObjects | speculative | Sets bit 7 in this[0x1be], clears a list, sets bits at +0x143/+0x146, releases an optional object at [0xff], makes two context-dependent calls through the rela… |
| `00963d80` | SetByteFlag0x140Bit0 | speculative | Sets bit 0 at this+0x140 when param_2 is nonzero; otherwise clears that bit. |
| `00963da0` | SetAttachmentStateFlags | speculative | Sets bit 6 at byte +0x145 when param_2 is nonzero; otherwise clears that bit and sets bit 0 at +0x140. |
| `00963f20` | SetCharacterBitAndPropagateByte | speculative | When bit 6 at byte +0x147 differs from param_2, updates that bit, obtains a related object through the virtual method, and writes param_2 to its byte at +0x226… |
| `009659e0` | LookupNodeByKeyAndTryCreate | speculative | Thiscall with a record argument param_3 and two further stack arguments. Returns 0 unless the vtable +4 predicate on param_3 succeeds. It then walks the 1024-b… |
| `00966790` | RemoveXorKeyedNode | speculative | Thiscall with two dword keys. Computes k = param_3 ^ param_2, selects the chain at this+0xf7c + (k & 3)*4, and walks it along each node's +0x14 link until a no… |
| `00968330` | HashList4InsertNode | speculative | Thiscall with ECX = map object and one stack argument. Calls the vtable +4 predicate on the map; when it returns zero the record stays null and the following s… |
| `009683e0` | HashList4EraseNode | speculative | Thiscall with ECX = map object and one stack argument, a record. Selects the bucket from record+0x1d0 & 3 (map+0x14 + bucket*4 base), walks that bucket chain a… |
| `009684b0` | ListUnlinkNodeAndRelease | speculative | Thiscall with ECX = list object (+4 head, +8 tail, +0xc count) and param_2 = a node (+0xc0 next, +0xc4 previous). Unlinks the node by patching its neighbours o… |
| `00968930` | InitNodeFieldsAndStoreKey1d0 | speculative | Thiscall (ECX = node) with five stack arguments. Calls 00964a40 with the values at *param_3, *param_4, *param_5 and *param_6, stores *param_2 at node+0x1d0, an… |
| `00968ab0` | HashList4UnlinkAndDestroy | speculative | Thiscall with ECX = map object (+8 head, +0xc tail, +0x10 count, four buckets at +0x14) and param_2 = a node. Finds the bucket from node+0x1dc & 3 and walks it… |
| `00968b80` | ListUnlinkNode30AndDestroy | speculative | Thiscall with ECX = list object (+4 head, +8 tail, +0xc count) and param_2 = a node (+0x30 next, +0x34 previous). Unlinks the node by patching its neighbours o… |
| `00969910` | ListRemoveKeyDecrementSlots | speculative | Thiscall with ECX = owner (its list head at +0x844) and param_2 = a key. Walks the list along each node's +0xc0 link; for every node whose dword at +0 equals t… |
| `0096c300` | SwitchTemporaryAppearanceFromRecord | speculative | Fastcall(ECX=this). When this[0x3ca] is positive and the vtable+0x24 object has bit 1 set in byte +0x10, it constructs a PlayerUpdatePacketUpdateTemporaryAppea… |
| `00bf1ee0` | ResolveAttachmentsForKey | speculative | Thiscall with four stack args (ret 0x10). Builds a 68-byte vector through 00776970, looks the key (param_3) up in the 10-bucket table at GlobalGet_EC()+0x10 an… |
| `00bf7b00` | ProxiedCharacterInitFields | speculative | Initializes a ProxiedCharacter-sized object: zeroes most fields, stores the constants DAT_018277ec, DAT_01b676bc, DAT_017ed82c, _DAT_0175b420, DAT_01809b68, _D… |
| `01051150` | PredicateFlag67AndFlag9c | speculative | Fastcall (this in ECX). Returns 1 when the byte at this+0x67 and the byte at this+0x9c are both nonzero, else 0. |
| `01051170` | TailCallIfFlag9cElse1 | speculative | Fastcall (this in ECX). If the byte at this+0x9c is zero, returns 1. Otherwise jumps (tail jump e9, not a call) to 0072f560 and returns its result. |
| `01051ca0` | MarkEntryReadyAndTryResolve | speculative | Thiscall (this in ECX; param_2 is a key). Calls 0072f920(key); when the result is nonzero sets its byte at +0x18d to 1. Walks the list at this+0x18 (next point… |
| `01051d00` | TryResolveIfAllEntriesReady | speculative | Fastcall (this in ECX). Walks the list at this+0x18 (next pointer at +0x190); returns when any entry has its +0x18d byte clear. Otherwise, when this+0x9c is ze… |
| `0095e890` | — | unknown | Calls the queue object?s virtual slot +8 with param_1 in ECX and no explicit stack argument to obtain a node. If nonnull, copies the supplied position, directi… |
| `00965f90` | — | unknown | Thiscall with three value arguments and a flag byte (param_4). Clears the held object at this+0x518: calls 0072a0d0 with ECX = this+0x58 and the held object as… |
| `009661d0` | — | unknown | Thiscall with one stack argument param_2 (a record whose C-string is at +4). Calls 006f9e60 with ECX = this+0x5e0 and param_2 as the source. It then walks the … |
| `00966ce0` | — | unknown | Fastcall (ECX = list object). Installs the SoeUtil::List<ProxiedCharacter::Destination,-1> vtable, zeroes the head, tail and count dwords at +4/+8/+0xc, then i… |
| `00968550` | — | unknown | Fastcall with ECX = list object (+4 head, +8 tail, +0xc count). Calls the vtable +8 predicate on the list; when it returns zero the node stays null and the fol… |
| `00968c80` | — | unknown | Thiscall (ECX = owner actor) with 14 stack arguments. Computes a heading with fpatan(param_7[0], param_7[2]) and walks the record list headed at param_13+4: fo… |
| `00973050` | — | unknown | Thiscall, no arguments. Calls virtual slot +0x10 on the object at this+0x54, then logs 'Dismount, rider ...' to ClientMountLog.txt via 00704db0. If the pointer… |
| `00bf0f80` | — | unknown | Walks a global group list, hashes each eligible group ID into one of 30 buckets, and repeatedly invokes the manager's virtual removal slot for matching IDs. |
| `01051660` | — | unknown | Thiscall with 11 stack args (ECX = this). Copies the 11th argument into a local, calls 0070bd80 with the 2nd argument, calls 01051580(local record, this+0xa0) … |
| `01051770` | — | unknown | Fastcall (this in ECX). Takes the actor from this+4 and, while this+0x64 is zero, adds each entry of the owner list at +0x60 (next at +0x16c) whose 0072f530 lo… |
| `01051ab0` | — | unknown | Fastcall (this in ECX). Sets the byte at this+0x9c to 1. If the dword at this+0xb0 is above 0: initialises an IStringFixed<char,128> at this+0xa8 via 0070bde0,… |

## Source shards

[[S0143 00701a20-00707400|S0143]] (4), [[S0145 0070ca40-00711ff0|S0145]] (26), [[S0146 00712020-00714060|S0146]] (12), [[S0147 007140c0-00715ea0|S0147]] (1), [[S0149 00719de0-0071efe0|S0149]] (13), [[S0150 0071f030-00725ff0|S0150]] (65), [[S0151|S0151]] (3), [[S0152|S0152]] (5), [[S0167|S0167]] (2), [[S0171|S0171]] (1), [[S0173|S0173]] (1), [[S0184|S0184]] (1), [[S0219 0089d990-008a3c80|S0219]] (3), [[S0222 008b1e00-008b7730|S0222]] (3), [[S0225 008c52a0-008ca980|S0225]] (2), [[S0254 00954ec0-0095a710|S0254]] (13), [[S0255 0095a840-00960520|S0255]] (7), [[S0256 009606e0-00965340|S0256]] (27), [[S0257|S0257]] (24), [[S0258 00969a10-0096de80|S0258]] (3), [[S0259 0096e490-00972d30|S0259]] (17), [[S0260|S0260]] (3), [[S0262 0097c960-00982e70|S0262]] (63), [[S0263 00982f10-009897a0|S0263]] (3), [[S0265 0098ec00-00995660|S0265]] (3), [[S0287 00a0c3a0-00a13140|S0287]] (1), [[S0305 00a707f0-00a75b20|S0305]] (5), [[S0332 00b006d0-00b05260|S0332]] (8), [[S0344 00b44fa0-00b4b360|S0344]] (2), [[S0366 00bb11b0-00bb7710|S0366]] (7), [[S0372 00bd37e0-00bd5d40|S0372]] (3), [[S0373 00bd5e90-00bd9850|S0373]] (7), [[S0374 00bd9880-00be0090|S0374]] (6), [[S0378 00becaa0-00bf1530|S0378]] (8), [[S0379|S0379]] (6), [[S0380 00bf5ff0-00bf9410|S0380]] (1), [[S0381 00bf9430-00bfcf30|S0381]] (6), [[S0382 00bfcfe0-00c056a0|S0382]] (4), [[S0384 00c0dcd0-00c171d0|S0384]] (33), [[S0414 00cbc8e0-00cc18f0|S0414]] (18), [[S0566 0102b8a0-01030cb0|S0566]] (2), [[S0569|S0569]] (154), [[S0570|S0570]] (220), [[S0571 01044480-0104a2b0|S0571]] (18), [[S0573|S0573]] (13), [[S0722|S0722]] (2)
