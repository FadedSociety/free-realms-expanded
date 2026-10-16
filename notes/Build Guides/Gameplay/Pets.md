---
type: build-guide
section: Gameplay
updated: 2026-10-12
---

# Pets

Part of [[Build Guides]] › Gameplay. Related: [[Mounts]].

<!-- NARRATIVE START -->
## How it works

**How to read this.** Confidence words (`confirmed`, `likely`, `speculative`) are the native-map labels for a function. Source tags: *client* = read from the shipped client in this pass or in a shard (function pseudocode and bytes, the Lua module table decoded from the binary, the shipped `Resources/Pet*.txt` files, the asset manifest, or client locale text); *vault* = an earlier research note (091, 011, 043), several of which lean on community material, so they are used only where the client agrees or where the line says otherwise; *video* = a reviewed frame record; *wiki* = the fan wiki. Precedence is client, then video, then wiki. *Reconstructed* (R) is our own server policy, not original behaviour. OSFR-held data (packet captures and server JSON) is not used. There is no original pet traffic, so every statement about what the server sent or decided is a hypothesis built from what the client requires. No player names, chat or player-chosen pet names are recorded here. Related: [[Mounts]], [[Housing]], [[Items and Inventory]], [[Coin Store and Purchases]], [[Combat and Abilities]], [[Quests and Objectives]], [[Fishing]], [[Trade]].

### Overview

Pets are companion creatures a player owns, summons beside their character and looks after. They are a native client feature (a packet family, a handler object and Lua-driven panels) with one native mini-game, the trick-drawing "gesture" round. The Pet Trainer job is the progression track for tricks.

- **What a pet is.** Pets spawn and despawn on command, follow the owner, need no cooldown, are not inventory items, cannot be sold or traded, may be owned in duplicate, and only one is in play at a time (wiki). The client has the same one-active-pet shape: the handler caches one active pet id and one active actor guid (client, `PetsHandler` +0x84 and +0x88/+0x8c), and the Lua call `Pet.GetActivePetId` returns 0 when nothing is out (vault 091, from the client Lua base script).
- **Needs and mood.** Each pet has a food, a groom (cleanliness) and a play (entertainment) bar plus a mood value (client data-source columns `FoodNeed`, `PlayNeed`, `GroomNeed`; wiki: hunger, cleanliness and entertainment bars). All bars drop slowly while the pet is idle and quickly while it is out (wiki). A full set of bars means a happy pet; a happy pet speeds the owner up (client locale "Speed boost given when your pet is happy", ranks 1 to 5 of "Pet Happiness Speed Boost"; wiki).
- **Care.** The owner feeds (food, vitamins, a bowl), grooms (shampoo, conditioner, a brush) and plays (a toy, tricks) with the pet. Shampoo, conditioner, food and vitamins are used up; the bowl and the brush are kept (client locale). Grooming is a scrubbing action in which the pet "will only hold still for a minute" (client locale).
- **Tricks.** Each pet has a trick list. Performing a trick opens the gesture round, where the player traces a shape on screen and the client grades it (client). Tricks level up the pet only while the player is in the Pet Trainer job, and the job level controls which tricks unlock (client locale; see Pet Trainer).
- **Clothes and kit.** Pets wear head, collar, chest and feet wearables and hold a toy (client data sources `BaseClient.PetInventory.HeadData`, `CollarData`, `ChestData`, `FeetData`, `ToyData`); cats and dogs can wear clothes and costumes (wiki). Eight utility slots hold the care kit (client, the record layout below).
- **Names.** Names are chosen from two premade lists (a first part and a second part) rather than typed (client resource `PetNames.txt`; wiki). Vanity pets can take a custom name and a "Pet Name Change" can be bought (client locale; Open questions).
- **Benefits beyond speed.** A summoned pet finds and digs up treasure chests and shows thought bubbles near battles, dungeons and collection spawns (wiki). The client locale has Common, Rare and Elite "Pet Treasure", treasure-hunting vitamins that raise the find chance, and a "happy face" prompt to encourage the pet when it takes interest in something. The video shows a thought bubble with a chest icon over a pet and a "Common Pet Treasure" prompt (video).
- **Where to get one.** Marketplace in Station Cash, Pet Adoption Centers (try for a short time or buy), the Mysterious Merchant, trading card virtual-reward cards, the Daily Wheel and limited events (wiki; client locale for the adoption centers and "Try Pet" / "Buy Pet").
- **Where they cannot go.** A pet cannot be used in a minigame, dungeon or duel, and returns on exit (wiki). Entering a house dismisses a following pet with "Your pet can't follow you, and has been dismissed." (client locale; video). The client also forces a dismiss when non-default area physics is applied to the local character (client, see Adopting).
- **Not covered here.** Mounts are a separate system ([[Mounts]]). Combat pets are also separate: the client locale has Tier 1 to 5 "Combat Pet" profiles summoned by abilities and a biscuit that "summons a pet companion to help you in battle for 1 minute" ([[Combat and Abilities]]).
- **Rewards.** The only recovered payoffs are the speed boost, treasure finds and Pet Trainer progress. No coin, experience or star values for tricks, care or treasure are recoverable from the client; the tutorials and quests are text only (client locale).

### How the client implements it

#### Main classes

| Class or object | Role | Evidence |
|---|---|---|
| `PetsHandler` (state setup `00b596f0`, teardown `00b59b90`) | The one pet manager, embedded at offset `0x19a60` of the game-state object the client keeps at client +0x3e0 (accessor `008a6660`, adjustor thunk `00a10950`); its first dword points back to the client object. Holds a 4-bucket hash table of pet records at +0x74 (node 0x130 bytes, next at +0x120, key at +0x124, bucket = key and 3), the active pet id at +0x84, the active pet actor guid pair at +0x88/+0x8c (default "none" pair `DAT_01be9810/14`), a cached active pair at +0x12c0/+0x12c4, a tracked target pair at +0x1308/+0x130c with its id at +0x1310, a state word at +0x12ac (values 2, 3 and 4 seen), the pending action id at +0x12b8, the can-go flag at +0x12f9, the trial timer in milliseconds at +0x12fc and the in-water byte at +0x1300. It registers the data sources `BaseClient.PetList` and `BaseClient.PetActive`. | likely labels; field offsets read from pseudocode in this pass |
| `HandlePetPacket` (`00b5ce60`, 9177 bytes) | The only receive switch for the pet family. The client packet dispatcher `009471a0` sends message id `0x35` here through `00a10950`, and only when the local player object (client +0x3e0) exists. 22 subtype cases (see Network). | likely; call chain and cases read this pass |
| Pet packet classes (35 constructors on base `00b50670`) | Each constructor writes `0x35` at +4 and its subtype at +8, then stores its own vtable. Slot 0 of each vtable is its scalar deleting destructor (RTTI-bound). Senders build the class, serialise it with a small body writer (`00b5a240` to `00b5a8d0`, `00b5b0c0`, `00c06e00`, `00c06ea0`, `00c06fc0`) and wrap it in a `PacketTunneledClientPacket` through a wrapper (`00b5bc20` to `00b5cd50`, `00c07090`, `00c071a0`, `00c072b0`), only when the client transport at client +0x280 exists. | confirmed constructors (S0346, S0349, S0383) |
| `ControllerPet` (ctor `00afd6f0`, base `ControllerTactileTarget` `00afe470`) | The player-controller state used while interacting with the pet (state selector `008b9c40`, state 0x0c). Shares slot 50 with `ControllerDraw`. Few members are labelled; a byte at +0x3b4 has a getter and setter. | confirmed ctor, speculative members (S0331) |
| `PetGesture` (ctor `00cc9020`) | The trick-drawing round (see Tricks and the gesture round). | confirmed |
| `PetTactile` (ctor `00cce850`) | A 20-bucket map of grooming spots (`HashListMap<int,PetGroomSpot>`) with two effect handles per spot; `00cce960` rebuilds the spots and `00cce490` releases the effects. | confirmed ctor, likely members |
| `PetTileTargetHandler` (ctor `00b54780`, `OnMessage` `00b60770`) | Click handler on the target processor. Message kind 1 goes to toy-target completion `00b5fbc0`; kind 0x1000 goes to the follow-location sender `00b5fcf0`. | confirmed ctor, likely dispatch |
| Pet data sources | `BaseClient.PetList`, `PetActive`, `PetAction`, `PetTrick`, `PetTrickFavorite`, `PetNameList`, `PetGesture`, `PetInventory` with `HeadData`, `CollarData`, `ChestData`, `FeetData`, `ToyData`, `ShampooData`, `ConditionerData`, `BrushData`, `FoodBowlData`, `FoodData`, `VitaminData`, `EquippedUtilityData`, `EquippedWearablesData`, and `PetTricks.TricksData` and `TrickEquippedData`. Column lists are under Data and formats. | registered by `00b596f0`, `00b5b470`, `00ccd800` to `00ccdac0`; confirmed |
| Lua module `Pet` (registrar `00c06ac0`, table `01829f60`, 24 entries) | The script surface the panels call. Full list under Network. | table decoded from the binary in this pass |
| Pet record container on `CharacterBase` (`01044560` CreatePetRecord, list at +0x1e0) | Allocates a 0x1f8-byte record, gives it the next per-owner pet id from +0x164 (logging "Pets.log" if the counter is not above every id in use) and links it into a 16-bucket list. | confirmed label (S0571) |
| UI movies | `petController.gfx`, `petpanels.gfx`, `PetNameBar.gfx`, `aboutpet.gfx`, `tutorial_petgestures.gfx`, `levelup_pettrainer.gfx` are in the asset manifest; the panels are Lua-driven. | client manifest |

#### Adopting, summoning and following

1. **Roster.** A `PetList` message (subtype 5) carries every owned pet. For each entry the client allocates a 0x130-byte record, reads the body (layout under Data and formats), copies it into the handler table, then reads a trailing active pet id and actor guid pair and calls `PetIsActive` (`00b58bd0`) if they differ from the cached pair, then refreshes the panels (`00b53450`). The roster therefore has to arrive before any other pet message makes sense (client).
2. **Choosing a pet.** The "Pet Browser" (opened from the game dock or the collar icon) lists pets; clicking one calls `Pet.Summon(id)` (`00c066a0`, requires the local player object). That reaches `SendPetSummon` (`00b60080`), which first runs the guard `00b549f0`, then sends subtype 4 with the pet id and action 1 (client; client locale for the browser text "Select a Pet below to summon it"). Members-only pets are gated by the panel script before the call (vault 091).
3. **Client-side summon gate.** `00b549f0` fires the Lua event `PetsHandler:OnSummonFailed` with a locale message and aborts when a predicate `00bf7c20` is true, or when the owner character's flag bits `0x60` (at +0x142) are set and its height is more than a threshold above a recorded height (airborne). The matching text is probably the locale line "You can't summon a pet when jumping." (likely; the link between that string and this branch is not traced). The threshold constant is not resolved.
4. **Spawn and activate.** The pet is a world actor found by guid pair in the proxied-character table (`00984960`; the client logs "EquipPetMannequin called with pet Guid=%llu, but that proxied character does not exist!" when the actor is missing). The server answers with `PetActive` (subtype 9: pet id plus actor guid). The client stores the pair and fires `PetsHandler:PetIsActive`. The pet then appears in the "Pet House" panel at the side of the screen with a paw button for tricks and an X to dismiss (client locale; the video shows the portrait panel with pet and paw buttons).
5. **Summon from an item.** `SendPetSummonByItemId` (`00b60110`, subtype 0x20) sends a pet id and an item id. Its only mapped caller is the item-record action walker `00a1ec00` (which also calls `SendPetEquipByItemRecord`), so it fires when a pet item is used (likely).
6. **Dismiss.** `Pet.Dismiss` (`00c07420`) sends subtype 4 with only the action byte set to 0 (the id field keeps its constructed default) and then resets the pet panel (`00b535b0`); `SendPetRecall` (`00b601a0`, no mapped caller) sends the pet id with action 0. The client can also dismiss on its own: `OnForcedDismiss` (`00b60610`) sends subtype 4 with the action byte alone (the id field stays at its default), shows a locale message, fires `PetsHandler:OnForcedDismiss`, clears the active pair and resets the panel. Its only mapped caller is `00ceafc0`, which applies area physics values (fields +0x490, +0x498, +0x49c, +0x4a0 of the local character) and calls the forced dismiss when one of them is set away from its default; the branch that restores defaults does not (likely). The "Your pet can't follow you, and has been dismissed." line appears on housing entry (video); it is probably the text of a forced dismiss, but that link is not traced.
7. **Following.** The mapped pet code has no follow logic of its own, so following is presumably done by the server-driven pet actor (speculative); the client only reports clicks. On a tile click `SendPetFollowLocation` (`00b5fcf0`) stores the new four-float position, requires the displacement from the last sent position to reach a threshold (`DAT_01b5d878`), resets the can-go flag (+0x12f9), reads the owner's position (`008a7d50`) and requires the distance from the owner to the click to fall between two limits (`DAT_0176b1b0`, `DAT_017ed93c`), requires one component of the supplied direction vector to be at or below `DAT_017649d4`, and requires a minimum time since the last send (a tick-time difference, `007027f0` and `008b0e20`, against `DAT_01b5d874`), then sends subtype 26 (four floats and a flag byte of 0). A server reply of subtype 26 with a non-zero flag sets +0x12f9 (client). None of the threshold constants is resolved.
8. **Water.** A character-state helper (`009607c0`, which sets bit 6 of the owner's +0x143) calls `00b60350`, which sets the in-water byte and sends subtype 28 with flag 1; the matching clear helper (`00960960`, labelled as water related) reaches `00b60420`, which sends flag 0 (client; the water trigger is inferred from those labels).
9. **Pet actors for previews.** `CreateActorFromPetTargetRecord` (`00b57170`) and `CreateActorForMannequinRecord` (`00b56dd0`) build a proxied actor from an item or target record. They are reached from the inbound subtype 0x26 handler and from `UpdatePetTarget` (`00b604b0`, called by `UseItemSelectedInUi` `00bdc060`), which sends subtype 0x26 with four dwords. This is the pet and pet-clothes preview path (likely).

#### Needs, care and utilities

- **State.** Four floats per pet at record +0x20 to +0x2c (food, groom, play and mood in some order; the columns are `FoodNeed`, `PlayNeed`, `GroomNeed` plus a mood value). `PetStatus` (subtype 3: pet id then four floats, NaN-checked) updates one pet and fires `PetsHandler:PetStatusUpdated`; `PetMoodList` (subtype 0x22: a counted list of five-dword records, id then four floats) updates many (client).
- **Starting an action.** `Pet.StartFeed` and `Pet.StartGroom` (`00c06980`, `00c069c0`) are thin wrappers that return whether the sender ran. The senders `00b5f720` (feed, subtype 0x13) and `00b5f8c0` (groom, subtype 0x12) send two dwords (the active pet id from handler +0x84, and a second dword they leave at 0) only when the in-water byte (+0x1300) is clear and the owner character's flag bit `0x20` at +0x143 is clear (a different test from `Pet.IsBusy`, which reads a busy byte at +0xb8 of the active pet's record and which the end of a gesture round clears); otherwise they fire `PetsHandler:UnlockUtilities` and a speaker/listener event (`00b56b80`) and return false (client). The same busy test guards the trick sender.
- **Server go-ahead.** `PetUtilityNotify` (subtype 0x14: pet id, a kind from 1 to 6, then more dwords) is the server's reply. Kind 1 fires `PetsHandler:BeginInteraction` and marks the proxied actor; kind 2 and kinds 4 to 6 run unlock and reset helpers (`00b56850`, `00b56920`, `00b56a00`); kind 3 sets the state word to 2, rebuilds the grooming spots with `PetTactile::RefreshSpots`, begins the interaction and starts an on-screen timer (`FlashTimerSetTimer` `00a8eff0`) from the next dword (client; the role names are inferred from the effects).
- **Equipping.** `Pet.EquipItem` and `Pet.UnequipItem` split on the id: ids up to 199 are wearables or the toy and go in a `PetEquip` packet (subtype 0x0b: flag byte then two dwords, `00b50e00`, wrapper `00b5c7e0`); ids of 200 and up are utility items, shifted down by 199, and go in a `PetUtility` packet (subtype 0x10: byte 0 equip or 1 unequip, then three dwords, `00b514e0`, writer `00c06ea0`). The server confirms with the same subtypes inbound (`0x0b`: flag and two dwords then a panel refresh; `0x10`: flag, pet id, slot and item id, stored at pet record +0x98 plus four times the slot, eight slots) (client). `SendPetEquipByItemRecord` (`00b607e0`, subtype 0x23) equips from an `ItemRecord`; it is called from the same item-record walker `00a1ec00` (likely).
- **Moods in the world.** `PetDecision` (subtype 1) tells the owner the pet took an interest: the client formats a "speaker/listener/target" text from the middle dword (a `Global.Text` id), shows it and fires `PetsHandler:PetTookAction` with the first dword, which looks like a decision id. The owner answers with `Pet.WasGood` or `Pet.WasBad` (`00c074c0`, `00c07630`), which send `PetResponse` (subtype 2: the decision id and a flag of 1 or 0) and appear to play a good or bad effect on the pet (`PetEffects.txt`: Good 5098, Bad 5099) (client; client locale "thumbs up or thumbs down").
- **Trial pets.** `PetExpireTime` (subtype 0x17: a guid pair, seven dwords and a flag) sets the trial timer (a dword times 1000 into +0x12fc). If the flag is 1 it fires `PetsHandler:TrialPetPurchased` and clears the timer; if the time is 0 it ends the trial, resets the trick-unlock state if a gesture object exists and posts an item-received style notice; otherwise the update `00b589c0` subtracts elapsed milliseconds from the timer and re-announces it through `PetsHandler:PetExpireTime` once per minute bucket (client). `PetTempLockoutError` (subtype 0x1d) shows a red system message with the remaining hours and minutes, which matches the locale "You cannot try another pet yet." (client).
- **Store tie-in.** `PetPacketOfferUpsell` (subtype 0x24) opens the marketplace (`MarketplaceHandler:Show`). `PetWorldToClientBuyResponse` (subtype 0x1f: two flags, a dword, an optional store-bundle id set) calls `CallStoreItemPreview` (`00b35320`); when the second flag is set that raises the Lua event `OnStorePetPreview` (client). Subtype 0x19 (`PetBuyRequest`) and 0x15 (`PetUpdateUtilityItemChange`) are read and discarded by the dispatcher.

#### Tricks and the gesture round

- **Trick list.** Each pet record has a list of trick rows (see Data and formats). A row carries an id, an icon (inferred from the column order), a name text id, a skill level and a "known" flag. `Pet.PerformTrick(id)` (`00c06890`) runs `DispatchPetActionRecord` (`00b5fb30`): the id must be positive, the active pet must have a matching row, and the row's known flag must be set. The id is saved at +0x12b8. A row whose fifth dword is 2 is a toy action and goes to the play-with-toy path (`00b5fa50`, subtype 0x21); any other row goes to `UnlockPetTricksAfterGesture` (`00cc7420`), which, if the owner is not busy and the pet is out, sends `PetPerformTrick` (subtype 7, one trick id) and then `PetTrickGestureResult` (subtype 8, the same value) back to back, and, when the controller is in the pet state (0x0c), also calls `00cc6710`, which switches towards controller state 14 (`ControllerDraw`, the drawing state) (client; the state switch is read from labels).
- **The round.** `PetGesture` loads `PetTrickGesturePoints.txt` and `PetTrickGestureSettings.txt` at construction (`00cc9020`, loader `00cc9bb0`). `BeginPetGestureRound` (`00cc9330`) finds the selected trick, scales its waypoints to the display, hides the HUD (`HUD:hideMain`), sets round state 4, sends `PetPerformTrick` with the trick id, and the round shows the prompts "Get Ready!" and "Draw!". If the trick has no gesture record the function scores nothing and sends nothing. The state machine (`00cc8d20`, `00cc97f0`, `00cc9950`, `00cca480`) samples the pointer, advances effects at each node, and ends on completion, the 10 second timeout or the 20 second hard stop. `AdvancePetGestureState` shows a tutorial the first time (client).
- **Scoring is local.** `ScorePetTrickGesture` (`00cc8890`) resamples both paths and combines four component scores (scale, centre error, segment direction match, time) with weights from constants into one number (`00cc8400`); three thresholds split it into grades 0 to 3, which pick one of four result strings (the `RESULT_*_STRING` ids) shown through `NotificationHandler`. Debug logging of the components is built in. The weights and thresholds are float constants that are not resolved (client).
- **What is sent.** `PetPerformTrick` goes out when the round begins (above); `FinishPetGestureState` (`00cc9a90`) sends `PetTrickGestureResult` through `00b5ffe0` with the selected trick id (the field the round stores at +0x94 and uses as its hash key), which also sets the handler state to 4. If the player drew nothing it only unlocks the panel and sends nothing. The grade is not in any outbound body found (client; the earlier vault note reaches the same reading).
- **Result back.** The server's `PetTrickNotify` (subtype 0x0f) has three kinds: 1 builds the `PetOrderDoTrick` text with the trick name and posts it as a world-say line for the owner, 2 fires `PetsHandler:TrickSuccessResult` with a text id and a numeric value, 3 fires `PetsHandler:UnlockTricks` to free the panel (client). `PetTrickSkillLevel` (subtype 0x0c: pet id, trick id, new level, announce flag) writes the level into the trick row (slot 3) and, if announced, posts a six second notice built from the pet name and the trick-name text (client; the locale has a line "[pet] has gotten better at [trick]!", not tied to this call). `PetTrickAdd` (subtype 0x0d) adds a row or marks it known and posts a learned notice, but only when the row was not already known and the trailing flag is clear (client).
- **Favourites and the equipped bar.** The pet has four favourite trick slots (record +0x88 to +0x94). `SendPetFavoriteTrick` (`00b5f670`) accepts slots 1 to 4 and sends subtype 0x0e with pet id, slot minus one and trick id; the inbound 0x0e stores the trick at +0x88 plus four times the slot and fires `PetsHandler:SetFavoritePetTrick` (client). The inbound guard accepts slot indexes 0 to 4, and index 4 would address +0x98, the first utility slot (Open questions).
- **Tutorial gate.** The tutorial quests (`Pet Tutorial: Basics`, `Happiness`, `Training`) must be finished, and the player must own a pet, before a trainer teaches more (client locale: "You must have a pet in addition to completing the pet tutorial").

#### Pet Trainer

- **The job.** Pet Trainer is a freestyle job whose experience comes from teaching tricks (wiki). The client locale says "You and your pet will only level up from tricks when you're in the pet trainer job. The higher level pet trainer you are, the faster you'll be able to teach your pet tricks." The client ships Pet Trainer gear parts, a level-up movie `levelup_pettrainer.gfx` and level-up sounds keyed by badge image set 1785 (vault 043, from the client Lua). It was removed from the game on 2009-12-14 but stayed usable on accounts that already had it (wiki); one video frame shows the Pet Trainer job at level 2 (video).
- **Unlocks by job level.** Client text: Sit Up at Pet Trainer level 5, Jump at 5, Peek-a-Boo at 10, Roll Over at 10. The wiki table adds the rest: dogs learn Speak at 1, Sit Up at 5, Roll Over and Peek-a-Boo at 10, Handstand at 15, Dance and Play Dead at 20; cats learn Speak at 1, Jump at 5, Box at 10, Spin and Handstand at 15, Dance and Play Dead at 20 (wiki).
- **Trainers and quests.** Three junior trainers start the job and run the three pet tutorials: Zachery (Stillwater Crossing, with his dog), Majorie (Cobblestone Village) and Mercy Merrywings (Highroad Junction). The main chain then runs through Roscoe (Sanctuary), Vidor Stormshroud (Shrouded Glade), Verill Shiverwings (Highroad Junction), Moanna (Seaside) and the Master Trainer Kirill Moonrunner (Sanctuary) (wiki; names also appear in client locale). Quest goals include scoring "Amazing" on a pet trick, maxing a trick such as Handstand and reaching Rank 4 in a trick (client locale).
- **Server-side progress.** Nothing on the client says how a trick gains levels or how many stars a round awards ("Keep practicing the same trick to gain stars for you and your pet", client locale). The XP curve and the job id are unrecovered (vault 043). The pet-side trick level is what `PetTrickSkillLevel` sets.

### Data and formats

**Client resources (read by the client, not sent).** Read in this pass from the shipped client baseline's `Resources` folder (`lab/runs/client-baseline-*/client/Resources`).

| File | Contents | Source |
|---|---|---|
| `PetNames.txt` | Caret-separated `GLOSSARY_ID^NAME^PART_ID^`. 60 first-part rows (`T4LIST_PET_FIRST_0001` to `0060`, the first is blank), 250 second-part rows (`T4LIST_PET_SECOND_*`) and a format row `T4LIST_PET_NAME_FORMAT^[*pet_first*] [*pet_second*]^0^`. Loader `00b5aa40` (via `00b5ae90`, resource directory `Resources/`) builds a sorted array of 0x24-byte two-string records. | client |
| `PetEffects.txt` | `Good^5098`, `Bad^5099`: effect ids for the owner's good and bad responses. Loader `00b562e0` stores them at handler +0x12b0/+0x12b4. | client |
| `PetTrickGesturePoints.txt` | `PET_TRICK_ID^SEQUENCE_ID^X^Y^`: 147 waypoint rows for 23 trick ids (1, 2, 11 to 20, 101 to 104, 501 to 507), coordinates in the unit square. Waypoints per id range from 3 to 15. | client |
| `PetTrickGestureSettings.txt` | `DRAW_DIST` 0.1, `SAMPLE_DRAW_RATE` 0.0005, `NUM_SEGMENTS` 128, effect ids `SAMPLE_DRAW_EFFECT` 5511, `PLAYER_DRAW_EFFECT` 5525, `SAMPLE_NODE_EFFECT` 5526, `PLAYER_NODE_EFFECT` 5527, `HIT_NODE_DIST_SQR` 0.0025, `TIMEOUT_SECONDS` 10, `FORCE_END_MILLISECONDS` 20000, result text ids 90155, 90157, 90161, 90163 and prompt ids 94617, 94618. | client |

The client ships no pet species, price, trick-definition or need-decay table. A pet's model, appearance and tricks come from the server record and the asset system (cat and dog model parts, for example `cat_longhair`, `cat_shorthair`, `dog_large_head_*` and `dog_small_head_*`, plus eight cat and dog `petparty` suit files and a Tiger the Party Animal model, are in the asset manifest).

**The pet record on the wire** (the reader is `00912cf0`, called once per entry of a `PetList`; widths and order are client-read, field names are inferred from the data sources and handlers, not from the binary).

| Order | Width | Client field | Notes |
|---|---|---|---|
| 1 | u32 | +0x00 | Pet id; the handler keys its table by this value. The list entry also sends the id as a separate u32 before the record. |
| 2 | u8 | +0x04 | Flag. |
| 3 | u32 | +0x08 | Unknown; the vault guesses a species id. |
| 4 to 7 | 4 x f32 | +0x20 to +0x2c | The four need and mood floats. A NaN sets the stream's NaN flag. |
| 8 | u8 | +0x30 | Flag. |
| 9 | i32 count, then rows | +0x34 list | Trick rows: u32 key, then eight u32 and one u8 (37 bytes each). |
| 10 | i32 count, then u32s | +0xbc array | Item guid style ids (purpose unresolved). |
| 11 | i32 count, then rows | +0xe4 list | u32 key then two u32 (12 bytes each); read as the pet's utility or profile items. |
| 12 | i32 length, bytes | +0x0c string | The first string read; the pet name by its use in messages. |
| 13 | u32 | +0x1c | Unknown. |
| 14 | u32 | +0x100 | Unknown. |
| 15 | i32 length, bytes | +0x108 string | The second string; a texture alias in the vault's reading. |
| 16 | u32 | +0xfc | Unknown (an icon id in the vault's reading). |
| 17 | u8 | +0x104 | Flag. |
| 18 | u32 | +0x118 | Unknown. |
| 19 | u8 | +0x11c | Flag. |
| 20 | 4 x u32 | +0x88 to +0x94 | Favourite trick ids (confirmed by the inbound 0x0e handler). |
| 21 | 8 x u32 | +0x98 to +0xb4 | Utility slot item ids (confirmed by the inbound 0x10 handler). |

The order and widths are those of the client reader above. Vault 091 lists field names reconstructed from a community serializer; this guide does not use that note for the record, and the names of the unknown fields stay open.

**Trick row.** Eight u32 and a u8. By the handlers: slot 0 repeats the trick id that keys the row, slot 2 is a text id that the learned message formats as the trick name, slot 3 is the skill level (`PetTrickSkillLevel` writes it), slot 4 equal to 2 marks a toy action, and the u8 at +0x20 is the "known" flag the perform path tests. The `PetTrickDataSource` columns are ID, icon, name, skill, busy, known and tooltip, which fits slots 0 to 3 as id, icon, name and skill (client). The remaining slots are unresolved.

**Chosen name request.** `00b5f2e0` takes two indexes into the name arrays (it rejects an index at or above the loaded count), reads the two name entries, looks up the `pet_first` and `pet_second` template keys and sends a `PetRequestName` packet (subtype 6) holding a string (client). The reply `PetSetName` (subtype 10) is: a guid pair (the pet actor), a length-prefixed string, then an accept flag (client reader `00b55730`); flag 0 shows the rejection text `@PetNameRejected`, otherwise the name is applied to the actor found by that guid pair (and the cached active pair is refreshed if it is the active pet). The roster record's name only changes through a new `PetList`.

**UI data-source columns (client).** `PetListDataSource`: pet name, sex, icon, `FoodNeed`, `PlayNeed`, `GroomNeed`, mood, id, `Members Only`, `Image Set ID`, `Tint Value`, `Is Vanity Pet`, `Attachment ItemGroup ID`, `Is Tutorial Pet` (indices 0 to 13, partly named). `PetActionDataSource`: ID, Target, Behavior. `PetTrickDataSource`: ID, icon, name, `SkillLevel`, `IsBusy`, `Known`, `TooltipString`. `PetTrickFavoriteDataSource`: ID, Slot, Icon, Name, SkillLevel (always four rows). `PetNameDataSource`: String, Id, Part, Selected. `PetGestureDataSource`: Trick, Order, X, Y. The shared pet item source `BasePetTrickDataSource` exposes item guid, icon, name, description, quantity, members only, tint, equippable, level, rarity, equipped, model name, texture alias, tint alias, activate enabled, ability ids, power rating and composite effect id. Pet and trick tooltips are built by `00a782d0` (7 rows) and `00a784e0` (3 rows).

### Network

**Envelope.** Pet traffic is message id `0x35` (53), carried tunneled inside the client packet (outer `PacketTunneledClientPacket`). The body starts with a u16 family id and a u8 subtype (reader `00b55280`, writer `00b595d0`), then the subtype's fields. The client defines 35 pet packet classes; the subtype numbers 0, 27, 30 and 37 have no class (constructors listed in shards S0346, S0347, S0383 and S0417; vault 091 agrees). No pet datagram has ever been decoded, so the datagram framing and the byte order on the wire are unverified. Strings in the client readers are an i32 length followed by the bytes.

**Server to client (handled by `00b5ce60`).**

| Sub | Class | Body (after the 3-byte header) | Client action |
|---|---|---|---|
| 1 | PetDecision | 3 x u32: decision id, text id, target (`00b552e0`) | Formats a speaker/listener/target text; fires `PetTookAction`. |
| 3 | PetStatus | u32 pet id, 4 x f32 (`00b555a0`) | Writes the four floats; fires `PetStatusUpdated`. |
| 5 | PetList | i32 count; per pet u32 key + record; u32 active pet id; u64 active actor guid (`00b5ba90`, `00b5aea0`, `00912cf0`) | Replaces the roster; sets active pet. |
| 9 | PetActive | u32 pet id, u64 actor guid (`00b556c0`) | Sets the active pair, refreshes panels. |
| 10 | PetSetName | u64 actor guid, string, u8 accept flag (`00b55730`) | Applies the name or shows rejection. |
| 11 | PetEquip | flag, then two u32 (`00b55c30`) | Refreshes the equip panel. |
| 12 | PetTrickSkillLevel | u32 pet id, u32 trick id, u32 level, flag (`00b557f0`) | Sets the level; optional notice. |
| 13 | PetTrickAdd | u32 pet id, 8 x u32 row, flag (`00b558a0`) | Adds or marks the trick; optional notice. |
| 14 | PetTrickFavoriteSet | 3 x u32: pet, slot, trick (`00b55a20`) | Stores the favourite. |
| 15 | PetTrickNotify | 5 x u32; the second is the kind, the fourth a text id (`00b55ab0`) | Kind 1 text, 2 result, 3 unlock. |
| 16 | PetUtility | flag, 3 x u32: pet, slot, item (`00b55b80`) | Stores utility slot; refreshes. |
| 20 | PetUtilityNotify | pet, kind, 4 more u32 (`00b55cc0`) | Begin or end an interaction. |
| 21 | PetUpdateUtilityItemChange | pair, 2 x u32, 3 skipped strings (`00b55db0`) | Read and discarded. |
| 23 | PetExpireTime | u64, 7 x u32, flag (`00b55370`) | Trial timer and purchase flag. |
| 25 | PetBuyRequest | u32 (`00b577d0`) | Read and discarded. |
| 26 | PetCanGoLocation | u64 pair, flag (`00b578d0`) | Sets the can-go flag. |
| 29 | PetTempLockoutError | 3 x u32 (`00b55510`) | Lockout message with hours and minutes. |
| 31 | PetWorldToClientBuyResponse | 2 flags, u32, optional bundle id set (`00b58780`) | Store preview event. |
| 33 | PetPlayWithToy | u32 plus a nested target (`00b55f50`) | Clears the toy target; sets the toy flag. |
| 34 | PetMoodList | i32 count of 5 x u32 (`00b57950`) | Updates each pet's four floats. |
| 36 | PetPacketOfferUpsell | none (`00b55280`) | Opens the marketplace. |
| 38 | PetPreviewRequest | 4 x u32 (`00b55fc0`) | Creates a preview actor. |

**Client to server (senders).**

| Sub | Class | Body | Sender |
|---|---|---|---|
| 2 | PetResponse | u32 id, u8 good flag | `Pet.WasGood`, `Pet.WasBad` (`00c074c0`, `00c07630`, writer `00c06e00`) |
| 4 | PetSummonRecall | u32 pet id, u8 action (1 summon, 0 recall) | `00b60080` (summon), `00b601a0` (recall with id); `Pet.Dismiss` `00c07420` and forced dismiss `00b60610` send the action byte alone |
| 6 | PetRequestName | name parts as strings (layout not decoded) | `00b5f2e0`, `00b5f500` |
| 7 | PetPerformTrick | u32 trick id | `00b5ff50`, from `00cc7420` and `00cc9330` |
| 8 | PetTrickGestureResult | u32 trick id | `00b5ffe0`, from `00cc7420` and `00cc9a90` |
| 11 | PetEquip | u8 flag (0 equip an item, 1 clear the slot), u32, u32 | `00b60220`, `00b602c0`, `Pet.EquipItem` and `UnequipItem` |
| 14 | PetTrickFavoriteSet | pet, slot minus one, trick | `00b5f670` |
| 16 | PetUtility | u8 (0 equip, 1 unequip), 3 x u32 | `Pet.EquipUtility`, `UnequipUtility` (`00c077a0`, `00c078a0`) |
| 17 | PetDelete | u32 id, then a trailing field written by `00c06fc0` | `Pet.Delete` (`00c07bc0`) |
| 18 | PetUtilityGroom | u32 active pet id, u32 left 0 | `Pet.StartGroom` via `00b5f8c0` |
| 19 | PetUtilityFeed | u32 active pet id, u32 left 0 | `Pet.StartFeed` via `00b5f720` |
| 22 | PetTactileNotify | pet handle and four values | `00ccebc0`, called only from the fishing controller in the mapped range |
| 24 | PetUiMode | u32 mode (0, 1 or 2) | `Pet.ModeSwitch` (`00b5f5d0`) |
| 26 | PetCanGoLocation | 4 x f32, u8 0 | `00b5fcf0` |
| 28 | PetInWater | u8 | `00b60350`, `00b60420` |
| 32 | PetSummonByItemId | u32 pet id, u32 item id | `00b60110` |
| 33 | PetPlayWithToy | u32 plus a nested target | `00b5fa50` |
| 35 | PetEquipByItemRecord | nested ItemRecord and a dword | `00b607e0` |
| 38 | PetPreviewRequest | 4 x u32 | `00b604b0` |

**Lua `Pet` module (24 entries, decoded this pass):** `SetName`, `SetNameCustom`, `Summon`, `Dismiss`, `WasGood`, `WasBad`, `GetActivePetId`, `CameraAtPet`, `CameraRestore`, `EquipItem`, `UnequipItem`, `PerformTrick`, `SetFavoriteTrick`, `EquipUtility`, `UnequipUtility`, `StartFeed`, `StartGroom`, `Delete`, `IsBusy`, `TutorialEnd`, `ModeSwitch`, `PlayWithToy`, `SetGestureData`, `StopGestureGame`. The client-to-script events the handler raises are `PetsHandler:` followed by `PetIsActive`, `PetExpireTime`, `PetStatusUpdated`, `PetTookAction`, `TrialPetPurchased`, `SetFavoritePetTrick`, `TrickSuccessResult`, `SetPetUtility`, `BeginInteraction`, `UnlockUtilities`, `UnlockTricks`, `OnSummonFailed`, `OnForcedDismiss`, `PopulatePetInventory`, `PetCanDraw`, `ThoughtBubble` and `RestoreGameDockFromPetError`.

### What the Rust server must do

Items marked (R) are reconstructed policy, not original behaviour.

1. **Persist the roster.** Per character: any number of pets, each with a per-owner increasing pet id (the client record class logs an error when the next-id counter is not above every id in use), the record fields in the table above (name, the four need floats, trick rows, utility and favourite slots, wearables, flags), and the species and model that the client does not hold. Keep the active pet in session state, not only on disk (R).
2. **Content tables the client lacks.** Species and model per pet, the catalogue (prices, member discount, availability channel, trick set), trick definitions (id, icon, name text id, skill rule, unlock job level), need decay and care effect values, and the Pet Trainer XP curve. None is in the client; derive them from our own design, using the wiki pet list as the roster and no emulator code or tables.
3. **Login.** Send `PetList` (5) with every pet, the active pet id and actor guid (or the default "none" pair). The client ignores all pet messages until its local player object exists, so send pet traffic after the player spawn (client).
4. **Summon and recall (4).** Validate ownership, a members-only rule where the pet is members-only, and the wiki restrictions (not in a minigame, dungeon or duel). For summon: spawn the pet as a world actor, then send `PetActive` (9) with the id and actor guid. For recall: despawn and send `PetActive` with the default pair (R, the exact "none" encoding is unconfirmed). Honour `SummonByItemId` (0x20) for trial and bundle pets. Dismiss the pet on house entry and on modified-physics areas (video, client). In houses honour the lot's pet-autospawn flag ([[Housing]]; client locale: it hides and prevents auto-spawned pets in all houses and does not affect summoned pets).
5. **Follow.** The pet actor's movement is server-owned. On `PetCanGoLocation` (26) decide whether the target is reachable, answer with a subtype 26 flag (the client sets its can-go flag), and move the pet. Track `PetInWater` (28) for the pet's swim behaviour (R).
6. **Care loop.** On feed (0x13), groom (0x12) and utility equip (0x10): require the owner not to be busy (the client already refuses while its busy bit is set or the owner is in water), consume food, vitamins, shampoo and conditioner (not the bowl or brush), and reply with `PetUtilityNotify` (0x14) kinds in the order the panel expects (begin, then end or reset), including the grooming timer duration. Apply the result with `PetStatus` (3) or `PetMoodList` (0x22). Run need decay over time, faster while the pet is out (wiki), and set the owner's speed buff from happiness rank (R).
7. **Tricks.** On `PerformTrick` (7) and `PetTrickGestureResult` (8), which the client may send back to back or split across a round (see Open questions): accept only a known, enabled trick the pet owns; treat the trick id as the claim and do not trust any client grade. Reply with `PetTrickNotify` (0x0f) kind 2 (result), `PetTrickSkillLevel` (0x0c) and `PetTrickAdd` (0x0d) when a trick is learned, and kind 3 to unlock the panel. Gate trick progress on the player being in the Pet Trainer job and its level (client locale) (R for the numbers).
8. **Favourites and equipment.** Store `PetTrickFavoriteSet` (0x0e) slots 1 to 4 and echo them. Handle `PetEquip` (0x0b) and `PetUtility` (0x10) with item ownership, slot rules (wearables up to id 199, utilities 200 and up) and the trade rule "can't be traded because it is equipped on a pet" (client locale). Echo the inbound forms the client expects.
9. **Names.** Accept `PetRequestName` (6) with the two chosen parts, validate them against the name lists, apply and reply with `PetSetName` (10). Reject with flag 0 to drive the `@PetNameRejected` text. Custom vanity names need a text filter and a paid "Pet Name Change" (R).
10. **Decisions and treasure.** Emit `PetDecision` (1) when the pet takes an interest and accept `PetResponse` (2). Run treasure finding (Common, Rare, Elite chests; vitamins raise the chance; thought bubbles near battles, dungeons and collection spawns), all tuning R (wiki, client locale).
11. **Adoption centers and store.** Support trial pets: a timed `PetExpireTime` (0x17) with the purchased flag, `PetTempLockoutError` (0x1d) when the player tries another trial too soon, and the `PetWorldToClientBuyResponse` (0x1f) store preview. Offer the marketplace upsell (0x24). The buy flow is store UI plus these replies; no pet purchase request sender was found (R for pricing and trial length).
12. **Delete.** Handle `PetDelete` (17); the trailing field is unresolved, so validate against the roster by id only (R).
13. **Previews.** Answer `PetPreviewRequest` (38) with the same subtype so the client builds the preview actor; the four-dword meaning is unresolved.
14. **Pet Party and events.** A recurring "Pet Party" community event gives a favour for bringing a pet (wiki). Pet party outfits exist as client assets (client manifest); nothing else is recovered.
15. **Errors.** Every refusal should end in a visible reply (an `OnSummonFailed` condition, a lockout message, a name rejection), never silence.

### Video and wiki evidence

**Video frames reviewed:** the reviewed frame records that show a pet panel, pet command bar or pet content, plus the store, quest and inventory frames that mention pets. Names shown over or beside pets are omitted throughout.

- **Species and roster (wiki).** The wiki lists 114 pet pages. By infobox category: 34 dogs (14 small, 20 large), 23 cats (11 long-hair, 12 short-hair), 6 ponies, 5 T-Rex, 4 pegasi, 3 unicorns, 3 penguins, 2 each of beetle, owl, dragon, groundhog, monkey, raptor, spider, reindeer, triceratops, tiger and treeble, and single bear, balloon pet, bat, coyote, duckbill, hippo, hydra, robgoblin mech, phoenix, pig, flying eyeball, robot and vulture (plus one boar page with no category). Infobox traits: 20 Friendly, 15 Lazy, 11 Excited, 7 Nervous, one Angry, one Curious; 59 pages have no trait. The client has no trait field in the data read here (Open questions).
- **Prices (wiki, video).** 99 pages list a Station Cash price, from 175 to 749. The commonest are 225 (26 pets), 400 (14) and 349 (11). Members pay about 10 percent less (93 of 95 pages with two prices; Raja 400 and 360, Phoenix 449 and 404, Hydra 550 and 495). Phoenix and Hydra were VIP-rank purchases until 2013-05-22 (wiki). Video Item Of The Day frames show **Raja (tiger) 200 Station Cash with 400 struck through**, **Hurricane (dog) 400 struck through, now 200**, and **Nightmare Pony** at 200 with the regular price struck through (its struck value was not read). The wiki regular price is 400 for all three, so the Raja and Hurricane sales are 50 percent. An in-game ad card says pets run "from 50 - 500 each" with a 30 percent weekend cut, which does not match the wiki range (Open questions).
- **Tricks (wiki, video).** 97 pages list tricks: 31 pages have the seven-trick cat set (Speak, Jump, Box, Dance, Spin, Handstand, Play Dead), of which 23 are cats and 8 are dogs obtained from the Mysterious Merchant, the Daily Wheel or a reward card; 26 dogs have the seven-trick dog set (Speak, Sit Up, Roll Over, Play Dead, Peek-a-boo, Handstand, Dance); 12 ponies, pegasi and unicorns have Stand, Bow, Kick; other species have small sets of three: tigers Fire Jump, Hand Stand, Kung Fu; the phoenix Spitfire, Burn, Rebirth; the bear Roar, Lightning, Grow; the flying eyeball Echo, Perch, Aerial; dragons Breathe Fire, Twirl, Bounce; penguins Glide, Fly, Spin. 17 pages (owls, bat, coyote, hydra and others) list none. Video shows pet command bars reading **Roar, Lightning, Grow** (with a green ability cloud on a desert lot) and **Echo, Perch, Aerial**; both triples match a wiki species, but the video frames did not read the species, so the match is an inference.
- **Pet panels (video).** The Pets window is a paged grid headed "Select a Pet below to summon it." with "Visit the Marketplace to buy more pets." and an Item Of The Day ad inside it. The portrait panel at the side shows a name bar, paw and X buttons and a stats panel with mood, food and energy bars with hearts floating around the pet. An apartment frame shows the dismissal line quoted in Overview.
- **Food and bowls (video).** The Coin Shop sells pet feeding bowls ("A feeding bowl for your pet."). One list view shows 14 bowls: Bones Bowl in Red, Green, Blue, Yellow, Purple and Pink, Daisies Bowl in the same six colours, and Paws Bowl in Red and Green (more are probably below the fold). **Regular price 15 coins, member price 13**; the detail pane shows "You could have saved 2". The same shop's furniture list shows a Pet Bed at 800 coins, a Cat Tower at 500 and a Dog House at 1100. Client locale supplies the care items: shampoo, conditioner, soft-bristle and other brushes, food, vitamins (including treasure-hunting vitamins in small, medium and large) and bowls. The video shows two pet vendor NPC titles, "Pet Supplies" and "Pet Clothing"; the locale names two Cobblestone Village pet vendors (video, client locale).
- **Events and quests (video, wiki, client).** A Pet Party crowd fills a cobblestone plaza (video). The wiki quest "Party for Pets" gives a "Pet Party Favor" from Tiger the Party Animal when the player brings a summoned pet; the client manifest has a Tiger the Party Animal model and cat and dog "petparty" outfits. Quest frames show "Pets on the Loose!" (find the lost pets in a camp, with markers labelled as lost animals), "Peep's Missing Bowl" (find a dog bowl on the Farnum Farmstead) and a "Pet Adventures" sign (video).
- **Adoption centers (client locale).** Adoption centers sit in Cobblestone Village, Stillwater Crossing, Highroad Vale and Junction, Seaside and Sanctuary. Players click a pet, then choose Buy Pet or Try Pet; a try lasts a short time, and "You cannot try another pet yet." follows a recent trial.

### Open questions and contradictions

- **No captured pet traffic.** Everything under Network is read from the client's own readers and writers. Framing, string encoding and the unlisted body layouts (subtype 6 string payload, subtype 17 trailing field, the trailing dwords of subtypes 1, 20, 23, 33 and 38) are unconfirmed.
- **Record fields.** Fields 2, 3, 8, 13, 14, 16, 17, 18, 19 of the pet record and trick row slots 4 to 7 are read but not named. The vault's names (food, groom, exercise, happiness, texture alias, icon id) come from a community serializer and an older recorded dump and are not used here; the client only confirms widths, order, the favourite and utility arrays, the trick key row and the four floats.
- **TrickAdd slot order.** The inbound `PetTrickAdd` handler (bytes at `00b5e486` to `00b5e4d5`) stores what the stack layout suggests are wire dwords 7, 5 and 6 into row slots 5, 6 and 7, while the list reader fills slots 0 to 7 in wire order. If that is real, a row sent in `TrickAdd` is permuted compared with the same row in a `PetList`. Needs a live check.
- **Favourite slot guard.** The inbound `PetTrickFavoriteSet` accepts indexes 0 to 4 but the favourite array holds four entries (+0x88 to +0x94); index 4 would overwrite utility slot 0. The sender only emits 0 to 3. Whether the server ever sent 4 is unknown. The `PetTricksEquippedDataSource` reads "one of five equipped trick slots", which also suggests a fifth slot exists somewhere.
- **Gesture ids and names.** The client has 23 gesture shapes, but the wiki lists many more trick names across species. The map from trick id to shape and to name is not in the client files. An older vault note maps seven ids to dog tricks from a community dump; this guide does not adopt that mapping.
- **Two trick send paths.** `UnlockPetTricksAfterGesture` sends subtypes 7 and 8 together, while a drawing round sends 7 at its start and 8 at its end (so a full round could emit subtype 8 twice if both paths ran). Which UI action takes which path, and whether the server ever got both, is untraced.
- **Grade handling.** The grade (0 to 3) is local and no packet found carries it. Whether the original server rewarded by grade, trusted the client or ignored it is unknown. The result labels behind ids 90155, 90157, 90161 and 90163 could not be resolved from the client locale file in this pass (it is keyed by hashed ids). Quest text calls the top grade "Amazing" (client locale); vault 011 reads 90155 as "Poor" from a community reader and leaves the middle two unknown.
- **Pet Trainer ids.** The client's item filter names Pet Trainer as job id 121 (client resource; see [[Character and Customization]]), but the job's own definition row is server data and is not in the client, so its server-side details are unrecovered. The level-up sound keys (1, 4, 8, 12, 16) and the audio file names (1, 5, 10, 15, 20) disagree (vault 043). XP thresholds and star rules are unrecovered.
- **Lua surface drift.** Vault 091 lists a `RaiseNeed` call that is not in the binary's 24-entry `Pet` module table; it may be a script-level name. The table holds the other 21 names from that note plus `UnequipItem`, `UnequipUtility` and `Delete`.
- **`PetTactileNotify` (22).** The only mapped sender is the fishing controller; the grooming flow (`PetTactile`) has no mapped sender, and the direction for pets is unproven.
- **Buy flow.** No client function sends subtype 25 (`PetBuyRequest`); the dispatcher reads and discards it. The earlier shard note guessed that subtype 31 is an accept or refuse code; the handler instead feeds the store preview, so purchase confirmation probably comes through the store system, not this family.
- **Traits.** The wiki's Friendly, Lazy, Excited, Nervous, Angry and Curious traits have no matching field found in the client record or data sources; they may be server-side or purely cosmetic.
- **Price range.** The in-game ad ("from 50 - 500 each") and the wiki range (175 to 749) differ; the ad may predate later pets or exclude premium ones.
- **Dismissal on physics areas.** The link from `ClientAreaPhysicsProperty` application to forced dismiss is read from one caller; which areas carry such properties is unknown.
- **Busy flag.** Vault 091 reads the `+0x143` bit tested by the feed, groom and trick senders as a pet-busy flag. The object tested is the one at client +0xc8, which `HandlePetPacket` also passes as the speaker, so it is the owner's character; what bit `0x20` means is unresolved. The pet record has its own busy byte at +0xb8, read by `Pet.IsBusy`; what sets it is not traced. The pair at handler +0x88/+0x8c, which shard S0349 could not explain, is the active pet's actor guid (it is looked up in the proxied-character table); the pair at +0x1308 is the preview target.
- **Speculative labels.** `ControllerPet` slot labels, `PetsHandler::RefreshCachedPetEntity`, `ResolveOwnActor`, the 8-byte thunks `00b52010` and `00b52030`, `PetGestureSetTerminalState`, and the reader adaptors for subtypes 0x19 and 0x15 rest on shape alone.

### Key functions

| Address | Role |
|---|---|
| `009471a0`, `00a10950`, `00b5ce60` | Client packet switch case 0x35, adjustor thunk, `HandlePetPacket` (likely) |
| `00b596f0`, `00b5b470`, `00b59b90` | Pet data-source registration and handler teardown |
| `00b50670`, `00b50610` | Pet base packet constructors (`0x35` at +4, subtype at +8) |
| `00b55280` to `00b55fc0`, `00b57c70` to `00b58450`, `00b5a130`, `00b5a1b0`, `00b5ba90`, `00b5cb20` | Field readers and their checked adapters |
| `00912cf0`, `00b5aea0`, `00b58830` | Pet record reader, list reader, record copy |
| `00b5a240` to `00b5a8d0`, `00b5b0c0`, `00c06e00`, `00c06ea0`, `00c06fc0` | Outbound body writers |
| `00b5bc20` to `00b5cd50`, `00c07090`, `00c071a0`, `00c072b0` | Tunnel wrappers and gateway templates |
| `00b60080`, `00b601a0`, `00b60110`, `00b549f0`, `00b60610` | Summon, recall, summon by item, summon gate, forced dismiss |
| `00b5fcf0`, `00b60770`, `00b5fbc0`, `00b5fa50` | Follow location, tile handler, toy target, play with toy |
| `00b60350`, `00b60420`, `00b5f5d0`, `00b604b0`, `00b607e0` | In-water, UI mode, preview request, equip by item record |
| `00b5f720`, `00b5f8c0`, `00b60220`, `00b602c0`, `00c077a0`, `00c078a0` | Feed, groom, equip and utility senders |
| `00b5fb30`, `00cc7420`, `00cc6710`, `00b5ff50`, `00b5ffe0`, `00b5f670` | Perform trick dispatch, immediate trick path, controller switch, trick senders, favourite sender |
| `00a1ec00` | Item-record action walker that calls summon by item and equip by item record (likely) |
| `00cc9020`, `00cc9bb0`, `00cc9330`, `00cc8d20`, `00cc8890`, `00cc8400`, `00cc9a90` | Gesture ctor, loader, round start, states, scoring, finish |
| `00cce960`, `00cce490`, `00ccdb70`, `00ccebc0` | Grooming spots, effect release, tactile packet ctor and sender |
| `00b562e0`, `00b5aa40`, `00b5ae90`, `0097c5c0`, `0097b770` | `PetEffects.txt`, `PetNames.txt` loaders and name templates |
| `00b589c0`, `00b58bd0`, `00b53450`, `00b54080`, `00b53560` | Trial timer, active pet, panel refresh, set and clear active pet |
| `00b35320` | `CallStoreItemPreview` (pet store preview event) |
| `00c06ac0`, `01829f60`, `00c066a0`, `00c07420`, `00c07990`, `00c07bc0` | Lua `Pet` registrar, table, Summon, Dismiss, EquipItem, Delete |
| `00ccacc0`, `00cca5c0`, `00ccd800` to `00ccdac0` | Pet data-source columns and constructors |
| `00a782d0`, `00a784e0` | Pet and trick tooltip rows |
| `01044560`, `010428d0` | `CharacterBase::CreatePetRecord`, pet record constructor |
<!-- NARRATIVE END -->

## Mapped functions (generated)

341 accepted functions: 150 confirmed, 95 likely, 29 speculative, 67 unknown. Network-linked (`wire`): 47.

Subsystems: `net.pet` (160), `ui.pets` (101), `game.pet` (80)

### `game.pet`

**BasePetTrickDataSource** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b5b1b0` | BasePetTrickDataSource::ScalarDeletingDtor | confirmed | Calls the GuiUserDataTable destructor and frees the object only when param_2 bit 0 is set. |
| `00b5b120` | — | unknown | Calls the GuiUserDataTable constructor, installs BasePetTrickDataSource vtables at three subobject offsets and stores param_2 at field 0x254. |

**CharacterBase** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01044560` | CharacterBase::CreatePetRecord | confirmed | Thiscall. Allocates 0x1f8 bytes with operator new (00d833a6, pushed size 0x1f8), and if non-null calls 010428d0 with the block in ECX. Stores param_2 at block+… |

**ControllerPet** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00afd610` | ControllerPet::~ControllerPet | confirmed | Fastcall dtor body (ECX=this, ret 0): stores ControllerPet::vftable at +0x00, sets the unwind state to -1, and calls 00afe290 on this (base teardown). |
| `00afd6f0` | ControllerPet::ControllerPet | confirmed | Thiscall ctor, 3 forwarded args to base 00afe470 then returns this (ret 0x14). Stores ControllerPet::vftable at +0x00, zeroes dword +0x3b0, sets byte +0x3b4 to… |
| `00afd7b0` | ControllerPet::ScalarDeletingDtor | confirmed | Scalar deleting dtor shape (thiscall, one flag byte arg, ret 4): stores ControllerPet::vftable, sets unwind state -1, calls 00afe290 on this, then calls _free(… |
| `00afd680` | ControllerPet::GetByte3b4 | speculative | Thiscall getter, no args, vtable slot 72 (+0x120): returns the byte at this+0x3b4 (mov al,[ecx+0x3b4]; ret). |
| `00afd690` | ControllerPet::SetByte3b4 | speculative | Thiscall setter with one byte arg (ret 4), vtable slot 73 (+0x124): stores the arg byte into this+0x3b4. |
| `00afd6c0` | ControllerPet::InvokeSlot53ThenSlot45 | speculative | Thiscall, vtable slot 46 (+0xb8). Calls this->vtable[+0xd4] with one stack arg this+0xe0, then this->vtable[+0xb4] with one stack arg 0. |
| `00afd810` | ControllerPet::DistanceOfFourFloatsToSlot5c | speculative | Thiscall with one arg, vtable slot 53 (+0xd4). Calls 0094d3b0(arg), this->vtable[+0x70](DAT_017eb654), this->vtable[+0xdc](1), 008a6660(), 00b536e0() to get an… |
| `00afd8d0` | ControllerPet::ForwardListNodes_c8 | speculative | Thiscall with one arg, vtable slot 50 (+0xc8): calls 00afe5d0(arg), then this->vtable[+0x128](arg), then walks the singly linked list headed at this+0xc8 (next… |
| `00afd920` | — | unknown | Thiscall with one int arg (time step), vtable slot 74 (+0x128). Calls 0094d650() and returns if null. Updates the counter at +0xec by the arg and flag byte +0x… |

**PetGesture** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cc7930` | PetGestureDtor | confirmed | Installs the PetGesture vtable, destroys tracked effects and both Vector2 arrays, frees their backing buffers, and releases gesture-owned runtime data. |
| `00cc8760` | PetGestureScalarDeletingDtor | confirmed | Calls PetGesture destructor and, when deleting flag is set, frees the object. Bytes confirm ordinary call wrapper followed by return. |
| `00cc9020` | PetGestureCtor | confirmed | Installs PetGesture vtable, initializes handler fields and two Vector2 arrays, sets effect and locale ids plus gesture limits, then loads PetTrickGestureSettin… |
| `00cc9bb0` | LoadPetGestureDataAndSettings | confirmed | For points file, parses caret-delimited trick and sequence ids plus float x/y into per-trick Vector2 arrays; for settings file, parses keys and writes effect i… |
| `00cca3f0` | PetGestureSetTerminalState | speculative | Writes value eight to PetGesture state field +0x60 and returns. Bytes show the field store and cookie epilogue. |

**PetTactile** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cce850` | PetTactile::Ctor | confirmed | Stores two constructor arguments, installs PetTactile vtable, clears tactile fields and initializes an embedded grooming spot hash-list map. |
| `00cce940` | PetTactile::ScalarDeletingDtor | confirmed | Destroys the PetTactile object and frees its allocation when the deleting flag is set. |
| `00cce490` | PetTactile::ReleaseSpotEffects | likely | For each grooming-map node, releases the two associated effect handles, then clears the grooming spot map. |
| `00cce620` | PetTactile::Dtor | likely | Restores PetTactile vtable, clears the shared pet panel state when active, then restores and clears its embedded grooming map. |
| `00cce960` | PetTactile::RefreshSpots | likely | Clears the spot map, resolves current pet tactile data, enumerates configured spots, inserts their ids and positions, and updates the activity distance state. |

**PetTileTargetHandler** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b60770` | PetTileTargetHandler::OnMessage | likely | Dispatches on the fifth argument: when param_5 == 1 it forwards all four other arguments to FUN_00b5fbc0, when param_5 == 0x1000 it forwards them to FUN_00b5fc… |

**PetTricksDataSource** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b5b210` | PetTricksDataSource::PetTricksDataSource | confirmed | Calls the BasePetTrickDataSource constructor, then replaces its three vtable pointers with PetTricksDataSource vtables. |
| `00b5b2e0` | PetTricksDataSource::ScalarDeletingDtor | confirmed | Destroys the GuiUserDataTable base and frees the object only when param_2 bit 0 is set. |

**PetTricksEquippedDataSource** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b5b340` | PetTricksEquippedDataSource::PetTricksEquippedDataSource | confirmed | Calls the BasePetTrickDataSource constructor, then installs PetTricksEquippedDataSource vtables at its three data-source subobjects. |
| `00b5b410` | PetTricksEquippedDataSource::ScalarDeletingDtor | confirmed | Destroys the GuiUserDataTable base and frees the object only when param_2 bit 0 is set. |

**PetsHandler** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b589c0` | PetsHandler::PetExpireTime | confirmed | Finds the proxied pet, updates its active mannequin and visual state, subtracts elapsed milliseconds from the trial timer, and emits the expiry-time UI event w… |
| `00b58bd0` | PetsHandler::PetIsActive | confirmed | Stores the supplied pet identifier pair, refreshes the cached active pair, builds a Lua parameter array and switches callback state for selected prior values. |
| `00b5ce60` | PetsHandler::HandlePetPacket 🌐 | likely | Dispatches pet subtype values through case-specific readers, updates pet/trick/utility state, changes Lua-visible data sources, emits notifications and log mes… |
| `00ccdda0` | PetsHandler::ResetPetPanel | likely | Clears the active pet guid at client state offset 0x12ac and invokes the UI reset helper when the shared client state is present. |
| `00b604b0` | — | unknown | For a non-null target record param_2 where FUN_00bae700 and FUN_00bae740 both return 0 and param_2+0x6c != 0x1d: if the tracked pair (+0x1308/+0x130c) still eq… |
| `00b60610` | — | unknown | Runs only when the tracked pair at +0x88/+0x8c differs from the shared default pair DAT_01be9810/DAT_01be9814: it constructs a PetSummonRecallPacket with FUN_0… |

**(no class)** (47)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b562e0` | LoadPetEffectsFile | confirmed | Formats a PetEffects.txt path, reads lines, skips comment lines, parses each remaining line and stores values in two fields at param_1+0x12b0 and +0x12b4. |
| `00b58d90` | EquipPetMannequinBundle | confirmed | Scans pet records against proxied characters, updates target mannequin actors, selects the cached active pet pair, sets its actor flag and applies the matched … |
| `00b5ae90` | LoadDefaultPetNamesFromResources | confirmed | Calls 00b5aa40 with the literal resource directory Resources/ and returns. |
| `00cc7080` | InitializePetTrickUnlockState | confirmed | Copies two globals into fields +0x128 and +0x12c, calls FUN_00cc6af0, then logs PetsHandler:UnlockTricks. |
| `00cc7420` | UnlockPetTricksAfterGesture | confirmed | Stores the current pair from shared pet context, checks availability and mode flags, resets trick state, invokes the gesture-result sender with param_2, and up… |
| `00cc8780` | GestureScaleRatio | confirmed | Computes maximum-axis span for two point sets, optionally logs both spans and their ratio, then returns sample span divided by player span. |
| `00cc8890` | ScorePetTrickGesture | confirmed | For paths longer than one point, resamples both paths, computes center error, segment match and elapsed time, combines scores, stores grade 0-3, and submits a … |
| `00cc8d20` | AdvancePetGestureState | confirmed | Stores requested state and handles cases 2,4,5,6,7 by updating gesture timing and point arrays, showing ready/go notices, initializing tutorial, or scoring com… |
| `00cc9330` | BeginPetGestureRound | confirmed | Finds selected trick record, transforms its points to active display dimensions, resets capture fields, initializes timing and effects, hides the main HUD, and… |
| `00cc9a90` | FinishPetGestureState | confirmed | Clears active pet gesture flag, releases pet and actor references, destroys tracked effects, then either unlocks tricks or sends gesture-result packet and retu… |
| `0097c5c0` | PetNameTemplateExpand | likely | Gets the handle for the template 'PetName' from the object at this+8+4 through its vtable slot 0x14, builds a temporary IString pair with 0097a330, assigns 'pe… |
| `00a127e0` | PetsHandler_PetCanDraw_Thunk | likely | Forwards both arguments to FUN_00b53530 and returns. |
| `00b56090` | InitTwoStringFieldsAndValue | likely | Initializes two adjacent IString-like fields at offsets 0 and 0x10, then stores param_4 at offset 0x20; returns param_1. |
| `00b56920` | UnlockUtilitiesResetB | likely | Invokes an additional setup callback, assigns the UnlockUtilities log label, clears a string/array-like member and proxied-actor flag, and zeros param_1+0x12ac. |
| `00b56dd0` | CreateActorForMannequinRecord | likely | Checks a cached pair against shared defaults, finds an entry for param_2, resolves its mannequin data and transform, creates an actor with string and transform… |
| `00b57170` | CreateActorFromPetTargetRecord | likely | Requires cached identifiers and a non-null record, resolves its model data, builds strings and transform data from the record, creates an actor, clears one act… |
| `00b59070` | UnlockPetTricks | likely | Assigns the UnlockTricks event label, clears the tracked trick count at param_1+0x12b8, builds a Lua parameter array, optionally invokes a callback, and destro… |
| `00b596f0` | InitializePetClientBindings | likely | Zeros pet-handler fields and state arrays, initializes PetName storage, creates Lua data sources named BaseClient.PetList and BaseClient.PetActive, and constru… |
| `00b59b90` | DestroyPetClientHandlerState | likely | Releases owned handler references, list nodes and PetName arrays; removes tracked hash/list entries and releases numerous retained objects while shared state r… |
| `00b5fb30` | DispatchPetActionRecord | likely | Rejects nonpositive action ids, finds the active pet and matching action record, rejects records whose enabled byte is clear, stores the action id, then routes… |
| `00cc6710` | SetFlag138AndNotifyForState14 | likely | Sets byte +0x138, calls 008b9f00, then checks the object at [this+0x4c]+0xec; when its virtual slot 0 returns 0x0e, calls 0094c8a0 with zero and 00960780. |
| `00cc6a80` | FindState14ObjectByPair | likely | Requires the object at [this+0x4c]+0xec to report virtual state 0x0e, obtains a candidate through 0094d650, reads its two-word key through nested vtable slot +… |
| `00cc6af0` | ClearFlag138AndResetDependentState | likely | When byte +0x138 equals one, conditionally calls 0094c8a0 with one for state 14, clears the flag, calls 008d2950, then for state 12 updates a pair through 008b… |
| `00cc81f0` | CompareGestureSegmentDirections | likely | Returns a sentinel when array lengths differ; otherwise examines corresponding segments, ignores sample segments below DAT_018212d0, and accumulates nonnegativ… |
| `00cc8630` | RegisterGestureSamplePoints | likely | For each Vector2 input, allocates a 0x20-byte node, copies coordinates, links it into an ordered list and hash bucket, and increments node count. |
| `00cc97f0` | UpdateGestureRoundState | likely | Checks play mode, initializes point capture when needed, records elapsed time, and switches to end state seven when configured duration expires. |
| `00cc9950` | SampleGestureRoundState | likely | Updates elapsed time, samples pointer positions and advances registered point effects while the relevant mode is active; switches to end state when limits are … |
| `009652a0` | AssignStringAndRefreshTransform | speculative | Assigns the supplied string through FUN_00402ce0, calls FUN_00726400, updates the transform through FUN_0097c280, and sets bit 0 at +0x140. |
| `00bda800` | AccessorThenTailJump_00b53690 | speculative | The 14-byte x86 body loads the pointer at [ECX] into ECX, calls 008a6660, copies EAX to ECX, then tail-jumps to 00b53690. This is a forwarding thunk; it does n… |
| `00cc7f60` | AverageVector2Points | speculative | Accumulates x and y from every 8-byte vector and writes each sum divided by array count to the two output pointers. |
| `00cc8040` | MaxAxisSpan | speculative | Scans Vector2 entries for minimum and maximum x and y, then returns the larger coordinate span. |
| `00cc8ff0` | AdvanceToGestureReadyState | speculative | If state is two and param_2 equals twelve, writes state three; otherwise returns without changing state. Bytes confirm the comparisons and guarded store. |
| `00cca480` | DispatchPetGestureUpdate | speculative | Checks gesture timeout and routes states three through seven to start, capture, sampling, finish, or cleanup routines. |
| `00ccdc90` | ReadPetUiMode12Object | speculative | Reads an interface pointer at a nested object offset, invokes its first virtual method, and returns the pointer only when that method returns 12. |
| `00ccdcc0` | SetPetObjectFlagZero | speculative | Checks a nested object twice for runtime id 12 and, when both checks pass, invokes virtual slot 0x124 with argument zero. |
| `00ccdd20` | SetPetObjectFlagOne | speculative | Checks a nested object twice for runtime id 12 and, when both checks pass, invokes virtual slot 0x124 with argument one. |
| `0097b770` | — | unknown | Looks up the 'pet_first' and then 'pet_second' template keys through FUN_00e54610 (with the strings at param_2/param_3 and a flag derived from param_5); return… |
| `00b56850` | — | unknown | Assigns the UnlockUtilities log label, clears a string/array-like member, clears a proxied-actor flag and zeros the dword at param_1+0x12ac. |
| `00b56a00` | — | unknown | Assigns the UnlockUtilities log label, clears a string/array-like member and proxied-actor flag, zeros param_1+0x12ac, and calls a stub cleanup routine. |
| `00b576e0` | — | unknown | Returns the record at index param_2 in a 0x24-byte array; extends capacity/count as needed and initializes intervening records with the two-string-pair constru… |
| `00b5aa40` | — | unknown | Opens the formatted PetNames.txt resource, skips comments, parses name/value records, inserts new entries in sorted order, replaces matching entries and stores… |
| `00cc7a00` | — | unknown | Measures input polyline length, sets output count to the configured segment limit, interpolates points at equal distances, and copies the final endpoint. |
| `00cc8400` | — | unknown | Clamps four component scores at zero, scales them by global weights, combines weighted terms, optionally logs them, and returns the combined score. |
| `00cc95f0` | — | unknown | Before the initial delay expires, increments timer; afterwards compares successive pointer positions to hit radius, appends or advances sampled points, and tra… |
| `00cca410` | — | unknown | Calls gesture reset helper; if state is not one, sets state seven, resets timer, checks actor, and scores or closes the current gesture flow. |
| `00ccdc70` | — | unknown | Checks the shared client state for a nonzero field at offset 0x858 and then invokes a global helper with arguments zero and one. |
| `00cce1d0` | — | unknown | Validates the nested activity object, obtains current activity context, refreshes 42 generated ids and two further ids, then copies coordinates and offsets fro… |

### `net.pet`

**PetActivePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b50d40` | PetActivePacket::~PetActivePacket | confirmed | Destructor: stores PetActivePacket::vftable then BasePacket::vftable at *this; the pet dispatcher 00b5ce60 and one unwind funclet call it. |
| `00b50d90` | PetActivePacket::ScalarDeletingDestructor | confirmed | Scalar deleting destructor: resets the vtable pointer at *this to PetActivePacket::vftable then BasePacket::vftable and calls _free(param_1) when bit 0 of the … |
| `00b53020` | PetActivePacket::PetActivePacket 🌐 | likely | Constructor: calls the pet base constructor 00b50670 with subtype 9, stores PetActivePacket::vftable, writes the sentinel 0xffffffff at +0xc and the shared def… |

**PetBasePacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b506e0` | PetBasePacket::~PetBasePacket | likely | Destructor body of the pet base packet: rewrites the pointer at *this with BasePacket::vftable and returns; no members are released. 119 MSVC unwind funclets c… |
| `00b50610` | PetBasePacket::PetBasePacket 🌐 | speculative | Default constructor of the pet base packet object: writes BasePacket::vftable at *this, stores the pet base opcode 0x35 at +4, rewrites *this with PetBasePacke… |
| `00b50670` | — 🌐 | unknown | Pet base-packet constructor taking the subtype: writes BasePacket::vftable, stores the pet base opcode 0x35 at +4, writes PetBasePacket::vftable and saves the … |
| `00b52db0` | — | unknown | Scalar deleting destructor: resets the vtable pointer at *this to BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set; returns … |

**PetBuyRequestPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b51ad0` | PetBuyRequestPacket::~PetBuyRequestPacket | likely | Destructor: resets the vtable pointer at *this to BasePacket::vftable and returns. |
| `00b531b0` | PetBuyRequestPacket::ScalarDeletingDestructor | likely | Scalar deleting destructor: resets the vtable pointer at *this to BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set; returns … |
| `00b51a70` | — 🌐 | unknown | Constructor: calls the pet base constructor 00b50670 with subtype 0x19 (25), then stores PetBuyRequestPacket::vftable at *this. |

**PetCanGoLocationPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b53210` | PetCanGoLocationPacket::ScalarDeletingDestructor | likely | Scalar deleting destructor: resets the vtable pointer at *this to BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set; returns … |
| `00b51b20` | — 🌐 | unknown | Constructor: calls the pet base constructor 00b50670 with subtype 0x1a (26), stores PetCanGoLocationPacket::vftable and zeroes the byte at +0x20 (the flag byte… |
| `00b51b80` | — | unknown | Destructor: resets the vtable pointer at *this to BasePacket::vftable and returns; the follow-location senders 00b5ce60/00b5fcf0 and two unwind funclets call i… |

**PetDecisionPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b50730` | PetDecisionPacket::PetDecisionPacket 🌐 | confirmed | Constructor: calls the pet base constructor 00b50670 with subtype 1, then stores PetDecisionPacket::vftable at *this inside an SEH frame. |
| `00b50790` | PetDecisionPacket::~PetDecisionPacket | likely | Destructor: resets the vtable pointer at *this to BasePacket::vftable and returns; the class has only scalar members, so no other teardown is emitted. |
| `00b52e10` | PetDecisionPacket::ScalarDeletingDestructor | likely | Scalar deleting destructor: resets the vtable pointer at *this to BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set; returns … |

**PetDeletePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c06510` | PetDeletePacket_DtorBody | confirmed | Destructor body for PetDeletePacket: rewrites the vtable to PetDeletePacket::vftable, then to BasePacket::vftable, and returns; reached from one MSVC exception… |
| `00c06560` | PetDeletePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor, slot 0 of PetDeletePacket::vftable: rewrites the vtable to PetDeletePacket::vftable then BasePacket::vftable and calls _free on the… |
| `00c06b40` | — 🌐 | unknown | Pet-family packet constructor for subtype 17: calls the pet base constructor FUN_00b50670(0x11) (base opcode 0x35, subtype 0x11), installs PetDeletePacket::vft… |

**PetEquipByItemRecordPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b50fa0` | PetEquipByItemRecordPacket::~PetEquipByItemRecordPacket | confirmed | Destructor: stores PetEquipByItemRecordPacket::vftable, resets the embedded ItemRecord at +0xc to ItemRecord::vftable, then stores BasePacket::vftable; one unw… |
| `00b50f30` | PetEquipByItemRecordPacket::PetEquipByItemRecordPacket 🌐 | likely | Constructor: calls the pet base constructor 00b50670 with subtype 0x23 (35), stores PetEquipByItemRecordPacket::vftable, then initialises an embedded ItemRecor… |
| `00b51000` | PetEquipByItemRecordPacket::ScalarDeletingDestructor | likely | Scalar deleting destructor: calls the class destructor 00b50fa0 (embedded ItemRecord reset plus vtable resets) and then calls _free(param_1) when bit 0 of the … |

**PetEquipPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b50e00` | PetEquipPacket::PetEquipPacket 🌐 | confirmed | Constructor: calls the pet base constructor 00b50670 with subtype 0xb (11), stores PetEquipPacket::vftable, zeroes the byte at +0xc, constructs an embedded mem… |
| `00b50e70` | PetEquipPacket::~PetEquipPacket | confirmed | Destructor: stores PetEquipPacket::vftable then BasePacket::vftable at *this; five unwind funclets and the senders 00b5ce60/00c07990 call it. |
| `00b50ec0` | PetEquipPacket::ScalarDeletingDestructor | confirmed | Scalar deleting destructor: resets the vtable pointer at *this to PetEquipPacket::vftable then BasePacket::vftable and calls _free(param_1) when bit 0 of the f… |

**PetExpireTimePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b53090` | PetExpireTimePacket::PetExpireTimePacket 🌐 | confirmed | Constructor: calls the pet base constructor 00b50670 with subtype 0x17 (23), then stores PetExpireTimePacket::vftable at *this. |
| `00b51970` | PetExpireTimePacket::~PetExpireTimePacket | likely | Destructor: resets the vtable pointer at *this to BasePacket::vftable and returns; no members are released. |
| `00b530f0` | PetExpireTimePacket::ScalarDeletingDestructor | likely | Scalar deleting destructor: resets the vtable pointer at *this to BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set; returns … |

**PetInWaterPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b51bd0` | PetInWaterPacket::PetInWaterPacket 🌐 | confirmed | Constructor: calls the pet base constructor 00b50670 with subtype 0x1c (28), then stores PetInWaterPacket::vftable at *this. |
| `00b53270` | PetInWaterPacket::ScalarDeletingDestructor | likely | Scalar deleting destructor: resets the vtable pointer at *this to BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set; returns … |

**PetListPacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b52f90` | PetListPacket::PetListPacket 🌐 | confirmed | Constructor: calls the pet base constructor 00b50670 with subtype 5, stores PetListPacket::vftable, zeroes the dwords at +0xc through +0x28, writes the sentine… |

**PetPacketOfferUpsell** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b51df0` | PetPacketOfferUpsell::PetPacketOfferUpsell 🌐 | likely | Constructor: calls the pet base constructor 00b50670 with subtype 0x24 (36), then stores PetPacketOfferUpsell::vftable at *this. |
| `00b51e50` | PetPacketOfferUpsell::~PetPacketOfferUpsell | likely | Destructor: resets the vtable pointer at *this to BasePacket::vftable and returns. |
| `00b53390` | PetPacketOfferUpsell::ScalarDeletingDestructor | likely | Scalar deleting destructor: resets the vtable pointer at *this to BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set; returns … |

**PetPerformTrickPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b50a70` | PetPerformTrickPacket::~PetPerformTrickPacket | confirmed | Destructor: stores PetPerformTrickPacket::vftable then BasePacket::vftable at *this, the full two-stage MSVC vtable reset; one unwind funclet calls it. |
| `00b50ac0` | PetPerformTrickPacket::ScalarDeletingDestructor | confirmed | Scalar deleting destructor: resets the vtable pointer at *this to PetPerformTrickPacket::vftable then BasePacket::vftable and calls _free(param_1) when bit 0 o… |
| `00b50a10` | — 🌐 | unknown | Constructor: calls the pet base constructor 00b50670 with subtype 7, then stores PetPerformTrickPacket::vftable at *this. |

**PetPlayWithToyPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b50ba0` | PetPlayWithToyPacket::~PetPlayWithToyPacket | confirmed | Destructor: stores PetPlayWithToyPacket::vftable, tears down the embedded Target sub-object through 0101b370, then stores BasePacket::vftable. Three unwind fun… |
| `00b50c00` | PetPlayWithToyPacket::ScalarDeletingDestructor | likely | Scalar deleting destructor: calls the class destructor 00b50ba0 (Target teardown plus vtable resets) and then calls _free(param_1) when bit 0 of the flag argum… |
| `00b50b30` | — 🌐 | unknown | Constructor: calls the pet base constructor 00b50670 with subtype 0x21 (33), stores PetPlayWithToyPacket::vftable, then initialises an embedded Target sub-obje… |

**PetPreviewRequestPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b51ea0` | PetPreviewRequestPacket::PetPreviewRequestPacket 🌐 | likely | Constructor: calls the pet base constructor 00b50670 with subtype 0x26 (38), stores PetPreviewRequestPacket::vftable and zeroes the four dwords at +0xc through… |
| `00b51f10` | PetPreviewRequestPacket::~PetPreviewRequestPacket | likely | Destructor: resets the vtable pointer at *this to BasePacket::vftable and returns; the pet dispatcher 00b5ce60 and two unwind funclets call it. |
| `00b533f0` | PetPreviewRequestPacket::ScalarDeletingDestructor | likely | Scalar deleting destructor: resets the vtable pointer at *this to BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set; returns … |

**PetRequestNamePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b543b0` | PetRequestNamePacket::PetRequestNamePacket 🌐 | confirmed | Constructor: calls the pet base constructor 00b50670 with subtype 6, stores PetRequestNamePacket::vftable and initialises an embedded IString at +0xc (vtable p… |
| `00b54420` | PetRequestNamePacket::~PetRequestNamePacket | confirmed | Destructor: stores PetRequestNamePacket::vftable, rewrites the embedded IString vtable at +0xc and, when the string owns a buffer (+0x18 positive), atomically … |
| `00b544b0` | PetRequestNamePacket::ScalarDeletingDestructor | likely | Scalar deleting destructor: calls the class destructor 00b54420 (IString release plus vtable resets) and then calls _free(param_1) when bit 0 of the flag argum… |

**PetResponsePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c06ae0` | PetResponsePacket::ScalarDeletingDtor | confirmed | Scalar deleting destructor, slot 0 of PetResponsePacket::vftable: rewrites the object's vtable to BasePacket::vftable and calls _free when bit 0 of the flag by… |
| `00c06460` | — 🌐 | unknown | Pet-family packet constructor for subtype 2: calls the shared pet base-packet constructor FUN_00b50670(2) (which writes base opcode 0x35 and the subtype) and i… |

**PetSetNamePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b54540` | PetSetNamePacket::~PetSetNamePacket | confirmed | Destructor: stores PetSetNamePacket::vftable, rewrites the embedded IString vtable at +0x18 and, when the string owns a buffer (+0x24 positive), atomically dec… |
| `00b545d0` | PetSetNamePacket::ScalarDeletingDestructor | likely | Scalar deleting destructor: calls the class destructor 00b54540 (IString release plus vtable resets) and then calls _free(param_1) when bit 0 of the flag argum… |
| `00b544d0` | — 🌐 | unknown | Constructor: calls the pet base constructor 00b50670 with subtype 10, stores PetSetNamePacket::vftable and initialises an embedded IString at +0x18 (vtable plu… |

**PetStatusPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b507e0` | PetStatusPacket::PetStatusPacket 🌐 | confirmed | Constructor: calls the pet base constructor 00b50670 with subtype 3, then stores PetStatusPacket::vftable at *this. |
| `00b50840` | PetStatusPacket::~PetStatusPacket | likely | Destructor: resets the vtable pointer at *this to BasePacket::vftable and returns. |
| `00b52e70` | PetStatusPacket::ScalarDeletingDestructor | likely | Scalar deleting destructor: resets the vtable pointer at *this to BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set; returns … |

**PetSummonByItemIdPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b50940` | PetSummonByItemIdPacket::PetSummonByItemIdPacket 🌐 | confirmed | Constructor: calls the pet base constructor 00b50670 with subtype 0x20 (32), then stores PetSummonByItemIdPacket::vftable at *this. |
| `00b509a0` | PetSummonByItemIdPacket::~PetSummonByItemIdPacket | likely | Destructor: resets the vtable pointer at *this to BasePacket::vftable and returns. |
| `00b52f30` | PetSummonByItemIdPacket::ScalarDeletingDestructor | likely | Scalar deleting destructor: resets the vtable pointer at *this to BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set; returns … |

**PetSummonRecallPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b50890` | PetSummonRecallPacket::PetSummonRecallPacket 🌐 | confirmed | Constructor: calls the pet base constructor 00b50670 with subtype 4, then stores PetSummonRecallPacket::vftable at *this. |
| `00b508f0` | PetSummonRecallPacket::~PetSummonRecallPacket | likely | Destructor: resets the vtable pointer at *this to BasePacket::vftable and returns. |
| `00b52ed0` | PetSummonRecallPacket::ScalarDeletingDestructor | likely | Scalar deleting destructor: resets the vtable pointer at *this to BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set; returns … |

**PetTactileNotify** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ccdb70` | PetTactileNotify::Ctor | confirmed | Initializes the pet packet base with subtype 22, installs PetTactileNotify vtable, and zeros four payload fields. |
| `00ccdf20` | PetTactileNotify::ScalarDeletingDtor | confirmed | Restores the BasePacket vtable and frees the packet allocation when the deleting flag is set. |

**PetTempLockoutError** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b51c80` | PetTempLockoutError::PetTempLockoutError 🌐 | likely | Constructor: calls the pet base constructor 00b50670 with subtype 0x1d (29), then stores PetTempLockoutError::vftable at *this. |
| `00b51ce0` | PetTempLockoutError::~PetTempLockoutError | likely | Destructor: resets the vtable pointer at *this to BasePacket::vftable and returns. |
| `00b532d0` | PetTempLockoutError::ScalarDeletingDestructor | likely | Scalar deleting destructor: resets the vtable pointer at *this to BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set; returns … |

**PetTileTargetHandler** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b54780` | PetTileTargetHandler::PetTileTargetHandler | confirmed | Constructor: calls the base constructor 008b3590 (TargetProcessorTileEventHandler, which installs the TargetProcessorEventHandler vtable), then stores PetTileT… |
| `00b54830` | PetTileTargetHandler::ScalarDeletingDestructor | confirmed | Scalar deleting destructor: resets the vtable pointer at *this to TargetProcessorEventHandler::vftable (the inlined base destructor) and calls _free(param_1) w… |
| `00b52070` | MatchKind1Or0x1000 | likely | Two-integer predicate that computes nothing else: returns 1 exactly when param_1 equals 1 and param_2 equals 0, or when param_1 equals 0x1000 regardless of par… |

**PetTrickAddPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b51150` | PetTrickAddPacket::PetTrickAddPacket 🌐 | confirmed | Constructor: calls the pet base constructor 00b50670 with subtype 0xd (13), stores PetTrickAddPacket::vftable, then zeroes nine dwords at +0xc through +0x2c an… |
| `00b511d0` | PetTrickAddPacket::~PetTrickAddPacket | confirmed | Destructor: stores PetTrickAddPacket::vftable then BasePacket::vftable at *this; the pet dispatcher 00b5ce60 and one unwind funclet call it. |
| `00b51220` | PetTrickAddPacket::ScalarDeletingDestructor | confirmed | Scalar deleting destructor: resets the vtable pointer at *this to PetTrickAddPacket::vftable then BasePacket::vftable and calls _free(param_1) when bit 0 of th… |

**PetTrickFavoriteSetPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b51290` | PetTrickFavoriteSetPacket::PetTrickFavoriteSetPacket 🌐 | confirmed | Constructor: calls the pet base constructor 00b50670 with subtype 0xe (14), stores PetTrickFavoriteSetPacket::vftable and zeroes the three dwords at +0xc/+0x10… |
| `00b512f0` | PetTrickFavoriteSetPacket::~PetTrickFavoriteSetPacket | confirmed | Destructor: stores PetTrickFavoriteSetPacket::vftable then BasePacket::vftable at *this; the pet dispatcher 00b5ce60, the sender 00b5f670 and two unwind funcle… |
| `00b51340` | PetTrickFavoriteSetPacket::ScalarDeletingDestructor | confirmed | Scalar deleting destructor: resets the vtable pointer at *this to PetTrickFavoriteSetPacket::vftable then BasePacket::vftable and calls _free(param_1) when bit… |

**PetTrickGestureResult** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b50c20` | PetTrickGestureResult::PetTrickGestureResult 🌐 | confirmed | Constructor: calls the pet base constructor 00b50670 with subtype 8, then stores PetTrickGestureResult::vftable at *this. |
| `00b50c80` | PetTrickGestureResult::~PetTrickGestureResult | confirmed | Destructor: stores PetTrickGestureResult::vftable then BasePacket::vftable at *this; one unwind funclet calls it. |
| `00b50cd0` | PetTrickGestureResult::ScalarDeletingDestructor | confirmed | Scalar deleting destructor: resets the vtable pointer at *this to PetTrickGestureResult::vftable then BasePacket::vftable and calls _free(param_1) when bit 0 o… |

**PetTrickNotifyPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b51420` | PetTrickNotifyPacket::~PetTrickNotifyPacket | confirmed | Destructor: stores PetTrickNotifyPacket::vftable then BasePacket::vftable at *this; the pet dispatcher 00b5ce60 and one unwind funclet call it. |
| `00b51470` | PetTrickNotifyPacket::ScalarDeletingDestructor | confirmed | Scalar deleting destructor: resets the vtable pointer at *this to PetTrickNotifyPacket::vftable then BasePacket::vftable and calls _free(param_1) when bit 0 of… |
| `00b513b0` | PetTrickNotifyPacket::PetTrickNotifyPacket 🌐 | likely | Constructor: calls the pet base constructor 00b50670 with subtype 0xf (15), stores PetTrickNotifyPacket::vftable and zeroes the five dwords at +0xc through +0x… |

**PetTrickSkillLevelPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b51020` | PetTrickSkillLevelPacket::PetTrickSkillLevelPacket 🌐 | confirmed | Constructor: calls the pet base constructor 00b50670 with subtype 0xc (12), stores PetTrickSkillLevelPacket::vftable and zeroes three dwords at +0xc/+0x10/+0x1… |
| `00b51090` | PetTrickSkillLevelPacket::~PetTrickSkillLevelPacket | confirmed | Destructor: stores PetTrickSkillLevelPacket::vftable then BasePacket::vftable at *this; the pet dispatcher 00b5ce60 and one unwind funclet call it. |
| `00b510e0` | PetTrickSkillLevelPacket::ScalarDeletingDestructor | confirmed | Scalar deleting destructor: resets the vtable pointer at *this to PetTrickSkillLevelPacket::vftable then BasePacket::vftable and calls _free(param_1) when bit … |

**PetUiModePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b53150` | PetUiModePacket::ScalarDeletingDestructor | likely | Scalar deleting destructor: resets the vtable pointer at *this to BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set; returns … |
| `00b519c0` | — 🌐 | unknown | Constructor: calls the pet base constructor 00b50670 with subtype 0x18 (24), then stores PetUiModePacket::vftable at *this. |

**PetUpdateUtilityItemChangePacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b545f0` | PetUpdateUtilityItemChangePacket::PetUpdateUtilityItemChang… 🌐 | confirmed | Constructor: calls the pet base constructor 00b50670 with subtype 0x15 (21), stores PetUpdateUtilityItemChangePacket::vftable, zeroes the dwords at +0x10/+0x14… |
| `00b546a0` | PetUpdateUtilityItemChangePacket::~PetUpdateUtilityItemChan… | confirmed | Destructor: releases the three embedded IStrings in the order +0x40, +0x30, +0x20 (each: rewrite the IString vtable, and when the string owns a buffer atomical… |
| `00b54de0` | PetUpdateUtilityItemChangePacket::ScalarDeletingDestructor | likely | Scalar deleting destructor: calls the class destructor 00b546a0 (three IString releases plus base vtable reset) and then calls _free(param_1) when bit 0 of the… |

**PetUtilityFeedPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b51730` | PetUtilityFeedPacket::PetUtilityFeedPacket 🌐 | confirmed | Constructor: calls the pet base constructor 00b50670 with subtype 0x13 (19), stores PetUtilityFeedPacket::vftable and zeroes the dwords at +0xc/+0x10. |
| `00b51790` | PetUtilityFeedPacket::~PetUtilityFeedPacket | confirmed | Destructor: stores PetUtilityFeedPacket::vftable then BasePacket::vftable at *this; the feed sender 00b5f720 and one unwind funclet call it. |
| `00b517e0` | PetUtilityFeedPacket::ScalarDeletingDestructor | confirmed | Scalar deleting destructor: resets the vtable pointer at *this to PetUtilityFeedPacket::vftable then BasePacket::vftable and calls _free(param_1) when bit 0 of… |

**PetUtilityGroomPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b51610` | PetUtilityGroomPacket::PetUtilityGroomPacket 🌐 | confirmed | Constructor: calls the pet base constructor 00b50670 with subtype 0x12 (18), stores PetUtilityGroomPacket::vftable and zeroes the dwords at +0xc/+0x10. |
| `00b51670` | PetUtilityGroomPacket::~PetUtilityGroomPacket | confirmed | Destructor: stores PetUtilityGroomPacket::vftable then BasePacket::vftable at *this; the groom sender 00b5f8c0 and one unwind funclet call it. |
| `00b516c0` | PetUtilityGroomPacket::ScalarDeletingDestructor | confirmed | Scalar deleting destructor: resets the vtable pointer at *this to PetUtilityGroomPacket::vftable then BasePacket::vftable and calls _free(param_1) when bit 0 o… |

**PetUtilityNotifyPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b518b0` | PetUtilityNotifyPacket::~PetUtilityNotifyPacket | confirmed | Destructor: stores PetUtilityNotifyPacket::vftable then BasePacket::vftable at *this; the pet dispatcher 00b5ce60 and one unwind funclet call it. |
| `00b51900` | PetUtilityNotifyPacket::ScalarDeletingDestructor | confirmed | Scalar deleting destructor: resets the vtable pointer at *this to PetUtilityNotifyPacket::vftable then BasePacket::vftable and calls _free(param_1) when bit 0 … |
| `00b51850` | PetUtilityNotifyPacket::PetUtilityNotifyPacket 🌐 | likely | Constructor: calls the pet base constructor 00b50670 with subtype 0x14 (20), stores PetUtilityNotifyPacket::vftable and zeroes the dwords at +0xc/+0x10. |

**PetUtilityPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b514e0` | PetUtilityPacket::PetUtilityPacket 🌐 | confirmed | Constructor: calls the pet base constructor 00b50670 with subtype 0x10 (16), stores PetUtilityPacket::vftable, zeroes the byte at +0xc and the dwords at +0x10/… |
| `00b51550` | PetUtilityPacket::~PetUtilityPacket | confirmed | Destructor: stores PetUtilityPacket::vftable then BasePacket::vftable at *this; three unwind funclets and the senders 00b5ce60/00c077a0 call it. |
| `00b515a0` | PetUtilityPacket::ScalarDeletingDestructor | confirmed | Scalar deleting destructor: resets the vtable pointer at *this to PetUtilityPacket::vftable then BasePacket::vftable and calls _free(param_1) when bit 0 of the… |

**PetWorldToClientBuyResponse** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b51d30` | PetWorldToClientBuyResponse::PetWorldToClientBuyResponse 🌐 | confirmed | Constructor: calls the pet base constructor 00b50670 with subtype 0x1f (31), stores PetWorldToClientBuyResponse::vftable, zeroes the bytes at +0xc and +0xd and… |
| `00b51da0` | PetWorldToClientBuyResponse::~PetWorldToClientBuyResponse | likely | Destructor: resets the vtable pointer at *this to BasePacket::vftable and returns. |
| `00b53330` | PetWorldToClientBuyResponse::ScalarDeletingDestructor | likely | Scalar deleting destructor: resets the vtable pointer at *this to BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set; returns … |

**PetsHandler** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b5f670` | SendPetFavoriteTrick 🌐 | confirmed | Accepts slots one through four, constructs PetTrickFavoriteSetPacket, converts the slot to zero-based form, writes a generated field and the trick id, and invo… |
| `00b5f8c0` | SendPetUtilityGroom 🌐 | confirmed | When the pet is not busy and actor flag bit 0x20 is clear, constructs PetUtilityGroomPacket and conditionally invokes the wrapper when transport at +0x280 exis… |
| `00b5ffe0` | PetsHandler::SendTrickGestureResult | confirmed | Constructs a PetTrickGestureResult packet through the pet base constructor FUN_00b50c20 (pet family sub-type 8), stores the caller's 32-bit value as its single… |
| `00b60080` | PetsHandler::SendPetSummon | confirmed | Calls the guard FUN_00b549f0 first and only proceeds when it returns non-zero; then constructs a PetSummonRecallPacket with FUN_00b50890, writes the pet id fro… |
| `00b60110` | PetsHandler::SendPetSummonByItemId | confirmed | Constructs a PetSummonByItemIdPacket with FUN_00b50940 (pet family sub-type 0x20) and writes both parameters into the payload, pet id from param_2 and item id … |
| `00b601a0` | PetsHandler::SendPetRecall | confirmed | Constructs a PetSummonRecallPacket with FUN_00b50890, writes the pet id from param_2 and the action byte 0 (recall) into the payload, and submits it through FU… |
| `00b60220` | PetsHandler::SendPetEquipVariant0 | likely | Constructs a PetEquipPacket with FUN_00b50e00 (pet family sub-type 0xb), copies the first dword of the record pointed to by param_2 into the payload, writes th… |
| `00b607e0` | PetsHandler::SendPetEquipByItemRecord | likely | Constructs a PetEquipByItemRecordPacket with FUN_00b50f30 (pet family sub-type 0x23), copies the three dwords at param_2+4, +8 and +12 into the payload, append… |
| `00b602c0` | PetsHandler::SendPetEquipVariant1 | speculative | Constructs a PetEquipPacket with FUN_00b50e00 (pet family sub-type 0xb), writes the byte 1 into the payload and the dword param_2 after it, then submits the pa… |
| `00b5f720` | — 🌐 | unknown | When the pet is not busy and actor flag bit 0x20 is clear, constructs PetUtilityFeedPacket, conditionally invokes the wrapper when transport at +0x280 exists, … |
| `00b60350` | — | unknown | Sets its in-water byte at +0x4c0 to 1, then (when FUN_00cca410 reports the pet/handler is usable) walks the linked list reached through *(*handler+500)+0x1828 … |
| `00b60420` | — | unknown | Clears its in-water byte at +0x4c0 to 0, constructs a PetInWaterPacket with FUN_00b51bd0 and submits it through FUN_00b5c8f0 with the flag byte read back from … |

**(no class)** (45)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b5f500` | SendPetNameRequest 🌐 | confirmed | Finds the active pet record in the hash table, constructs PetRequestNamePacket, assigns param_2 to its string payload, and invokes the tunneled-client wrapper … |
| `00b5f5d0` | SendPetUiMode 🌐 | confirmed | Constructs PetUiModePacket, maps param_2 to a mode value with special handling for zero and five, then sends the serialized packet when the handler has a trans… |
| `00b5fcf0` | SendPetFollowLocation 🌐 | confirmed | Stores the new four-float location, checks displacement and path/ground conditions, updates cached coordinates, and on qualifying routes constructs PetCanGoLoc… |
| `00b5ff50` | SendPetPerformTrick 🌐 | confirmed | Constructs PetPerformTrickPacket, stores param_2 as its single dword trick identifier, and submits it through the tunneled-client packet wrapper when the handl… |
| `00b54330` | InitStringPairRecord | likely | Initialises a record holding two reference-counted IString objects: writes SoeUtil::IString<char>::vftable at +0 and +0x10 with the shared empty-string data po… |
| `00b54890` | DestroyStringPairRecord | likely | Destructor of the two-IString record initialised by 00b54330: releases the second IString at +0x10 first (rewrite its vtable, and when the string owns a buffer… |
| `00b54cd0` | ReadFiveDwordsChecked | likely | Packet-field reader for a five-dword record: for each dword it checks the cursor at +8 against the end pointer at +0xc, and on overrun writes 0, sets the error… |
| `00b54e00` | ScalarDeleteStringPairRecord | likely | Scalar deleting destructor of the 36-byte two-IString name record: calls the record destructor 00b54890 (release of both embedded strings) and then calls _free… |
| `00b55100` | CopyStringPairRecord | likely | Copy constructor of the 36-byte two-IString record: writes the IString vtable and empty-string data pointer at +0/+4 and again at +0x10/+0x14, zeroes the curso… |
| `00b55280` | ReadShortThenByte | likely | Packet-field reader used by 23 call sites: reads a 16-bit value into +4 and an 8-bit value into +8 from the stream (cursor at +8, end pointer at +0xc), sign-ex… |
| `00b552e0` | ReadShortByteAnd3Dwords | likely | Packet-field reader: calls 00b55280 for the leading short and byte and then reads three dwords into +0xc/+0x10/+0x14, each guarded by the cursor/end comparison… |
| `00b55370` | ReadShortBytePairAnd7DwordsAndFlag | likely | Packet-field reader: calls 00b55280, then reads an 8-byte pair into +0x10/+0x14, seven dwords into +0x18 through +0x30 and finally a bool byte into +0x34, ever… |
| `00b55510` | ReadShortByteAnd3Dwords | likely | Packet-field reader: calls 00b55280 for the leading short and byte and then reads three dwords into +0xc/+0x10/+0x14 with the same cursor/end guard and error f… |
| `00b556c0` | ReadShortByteDwordAndPair | likely | Packet-field reader: calls 00b55280, reads one dword into +0xc and then an 8-byte pair into +0x10/+0x14, each read guarded by the cursor/end comparison and set… |
| `00b558a0` | ReadShortByte9DwordsAndFlag | likely | Packet-field reader: calls 00b55280, reads nine dwords into +0xc through +0x2c and finally a bool byte into +0x30, each read guarded by the cursor/end comparis… |
| `00b55a20` | ReadShortByteAnd3Dwords | likely | Packet-field reader: calls 00b55280 for the leading short and byte and then reads three dwords into +0xc/+0x10/+0x14 with the same cursor/end guard and error f… |
| `00b55b80` | ReadShortByteFlagAnd3Dwords | likely | Packet-field reader: calls 00b55280, reads a bool byte into +0xc and then three dwords into +0x10/+0x14/+0x18, each read guarded by the cursor/end comparison a… |
| `00b55c30` | ReadShortByteFlagAnd2Dwords | likely | Packet-field reader: calls 00b55280, reads a bool byte into +0xc, then a dword into +0x14 before the dword at +0x10 (the two fields are read out of offset orde… |
| `00b55cc0` | ReadShortByteAnd6Dwords | likely | Packet-field reader: calls 00b55280 and then reads six dwords into +0xc through +0x20 with the cursor/end guard and stream error flag. |
| `00b55db0` | ReadShortBytePair2Dwords3Skips | likely | Packet-field reader: calls 00b55280, reads an 8-byte pair into +0x10/+0x14 and dwords into +0x18/+0x1c, then performs three consecutive length-prefixed skips t… |
| `00b55f50` | ReadShortByteDwordThenNested | likely | Packet-field reader: calls 00b55280, reads one dword into +0xc and then delegates to the nested object stored at +0x10 by calling its virtual at slot +4 with t… |
| `00b55fc0` | ReadShortByte4DwordsFloatCheck | likely | Packet-field reader: calls 00b55280 and then reads four dwords into +0xc/+0x10/+0x14/+0x18 with the cursor/end guard and error flag; only the last dword carrie… |
| `00c064c0` | ResetVtableToBasePacket | likely | Rewrites the object's vtable pointer to BasePacket::vftable and returns; reached from two MSVC exception-unwind funclets (015f3f90, 015f3fd0) of the PetRespons… |
| `00c06e00` | PetResponsePacket_SerializeBody | likely | Packet body writer used by the response send template: calls FUN_00b595d0(buffer) for the common preamble, then appends the packet's 32-bit field at +0xc and t… |
| `00b52010` | Wrapper00cc8ff0 | speculative | Tail-call thunk: loads the dword at +8 of the incoming this into ECX (mov ecx,[ecx+8]) and jumps to 00cc8ff0; the whole body is 8 bytes (8b 49 08 e9 d8 6f 17 0… |
| `00b52030` | Wrapper00cca410 | speculative | Tail-call thunk: loads the dword at +8 of the incoming this into ECX (mov ecx,[ecx+8]) and jumps to 00cca410; the body is 8 bytes (8b 49 08 e9 d8 83 17 00). Th… |
| `00b52100` | ZeroInit80ByteRecord | speculative | Initialises an 80-byte record: zeroes the dwords at +4/+8/+0xc, memsets the 64 bytes at +0x10 to zero through 00d836a0 (_memset with size 0x40) and zeroes the … |
| `00b555a0` | ReadShortByteAnd4Floats | speculative | Packet-field reader: calls 00b55280, then reads one unchecked dword into +0xc and four dwords into +0x10 through +0x1c, each of the four carrying a NaN test th… |
| `00b557f0` | ReadShortByte2DwordsAndFlag | speculative | Packet-field reader: calls 00b55280, reads dwords into +0xc/+0x10/+0x14 and finally a bool byte into +0x18, each read guarded by the cursor/end comparison and … |
| `00b55ab0` | ReadShortByteAnd5Dwords | speculative | Packet-field reader: calls 00b55280 and then reads five dwords into +0xc through +0x1c with the cursor/end guard and error flag; unlike 00b555a0 none of the fi… |
| `00b5fbc0` | CompletePetPlayWithToyTargetSelection 🌐 | speculative | If the toy action is pending and the handler has a live actor, removes the old target entry, clears the pending flag, restores actor state, stores the supplied… |
| `00b51a20` | — | unknown | Vtable-reset body of a packet cleanup routine: installs an MSVC SEH frame, stores BasePacket::vftable at *this and returns; the raw bytes hold only the frame p… |
| `00b51c30` | — | unknown | Vtable-reset body of a packet cleanup routine: installs an MSVC SEH frame, stores BasePacket::vftable at *this and returns; the bytes are identical to 00b51a20… |
| `00b52040` | — | unknown | Predicate: returns 1 when the argument pointer is non-null and 00b87e00(*(arg+0x28)) returns non-zero, otherwise returns 0. It reads no other field and writes … |
| `00b547e0` | — | unknown | Vtable-reset body: installs an MSVC SEH frame, stores TargetProcessorEventHandler::vftable at *this and returns. No callers are recorded and the body is indist… |
| `00b54930` | — | unknown | Looks up the handler's proxied character by its id (bucket chain at +0x74 + (id&3)*4, nodes linked at +0x120 and keyed at +0x124) and then scans that character… |
| `00b55730` | — | unknown | Packet-field reader: calls 00b55280, reads an 8-byte pair into +0x10/+0x14, then a 4-byte length whose bytes are skipped with 00894b10 (setting the stream erro… |
| `00b5f2e0` | — 🌐 | unknown | Locates a pet record by its key, rejects invalid name or state indices and a missing handler, constructs PetRequestNamePacket, and when a controller exists for… |
| `00b5fa50` | — 🌐 | unknown | Finds the active pet record, resolves its target identifier, and constructs PetPlayWithToyPacket when both lookups succeed; calls the tunneled-client wrapper o… |
| `00c06ea0` | — | unknown | Packet body writer: calls FUN_00b595d0(buffer), appends the byte at +0xc, then the three 32-bit fields at +0x10, +0x14 and +0x18 through the bounded copy FUN_0… |
| `00c06fc0` | — | unknown | Packet body writer: calls FUN_00b595d0(buffer), appends the 32-bit field at +0xc through the bounded copy FUN_007cc400, then delegates the field beginning at +… |
| `00c07090` | — | unknown | ClientGateway send template for the pet response packet: reserves the 64KB stack scratch (__alloca_probe), serialises the source packet through FUN_00c06e00 in… |
| `00c071a0` | — | unknown | ClientGateway send template for the pet utility packet: serialises the source packet through FUN_00c06ea0 into a local buffer, finalises it, constructs ClientG… |
| `00c072b0` | — | unknown | ClientGateway send template for the pet delete packet: serialises the source packet through FUN_00c06fc0 into a local buffer, finalises it, constructs ClientGa… |
| `00ccea80` | — | unknown | Writes the pet object handle and four additional values into a bounded writer, advancing the output cursor after each field. |

### `ui.pets`

**BasePetTrickDataSource** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ccacc0` | BasePetTrickDataSource::GetColumnName | confirmed | Maps indices zero through twenty to item guid, icon, name, description, quantity, equip state, model, abilities, power rating and effect labels; other indices … |

**PetActionDataSource** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cca690` | PetActionDataSource::GetColumnName | confirmed | Returns ID, Target, or Behavior for indices zero, one, or two; other indices return <error>. |
| `00ccd3d0` | PetActionDataSource::Dtor | confirmed | Restores the PetActionDataSource vtables at the primary and embedded table offsets, then destroys the embedded GUI table holder. |
| `00ccd800` | PetActionDataSource::Ctor | confirmed | Initializes the embedded GUI data table, installs three PetActionDataSource vtables, stores two constructor arguments, and clears its state byte. |
| `00ccd890` | PetActionDataSource::ScalarDeletingDtor | confirmed | Calls the pet action data source destructor, then frees the allocation when the deleting flag is set. |
| `00ccb2c0` | PetActionDataSource::RefreshAllRows | speculative | Refreshes backing source, walks every row, invokes update callback with each row id, and marks the source dirty. |
| `00ccb980` | — | unknown | If source dirty flag is clear, refreshes all action rows; returns row count at source record +0xc. |
| `00ccc940` | — | unknown | Resolves the requested row and current pet, returns selected pet id or localized action text, and reports missing row, pet, or string values. |

**PetActiveDataSource** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ccb210` | PetActiveDataSource::RefreshActivePetRows | confirmed | Refreshes source, finds active pet by id, walks display rows, and invokes row update callback for rows with positive count and matching pet id. |
| `00ccd6d0` | PetActiveDataSource::Ctor | confirmed | Constructs the PetListDataSource base, replaces its vtables with PetActiveDataSource vtables, and returns the initialized object. |
| `00ccd7a0` | PetActiveDataSource::ScalarDeletingDtor | confirmed | Destroys the PetListDataSource base state and frees the allocation when the deleting flag is set. |

**PetEquipScreenDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cca880` | PetEquipScreenDataSourceCtor | confirmed | Calls base setup with constructor arguments, installs class vtables at main and embedded offsets, and stores two supplied values. |
| `00cca910` | PetEquipScreenDataSourceDtor | confirmed | Restores class vtables at four embedded offsets and calls FUN_00cb1b10 to release base data-source state. |
| `00ccb200` | PetEquipScreenDataSource::DtorAdjustorThunk | confirmed | Adjusts ECX backward by 0x254 bytes, then jumps to 00ccb380; bytes decode as sub ecx,0x254 and relative jump. |
| `00ccb380` | PetEquipScreenDataSourceScalarDeletingDtor | confirmed | Calls PetEquipScreenDataSource destructor and frees object when deleting flag is set; bytes show a normal call wrapper. |
| `00ccb3a0` | PetEquipScreenDataSource::CanEquipItem | confirmed | Requires category three, checks optional id filter, resolves item definition, verifies pet/equipment context and item metadata, then returns final eligibility … |
| `00ccc480` | PetEquipScreenDataSource::HandleEquipSelection | confirmed | For selection value 12, searches the current pet's utility ids for the selected item id and chooses one of two UI values; otherwise delegates to the base handl… |

**PetEquippedUtilityDataSource** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ccabc0` | PetEquippedUtilityDataSourceCtor | confirmed | Initializes shared base, installs class vtables at main and embedded offsets, stores one value, and clears the other. |
| `00ccac60` | PetEquippedUtilityDataSourceDtor | confirmed | Restores class vtables at main and embedded offsets and invokes FUN_00cb1b10 to release base data-source state. |
| `00ccb1d0` | PetEquippedUtilityDataSource::DtorAdjustorThunk | confirmed | Adjusts ECX backward by 0x254 bytes, then jumps to 00ccb490; bytes decode as sub ecx,0x254 and relative jump. |
| `00ccb490` | PetEquippedUtilityDataSourceScalarDeletingDtor | confirmed | Calls equipped-utility data-source destructor, conditionally frees object, and returns it; bytes confirm call wrapper. |
| `00ccb4b0` | PetEquippedUtilityDataSource::MatchesSlot | confirmed | Requires category four, finds active pet, scans eight item slots from record +0x98, and returns one if a slot id equals this+0x10. |
| `00ccb9d0` | PetEquippedUtilityDataSource::RefreshRows | speculative | Refreshes backing source, scans inventory rows, selects entries whose type is one or two, invokes row callback for each, and marks source dirty. |
| `00ccc100` | — | unknown | Rejects indices above six, resolves the current pet record by key, reads the indexed utility id, resolves its item, and delegates the requested field lookup. |

**PetEquippedWearablesDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ccaab0` | PetEquippedWearablesDataSourceCtor | confirmed | Initializes shared data source, installs its vtables at four object offsets, stores passed value, and clears its second state field. |
| `00ccab50` | PetEquippedWearablesDataSourceDtor | confirmed | Restores class vtables at main and embedded offsets and releases base data-source state through FUN_00cb1b10. |
| `00ccb1f0` | PetEquippedWearablesDataSource::DtorAdjustorThunk | confirmed | Adjusts ECX backward by 0x254 bytes, then jumps to 00ccb470; bytes decode as sub ecx,0x254 and relative jump. |
| `00ccb470` | PetEquippedWearablesDataSourceScalarDeletingDtor | confirmed | Calls equipped-wearables data-source destructor, conditionally frees object, and returns it; bytes confirm call wrapper. |
| `00ccc560` | PetEquippedWearablesDataSource::FindByWearable | likely | Resolves the current pet, iterates its equipped utility item ids, resolves each item, matches its category to the requested value, and returns the selected res… |

**PetGestureDataSource** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ccadf0` | PetGestureDataSource::GetColumnName | confirmed | Maps indices zero through three to Trick, Order, X, and Y; other indices return <error>. |
| `00ccba50` | PetGestureDataSource::GetRowCount | confirmed | Refreshes gesture data when needed, finds selected gesture key in hash table, and returns its point count or zero if absent. |
| `00ccd5b0` | PetGestureDataSource::Dtor | confirmed | Restores PetGestureDataSource vtables at the primary and embedded table offsets, then destroys the embedded GUI table holder. |
| `00ccdac0` | PetGestureDataSource::Ctor | confirmed | Initializes the GUI table, installs three PetGestureDataSource vtables, stores constructor arguments, clears its state byte, and zeros another field. |
| `00ccdb50` | PetGestureDataSource::ScalarDeletingDtor | confirmed | Destroys the pet gesture data source and frees the allocation when the deleting flag is set. |
| `00ccb580` | — | unknown | Refreshes backing table, finds selected gesture record by key, calls row callback for each entry, and marks data source dirty. |
| `00ccd220` | — | unknown | Resolves the row and gesture bucket keyed by this+0x230, verifies a matching gesture entry, then returns the requested gesture fields. |

**PetListDataSource** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cca5c0` | PetListDataSource::GetColumnName | confirmed | Maps indices zero through thirteen to pet name, sex, icon, needs, mood, id, membership, appearance, and tutorial labels; other indices return <error>. |
| `00ccc200` | PetListDataSource::Refresh | confirmed | Counts pet records with a positive field, collects their ids, sorts the temporary list, updates bound UI rows, and sets the dirty flag when needed. |
| `00ccd350` | PetListDataSource::Dtor | confirmed | Restores PetListDataSource vtables at the primary and two embedded table offsets, then destroys the embedded GUI table holder. |
| `00ccd620` | PetListDataSource::Ctor | confirmed | Initializes the embedded GUI table, installs three PetListDataSource vtables, stores two constructor arguments, and clears the initialization flag. |
| `00ccd6b0` | PetListDataSource::ScalarDeletingDtor | confirmed | Calls the PetListDataSource destructor and frees the object when the deleting flag is set. |
| `00cca5a0` | — | unknown | If dirty byte +0x22c is clear, calls owner virtual method at offset +0x38, then calls FUN_010c8f30. |
| `00ccc680` | — | unknown | Resolves a row id and pet record, then switches over field indices to update the matching pet fields or localized string data. |
| `00ccd750` | — | unknown | Creates an exception-cleanup frame, invokes PetListDataSource destruction on a stack temporary, and restores the prior exception list. |

**PetNameDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cca820` | PetNameDataSource::GetColumnName | confirmed | Maps indices zero through three to String, Id, Part, and Selected; other indices return <error>. |
| `00ccc0e0` | PetNameDataSource::GetField | confirmed | When the readiness byte at this+0x22c is clear, calls the refresh helper, then returns the value stored at this+0x230. |
| `00ccd530` | PetNameDataSource::Dtor | confirmed | Restores PetNameDataSource vtables at primary and embedded table offsets, then destroys the embedded GUI table holder. |
| `00ccda10` | PetNameDataSource::Ctor | confirmed | Initializes the embedded GUI table, installs three PetNameDataSource vtables, stores constructor arguments, and clears the state byte. |
| `00ccdaa0` | PetNameDataSource::ScalarDeletingDtor | confirmed | Destroys the pet-name data source and frees the allocation when the deleting flag is set. |
| `00cccdb0` | — | unknown | Resolves the name row, returns a stored name or localized value, and formats a key from player name, active pet id, and pet-name id. |

**PetTrickDataSource** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cca6f0` | PetTrickDataSource::GetColumnName | confirmed | Maps indices zero through six to ID, icon, name, skill, busy, known, and tooltip labels; other indices return <error>. |
| `00ccc420` | PetTrickDataSource::GetCurrentPetField | confirmed | Refreshes when stale, resolves the current pet by its key, returns zero when absent, otherwise returns the field at pet offset 0x44. |
| `00ccca10` | PetTrickDataSource::GetField | confirmed | Resolves a trick row and current pet, looks up the matching trick record, then returns selected fields or localized string values for that trick. |
| `00ccd450` | PetTrickDataSource::Dtor | confirmed | Restores the PetTrickDataSource vtables at the primary and embedded table offsets, then destroys the embedded GUI table holder. |
| `00ccd8b0` | PetTrickDataSource::Ctor | confirmed | Initializes the embedded GUI table, installs three PetTrickDataSource vtables, stores two arguments, and clears the state byte. |
| `00ccd940` | PetTrickDataSource::ScalarDeletingDtor | confirmed | Destroys the pet trick data source and frees the allocation when the deleting flag is set. |
| `00ccc300` | — | unknown | Looks up the current pet by key, collects its trick records, applies the callback-based filter, updates bound rows, and marks the data source dirty. |

**PetTrickFavoriteDataSource** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cca7b0` | PetTrickFavoriteDataSource::GetColumnName | confirmed | Returns ID, Slot, Icon, Name, or SkillLevel for indices zero through four; other indices return <error>. |
| `00ccd4c0` | PetTrickFavoriteDataSource::Dtor | confirmed | Restores PetTrickFavoriteDataSource vtables at the primary and embedded table offsets, then destroys the embedded GUI table holder. |
| `00ccd960` | PetTrickFavoriteDataSource::Ctor | confirmed | Initializes the GUI table, installs three favorite-trick data source vtables, stores constructor arguments, and clears its state byte. |
| `00ccd9f0` | PetTrickFavoriteDataSource::ScalarDeletingDtor | confirmed | Destroys the favorite-trick data source and frees its allocation when the deleting flag is set. |
| `00ccb300` | — | unknown | Refreshes backing source, locates active pet record, calls row callback for each of four favorite entries, and marks source dirty. |
| `00ccb9b0` | — | unknown | Refreshes favorite data when dirty flag is clear, then returns fixed count four. |
| `00cccc10` | — | unknown | Resolves a table row and current-pet favorite-trick record, then returns the requested field or localized text for that favorite. |

**PetTricksDataSource** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ccb530` | PetTricksDataSource::GetActivePetField44 | confirmed | Finds active pet record by id and returns 32-bit field at record +0x44, or zero if no match. |
| `00ccd120` | PetTricksDataSource::GetField | confirmed | Resolves the current pet, walks its trick list to the requested index, and delegates row-field extraction to the shared trick-row helper. |

**PetTricksEquippedDataSource** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ccd1a0` | PetTricksEquippedDataSource::GetField | confirmed | Resolves the current pet, retrieves one of five equipped trick slots, and delegates field extraction when the slot resolves to a trick. |

**PetUtilityScreenDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cca970` | PetUtilityScreenDataSourceCtor | confirmed | Initializes shared base, installs class vtables at main and embedded offsets, stores constructor values, and clears an additional field. |
| `00ccaa10` | PetUtilityScreenDataSourceDtor | confirmed | Restores class vtables at embedded offsets and invokes FUN_00cb1b10 to release base data-source state. |
| `00ccb1e0` | PetUtilityScreenDataSource::DtorAdjustorThunk | confirmed | Adjusts ECX backward by 0x254 bytes, then jumps to 00ccb450; bytes decode as sub ecx,0x254 and relative jump. |
| `00ccb450` | PetUtilityScreenDataSourceScalarDeletingDtor | confirmed | Calls utility-screen data-source destructor, conditionally frees object, and returns it; bytes confirm standard scalar deleting wrapper. |
| `00ccaa70` | PetUtilityScreenDataSource::MatchesUtilityId | likely | Requires lookup category four, then compares another lookup value with this+0x2e0 and returns one only on equality. |

**PetsHandler** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b53450` | PetsHandler::PopulatePetInventory | confirmed | Refreshes the whole pet UI after an inventory change: calls 00cca760, 00cca810, 00cca680 twice and 00ccae50; refreshes the two sub-objects at +0x1368/+0x136c t… |
| `00b53530` | PetsHandler::PetCanDraw | confirmed | If the two guid halves passed in equal the handler's stored pair at +0x88/+0x8c, dispatches the Lua handler PetsHandler:PetCanDraw through 009797a0 with no arg… |
| `00b53560` | PetsHandler::ClearActivePetGuid | likely | If the guid pair at +0x12c0/+0x12c4 differs from the shared default pair DAT_01be9810/DAT_01be9814, calls 00a8e540 with the old pair and then stores the defaul… |
| `00b535b0` | PetsHandler::ResetPetPanel | likely | Resets the pet panel state: clears the active pet guid through 00b53560, calls the pet UI helper 00cca410, zeroes the byte at +0x1300 and calls 00ccdda0. |
| `00b535d0` | PetsHandler::SetPendingValue | likely | If the sub-object pointer at +0x132c is set, writes param_2 into that sub-object's field at +0x260 and calls the pet UI helper 00ccae50; otherwise returns with… |
| `00b53600` | PetsHandler::RefreshInventoryUi | likely | Calls 00cca760 and 00cca810 and then refreshes the two sub-objects at +0x1368/+0x136c through their vtable slot +0x10; returns nothing. The decompiler flags an… |
| `00b53710` | PetsHandler::SetProxiedActorFlag | likely | Walks the hash chain of proxied characters (bucket base +0x74 indexed by (id at +0x84) & 3, nodes linked at +0x120 and keyed at +0x124) and, when the matching … |
| `00b54080` | PetsHandler::SetActivePet | likely | When the guid pair passed in differs from the cached pair at +0x12c0/+0x12c4 it first clears the previous cache through 00b53560 (only when the cached pair is … |
| `00b53690` | PetsHandler::RefreshCachedPetEntity | speculative | If the tracked guid pair at +0x1308/+0x130c equals the shared default pair the function returns immediately. Otherwise it resolves the pair through 00984960 an… |
| `00b536e0` | PetsHandler::ResolveOwnActor | speculative | Resolves the entity for the handler's own guid pair at +0x88/+0x8c through 00984960 and discards the returned pointer. |
| `00b53640` | — | unknown | If the guid pair at +0x1308/+0x130c differs from the shared default pair DAT_01be9810/DAT_01be9814, calls 009855d0 with the old pair and stores the default pai… |

**(no class)** (18)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a782d0` | PetTooltip_BuildRows | confirmed | Method with this in ECX and a pet record argument: sets the row count to 7 through 00a76870 or 00a75d50, resets the state fields at +0x280 to +0x294, calls 008… |
| `00a784e0` | PetTrickTooltip_BuildRows | confirmed | Method with this in ECX and a trick id: sets the row count to 3 through 00a76870 or 00a75d50, resets the state at +0x280 to +0x294, calls 008356b0 with 0, incr… |
| `00b549f0` | PetsHandlerOnSummonFailed | likely | Summon-failure gate: reads the pet pointer at *(this+0xc8); when it is non-null and not the -0x440 sentinel it evaluates two failure conditions, the predicate … |
| `00c066f0` | Pet_TutorialEnd | likely | Lua 'TutorialEnd' handler: reads one or two optional numeric arguments (FUN_00e3a770 + FUN_00d83c90) only when the stack holds two, resolves the pet handler th… |
| `00c06a60` | Pet_SetGestureData | likely | Lua 'SetGestureData' handler: requires one argument, resolves the pet handler via FUN_008a6660 and calls FUN_00b535d0(value) with it; returns 0. |
| `00c06ac0` | RegisterPetModule | likely | Registration wrapper: pushes nup=0, the 24-entry table 0x01829f60 and the module-name pointer at [01b6a130] ('Pet'), calls the Lua module registrar FUN_00e39e4… |
| `00c07990` | Pet_EquipItem | likely | Lua 'EquipItem' handler: requires 2 arguments; reads the id number and when it is above 199 forwards the whole call to the utility path 00c077a0 (utilities use… |
| `0096c120` | ShowThoughtBubbleAndTrack | speculative | Thiscall(this=param_1, param_2, param_3, param_4 flag). When this+0x738 is nonzero and the object from 008a6660 has +0x88/+0x8c equal to this+0x78/+0x7c, it ca… |
| `00c066a0` | — | unknown | Lua 'Summon' handler: requires exactly one argument, checks the local player object at client+0x3e0 exists, reads the argument as an integer (FUN_00e3a770 + FU… |
| `00c06a00` | — | unknown | Lua 'ModeSwitch' handler: requires exactly one argument, resolves the pet handler (FUN_008a6660) and reads the argument as an integer (FUN_00e3a770 + FUN_00d83… |
| `00c07420` | — 🌐 | unknown | Lua 'Dismiss' handler: constructs a PetSummonRecallPacket (FUN_00b50890), sets the action byte to 0 (recall/dismiss), and when the gateway link is up (client+0… |
| `00c074c0` | — | unknown | Lua 'WasGood' handler: requires one argument, builds a PetResponsePacket (FUN_00c06460), stores the read id (FUN_00e3a770 + FUN_00d83c90) and flag 1 in the pac… |
| `00c07630` | — | unknown | Lua 'WasBad' handler: identical to 00c074c0 but stores flag 0 in the PetResponsePacket before sending it through 00c07090 when the gateway link is up, then run… |
| `00c077a0` | — | unknown | Lua 'EquipUtility' handler: requires 2 arguments (an item string at 2 and a number at 1), builds a PetUtilityPacket (FUN_00b514e0, subtype 0x10), sets the pack… |
| `00c078a0` | — | unknown | Lua 'UnequipUtility' handler: like 00c077a0 but sets the PetUtilityPacket's byte to 1 (unequip) before filling the id/string fields and sending through 00c071a… |
| `00c07ae0` | — | unknown | Lua 'UnequipItem' handler: requires 2 arguments; when the id is above 199 it forwards to the utility path 00c078a0, otherwise builds a PetEquipPacket (FUN_00b5… |
| `00c07bc0` | — | unknown | Lua 'Delete' handler: requires one argument, builds a PetDeletePacket (FUN_00c06b40), stores the argument string as the packet name field, copies the two clien… |
| `00cccfe0` | — | unknown | Returns fields zero and one directly, formats field two through a string table, returns one for supported remaining indices, and zero for unknown indices. |

## Source shards

[[S0256 009606e0-00965340|S0256]] (1), [[S0258 00969a10-0096de80|S0258]] (1), [[S0261 00976740-0097c910|S0261]] (2), [[S0287 00a0c3a0-00a13140|S0287]] (1), [[S0306 00a75b70-00a7b4d0|S0306]] (2), [[S0331 00afb7a0-00b005c0|S0331]] (9), [[S0346 00b50330-00b55fc0|S0346]] (138), [[S0347 00b56090-00b5b410|S0347]] (22), [[S0348 00b5b470-00b5ff50|S0348]] (12), [[S0349 00b5ffe0-00b66a80|S0349]] (12), [[S0374 00bd9880-00be0090|S0374]] (1), [[S0383 00c05780-00c0db00|S0383]] (25), [[S0415 00cc1a00-00cc6bc0|S0415]] (3), [[S0416 00cc6c40-00ccbaa0|S0416]] (60), [[S0417 00ccbd60-00cd11e0|S0417]] (51), [[S0571 01044480-0104a2b0|S0571]] (1)
