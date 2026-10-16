---
type: build-guide
section: Gameplay
updated: 2026-10-12
---

# Items and Inventory

Part of [[Build Guides]] › Gameplay. Related: [[Coin Store and Purchases]], [[Trade]], [[Rewards and Collections]].

<!-- NARRATIVE START -->
## How it works

**How to read this.** Confidence words (`confirmed`, `likely`, `speculative`) are the native-map labels for a function. Source tags: *client* = read from the shipped executable in a shard or in this pass; *client Lua* = the client's own UI script bytecode, losslessly decompiled in the lab (not redistributed); *client resource* = a data file from the shipped client's `Resources` folder; *client locale* = the shipped text table; *video* = a reviewed frame record (5,760 records, 656 videos; no player names or chat are recorded); *wiki* = the fan wiki; *vault* = a project research note, used here only for facts it derived from the client or from our own native runs. Precedence is client, then video, then wiki. *(R)* marks reconstructed policy that is ours, not original behaviour. No original server code exists, so every statement about what the server did is a hypothesis built from what the client requires. The community emulators' item tables, packet writers and catalogue rows are not used anywhere in this guide. OSFR-held data (packet captures and server JSON) is not used. Packet family and sub-opcode numbers below are the ones the client's own constructors and dispatchers show. Related: [[Coin Store and Purchases]], [[Trade]], [[Rewards and Collections]], [[Housing]], [[Pets]], [[Mounts]], [[Combat and Abilities]], [[Character and Customization]], [[Minigame Framework]], [[Quests and Objectives]], [[Text and Localization]], [[Scaleform and Flash UI]], [[Scripting (Lua)]], [[Gateway and Transport]], [[User Data and Storage]], [[Content]].

### Overview

Seven player-facing pieces share this guide. They share one item model but are separate screens and separate packets.

1. **The "Stuff" window** (the My Stuff icon on the game dock; client locale 38887 is "Stuff"). A category tree on the left, a list or grid of owned items, a search box, a "New Items" view and a detail panel.
2. **The equipment screens** (the job screen's "Jobs and Equipment" tabs): per-job slot tabs with a live model preview, an Accessories tab, a Power Shards tab and a Consumables tab with four item-bar slots.
3. **The Style Cards panel** (the appearance or wardrobe screen): Hair Styles, Hair Colors, Eye Colors, Facial Hair Styles, Face Paints and Wing Patterns. Each is an inventory category of style-card items (client Lua `StyleCardHandler`, categories 141 to 146 under 140; video).
4. **Storage**: a second window that moves items between the bag and a capped storage area.
5. **Members items and rentals**: members-only gear, items that expire.
6. **Item tooltips**, which exist for about a dozen kinds of thing, not only inventory items.
7. **Loot, mystery chests and prize wheels.**

What the player sees:

- **The Stuff window.** Title "Stuff". Breadcrumb categories such as "Clothes & Armor > Gloves", "Clothes & Armor > Shirts", "Housing > Walls" (video, 87 inventory frames in 44 videos). Top-level groups seen in video: Clothes & Armor (Head, Chest, Hands, Legs, Feet, Hats, Gloves, Shirts), Weapons & Tools, Jewelry & Shards (Jewelry, Power Shards, Flair Shards), Style Cards, Supplies, Crafts, Karts, Pets, Housing, Tickets, Extras. An empty list reads "Visit the Marketplace or Coin Shop to get items!" (video, 8 frames). The tree also has "New Items" (client locale 402380) and "All Items" (client Lua `MarketplaceStrings.ALL_ITEMS`, id 0.1, the default).
- **List columns and the detail panel.** Name, Worth, Rank Required, Quantity, Power, Equippable State (video; client locale 76662 Worth, 71107 Quantity, 141511 Rank Required, 402381 Power, 141512 Equippable State). Example frames: a pair of gloves "Worth 75, Rank Required 12, Quantity 1, Power 4"; leggings "Worth 40, Rank Required 4, Quantity 1, Power 2". Tooltip lines seen in video: "Resell Value: 27", "You Own: 1", "Cannot Sell", "Cannot Trade", "Required: Level 1, Freestyle Jobs, Boys only", "Required: Membership, Level 1, Freestyle Jobs", "Required: All Racing Karts", "Required: Level 1, Any Job".
- **Equippable State text** has six messages: "You can use this right now.", "You can use this item if you change jobs.", "You cannot use this right now.", "You can never use this item.", "Your current pet can use this." and "Your current pet can not use this.", plus "This item is currently equipped." for the equipped state (client locale 141370, 385977, 141371 to 141374, 385986; video shows the first, second and last).
- **Equip.** An Equip button in the list, a live preview that tries the item on before the player commits, and a character reaction on equip for the first six slots (client Lua: it fires a quick-chat command for slots below 7). Members-only items show "Membership" in the requirement line; a non-member who tries to equip one is silently ignored (client log lines, see below).
- **Job tabs.** The equipment screen lists the player's jobs (video: Warrior, Wizard, Adventurer, Archer, Blacksmith and others, plus "None") on one side and slot tabs on the other (Head, Hands, Chest, Legs, Feet, Costume, Hairstyle, Power Shards, Flair Shards). Equipment is kept per job (every equip request carries a key read from `player+0x504`, which the client uses as the active profile key).
- **Consumables.** Four item-bar slots on the Consumables tab and on the game dock. Food, potions and battle packs are assigned to a slot and used from the dock (client Lua).
- **Bags.** The client has no bag slots and no bag size: the Stuff window lists every owned item with no capacity figure, and the only capacity the client knows is the storage usage and cap pair. A "200 Slot Storage Upgrade" exists as an item name in the locale (client locale 610365668), which suggests storage capacity was raised by buying an item (R). "Backpack" and "Quiver" are cosmetic gear (video: Seeker's Backpack as a reward, Domo Backpack in a store list; dungeon pages: Trauma Care Medic Backpack, Cross Wind Archer Quiver in job clothing sets), and loot bags are world props, not containers the player carries.
- **Rentals.** Rental items show "time left" text in days, hours and minutes, minutes and seconds, or seconds, and "Expired" (client locale 419033 to 419048). An expired rental can send the player to the store bundles that contain it.
- **Loot.** Coins and items arrive as "You receive ..." lines ("You receive 1 coin", "You receive 1 Seeker's Hat"; video, 40 frames containing "You receive 1"; the Bandit Hideout page records "You receive 15 coins" pickups during the fight). The client ships loot props (`loot_bag_small_01`, `loot_bag_big_01`, `loot_chest_01`, `loot_coins_01`) in its asset manifest (client resource `Assets_manifest.txt`).
- **Prize wheels.** After a dungeon or a minigame win a ring-shaped wheel shows the prize slots; one slot is Coins and one is a "Battle Item Mystery Pack"; a pop-up says "This prize grants the winner 3 battle items!" (video, Robgoblin Treasure Trove and Artifact Hunt frames). Heading text: "Congratulations!", "You won", "Click here!" (client locale 119055, 408044, 425248). The dungeon pages in this guide set list each wheel by job.
- **Mystery chests.** A chest item whose Use asks for confirmation, consumes a key, then shows a reward panel (vault, native run; see Network). A Mystery Chest was also a plain quest or dungeon reward (video: "You receive 1 Sci-Fi Mystery Chest").
- **Gear naming and tiers.** Dungeon wheels carry job-specific gear named `<Job>'s <Weapon> of <Suffix>`, a necklace, a ring, a Power Shard and a clothing piece from a six or seven piece set, with a Roman numeral tier I to V that equals the dungeon difficulty (wiki; the names exist in the client locale, for example "Warrior's Power Shard of Regeneration V" and "Brawler's Necklace of Vitality V"). The coin slot size is not tied to difficulty (see the dungeon pages: 8 coins at difficulty 1 in Sheep Watch, 343 at difficulty 3 in Howling Hills, 307 at difficulty 5 in Trail of Betrayal).

### How the client implements it

#### Main classes

| Class or object | Role | Evidence |
|---|---|---|
| `ClientPlayerItemManager` (constructor `009918d0`, at `player+0x35748`) | Per-player item store. Builds the data sources `BaseClient.InventoryItemsData`, `BaseClient.SearchInventoryItemsData` (same class, flag `+0x2e8` set), `BaseClient.StorageItemsData`, Consumables, New Items, Rentals, Expired Rentals, ItemStats, one `BaseClient.Inventory.<Slot>Data` source per equipment slot and 16 `BaseClient.Vehicle.<Part>Data` sources. After any definition arrives `RefreshDerivedState` (`009912f0`) recomputes boost-currency and the character, pet and house name-change entitlements from item totals. | confirmed (`009918d0`, `009912f0`) |
| `ClientItem` / `ItemInstance` / `ItemRecord` | The owned-item record. `ItemRecord` (vtable shared by `ItemInstance` and `ClientItem`) holds Definition at `+4` and Tint override at `+8`. `ItemInstance` adds Guid `+0x10`, Count `+0x14`, ConsumedCount `+0x18`, stored count `+0x1c`, last-cast time `+0x20`, and a 16-byte rental block at `+0x28` allocated on demand. `ClientItem` adds flags at `+0x30` and `+0x31`. The format strings print `Item: {Guid, Definition, TintOverride, Count, ConsumedCount, LastCastTime, RentalExpirationTime, RentalUpsellsSent, MarketplaceBundleId}`. | confirmed (`0103dc60`, `0103e5e0`, `0103e590`, `008b1e00`); layout also in S0569 |
| `ClientItemDefinitionManager` (`00a3d6e0`, at client `+0x20c`) | The runtime definition cache: a 1,024-bucket table keyed by definition id (`00a3d870`), a pending-download tree, and an update dispatcher (`ItemDefinitionDsUpdateDispatcher`, `00c93410`, `00c93c90`) that tells the UI when a definition lands. | confirmed |
| `ClientItemDefinition` / `BaseItemDefinition` / `ItemDefinition` (runtime `ClientItemDefinition` objects are 0x230 bytes, `0090b720`; the character-create factory `00724a80` allocates 600 bytes) | The definition record. `BaseItemDefinition` (`007230f0`) embeds `IconData` (ImageSetId `+4`, TintId `+8`) and three strings. `ItemDefinitionStyleCard` is built for type `0xc` and `ItemDefinitionCustomization` for type `0x11`; every other type is a plain `ItemDefinition`. | confirmed (`007230f0`, `00724300`, `00724820`, `00724960`) |
| `ItemClassDefinitionManager` (client `+0x1b8`), item category manager (client `+0x1bc`) | Item classes (wield and stat metadata, owning-job name) and the category tree. Filled from the server's reference-data packets 44/1 and 44/2 (`00928d00`). File loaders for the same tables exist (`01048570` for `ItemClasses.txt`, `010456c0` for `ItemCategories.txt` and `ItemCategoryGroups.txt`) but have no direct caller and the files are not in the shipped `Resources` folder. | confirmed (`00928d00`, `01048570`); `010456c0` unlabelled |
| `EquipmentSlotDefinitionManager` (`007989a0`) | The slot table loaded from `EquipmentSlotDefinitions.txt`; logs to `EquipmentSlotDefinitionManager.log`. | confirmed |
| `BaseInventoryDataSource`, `InventoryDataSource`, `StorageDataSource` (`00a6d870`, `00a6deb0`, `00a6e000`) | The row sources. Rows are the owned instances that pass a per-row predicate (vtable slot 15). `00a6e910` (slot 14) rebuilds; `00a6e7b0` is the loop that calls the predicate. | confirmed constructors; predicates `00a6d9b0` (unknown), `00a6db90` (likely) |
| `InventoryCategoriesDataSource` and `CategoriesTreeNode` (`00a6e9d0`, `00a6e480`) | The category tree the Stuff window binds to (`BaseClient.InventoryCategoriesData`). | confirmed constructor; rebuild likely |
| `InventoryNavigationDataSource` (`00a7c6e0`; registered as `Inventory.Navigation` and `BaseClient.Inventory`) | A second tree of categories and **item groups** with Total Count, Owned Count, On Sale and tag columns. Its columns suggest a catalogue-style browser rather than the bag list (see Item groups and Open questions). | confirmed |
| `BaseItemDefinitionDataSource` family (`00c96a30`, value getter `00c956c0`, column names `00c92c10`) | One 41-column view of definitions shared by the all-items, single-item, fixture, customization and coin-store row sources. | confirmed |
| `TooltipDataSource` (`00a79880`) and the builders `00a76a60` to `00a79620` | One tooltip store with a kind field at `+0x280` and a 32-byte two-string row array. Fires `TooltipMethods:OnTooltipRequest`. | confirmed constructor; builders confirmed or likely |
| `ItemAbilitiesDataSource` (`00a7ee80`, `BaseClient.ItemAbilities`) | The ability list of the selected item (ability id, icon, name, description, effect count). Missing ability definitions are requested through the ability family. | confirmed |
| `BaseItemSearchEngine` (`00c30040`) | Local text search: tokenizer, lowercasing, weighted per-field scores. | confirmed constructor; scoring likely |
| `ClientItemFilterManager` (`00bb2350`) | Job and gender filter definitions behind `BaseClient.ItemFilters`; the Lua that binds them was not traced. | confirmed constructor |
| Lua objects `InventoryHandler`, `StorageHandler`, `JobHandler` (equipment panes), `EquipmentHandler`, `TooltipMethods`, `ScoreScreen` | The presentation layer. | client Lua |
| Native Lua module `InventoryScreen` (registrar `00c09060`, table `0182a4a8`, 21 entries) | The native entry points of the Stuff window. | table read in S0383 |

The `InventoryScreen` entries decoded in S0383 are `FilterItemGroupByPage` (`00c09080`), `SelectItemGroup` (`00c090c0`), `GetGroupIdFromItemId` (`00c091c0`), `GetItemDefinitionIndex` (`00c09220`), `CreateItemGroup` (`00c09280`), `SelectInventoryCategory` (`00c092c0`), `SelectAltInventoryCategory` (`00c09300`), `SelectInventoryGenderFilter` (`00c09340`), `HideItem` (`00c09380`), `ShowItemGroup` (`00c09720`), `ShowItemId` (`00c098b0`), `SearchInventoryItems` (`00c09a80`) and `RequestItemRequirementStatus` (`00c09d30`) (client). The table has 21 entries; 13 handlers are in the shard.

#### Item definitions: where they come from

1. **The client ships no item catalogue.** Its `Resources` folder has the equipment-slot table, filter, group, sort, rarity and requirement tables listed under Data and formats, and one small starter-clothing file used only by character creation (`CharacterCreate/ItemDefinitions.txt`, 302 rows). Every other definition arrives from the server (client resource listing; client `00724b80`).
2. **Startup catalogue, 35/37.** The base dispatcher (`0092f460`, jump table `00934618`, index = sub-opcode minus 1) builds `PlayerUpdatePacketItemDefinitions` and calls `ItemDefinitions_ReadBatchFromReader` (`00a3db70`). The packet is header, then one **length-prefixed payload**; inside it is an `i32` count followed by that many definitions back to back (no per-definition length). Each definition is constructed (`0090b720`), read (`008e3760`, `00919e10`) and cached by `ItemDefinition_OnReceived` (`00a3c290`), which logs "Received item definition data, id: %d." to `ItemDefinitions.log`, replaces an existing entry ("Overwriting existing item definition in cache") and drops the id from the pending-download list. A missing length prefix makes the client reject the whole catalogue before registering it (vault: the framing repair, native run 173814; the reader chain is `008f81f0` to `008d6830` and `008bfc00`).
3. **Single definition on demand, 35/34.** When a screen needs a definition that is not cached, `ItemDefinition_RequestById` (`00a3ddd0`) checks the cache and the pending tree, logs "Item definition not found or pending, will issue new request" and sends `PlayerUpdatePacketItemDefinitionRequest` (35/34, one `i32` id at `+0x14`) through the generic tunnel sender `00a3dca0` when the gateway pointer at client `+0x280` is set. A pending id is dropped silently on repeat ("will drop request"). The same 35/34 class is what the dispatcher uses to **read** the answer (case at `00933501`): header, a length-prefixed payload holding exactly one definition, then the same `OnReceived` path.
4. **Inline with ItemAdd, 38/2.** The item-add update carries the definition after the instance (below), so a new item never needs a separate fetch.
5. **Visibility rule.** The inventory row predicate resolves the instance's definition first and returns "hidden" when it cannot (`00a6d9b0`). A missing definition therefore hides the item silently (vault: the restart-172324 run showed an empty All Items list until the catalogue framing was fixed; native run 173814 showed All Items, Hats and Chef Head populated).

#### Item instances: login, add, update, delete

- **Login.** Message id 12 (`cPacketIdSendSelfToClient`) is handled by the main dispatcher `009471a0` (case `0xc`): it creates `ClientPcData` (`00a1de60`) and reads the whole body with `00944fe0`. The body includes the item list: `0090a630` reads an `i32` count and, per entry, the instance record below plus one flag byte stored at `+0x31`, inserting each node into the manager at `player+0x35748`. The same body also fills the action bars (`00a16d50` is called from the reader). Whether equipment and profile blocks sit in this same body is described from the client's own readers in [[Character and Customization]]; an earlier vault comparison against a recorded server payload is not used here.
- **38/2 ItemAdd** (`HandleClientUpdate` `009392c0`, case 2). Header, one length-prefixed payload containing the instance record, then a full definition. The handler allocates a node, reads the record, sets `+0x30` to 1, hands it to `00991630` on the manager (which first removes any existing record with the same instance id and then inserts the new one), then constructs and reads the definition and caches it (`00a3c290`).
- **38/3 ItemUpdate.** Header and seven `i32` (packet offsets `+0xc` to `+0x24`, reader `008d8350`). Reading of the disassembly (R for the field names): the first is the item key; the second is written to count (`+0x14`), the fourth to consumed count (`+0x18`) and the fifth to stored count (`+0x1c`) through keyed setters (`009904c0`, `00990500`, `00990540`); the last two form an 8-byte value written through `008d0d90` (the last-cast field); the third defaults to -1 in the constructor (`008b25e0`) and, when it is not negative, is passed with the fifth value to `HandlerActionBar:setItemCount` (`009cfb80`) as the item-bar slot.
- **38/4 ItemDelete.** Header and one `i32` item key (`0089d6c0`, reader `008e3e70`). `RemoveItemAndRefreshNamePermission` (`00991690`) removes the node, recycles it, re-evaluates `NameChange.CanChangeName` when the removed key is a name-change item, and tells the owner. The handler then calls `00bdb400` on the object at client `+0x138` (speculative label; it looks like an equipment-screen refresh).
- **Row predicate** (`00a6d9b0`, main bag): resolve the definition; require that the item's category exists and has its `+0x14` flag non-zero (when the category table is loaded); require the current category filter to accept the definition's category (`008a6920`, recursive over parent categories by `00a6dd70`); show only when `Count - StoredCount - ConsumedCount` is positive; and, when a flag at `+0x8c` on the item manager is set, hide a last single unit that `00a15590` marks (not resolved). Storage (`00a6db90`) uses the same category test and requires `StoredCount > 0`.

#### The Stuff window (client Lua `InventoryHandler`)

`Show` loads the SWF, sets the title (`InventoryLabel_MyStuff`), the member status, the coin total and the last view mode, then builds the category tree from `DsTree` `DST_INVENTORY_CATEGORIES_DATA`: a fixed "New Items" node (`-1`), an "All Items" node (`0.1`, the default), then every category the data source holds, skipping the housing-fixture category and categories with no owned items. Selecting a category calls `InventoryScreen.SelectInventoryCategory`, stores `LastSelectedCategory` and `LastSelectedViewMode` in user options and refreshes. The SWF pulls rows in pages (`requestData(offset, count)`), and each row is serialized as `column^value` pairs joined by `::` and rows joined by `##`. The detail panel asks for one row at a time (`SelectItem`, `UpdateItem`). Events the panel can raise: equip (`EquipScreen.EquipItem`), use (`EquipScreen.CheckImmediateActivationItem`), offer to a trade (`Trade.OfferItems`), buy a rental again, go to the members page, clear the new-item flag (`Ui.ClearItemNewFlag`, queued while the New Items view is open and flushed on leaving it), search, cancel search, and close.

- **Data source columns, Stuff window (31).** Item Guid, Icon ID, Name, Description, Quantity, Members Only, Tint Value, Equippable State, Slot, New, Is Activatable, Rarity, Is Equipped, Model Name, Item Type, Override Tint Id, Activate Enabled, Power Rating, Gender Usage, List View Stat Id, List View Stat Value, Min Profile Rank, Text Color, Item ID, In Storage, In Inventory, Is Equipped On Any Profile, Price, No Sale, Is Rental, Rental Seconds Remaining (client Lua; native names in `00cb09d0` for 0 to `0x1a`, `00a6d960`, `00a6db10`). The New Items source has no Price or No Sale columns.
- **Search is local.** `InventoryScreen.SearchInventoryItems(text)` rebuilds `BaseClient.SearchInventoryItemsData` through `BaseItemSearchEngine`: the query is split into tokens (quotes allowed, `00c2f490`), lowercased, scored per configured field with weights and a log transform (`00c2fc80`, `00c2f080`) and ranked (`00c30a70`). No packet is sent (the crashed run of vault note 082 sent none).
- **Style Cards panel** (client Lua `StyleCardHandler`, a mode of the equipment screen). Category 140 is the style-card parent and 141 to 146 are its six children (Hair Styles, Hair Colors, Eye Colors, Facial Hair, Face Paints, Wing Patterns). Opening the panel selects category 141 and sets the inventory gender filter to the player's gender (`InventoryScreen.SelectInventoryGenderFilter`), restricts some slots by race, and previews a card locally (`EquipScreen.PreviewStyleCard`). "Accept" walks the six categories and calls `EquipScreen.UseStyleCard(itemGuid)` for each changed slot (**42/10**), then clears the new-item flag; "Revert" clears the previews. Style cards are definitions of type `0xc` (`ItemDefinitionStyleCard`). The equipment screen's second tab label starts as the placeholder "[Style Cards]" and is replaced by the name and icon of the matching category in the tree (`JobHandler.GetStyleCardIconAndLabel`).

#### Equipment slots and the equip request

- **Slot table** (client resource `EquipmentSlotDefinitions.txt`, 20 rows). Slot ids 1 to 7 are Head, Hands, Chest, Legs, Feet, Shoulders, Weapon; 8 and 9 are Accessory1 and Accessory2 (client locale names "Ring" and "Necklace"); 10 is Accessory3 (no name); 11 to 16 are the Triangle, Square, Diamond, Star, Moon and Circle runes (client Lua labels Square "Power Shards", Diamond "Flair Shards", Triangle "Triangle Shards", Circle "Circle Shards", Star "Star" and Moon "Moon"; locale 2414, 2419, 2413, 2415, 2416, 2242); 17 is WholeBodyCostume; 18 to 20 are CustomizationHead, CustomizationHair and CustomizationModel. Chest and Legs are the only `IS_REQUIRED` slots. Star Rune needs profile rank 10 and Moon Rune rank 20 (`MIN_PROFILE_RANK`). Weapon is the only `IS_WEAPON` slot. No slot is flagged `MEMBERS_ONLY` in the shipped table, although the equip code tests that flag (below). The Lua `JobHandler.InventorySlotMap` has 21 entries: slots 1 to 8 map to the data sources `HeadData`, `HandsData`, `ChestData`, `LegsData`, `FeetData`, `ShouldersData`, `WeaponData`, `Accessory1Data`; 9 to 11 map to nothing; 12 and 13 to `SquareRuneData` and `DiamondRuneData`; 14 to 16 to nothing; 17 to `WholeBodyCostumeData`; and 18 to 21 to the four `Consumables` item-bar positions (client Lua; positions 18 to 21 here are item-bar positions, not the customization slots 18 to 20 of the table; see Open questions). The equipment panes are opened with (first slot, count): equipment `1, 14`, accessories `8, 9`, consumables `18, 4` (client Lua).
- **Pane flow** (`JobHandler:EquipmentPaneOnUserEvent`). `previewItem` calls `EquipScreen.PreviewEquipItemByGuid(guid)` (local only) and sets a dialog camera offset; `clearPreview` clears it; `equipItem` plays a sound, tells the first-time-event handler, and either assigns to the item bar (consumables) or calls `EquipScreen.EquipItem(guid, slot)`; `changeSelectedJob` calls `JobScreen.SelectJob`.
- **Native equip** (`EquipItemSelectedInUi`, `00bdbc50`, reached from `EquipScreenEquipItem` `00c10bf0`): the guid string is resolved to an owned instance (`009909c0`). If the instance is members-only (`def+0x74`) and the player is not a member (`008a6990`), the client logs "EquipItemSelectedInUi: Ignoring equip item request ItemDefinition = %d for MO item for non-member." to `Items.log` and sends nothing. Otherwise the slot is looked up in the slot manager; a members-only slot (`+0x25`) with a non-member logs "...on MO slot = %d for non-member." and stops. With a resolved instance it sends **42/3 EquipByGuid** (`00bda510`, `00bdb450`: three `i32` after the 4-byte header: instance id, the player's active profile key at `player+0x504`, slot). With no instance it sends **42/2 EquippedRemove** (`00bda720`, `00bdb530`: slot and profile key), which is how the screen unequips a slot.
- **Equip by record.** After a store purchase or a claim, `ProcessItemRecordListActions` (`00a1ec00`, callers `00b38ea0`, `00a041b0`) walks a list of `ItemRecord` (definition, tint) entries and, by definition type (`1, 2, 3, 4, 7, 0xc, 0xf, 0x10, 0x13, 0x1b, 0x1c, 0x1d` are the cases of its switch), sends **42/8 EquipByItemRecord**, **42/9 ItemActionBarAssignByItemRecord**, **42/12 UseStyleCardByItemRecord**, and the housing, mount and pet equivalents. The server therefore has to resolve a definition id and a tint to an owned instance (client `00a1ec00`; S0290).
- **Other equip requests.** **42/10** UseStyleCard, **42/11** PreviewStyleCard, **42/13** EquipLightsaber (by guid, `00bdbe30`), **42/14** EquipItemGroup (by group id, `00bdbf90`, walking the 10-bucket group table). All use the same tunneled-client send template (client `00bdb790`, `00bdb8a0`, `00bdbac0`).
- **What the server sends back.** For the owner: **38/5 EquipItem** (`ClientUpdatePacketEquipItem`, `008dc810`): header, a profile id, an attachment record of three length-prefixed strings (model name, texture alias, tint alias) and three `i32` (the log string prints Slot, TintIdOverride and Effect; their order on the wire was not separately checked), then two `i32` and a bool. The handler logs `EquipItem: Slot=%d, ModelName=%s, TextureAlias=%s, TintAlias=%s, TintIdOverride=%d, Effect=%d.`, caches the attachment under that profile and applies it to the model only when the profile equals the active one. **38/6 UnequipSlot** carries a profile id and a slot and removes it. For other players: **35/6 EquipItemChange** and **35/7 EquippedItemsChange** (decoded here only to the dispatcher level: both locate the actor by a key pair and update its attachment groups). `ProxiedCharacterAttachmentGroup` logs "multiple items in equipment slot" when a profile lists two items for one slot (`00bf75d0`).
- **Equippable State** is computed on the client (`ItemDefinition_EvaluateUsability` `00a15690`, classifier `00a142c0`, whose branch structure is summarised here, not proven line by line): state 4 "never" when the definition's gender usage excludes the player's gender (`00bae6a0`); state 3 for a members-only item and a non-member; state 1 "right now" when the active profile meets the definition's rank (`def+0x84` against a field at `+0x14` of the profile node) and the slot's and the item's requirement sets validate (`00bd1260`); state 2 when another profile in the player's list at `player+0x49c` qualifies (`00ae19f0`); otherwise 3. Style cards (type `0xc`) only test gender (`def+0x68` of 0 any, 1 male, 2 female against the player's gender at `player+0x458`) and two slot-specific rules (slot 5 and slot 7 cards need extra player state). An expired rental forces state 7 (`00a156f0`), whose text slot reuses "You cannot use this right now". Invalid slots log `Item Definition=%d is using invalid/missing slot %d` to `#ItemErrors.log`. The server does not send this value (client; S0295, S0288).
- **Requirement text** (`00baec20`): the format string (client locale 3908) is `[membership][level][class][vip rank][gender]`; level text appears when `def+0x84 > 0` ("Level 5, "), VIP text when `def+0xa4 != 0` and the VIP system is enabled, "Membership, " when `def+0x74` is set, ", Boys only" or ", Girls only" from `def+0x68`, and the class name: the item class's owner-name string (`class+0x18`), or "Any Job" when the class id is not in the class table and a level is set. A requirement-set id at `def+0xa8` adds extra race lines from a table at client `+0x13c`.

#### Consumables, the item bar and immediate use

- **Assign.** `Ui.AssignToItemActionBar(slot, guid)` (`00c23ed0`) builds **42/6 ItemActionBarAssign** (`00c14720`: slot at `+0xc`, instance id at `+0x10`; vault: 12 bytes with the 4-byte header) and sends it through `00c22ad0`/`00c20a30`. Slots are zero-based within the four item-bar slots (the Lua passes `slot - 18`).
- **Bars.** `Client_GetOrCreateActionBar` (`00a16d50`) keys action bars by type in a table at client `+0x35434`: type 1 `BaseClient.AplActionBar`, type 2 `BaseClient.ItemActionBar`, anything else `BaseClient.InvalidActionBar`. The server updates a slot with **38/25 UpdateActionBarSlot** (`0089e7f0`; vault: the original reader consumes 63 bytes, header plus bar and slot plus a 51-byte slot record, and the slot's enabled flag at offset 41 is independent of the inventory row's Activate Enabled).
- **Use from the dock.** Activation is the ability path: the dock's `activateItem` calls `Ui.ClickedAbility(button - 5, 2)` (client Lua) and the native chain gates on the slot (empty, disabled, recast, mana, status) before it sends the common ability activation (**36/10**, vault: `i32` bar, `i32` slot, `i32` target mode, optional 8-byte guid or Vector4; native bytes `00c173c0`, `00a12840`, `00c44160`, `00a35600`, `00a35150`). Whether an item has an activation ability is data in the definition (see Data and formats).
- **Immediate-use items** (chests, packs): the Lua `useItem` calls `EquipScreen.CheckImmediateActivationItem(guid)`, which sends **42/16 CheckImmediateActivationItem** (`00c109f0`, one `i32`). The server answers **38/30 ImmediateActivationCheck** (`0089ea30`, reader `008f9270`): the handler `00a1a7d0` raises `InventoryHandler:OnIsItemUsable(success, guid, categoryId, bundleId, titleStringId, bodyStringId)`. Success shows a confirm window and, on Okay, `EquipScreen.UseImmediateActivationItem(guid)` sends **42/15 UseImmediateActivationItem** (`00c10940`). Failure shows the title and body strings and an Okay that opens the marketplace at that category and bundle. The client sends no separate cancel (vault: native run, Mystery Chest).
- **Consume effects.** The client has no consume rule; the server decides what a stew, potion or pack does. Original numbers are not recovered (vault: strength stew, health potions; the wiki and the client's generic tooltip disagree, see Open questions).

#### Storage

`StorageDataSource` shows items with a non-zero stored count. **42/17 MoveItemToStorage** (`00c1a8c0`, from `00c24270`) carries an 8-byte value obtained from the local player object (packet `+0x10`/`+0x14`; most likely the player's guid, R), an item value parsed from the Lua string argument (`+0x18`) and a number (`+0x1c`). **26/0x2c RequestStorageCheck** (`00c14a80`, `00c24370`, one zero) is a command-family request. The server opens the window with **42/18** (the first short of the 42-family body is `0x12`: `00991730` raises `StorageHandler:Show`, nothing else is read), publishes usage and capacity with **38/31** (`ClientUpdatePacketStorage`, `0089d5e0`: two `i32` stored at client `+0x84` and `+0x88`, then `StorageHandler:PopulateStorageInfo`) and can send **38/32** (no body) to raise `StorageHandler:PromptPurchaseStorage`. The Lua window compares `Ui.GetStorageUsage()` with `Ui.GetStorageCap()` to glow the add-space button, and `CheckCSTokens` scans coin-store category 62 for something buyable (client Lua; which of the two stored values is usage was not traced).

#### Rentals, new items, derived entitlements

- Rental block: a flag byte, then an 8-byte expiry, an `i32` upsell count and an `i32` marketplace bundle id. `IsBlock28BeforeCachedNow` (`0103dfe0`) compares the expiry with the client's cached 64-bit clock (`00702670`, a throttled `__time64`), so the expiry must be in that clock's base (S0569; R for the exact unit). `IsRental` and `RentalSecondsRemaining` are computed by the inventory data source (`00a6db10`, `00a6e6b0`).
- "New" is a client-local flag (`+0x30`/`+0x31` on the node plus the Lua clear queue); a restart can leave New Items empty without any loss (vault, restart audit).
- `RefreshDerivedState` (`009912f0`) re-derives `NameChange.CanChangeName`, `NameChange.CanChangeCharacterName`, `NameChange.CanChangePetName`, the house variant (`00991440`) and `BaseClient.MiniGame.BoostCurrency` from the owned count of particular item definitions (`0098f850`); the definition ids come from a tag list keyed by hashed constants (`00a145e0`, selectors 0 to 2; the constants were not read). The Factory tools source is refreshed from the same call.

#### Tooltips

`TooltipData` is one store with a kind in `+0x280` and rows of two strings (32 bytes each). Builders, with the kind they set where it was read: InventoryItem by id (`00a799d0`, kind 9; builder `00a76a60`), another item-by-id builder (`00a77a80`, kind 1), AbilityItem (`00a77d40`, kind 2, 11 rows), Recipe (`00a78a40`, kind 6) and a second recipe form (`00a79620`, kind 13), Collection entry (`00a78040`, kind 8, 10 rows), ItemGroup (`00a77b40`, kind 15, 7 rows), StoreItem (`00a771b0`, 12 rows), ClaimItem (`00a77680`, 8 rows), Pet (`00a782d0`), PetTrick (`00a784e0`), Ability (`00a786d0`), Job (`00a78860`), ProxiedCharacter (`00a78ca0`), Trophy (`00a78e80`) and FactoryNpc (`00a79250`, 21 rows). The item builder never blocks: when the definition is not cached it remembers the id, issues the 35/34 request and rebuilds when the definition lands. The Lua entry points are `TooltipData_Populate*` (`00c1fdc0` to `00c20480`). Tooltip kinds other than items are covered in their own guides.

#### Item groups and navigation

`ItemGroups.txt` (28 rows, all name and description ids zero in the shipped file) and `ItemGroupMappings.txt` (2,723 rows over 28 groups and 1,796 distinct item ids, every slot override 0) are loaded by `00a3dab0` into the `BaseClient.ItemGroup.Items` source (group record 0xf8 bytes, 10-bucket map). `InventoryNavigationDataSource` builds a tree of category nodes and item-group nodes whose columns are ID, Parent Row, Node Type, Name, Description, Icon ID, Total Count, Owned Count, Members Only, Currency Type, On Sale, New, Tag String, Tag Color (`00a7a5e0`). `ItemGroupNode` filters by flags and by a three-way selector (`00a7ba90`). No client Lua calls `ShowItemGroup`, `ShowItemId`, `CreateItemGroup`, `FilterItemGroupByPage`, `SelectItemGroup`, `GetGroupIdFromItemId`, `GetItemDefinitionIndex`, `HideItem` or `SelectAltInventoryCategory`; the Lua only uses `SelectInventoryCategory`, `SelectInventoryGenderFilter` and `SearchInventoryItems` (client Lua). The group tables and these entries are therefore either used by SWF code or unused. The group names are not in the client files (see Open questions).

#### Loot, chests and prize wheels

- **Loot event 35/29** (`PlayerUpdatePacketLootEvent`, constructor `008d0a00`, case at `009332fa`): a string and two vectors. The handler builds a `ProjectileParameters` block from the vector difference and starts a projectile emitter. It is a visual effect; the coins and items themselves arrive in a reward bundle (family 50, see [[Rewards and Collections]]). The packet's field order was not decoded (client).
- **Prize wheel** (client Lua `ScoreScreen`). `PopulateLootWheel` reads two data sources, `BaseClient.MiniGame.RewardPreview.Entries` (the candidate prizes) and `BaseClient.MiniGame.RewardPreview` (the coin and experience bundle), and calls `AddLootWheelItemData` for each entry, then adds a Coins slot (icon id 22663, label locale 6513 "Coins") and an Experience slot (icon id 9, label locale 426522 "Experience") when those amounts are positive. With no entries the wheel is skipped. The entry columns used are Type, Name, Description, Icon ID, Quantity, Tint Value, Item Id, Parameter 2, Rarity, Members Only, Guid, Hidden. The player spins (`OnLootWheelSpinRequest`); the server's result arrives through the minigame family (`StopMiniGameLootWheelAt` `009b6da0`, called from `DispatchMiniGameMessage` `009bfea0`), which compares the awarded reward against the preview entries to compute the stop index and raises `ScoreScreen:StopLootWheelAt`. When the wheel stops the Lua calls `Ui.SendLootWheelRotationStopPacket()` and then shows the reward list (client; the sender of that packet was not traced).
- **Mystery chests.** The client has a `BaseClient.MysteryChest` source (columns include `ChestId`, `ChestImageId`, `ChestNameId`, `ChestDescriptionId`; `009c9310`, `009c6e10`), a `MysteryChest.log` and `CallHandler:ShowMysteryChestReward` (`00b899f0`). The chest flow itself is the immediate-use pair above followed by a reward bundle of kind 16 (vault, native runs: Cancel kept the chest and key; Okay consumed one of each and showed the reward panel).
- **Reward text.** "You receive #count([*item*])" and "You receive #count([*experience*]) and #count([*coins*])" (client locale 2 and 3).

### Data and formats

**Client resource files that touch items** (counts are data rows, from the shipped `Resources` folder of the baseline client):

| File | Contents | Source |
|---|---|---|
| `EquipmentSlotDefinitions.txt` | 20 rows. Columns `ID, IS_REQUIRED, MEMBERS_ONLY, MIN_PROFILE_RANK, REQUIREMENT_ID, NAME_ID, SLOT_NAME, DEFAULT_ADR, DEFAULT_TEXTURE_ALIAS, DEFAULT_TINT_ALIAS, IS_EQUIPMENT, BONE_ATTACH_NAME_OVERRIDE, DUAL_WIELD_ATTACH_NAME, CUSTOMIZATION_SLOT_OVERRIDE, SKINTONE_TEXTURE_ALIAS, CLIENT_REQUIREMENT_ID, IS_WEAPON, ANIMATION_PRIORITY`. Slots 1 to 8 and 17 have `IS_EQUIPMENT` 1; animation priorities are Head 8, Hands 6, Chest 5, Legs 7, Feet 2, Shoulders 4, Weapon 3, slots 8 to 16 in order 9 to 17, WholeBodyCostume 1, customization slots 18 to 20. WholeBodyCostume has `REQUIREMENT_ID` 241 and `CLIENT_REQUIREMENT_ID` 25. | client resource; loader `007989a0` |
| `ItemFilters.txt`, `ItemFilterOptions.txt`, `ItemFilterOptionMap.txt` | 2, 22 and 18 rows. Filter 1 is "Job" (type 1), filter 2 is "Gender" (type 2). Job options carry the job profile ids as values: Adventurer 1, Ninja 2, Postman 4, Medic 11, Wizard 12, Miner 14, Blacksmith 16, Warrior 32, Archer 35, Brawler 43, Chef 45, Kart Driver 48, Demo Derby Driver 49, Soccer Star 52, Card Duelist 120, Pet Trainer 121, Fisherman 137. Gender options are Male (value 1) and Female (value 2). Options 5 to 7 ("1", "2", "4-8", the last with a 4 to 8 range) are defined but not mapped to any filter. | client resource; locale names; loader `0107f4e0`, predicate `00bb24c0` (type 1 = id-list membership, type 2 = the check `00a0fc30`, type 3 = compare `def+0x84`); the option values are read as profile ids (likely: `CheckCurrentProfile` in `Requirements.txt` uses 120 and 121) |
| `ItemGroups.txt`, `ItemGroupMappings.txt`, `ItemGroupPromos.txt` | 28 placeholder rows (columns `ID, NAME_ID, DESCRIPTION_ID, SORT_ORDER, ICON_ID, MODEL_NAME, CATEGORY_ID, PAGE_ID, IS_NEW, MODEL_ID, ANIM_ID, NOT_LINK_SET, TAG_STRING, TAG_COLOR`); 2,723 group-to-item rows (`ITEM_GROUP_ID, ITEM_DEFINITION_ID, SLOT_OVERRIDE`; largest groups 1 with 1,116 items, 19 with 540, 18 with 505, 20 with 299); promos file is header only. | client resource; `00a3d350`, `00a3cf00` |
| `ItemSortTypes.txt` | 5 rows: Default, Alphabetical A to Z, Z to A, Price Low to High, Price High to Low. | client resource; locale names |
| `ItemRarityDefinitions.txt` | 4 rows of RGB: rarity 0 white (255,255,255), 1 green (160,255,118), 2 blue (133,239,255), 3 orange (255,162,0). | client resource |
| `Requirements.txt`, `RequirementGroups.txt`, `RequirementSets.txt` | 43, 39 and 34 rows. Requirement types by factory name: `CheckRankLastProfileByType` (24 rows, operators 4 and 6, profile type and rank parameters), `AlwaysFails`, `SourceVipRank`, `CheckCurrentProfile` (including profile ids 120 and 121), `CheckRenamePending`, `CheckNudge`. Sets map to groups and groups to requirements. Items and slots reference sets by id. | client resource; loaders `00bcfa70`, `00bcfef0` |
| `WieldTypeAnimationMappings.txt` | 26 wield types (Hand to Hand 1, One-Hand Slashing 2, Two-Hand Slashing 3, Dual-Wield Slashing 4, Bow 5, Thrown 6, Wand 7, Two-Hand Polearm 8, job outfits 9 to 17, shooters and boots 20 to 24, Parachute 27, whip 28, ice skate 29, Bunny Suit 30) with melee and idle animation groups. | client resource |
| `VehicleItemClasses.txt` | 124 rows linking a vehicle id to item class ids. | client resource; `0104aea0` |
| `TintGroupMappings.txt`, `GenderDefinitions.txt`, `Mannequins.txt`, `ContentPacks.txt` | 269, 2, 2 and 94 rows. Genders: 1 male (`m`), 2 female (`f`). Content packs carry per-environment unlock flags and gate categories (`010463c0`). | client resource |
| `CharacterCreate/ItemDefinitions.txt` | 302 starter-clothing rows with the 51-column definition schema below. Item types seen: 12 (203 rows) and 1 (99 rows). | client resource; loader `00724b80` |

**The definition schema as the client names it** (51 columns in `CharacterCreate/ItemDefinitions.txt`): `ID, ITEM_TYPE, NAME_ID, DESCRIPTION_ID, IMAGE_SET_ID, ACTIVATABLE_ABILITY_ID, ACTIVATABLE_RECAST_SECONDS, PASSIVE_ABILITY_ID, COST, ITEM_CLASS, MAX_STACK_SIZE, PROFILE_OVERRIDE, SLOT, RANGE, NO_TRADE, SINGLE_USE, MODEL_NAME, GENDER_USAGE, TEXTURE_ALIAS, TINT_ID, CATEGORY_ID, EQUIP_REQUIREMENT_ID, MEMBERS_ONLY, NON_MINI_GAME, PARAM1` to `PARAM8, NO_SALE, ITEM_MATERIAL_TYPE, WEAPON_TRAIL_EFFECT_ID, USE_REQUIREMENT_ID, COMPOSITE_EFFECT_ID, POWER_RATING, MIN_PROFILE_RANK, RARITY, CONTENT_ID, TINT_ALIAS, COMBAT_ONLY, PREFERRED_ITEM_BAR_SLOT, IS_TINTABLE, STRING_PARAM1, FORCE_DISABLE_PREVIEW, MEMBER_DISCOUNT, RACE_SET_ID, VIP_RANK_REQUIRED, CLIENT_EQUIP_REQ_SET_ID`. `^` separates cells and `#` starts a comment. The starter rows use model names with `<race>` placeholders (for example `freestyle_<race>_m_legs_baggypants.adr`) and texture aliases such as `freestyle-baggyjeans-L1`; this schema is the closest the client gives to a definition spec.

**Runtime definition record: wire order and client offsets.** Read by `008e3760` (base) and `00919e10` (client tail). All integers are little-endian `i32`; strings are an `i32` byte length plus bytes. Names are the data-source columns of `00c92c10`/`00c956c0` where the client prints the field, otherwise the file column the position suggests (R) or "unnamed".

| Step | Offset | Field | Basis |
|---|---|---|---|
| 1 | `+0x04` | Item definition id | client (`00c956c0` col 0; cache key) |
| 2, 3 | `+0x08`, `+0x0c` | Probably name id and description id; the displayed Name (`+0xd4`) and Description (`+0xe4`) are resolved strings filled after the read (`00bae660`) | R |
| 4 | `+0x14`, `+0x18` | Icon image-set id, tint id (`IconData`) | client (`00720cd0`, cols 1 and 0xd) |
| 5 | `+0x1c`, `+0x20` | Two ids that fall between the icon and the price; by the file order, the activatable and passive ability ids | R |
| 6 | `+0x24` | Coin purchase price | client (col 0x19) |
| 7 | `+0x28` | Item class id | client (col 0x22; `01048390` lookup) |
| 8 | `+0x2c` | unnamed (file order: profile override) | R |
| 9 | `+0x30` | Equipment slot id | client (col 0xa) |
| 10 | `+0x34`, `+0x35` | Flag byte (unnamed); No Sale | client (col 0x1c reads `+0x35`) |
| 11 | strings at `+0x3c`, `+0x4c` | Model name, texture alias | client (cols 0xb, 0xc) |
| 12 | `+0x68`, `+0x6c`, `+0x70` | Gender usage (0 any, 1 boys, 2 girls), item type, category id | client (cols 7, 0x23, 4; text `00baec20`) |
| 13 | `+0x74`, `+0x75` | Members only; unnamed byte | client (col 5) |
| 14 | `+0x78`, `+0x7c`, `+0x80`, `+0x84` | unnamed; composite effect id; power rating; minimum profile rank (also used as the level shown in the requirement text) | client (cols 0xf, 0xe, 0x13) |
| 15 | `+0x88`, `+0x8c`, `+0x90`, byte `+0x94` | Unnamed; by file order rarity, content id and a flag | R |
| 16 | `+0x98`, `+0x9c` byte | Max stack size; Is Tintable | client (cols 0x25, 0x16) |
| 17 | string at `+0x58` | Third string; by file order the tint alias | R |
| 18 | byte `+0x9d`, `+0xa0`, `+0xa4`, `+0xa8`, `+0xac` | unnamed byte, unnamed, VIP rank requirement, equip requirement set id, a second set id (alias) | client (col 0x17; `00baec20`, `00a142c0`) |
| 19 | `+0xf0`, `+0xf4`, `+0xf8`, `+0xfc` | Coin sell price ("Worth"), then three unnamed values | client (col 0x1a; the inventory "Price" column reads `+0xf0`) |
| 20 | list at `+0x108` | Item stat list: `i32` count, then per entry a key `i32` and a stat record that `008fc430` reads as a float with a NaN-or-flag convention followed by a bool (`00911a90`, `008fa830`) | client (`00911a90`); entry layout partly open |
| 21 | list at `+0xc0` | Rows of four `i32` after a count (`008e3c30`), the item's ability entries | R |

The tail after step 19 and the position of the stat and ability lists relative to steps 1 to 18 follow the read order in `00919e10` (four dwords, the stat list, then the ability rows); the first 18 steps come from `008e3760`. The 35/37 batch and the single 35/34 and 38/2 definitions all use this one record.

**Stats on items.** A stat record has the value as a float at `+8` and a type at `+0xc`: type 0 is displayed as an additive value (`ItemStatAdditiveValue`), type 1 as a multiplier, converted by `(value - constant) * scale` (`ItemStatMultiplicativeValue`, `00baeae0`). The item class decides which stat the list view shows: class `+0x14` is "List View Stat Id" and the value is looked up in the item's stat list (`00c956c0` cols 0x10, 0x11). `HashList<ItemStatDefinition,71>` holds 71 slots (S0150). This closes part of what the vault left open: the Type field selects the display, and the class field selects the stat shown in lists.

**Item class record** (0x2c bytes, 20-bucket map keyed at `+0x20`): the 44/1 body is an `i32` count, then per class a key `i32` and six `i32` stored at `+0`, `+4`, `+8`, `+0xc`, `+0x14`, `+0x18` (`008fcfa0`, `008dc010`); the text loader (`01048470`) reads seven cells into the same fields plus `+0x10`. `+0x14` is the list-view stat id and `+0x18` the owning-job name string id, shown in "Profile Name" and in requirement text. The other fields are not named by any client reader traced here.

**Item category record** (0x28 bytes, 100-bucket map, key at `+0x1c`): the 44/2 body is an `i32` count of categories, each a key `i32` and four `i32`, a bool and an `i32` (`008e98b0`, `008ce570`), then an `i32` count of category-group pairs (`008e9a10`, 0x18-byte records, 20 buckets, two `i32` each). Reads in the client: `+4` is the name id and `+8` the icon id (navigation columns 3 and 5); the bool at `+0x10` decides whether a category node appears in the navigation tree (`00a7b4d0`); the `i32` at `+0x14` must be non-zero for an item of that category to appear in the main bag (`00a6d9b0`). The parent link and the meaning of the group pairs are not read by anything traced here.

**Item instance record** (self-data entry, 38/2 first half), in order: `i32` definition id, `i32` tint override, `i32` instance id, `i32` count, `i32` consumed count, `i32` last-cast value (a signed offset the client subtracts from its clock; `0x7fffffff` means never; `008cb5a0`), `u8` rental flag, if set an 8-byte expiry, `i32` upsells sent and `i32` marketplace bundle id (`008cd5e0`), `i32` stored count, and, for self-data and 38/2, one final `u8` stored at `+0x31`. The instance id is read and printed as a signed 32-bit value (client). Nothing in the client says how large the ids are, whether they are dense or sparse, or whether the original server ever reused a retired id; the only source for that was a recorded server payload, which is not used, so the original allocation rule is unknown.

**Equipment attachment record** (38/5): three length-prefixed strings (model name, texture alias, tint alias), three `i32`, two more `i32` and a bool, behind a header and a profile id (client `0090a460`, `008e86f0`). Model names carry `<race>` and `<gender>` placeholders that the client expands (client resource: 142 of the 302 starter rows contain `<race>` and 26 contain `<gender>`; the six skin-tone rows carry a token like "SkinTone 1" instead of a model).

**Prize wheel contents** (dungeon and job content pages, wiki with client-checked names). Every wheel has one coin slot (the amount is per dungeon), a "Battle Item Mystery Pack" and, per job, a weapon named `<Job>'s <Weapon> of <Suffix>`, a necklace, a Power Shard of Regeneration with the tier numeral, and one clothing piece from the dungeon's bonus-clothing set; the six job rows (Archer, Brawler, Medic, Ninja, Warrior, Wizard) each list five items, or six at some difficulty-5 dungeons, which add a Ring. The tier numeral on necklace and shard equals the dungeon difficulty (I at Bixie Hive, III at Deep Mines, IV at Bandit Hideout, V at Snowy Canyon); in the pages checked the necklace line is "of Bark Skin" at tiers I and IV and "of Vitality" at tiers III and V. Clothing pieces include caps, helms, shoulder pads, backpacks and quivers. The bonus-goal clothing piece (random slot and colour) goes to every player in the instance, and members get an extra coin amount on the results screen (wiki). The client carries none of this: it only displays the entries it is sent.

**Locale.** Item names and descriptions arrive as locale ids (resolved with `Ui.GetStringById`); weapons and clothing are stored as `{n}Name{pl=...}` entries in the locale, so singular and plural come from one entry (dungeon pages).

### Network

Every client-to-server item sender traced here (`00bdb790`, `00bdb8a0`, `00bdbac0`, `00c22ad0` and the storage and chest senders) wraps the packet in `PacketTunneledClientPacket` (gateway id 5, `0089adc0`) after a body writer that starts with the family and sub-opcode (`00a1ada0` writes the 42-family head; see [[Gateway and Transport]]). The world-tunnel variant (gateway id 6, `0089ae70`) is used by activity and matchmaking senders, not by these. Directions are proven when a sender builds the packet (C>S) or a dispatcher case parses it (S>C). The map's `wire` field is set on only 12 of the 332 functions because the packet catalogue the validator uses lacks most of these classes.

| Fam/sub | Name (client RTTI) | Dir | Body | Handler or sender |
|---|---|---|---|---|
| message id 12 | SendSelfToClient | S>C | whole player body, includes the item list | `009471a0` case 12, `00944fe0`, `0090a630` |
| 35/34 | ItemDefinitionRequest | C>S | `i32` definition id | `008a1100`, `00a3ddd0`, `00a3dca0` |
| 35/34 | (same class used as the answer) | S>C | length-prefixed payload with one definition | case `00933501` |
| 35/37 | ItemDefinitions | S>C | length-prefixed payload: `i32` count then definitions | case `009335e3`, `00a3db70` |
| 35/6, 35/7 | EquipItemChange, EquippedItemsChange | S>C | other players' attachments; not decoded | cases `009314e3`, `0093047e` |
| 35/29 | LootEvent | S>C | string and two vectors; not decoded | case `009332fa`, `008d0a00` |
| 38/2 | ItemAdd | S>C | length-prefixed payload: instance record, definition | `009392c0`, `0089d410` |
| 38/3 | ItemUpdate | S>C | seven `i32` | `009392c0`, `008b25e0`, `008d8350` |
| 38/4 | ItemDelete | S>C | one `i32` item key | `009392c0`, `0089d6c0`, `008e3e70` |
| 38/5 | EquipItem | S>C | profile id, attachment record | `008dc810`, `0090a460` |
| 38/6 | UnequipSlot | S>C | profile id, slot | `0089d7a0`, `008f8ad0` |
| 38/19 | CoinCount | S>C | one `i32` | `0089e670` |
| 38/25 | UpdateActionBarSlot | S>C | bar, slot, slot record | `0089e7f0` |
| 38/30 | ImmediateActivationCheck | S>C | six values: success flag, guid, category id, bundle id, title string id, body string id | `0089ea30`, `00a1a7d0` |
| 38/31, 38/32 | Storage, storage-full prompt | S>C | two `i32`; none | `0089d5e0`, `009392c0` |
| 42/2 | EquippedRemove | C>S | slot, profile key | `00bda720`, `00bdb530` |
| 42/3 | EquipByGuid | C>S | instance id, profile key, slot | `00bda510`, `00bdb450` |
| 42/4 | ItemRequirementRequest | C>S | one `i32`; no Lua caller found | `00c08fa0`, `00c09d30` |
| 42/6 | ItemActionBarAssign | C>S | slot, instance id | `00c14720`, `00c23ed0` |
| 42/8, 42/9, 42/12 | EquipByItemRecord, ActionBarAssignByItemRecord, UseStyleCardByItemRecord | C>S | an `ItemRecord` (definition, tint) | `00a10470`, `00a122e0`, `00a105a0` |
| 42/10, 42/11 | UseStyleCard, PreviewStyleCard | C>S | one item value (body not read here) | `00c10890`, `00bd58c0` |
| 42/13, 42/14 | EquipLightsaber, EquipItemGroup | C>S | guid; group id (R, from the senders' names and tables) | `00bda5c0`, `00bda670` |
| 42/15, 42/16 | UseImmediateActivationItem, CheckImmediateActivationItem | C>S | one `i32` instance id | `00c10940`, `00c109f0` |
| 42/17 | MoveItemToStorage | C>S | 8-byte value, item, number | `00c1a8c0`, `00c24270` |
| ability family sub 12 | Ability definition request (for the ability list of an item) | C>S | not read here | `00a34410`, `00a2d270`, `00a7e660` |
| 42/18 | Storage show | S>C | none read | `00991730` (case `0x2a`) |
| 26/0x2c | RequestStorageCheck | C>S | none | `00c14a80`, `00c24370` |
| 44/1, 44/2 | Item classes, item categories | S>C | see Data and formats | `00928d00` (case `0x2c`) |
| 36/10 | Ability activation (item bar use) | C>S | bar, slot, target mode, target | vault, `00a35150` |
| 50/1 | Reward bundle (kind 16 = mystery chest panel) | S>C | see [[Rewards and Collections]] | vault, native run |
| 39 family | Minigame messages, loot-wheel stop | S>C | not decoded here | `009bfea0`, `009b6da0` |

Other families the item screens lean on and that have their own guides: coin store (family 165, reached from rental re-buys and the storage add-space button), trade (offering an item from the Stuff window), housing (fixture categories are hidden from the Stuff tree), pets and mounts (pet gear lives in `BaseClient.PetInventory.*`, mount items spawn a mount), claim items (family 108) and ability definitions.

### What the Rust server must do

Items marked (R) are reconstructed policy, not original behaviour.

1. **Own the item catalogue.** The client has none beyond a starter file. Keep one record per definition id with every field in the table above, locale ids for name and description, a model name with `<race>`/`<gender>` placeholders, texture and tint aliases, slot, class, category, gender usage, type, members-only, rank, VIP rank, requirement-set ids, price and worth, stack size, ability ids and stat list. Our own content pages supply the names; do not import community tables.
2. **Send reference data before inventory is used.** Item classes (44/1) and categories (44/2) first (44/2 also rebuilds the category tree and navigation), then the startup catalogue 35/37 with its **byte-length prefix** and a definition for every item the player owns, then the self body (id 12). Any item whose definition or category is missing, or whose category flag is zero, is invisible with no error.
3. **Category table.** Every category referenced by any definition must exist in 44/2 with the in-bag flag set (the `+0x14` field) for its items to appear. Send the tree with real parents once the parent field is confirmed (Open questions).
4. **Instance ids.** Unique `i32`, allocated from a durable counter and never reused after a delete (R; our own policy, chosen so that a retired id never names a different item later; the original allocation rule is unknown). Merge stacks by definition and tint and honour max stack.
5. **Persist per character.** Instances with count, consumed count, stored count, last cast, rental block; equipment per profile (slot to instance); item-bar slots; storage usage and capacity; the new-item set is client-local and needs nothing.
6. **Equip.** Accept 42/3, 42/2, 42/8, 42/10 to 42/14. Validate: the instance is owned, its definition's slot equals the requested slot (the client sends the slot the UI chose), the profile is the player's active profile, members-only definitions and slots, class, rank, gender. The client already ignores members-only equips from non-members; the server must not rely on that. Reply to the owner with 38/5 or 38/6 and to the zone with 35/6 or 35/7. Persist before replying (R, matches the order of the working native runs).
7. **Required slots and ranks.** Chest and Legs are the required slots in the shipped table; Star Rune needs rank 10, Moon Rune rank 20. Whether the original server enforced "required" on unequip is unknown; the client sends 42/2 for any slot (R: refuse to empty Chest or Legs only if a native run shows the client expects it).
8. **Add, update, delete.** Send 38/2 with the instance and its definition inline, 38/3 for count changes (the item key plus the changed counts) and 38/4 on removal. Send each once; the client has no duplicate check beyond replacing by key. Keep coins in 38/19.
9. **Definition requests.** Answer 35/34 with exactly one definition (or fold it into a 35/37). An unanswered id stays in the client's pending tree and every later request for it is dropped silently, so tooltips for that item never fill.
10. **Item bar.** Persist 42/6 assignments (zero-based slot among four), send 38/25 with an enabled flag the client does not derive from the inventory row, and answer 36/10 activations. Pick consume effects explicitly; none are recoverable from the client (R).
11. **Immediate use.** For 42/16 reply 38/30: success true when the item may be used now, otherwise false with the marketplace category, bundle and title and body string ids the client then shows (for example a chest without its key, R). On 42/15 re-check, consume atomically, grant, and send the reward bundle (kind 16 for a chest panel). A replayed 42/15 must not open a second chest without a new check (R).
12. **Storage.** Open with 42/18, publish 38/31 usage and capacity once the field order is settled, and 38/32 when full. Moving an item sets its stored count; the bag shows `count - stored - consumed`.
13. **Rentals.** Send the rental block with an expiry in the client clock's base, expire server-side, and delete or refuse the row (R). Do not rely on the client to expire the item.
14. **Loot and wheels.** Decide drops and wheel contents server-side; the client only shows them. Send the preview entries and bundle before the score screen, the awarded reward so the client computes the stop index, and the reward bundle exactly once. Per dungeon the wheel set is one coin slot plus five or six items per job with the difficulty tier (content pages); amounts are wiki data (R for exact odds).
15. **Members and VIP.** Send the membership activation message (id 202, which sets `player+0x464`) so the client knows the player is a member; it drives Equippable State 3 and the members-only equip block. VIP rank text appears only when the VIP system is enabled in the client settings.
16. **Names and tokens.** Name-change and boost-currency entitlements are derived on the client from totals of particular item definitions; those definitions must exist and be granted as items.
17. **Style cards.** Define categories 140 to 146 in 44/2 (parent and the six children), give each style card definition type `0xc` with gender usage and a category in 141 to 146, and accept 42/10 per changed slot. The panel applies one card per category, so the server should apply them as a set and send the resulting appearance (R).
18. **Fixtures and pets.** Housing fixtures are excluded from the Stuff tree; pet gear uses the pet inventory sources. Send them through their own guides' packets.
19. **Locale.** Every name, description, requirement and category string id must exist in the locale we ship or the client prints a not-found marker.

### Evidence

- **Client.** 332 accepted functions across 36 shards (S0150 to S0677): 121 confirmed, 104 likely, 56 speculative, 51 unknown; subsystems `game.item` 139, `ui.inventory` 77, `ui.tooltip` 49, `ui.items` 36, `game.inventory` 31. The table is mostly plumbing (destructors, trivial accessors, container helpers): 83 of the 139 `game.item` rows have no class, and shards S0304, S0305, S0307, S0308, S0411, S0658 and S0677 are mostly other systems (Pirates, cinematics, groups, matchmaking, STL helpers) that happen to hold a few item rows. Extra reads in this pass: the base dispatchers (`009471a0` cases 12, 38, 42, 44, 108; `0092f460` jump table; `009392c0` cases 2 to 6 and 30 to 32), the definition reader chain (`008e3760`, `00919e10`, `00911a90`, `008e3c30`), the instance reader (`008fa4b0`, `008cb540`, `008cb5a0`, `008cd5e0`), the reference-data reader (`00928d00`, `008fcfa0`, `009255c0`, `008e98b0`, `008e9a10`), the equip senders (`00bdbc50` family), the usability and requirement code (`00a15690`, `00a142c0`, `00baec20`, `00baeae0`), the definition column getter `00c956c0`, the inventory predicates (`00a6d9b0`, `00a6db90`) and the storage, loot-event and chest paths.
- **Client Lua and resources.** `ScriptsBase.lua` for `InventoryHandler`, `StorageHandler`, the equipment panes, the 31-column and 25-column data sources and `ScoreScreen`; the shipped `Resources` files in the table above; `Assets_manifest.txt` for the loot props; the locale for every string quoted.
- **Video.** 87 frames with screen "inventory" in 44 videos; two prize-wheel frames (Robgoblin Treasure Trove, Artifact Hunt) and one prize pop-up; 40 "You receive 1" frames. All are curated reads, not counts of every frame.
- **Wiki and content pages.** The dungeon pages (70 pages, all with a prize-wheel section; 39 list the coin slot and 28 state the gear tier) and the job pages supply the wheel contents, the I to V tier rule and member bonuses; the wiki is secondary and its numbers are not used for rules.
- **Vault.** Used for client-derived facts only: Inventory Catalog Framing Compatibility (35/37 byte-length prefix, proven by our own restart 172324 and later native runs; the note's comparison with a recorded template and a community writer is not used), Native Inventory Repair 173814 (native acceptance of All Items, Hats and Chef Head), Pieces 081 (only our own durable instance-id counter that replaced the allocator which reused retired ids; the note's sparse-id evidence comes from a recorded server payload and is not used), 082 (inventory search crash, still without a fault address), Mystery Chest Restoration (42/15 and 42/16, 38/30, kind 16), Original Inventory Food and Potion Use (the native-byte parts: 42/6, 36/10, the 38/25 reader). Not used for facts: Pieces 045, 047 and 057, whose slot, category, stat and drop tables come from community catalogues; the community packet writers and catalogue rows named in the other notes; and any FreeRealms-Legacy or OpenSourceFreeRealms material. Their client-derived corner facts were re-checked in the binary here instead.
- **What is verified and what is reconstructed.** Verified in the original client by our native runs: startup catalogue load, All Items, Hats and Chef Head browsing, equipped Student Chef Shirt and Hat visible and persisted, Mystery Chest check, confirm, consume and reward panel, restart with chest and key counts intact. Live and exercised in native runs but reconstructed policy: the durable instance counter (the economy run minted ids 36 to 51 without reusing retired ones) and the grant of earned items. Reconstructed and accepted only at unit level (artifacts, not integrated or not run natively): the item-bar request probe, cosmetic validation, gear-stat tables and the loot ledger. Not yet accepted natively: consumable effects, item-bar assignment and activation, storage, rentals, loot drops and wheels (selling and buying are in [[Coin Store and Purchases]]).

### Open questions and contradictions

- **No original server code.** Catalogue contents, ids, gear stats, drop tables and consume effects are inferred. The vault's native runs prove the client accepts the reconstructed messages, not that the original sent them.
- **Slot numbering.** The Lua `InventorySlotMap` has 21 entries (slots 1 to 17 plus four consumable positions 18 to 21), while the resource table has 20 rows whose ids 18 to 20 are customization slots. The two use the same numbers for different things. The Lua `EquipmentSlotLabels` array (26 entries starting with two "unused") lines up with the table as label index = slot + 2 for slots 1 to 5, 7 to 13 and 15 to 17; slot 6 reads "Misc" (the table says Shoulders) and slot 14 reads "Costume" (the table says Star Rune). The vault said the alignment was unproven; it is now supported for 15 of 17 positions. How the array is indexed at run time was not traced.
- **Shoulders versus Misc.** The slot table names slot 6 "Shoulders" (client locale 16364) and the Lua labels the same position "Misc" (412828); video inventory frames also show a "Misc" label (8 mentions). Which one the player sees on that slot was not captured.
- **Stat entry encoding.** The client reads a stat's value as a float and a type of 0 or 1 selects the display (`00baeae0`). Which numeric form the original server wrote for each type is unknown (an earlier vault claim of an integer for type 0 came from community sources and is not used). The wire order inside a stat entry (`008fc430` is labelled "float, NaN flag, then bool") was not resolved. Treat the stat list as undecoded on the write side.
- **Definition fields not named.** Steps 2, 3, 5, 8, 15, 17, 18 and 19 above are position-based (R); the values at `+0x78`, `+0x88`, `+0x8c`, `+0x90`, `+0x94`, `+0x9d`, `+0xa0` and `+0xf4` to `+0xfc` have no client reader traced here.
- **Item category parent and group pairs.** The category record has four `i32` and a bool and the packet carries group pairs, but no traced reader uses a parent id or the group pairs (an earlier vault count of category ids that group rows reference without defining them, and of definitions whose category is absent, came from a recorded server payload and is not used; whether the original data had such gaps is unknown, but under the row predicate any item whose category is absent would be hidden).
- **Shipped item groups are unnamed.** `ItemGroups.txt` has 28 rows with every name, description and icon id zero, and the mappings file lists 1,796 items in them. Which group is which is not in the client. The strings `ItemGroup.PlayerCustomization{Hair, HairColor, FacialHair, SkinTone, EyeColor, FaceDesign, Species}` in `00bd9e20` name seven customization data sources (see [[Character and Customization]]); whether they use these 28 groups is open. The Style Cards panel does not use item groups; it uses inventory categories 140 to 146.
- **`InventoryNavigationDataSource` role.** Its columns (Total, Owned, On Sale, Currency Type, Tag) look like a catalogue browser; whether the Stuff window ever binds to it is not shown. The Stuff window binds `InventoryCategoriesData`.
- **Search crash.** The vault's search note names `00a6e7b0` as the predicate and the search text as wide-character at `+0x2f0`. The shard S0304 found `00a6e7b0` is the rebuild loop, the predicate is `00a6d9b0` (`00a6db90` for storage), and the search text is a narrow string at `+0x2ec`. The crash cause is still open; `BaseItemSearchEngine` is not shown to be on that path (S0388).
- **42/4 ItemRequirementRequest.** The packet and its native handler exist, but no client Lua caller was found and no reply form is known. The immediate-use check (42/16 to 38/30) is the working path.
- **Storage fields.** Which of the two `i32` in 38/31 is usage and which is capacity, and what 42/17's last number means (a count or a direction), were not traced.
- **Rental clock.** The expiry is compared with a cached 64-bit clock, but the unit and epoch were not confirmed against a live value.
- **Equip-by-record routing.** The case values of `00a1ec00` are known (`1, 2, 3, 4, 7, 0xc, 0xf, 0x10, 0x13, 0x1b, 0x1c, 0x1d`); which type is which (equipment, style card, consumable, housing, mount, pet, chest) is only partly proven. Types `0xc` (style card) and `0x11` (customization) come from the factory, and `0x1c` is the chest type per the vault's native run.
- **Consume effects.** The client's generic text for health potions says healing over five seconds, the wiki says three-second values for three potions; no original effect database exists. Food effects (the strength stew) have no recovered numbers.
- **What the New flag does on the wire.** `+0x30` and `+0x31` are set by the add path and the self-body; which one is "new" and which is "in storage" is read from the data-source columns but not proven.
- **Prize wheel packets.** The score-screen Lua is complete but the packets that carry the preview entries and the stop result sit in the minigame family and were not decoded here; `Ui.SendLootWheelRotationStopPacket`'s sender was not traced. A separate daily "coin wheel" (`HUD:ShowCoinWheel`, locale "Come back tomorrow for another spin!") exists and is not covered here.
- **Loot drops.** No drop table, odds or quantity exists in the client. The 35/29 loot event's fields and the pickup interaction (bag, chest, coin props) are undecoded.
- **Wheel contents.** The coin slot per dungeon is wiki data and is not consistent with difficulty (Howling Hills, 343 coins at difficulty 3, is far above its tier); the wiki gives the members-only bonus coin amount but the client has no matching "Bonus: N Coins" strings in several cases (see the dungeon pages).
- **Starter file versus live catalogue.** `CharacterCreate/ItemDefinitions.txt` has 302 rows; our native runs sent a prepared startup catalogue (the vault records 1,647 definitions; its source is not claimed here). The client does not say what the original catalogue size was, and nothing here relies on that figure.
- **Duplicated or mislabelled functions.** `009921a0` and `00991430` are both `ClientPlayerItemManager::Refresh`; `00724660` to `007247f0` are sixteen speculative field accessors present in the vtable; `00724880` and `007249c0` are identical return-argument stubs. `00a6e0a0`, `00a6dcf0`, `00a6df60` and `00a6e090` are adjustor thunks. The `game.item` rows `00a156f0` and `00b2a4d0` are `speculative`; `00b2a4d0` (`CheckItemRecordConditions`) is not tied to any feature.

### Key functions

| Address | Role |
|---|---|
| `009471a0` | Main inbound dispatcher; case 12 self body, 38, 42, 44, 108, 165 and others |
| `00944fe0`, `0090a630` | Self-body reader and its item-list reader |
| `0092f460`, `00934618` | Player-update dispatcher and its jump table (35/6, 35/7, 35/29, 35/34, 35/37) |
| `009392c0` | `HandleClientUpdate`, family 38 (item add, update, delete, equip, unequip, storage, immediate-use check, action bar) |
| `00928d00` | Reference-data dispatcher (44/1 item classes, 44/2 categories, 44/3 profile data) |
| `00a3db70`, `00a3c290`, `00a3ddd0`, `00a3dca0`, `00a3df00` | Definition batch read, cache insert, request, tunnel send, lookup-or-request |
| `008e3760`, `00919e10`, `00911a90`, `008e3c30` | Definition record readers |
| `008fa4b0`, `00908eb0`, `008cb540`, `008cb5a0`, `008cd5e0` | Item instance record readers |
| `009918d0`, `009912f0`, `00991690`, `00991630` | Item manager constructor, derived-state refresh, remove and add |
| `0103dc60`, `0103e5e0`, `0103dfe0`, `0103df50` | `ItemInstance` constructor, format, rental-expiry test, rental block |
| `00a6d9b0`, `00a6db90`, `00a6e910`, `00a6e7b0` | Bag and storage row predicates, refresh, row-rebuild loop |
| `00a6e9d0`, `00a6e480`, `00a7c6e0`, `00a7b4d0`, `00a7ba90` | Category tree and navigation data sources |
| `00c92c10`, `00c956c0`, `00cb09d0` | Definition and inventory column names and values |
| `00bdbc50`, `00bda510`, `00bda720`, `00bdb450`, `00bdb530`, `00c10bf0` | Equip: native body, 42/3 and 42/2 constructors and writers, Lua entry |
| `00a1ec00` | Equip and use by item record list |
| `00bdbe30`, `00bdbf90`, `00bdc060`, `00bdc420` | Lightsaber, item-group equip, use selected item, group array |
| `00c23ed0`, `00c14720`, `00c22ad0`, `00c20a30` | Item-bar assign and its packet |
| `00c109f0`, `00c10940`, `0089ea30`, `00a1a7d0`, `00a1a4b0` | Immediate-use check, use, 38/30 packet and handlers |
| `00c24270`, `00c1a8c0`, `00c24370`, `00c14a80`, `00991730`, `0089d5e0` | Storage move, check and open, storage update |
| `00a15690`, `00a142c0`, `00a156f0`, `00baec20`, `00baeae0` | Equippable state, requirement text, stat display |
| `00a799d0`, `00a76a60`, `00a77d40`, `00a77b40`, `00a76700` | Item tooltip by id, builders, request hook |
| `00c30040`, `00c30a70`, `00c2f490`, `00c09a80` | Item search engine and the Lua search entry |
| `007989a0`, `00724b80`, `01048570`, `010456c0`, `0107f4e0` | Slot table, starter definitions, item class, category and filter loaders |
| `00a3dab0`, `00a3d350`, `00a3cf00` | Item group source and its two file loaders |
| `009b6da0`, `009bfea0`, `008d0a00`, `009332fa` | Loot-wheel stop, minigame dispatcher, loot-event packet and case |
<!-- NARRATIVE END -->

## Mapped functions (generated)

332 accepted functions: 121 confirmed, 104 likely, 56 speculative, 51 unknown. Network-linked (`wire`): 12.

Subsystems: `game.item` (139), `ui.inventory` (77), `ui.tooltip` (49), `ui.items` (36), `game.inventory` (31)

### `game.inventory`

**ClientItemRootDataSource** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0098f7e0` | ClientItemRootDataSource::ScalarDeletingDtor | confirmed | Calls the base cleanup routine and frees the object when the deleting flag is set; otherwise returns the object pointer. |

**ClientPlayerItemManager** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009912f0` | ClientPlayerItemManager::RefreshDerivedState | confirmed | Notifies the manager listener, refreshes boost-currency and name-change derived values from item totals, then checks the Factory.Tools data source and updates … |
| `00991430` | ClientPlayerItemManager::Refresh | confirmed | Calls the client item manager's derived-state refresh routine and returns. |
| `009918d0` | ClientPlayerItemManager::Ctor | confirmed | Initializes the generic item manager, installs the client manager vtable, creates named inventory/search/storage data sources and lists for equipment and vehic… |
| `009921a0` | ClientPlayerItemManager::Refresh | confirmed | Calls the client item manager derived-state refresh and returns. |
| `009921c0` | ClientPlayerItemManagerDtor | confirmed | Restores the client manager vtable, tears down the base item manager, releases the inventory-derived data-source references, clears vehicle/equipment lists, an… |
| `009924e0` | ClientPlayerItemManagerScalarDeletingDtor | confirmed | Calls the client item manager destructor, conditionally frees the object when the deleting flag is set, and returns the original pointer otherwise. |
| `00991440` | — | unknown | Calls the derived-state refresh, then sets character, pet, and house name-change properties to true when the changed key matches each corresponding item key. |

**PlayerItemManager<ClientItem>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0098fb10` | PlayerItemManagerCtor | confirmed | Installs the manager vtable, zeroes two 100-bucket tables and associated fields, and sets the final initialization flag to 1. |
| `00991860` | PlayerItemManagerDtor | confirmed | Reinstalls the generic player item manager vtable, prepares exception cleanup state, calls base item-manager teardown, and restores the exception frame. |
| `009918b0` | PlayerItemManagerScalarDeletingDtor | confirmed | Calls the manager destructor and frees the object when the deleting flag is set; otherwise returns the original pointer. |

**(no class)** (20)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00991690` | RemoveItemAndRefreshNamePermission | confirmed | Finds an item record, refreshes NameChange.CanChangeName when the removed key matches the name-change item key, runs item cleanup, recycles the item, and notif… |
| `00c23ed0` | AssignToItemActionBar 🌐 | confirmed | Validates two indexed arguments, constructs InventoryPacketItemActionBarAssign from the slot and item instance, and sends through the gateway wrapper if the co… |
| `00bdaae0` | NodeDefinitionUsabilityResultByGuid | likely | Looks up the inventory node whose key (+0xb4) equals the guid param_2 in the 1024-bucket table at *(*this+0x20c)+0x64 and, when found, calls FUN_00a15690 on it… |
| `00bdab30` | NodeFlagAndKindPredicate | likely | Looks up the node by guid in the 1024-bucket table; returns 0 when absent or when the byte at node+0x9d is nonzero. It switches on node+0x6c: kind 1 requires a… |
| `00bdac10` | InventoryNodeRemainingCount | likely | Looks up the inventory node by guid in the 1024-bucket table at *(*this+0x20c)+0x64; when found, returns the node field at +0x98 unchanged if it is -1, otherwi… |
| `00bdae10` | InventoryNodePlayerEligibilityPredicate | likely | Looks up the guid in the 1024-bucket node table. For node kind 0xc it compares node+0x68 with the field at player+0x458, allowing node values 0, or 1/2 only wh… |
| `00bdaf00` | ItemGroupUsabilityAll | likely | When *(*this+0x1c8) is non-zero and 00c388b0(param_2) resolves: if the set field at +0x118 is non-zero, returns whether it equals 2; otherwise walks the set li… |
| `00bdbe30` | EquipLightsaber_ByGuid 🌐 | likely | Looks up the inventory node by guid in the 1024-bucket table and returns when absent; otherwise constructs a 0x2a0-byte scratch with 009e28e0, resolves it for … |
| `00bdbf90` | EquipItemGroup_ById 🌐 | likely | Walks the 10-bucket item-group table at *(*this+0x20c)+0x20 (bucket id % 10, node key at +4, chain next at node[0]) and returns when the id is absent; when fou… |
| `00bdc060` | UseItemSelectedInUi 🌐 | likely | Takes an ItemRecord-shaped argument whose field +4 is the item guid; when 00bdae10 marks it usable it looks the node up in the 1024-bucket table and, when the … |
| `00c24270` | MoveItemToStorage | likely | Reads two values, obtains the current local player object, builds an InventoryPacketMoveItemToStorage instance through 00c1a8c0, and wraps it when command serv… |
| `00c24370` | RequestStorageCheck | likely | Constructs CommandPacketRequestStorageCheck with zero, then sends the wrapped command if the client command service is active. |
| `00bdc3c0` | ClearPreviewAndFlushMount | speculative | Calls FUN_008a6660 and FUN_00b53690 (cached pet-entity refresh), and when the byte at *(*(*this+0xc8)+0x18d5) is zero calls FUN_0095d6c0; then clears the pendi… |
| `00bdf150` | ItemProducerVisible | speculative | Returns 0 when the argument is null; when the argument field at +0x34 is non-null it calls FUN_00bd1180 with that value and the dword at *(*(this+0x18cbc)+200)… |
| `00bdac80` | — | unknown | Resolves the item set for param_2 with 00c388b0(param_1,param_2); when the set exists and its count at +0x58 is positive it walks the set list at +0x50 (next a… |
| `00bdad00` | — | unknown | When *(*this+0x1c8) is zero or 00c388b0(param_2) does not resolve, returns 4; otherwise walks the set list at +0x50 and, for each entry guid at +0xc, looks the… |
| `00bdadb0` | — | unknown | When *(*this+0x1c8) is non-zero and 00c388b0(param_2) resolves, walks the set list at +0x50 and returns the first non-zero result of 00bdab30(entry guid at +0x… |
| `00bdbc50` | — 🌐 | unknown | Native body of the UI action EquipItemSelectedInUi(itemDefinitionName, slot): copies the name string into an SoeUtil::IString, resolves it with 009909c0 (retur… |
| `00bdc420` | — 🌐 | unknown | When 00bdaf00 approves the item set for the group id it builds an array of ItemRecord entries with FUN_00a035f0, walking the group's item list from 00c388b0 (e… |
| `00bdc7f0` | — | unknown | Calls 00bdc3c0 and then the preview-id setter FUN_00a10750(-1). |

### `game.item`

**BaseDefinitionManager** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00722720` | BaseDefinitionManager::ScalarDeletingDestructor | confirmed | Installs BaseDefinitionManager::vftable into this and calls FUN_00401c00 (FreeMemory) when bit 0 of the flag byte is set. |
| `00720d50` | BaseDefinitionManager::~BaseDefinitionManager | likely | Writes BaseDefinitionManager::vftable into *param_1 and returns; the only callers are EH unwind funclets. |

**BaseItemDefinition** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00723040` | BaseItemDefinition::~BaseItemDefinition | confirmed | Installs BaseItemDefinition::vftable, destroys the three embedded SoeUtil strings through 00722bf0 and finally rewrites the Loggable vtable of the embedded Log… |
| `007230f0` | BaseItemDefinition::BaseItemDefinition | confirmed | Installs BaseItemDefinition::vftable, clears the leading fields, constructs the embedded IconData at +0x10 through 00720bf0(0,0), clears the trailing scalar fi… |
| `007230d0` | BaseItemDefinition_ScalarDeletingDtor | likely | Calls the BaseItemDefinition destructor body 00723040 for this and calls FUN_00401c00 (FreeMemory) when bit 0 of the flag byte is set. |

**BaseItemSearchEngine** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c2fa70` | BaseItemSearchEngineDtor | confirmed | Restores engine vtable, clears its client-profile hash map, releases a framework object, restores the token-array vtable, destroys tokens and frees token stora… |
| `00c30040` | BaseItemSearchEngineCtor | confirmed | Installs the engine vtable, copies eight globals, initializes the ItemSearchToken array and supporting map, resets a byte flag, and returns this. |
| `00c30130` | BaseItemSearchEngineScalarDeletingDtor | confirmed | Calls engine-member teardown and frees the object only when flag bit 0 is set. |

**ClientItem** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b1e20` | ClientItem::~ClientItem | confirmed | Destructor body: stores ClientItem::vftable into *this and then runs the base-object cleanup at 0103dd90 (which resets ItemInstance::vftable, frees the pointer… |
| `008b1e70` | ClientItem::ScalarDeletingDestructor | confirmed | Deleting destructor: stores ClientItem::vftable, runs the base cleanup at 0103dd90, then calls operator delete (_free) on this when bit 0 of the flag argument … |

**ClientItemDefinitionManager** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a3d6e0` | ClientItemDefinitionManager::ClientItemDefinitionManager | confirmed | Constructor of the item-definition manager: installs ClientItemDefinitionManager::vftable, stores the owning client at +4, zeroes +8, initialises the member bl… |
| `00a3d870` | ClientItemDefinitionManager::FindDefinitionById | confirmed | Uses id&0x3ff to select the bucket at this+0x64, walks node+0xb0 links, returns the node whose key at +0xb4 matches the id, or returns null when the chain ends. |
| `00a3d8a0` | ClientItemDefinitionManager::~ClientItemDefinitionManager | confirmed | Destructor body: installs the manager vtable, empties the definition cache with 00a3b530, deletes the lazy global object DAT_01be69c4 through its vtable slot 0… |
| `00a3ddd0` | ItemDefinition_RequestById 🌐 | confirmed | Request-if-missing path for one definition id: it searches the cache chain at this+0x64 (key at +0xb4, chain +0xb0) and on a hit calls FUN_00c93c90; else it ch… |

**ClientItemFilterManager** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0107f4e0` | — | unknown | Clears the list at this+0xb0 through 0107dce0, then loads three text tables with the string formatter and the row reader: ItemFilters.txt (rows split on caret,… |

**GameItemEntry** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01060e60` | GameItemEntry::GameItemEntry | confirmed | Fastcall with the 0x18-byte entry in ECX. Stores Loggable::vftable, then GameItemEntry::vftable at +0, stores ItemRecord::vftable at +4, zeroes the words at +8… |
| `01060ed0` | GameItemEntry::~GameItemEntry | confirmed | Fastcall with the entry in ECX. Stores GameItemEntry::vftable at +0, ItemRecord::vftable at +4, then Loggable::vftable at +0. Destroys no members. Plain ret. |
| `01060f30` | GameItemEntry::GameItemEntry | confirmed | Thiscall with two stack arguments (ret 8): a pointer to a source record and a word. Stores Loggable::vftable, GameItemEntry::vftable at +0 and ItemRecord::vfta… |
| `01060fb0` | GameItemEntry_ScalarDeletingDtor | confirmed | Thiscall with a flag byte (ret 4). Calls the GameItemEntry destructor body 01060ed0 on this, then when bit 0 of the flag is set calls _free on this. Returns th… |
| `01061000` | GameItemEntry::GameItemEntry | confirmed | Thiscall with one stack argument, a source GameItemEntry pointer (ret 4). Stores Loggable::vftable, GameItemEntry::vftable at +0 and ItemRecord::vftable at +4,… |

**IconData** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00720bf0` | IconData::IconData | confirmed | Installs Loggable::vftable and then IconData::vftable into param_1 and stores param_2 at +4 and param_3 at +8. |
| `00720cd0` | IconData::Print | confirmed | Logs the header --- IconData --- and then ImageSetId=%d with the dword at +4 and TintId=%d with the dword at +8 through 00720c60. |

**ItemDefinition** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0072e4d0` | ItemDefinition::DtorBody | confirmed | Installs ItemDefinition::vftable, drains the node chain at this+0xdc through 00722b20 (one head at a time), stores the SoeUtil::List<ItemDefinition::ItemAbilit… |
| `00724300` | — | unknown | Constructs the BaseItemDefinition base via 007230f0, installs ItemDefinition::vftable, clears +0xb0..+0xd4 (two dwords default to 1), constructs the embedded H… |

**ItemDefinitionCustomization** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00724960` | ItemDefinitionCustomization::ItemDefinitionCustomization | confirmed | Constructs the ItemDefinition base through 00724300 and then rewrites the vtable to ItemDefinitionCustomization::vftable. |

**ItemDefinitionManager** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007218e0` | ItemDefinitionManager::ItemDefinitionManager | confirmed | Constructs the definition-manager base chain in place: writes BaseDefinitionManager::vftable then BaseItemDefinitionLookup::vftable into +0/+4 and then the two… |
| `00722750` | ItemDefinitionManager::ScalarDeletingDestructor | confirmed | Restores the two ItemDefinitionManager vtables at +0/+4, the BaseDefinitionManager vtable at +0, and frees through FUN_00401c00 when bit 0 of the flag byte is … |
| `00720d60` | ItemDefinitionManager::~ItemDefinitionManager | likely | Restores ItemDefinitionManager::vftable at +0 and at +4 (the secondary base slot) and then writes BaseDefinitionManager::vftable at +0, i.e. the base-object de… |
| `007227c0` | ItemDefinitionManager::RemoveAll | likely | Walks the doubly-linked list from this+4 (tail fields +0x250/+0x254 on the nodes): unlinks every node from the 10000-bucket hash (00721490), fixes the neighbou… |

**ItemDefinitionStyleCard** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00724820` | ItemDefinitionStyleCard::ItemDefinitionStyleCard | confirmed | Constructs the ItemDefinition base through 00724300 for the given object and then rewrites the vtable to ItemDefinitionStyleCard::vftable. |

**ItemInstance** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0103dc60` | ItemInstance::ItemInstance | confirmed | Constructor body: stores ItemRecord::vftable then ItemInstance::vftable into *this, zeroes +4, +8, +0x14, +0x18, +0x1c and +0x28, zeroes the dword at +0x10 thr… |
| `0103dcf0` | ItemInstance::ItemInstance | confirmed | Three-stack-argument constructor (ret 0xC): stores ItemRecord::vftable then ItemInstance::vftable into *this, stores arg1 at +4, arg2 at +8, arg3 at +0x14, zer… |
| `0103dd90` | ItemInstance::~ItemInstance | confirmed | Destructor body (no delete): stores ItemInstance::vftable, frees the pointer at +0x28 through _free when it is non-null, then stores ItemRecord::vftable. Retur… |
| `0103de60` | ItemInstance::SetField14ClampTrack18 | confirmed | Virtual slot 3 of ItemInstance: clamps the argument at 0, stores it at this+0x14, and when it is below the +0x18 dword lowers +0x18 to it, otherwise clamps +0x… |
| `0103e110` | ItemInstance::ScalarDeletingDestructor | confirmed | Scalar deleting destructor: calls the ItemInstance destructor body 0103dd90, then calls _free on this when bit 0 of the flag argument is set; returns this. |
| `0103e460` | ItemInstance::ItemInstance | confirmed | Copy constructor: stores ItemRecord::vftable then ItemInstance::vftable into *this, zeroes the dword at +0x10 through 0101cd40 (ECX = this+0x10), zeroes +0x28,… |
| `0103e590` | ItemInstance::FormatShortString | confirmed | Virtual slot 2 of ItemInstance: formats an item string into the output object at param_2 through 00402fd0 using the Guid form (Guid, Definition, Tint, Count) w… |
| `0103e5e0` | ItemInstance::FormatFullRecordString | confirmed | Virtual slot 4 of ItemInstance: formats the full item text through 00402fd0 with the fields Guid (+0x10), Definition (+4), TintOverride (+8), Count (+0x14), Co… |
| `0103e130` | — | unknown | Copies a source ItemInstance (param_2) into this: +0x10 and +4, +8, +0x20, +0x24 are copied; +0x14 is set through vtable slot 3 (0103de60) from the source +0x1… |

**ItemRecord** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b1e00` | ItemRecord::Field4PositiveAndField8NonNegative | confirmed | Returns 1 when the signed dword at this+4 is greater than zero and the signed dword at this+8 is >= 0, and 0 otherwise. It reads only those two fields, calls n… |
| `009ff220` | ItemRecord::~ItemRecord | confirmed | Stores ItemRecord::vftable at *this and returns; it performs no field cleanup or allocation. |
| `00c707e0` | ItemRecord::CopyCtor | confirmed | Stores ItemRecord::vftable, then copies the five words at source +4..+0x14 into words [1]..[5] of the new object (six words total). Returns nothing. |
| `01045c10` | ItemRecord::FormatDebugString | confirmed | Thiscall, one stack arg (param_2, an IString-like object). Calls 00402fd0 with param_2, the format string "ItemRecord: {Definition: %d, Tint: %d}" and the dwor… |
| `01086a40` | ItemRecord::DtorBodyAt20 | confirmed | Fastcall, ECX = the owning object. Installs an SEH frame, sets state -1, stores ItemRecord::vftable (0x18119b0) at this+0x20 (the embedded ItemRecord base), re… |
| `01086b20` | ItemRecord::DtorBodyAt20_Alt | likely | Fastcall, ECX = the owning object. Same body as 01086a40 (SEH frame, state -1, store ItemRecord::vftable at this+0x20, restore the frame, return); it has no ca… |
| `00b2a4d0` | CheckItemRecordConditions | speculative | Checks several runtime prerequisites and item flags, consults item definitions for linked records, and returns 0, 1, or 2 according to the tested conditions. |

**ItemStatDefinition** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0072da50` | ItemStatDefinition::ScalarDeletingDtor | confirmed | Installs ItemStatDefinition::vftable, then BaseItemStatDefinition::vftable, and when bit 0 of param_2 is set frees this through 00401c00 with alignment 0. Retu… |
| `0072dcc0` | ItemStatDefinition::ItemStatDefinition | speculative | SEH-framed constructor: runs the base constructor 0072d420 on this, then stores the dword *param_2 (the key) at this+0x28. Returns this. |

**ProxiedCharacterAttachmentGroup** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bf6d60` | ProxiedCharacterAttachmentGroup::Ctor | confirmed | Constructor body. Calls 01051490 with the forwarded arguments, stores ProxiedCharacterAttachmentGroup::vftable, stores arg4 at this+0x174, clears the byte at +… |
| `00bf75d0` | ProxiedCharacterAttachmentGroup::CheckDuplicateSlots | confirmed | Shared vtable slot [21] of the three attachment group classes. Builds a bool table (Array<bool,32,1>, 00bf6b70) and walks the chain at this+0x18 (slot index at… |
| `00bf6980` | AttachmentGroupSlot4Forward | speculative | Shared vtable slot [4] of ProxiedCharacterAttachmentGroup, PlayerSwitchProfileAttachmentGroup and GuiModelAttachmentGroup. Forwards its stack arguments to the … |
| `00bf6e10` | — | unknown | Shared vtable slot [20] of ProxiedCharacterAttachmentGroup, PlayerSwitchProfileAttachmentGroup and GuiModelAttachmentGroup. Sets this+0x6a = 1, then walks the … |
| `00bf7240` | — | unknown | Shared vtable slot [17] of the three attachment group classes. Returns 1 early when its flag byte at +0x9c is clear. When the group has no entries (count at +0… |

**RecipeComponentData** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c8b7c0` | RecipeComponentData::Copy | confirmed | Copies fields and embedded icon data, initializes an empty intrusive component list, then replaces that nested list from the source. |

**SoeUtil::HashListMap<int,ItemFilterDefinition,32,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0107cf90` | SoeUtil::HashListMap::Construct | confirmed | Fastcall constructor (this in ECX, returns this): stores the HashListMap vtable for ItemFilterDefinition, zeroes +0x10, +8 and +0xc, clears the 0x80-byte bucke… |

**SoeUtil::HashMap<int,ItemFilterOptionDefinition,256,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0107cfc0` | SoeUtil::HashMap::Construct | confirmed | Fastcall constructor (this in ECX, returns this): stores the HashMap vtable for ItemFilterOptionDefinition, zeroes +8 and +4, and clears the 0x400-byte table a… |

**(no class)** (83)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00724b80` | LoadItemDefinitions | confirmed | Opens ItemDefinitions.txt through the text-file reader, and per non-comment line tokenizes the id and the type id with two 007065d0 calls into the same line bu… |
| `00a3c290` | ItemDefinition_OnReceived | confirmed | Item-definition receive path: logs "Received item definition data, id: %d." with the id at definition+4, searches the 1024-bucket cache at this+0x64 for an ent… |
| `00c2f490` | ParseQuotedWideToken | confirmed | Skips leading whitespace, parses quoted tokens with doubled-quote escapes or unquoted text through CR/LF/space, trims unquoted trailing whitespace, and passes … |
| `00c952f0` | HashListMapScalarDeletingDtor | confirmed | Destroys the item-entry hash-list map and frees its object when the deleting flag's low bit is set. |
| `007217b0` | CallItemStatDefinitionDtor | likely | Under an SEH frame forwards its incoming this pointer (ECX) straight into the ItemStatDefinition/BaseItemStatDefinition destructor 0072d490 and returns; nothin… |
| `007225e0` | ScalarDeletingDtor_ItemStatDerived | likely | Under an SEH frame calls the ItemStatDefinition destructor 0072d490 for this and, when bit 0 of the flag byte is set, calls _free; returns this. |
| `007233b0` | DefinitionList_Append | likely | Appends definition node param_3 at the tail of the list (head this+4, tail this+8), stores the id param_2 at node+0x24c, prepends the node to the 10000-bucket … |
| `00724800` | ScalarDeletingDtor_ItemDefinition | likely | Calls the ItemDefinition destructor 0072e4d0 for this and frees through FUN_00401c00 when bit 0 of the flag byte is set. |
| `007248b0` | ItemDefinitionDtorCall | likely | Under an SEH frame forwards its incoming this pointer to the ItemDefinition destructor 0072e4d0 and returns. |
| `00724900` | ScalarDeletingDtor_ItemDefinition_B | likely | Under an SEH frame calls the ItemDefinition destructor 0072e4d0 for this and frees through FUN_00401c00 when bit 0 of the flag byte is set. |
| `007249d0` | ItemDefinitionDtorCall_B | likely | Under an SEH frame forwards its incoming this pointer to the ItemDefinition destructor 0072e4d0 and returns; second trampoline instance. |
| `00724a20` | ScalarDeletingDtor_ItemDefinition_C | likely | Under an SEH frame calls the ItemDefinition destructor 0072e4d0 for this and frees through FUN_00401c00 when bit 0 of the flag byte is set; third variant. |
| `00724a80` | NewItemDefinition | likely | Allocates 600 (0x258) bytes through the memory-manager chain and constructs by type id param_2: 0xc allocates through the global manager path and constructs an… |
| `00990d80` | CollectMatchingItemCandidates | likely | When param_3 is nonzero, initializes temporary key arrays, derives keys from records in param_1's chain, invokes ItemSearchEngine helpers with param_2, scans k… |
| `00a15690` | ItemDefinition_EvaluateUsability | likely | Dispatches on the item definition type at +0x6c: type 3 goes to FUN_00a125d0; type 1 passes through FUN_00a12580 on its slot id at +0x30 first; types 0xc and 7… |
| `00a1ec00` | ProcessItemRecordListActions 🌐 | likely | Walks input record lists from the head at param_2+4 through next at +0x10, resolves each handle in the base client's item-record hash table using a 10-bit buck… |
| `00a3ac00` | ItemDefinitionCache_Construct | likely | Zero-initialises the item-definition cache header: clears the pointer at +0, the counters at +4/+8/+0xc, and 0x1000 bytes of bucket heads starting at +0x10 (10… |
| `00a3b000` | ItemDefinitionCache_UnlinkFromList | likely | Removes one cache node from the ordered list: it rewires the node's predecessor link at +0xb8 and successor link at +0xbc into the list head held at container+… |
| `00a3b070` | ItemDefinitionCache_UnlinkFromBucket | likely | Removes one cache node from its hash bucket: the bucket is selected by (node key at +0xb4) & 0x3ff within the 1024 bucket heads starting at container+0x10, the… |
| `00a3b390` | MemberInit_ZeroFields0_44 | likely | Member initialiser: stores the argument in the first field (+0) and zeroes the following 17 dwords up to +0x44, leaving the rest of the object untouched. |
| `00a3b410` | FindListedRecordValueById | likely | Walks the list from container+8 via node+0xc; for each node with flag +0x40 clear, it searches the 32-bucket table at node+0x70 using id&0x1f and +0x10 chain l… |
| `00a3b460` | ListUnlinkAll | likely | Drains a doubly-linked list described by head at +0, tail at +4 and count at +8: for every node it splices the node out using the +0x10 predecessor and +0x14 s… |
| `00a3b530` | ItemDefinitionCache_RemoveAll | likely | Empties the item-definition cache: for every node starting at the head held in container+4 it unlinks the node from its bucket chain ((key & 0x3ff) at +0x2c/+0… |
| `00a3c1f0` | ItemDefinitionCache_InsertNode | likely | Inserts a cache node: links it at the tail of the ordered list (+0xb8 previous from container+8, +0xbc written through, container tail +8 updated), stores the … |
| `00a3db70` | ItemDefinitions_ReadBatchFromReader 🌐 | likely | Bulk definition reader: takes the remaining-size field at this+0x10 and the cursor at this+0xc, reads a 32-bit count, then for each element allocates 0x230 byt… |
| `00bf6a60` | ListIntContainsAt180 | likely | Walks the singly linked list whose head is at this+0x180 (node[0] = value, node[1] = next). Returns 1 when a node's value equals arg1, else 0 (ret 4). |
| `00c2e940` | SumMatchedValuesCapped | likely | Walks linked entries under this+0x1050, finds matches for param_2, adds matched integer values times this+0x18, then returns the sum capped by this+0x1c. |
| `00c2eaf0` | AppendSearchToken | likely | Increments token count, grows storage through FUN_00c2ddf0 when needed, then initializes the new record by passing its first field to FUN_00cf34d0. |
| `00c2f420` | InitSearchResultState | likely | Stores the supplied vtable/value, constructs an inline ScoreCard array, clears dwords at indices 0x36 and 0x37, and returns this. |
| `00c2fb00` | BuildNormalizedTokenList | likely | Scans wide input, extracts tokens with FUN_00c2f490, collapses whitespace, lowercases nonempty tokens, and appends them to the index; releases temporary string… |
| `00c2fc80` | AccumulateMatchedFieldScores | likely | Lowercases field text, tests each configured field via FUN_00cf27d0, increments match counters, adds weighted scores to the ScoreCard, and returns total matche… |
| `00c30450` | RebuildItemSearchIndex | likely | Clears prior result state, tokenizes the supplied query, copies the active source vector through FUN_009908a0, then rebuilds profile terms with FUN_00c30150. |
| `00c75210` | DerivedHashList4096Ctor | likely | Constructor of a derived object: stores the vtable passed in param_2 at offset 0, calls the base constructor FUN_00c749d0 (SoeUtil::HashList of ItemDefinitionM… |
| `0103de00` | Field14AddDeltaClamp | likely | Adds the signed delta arg to the dword at this+0x14 (skipped when the delta is positive and the sum would be negative), clamps +0x14 at 0, then lowers +0x18 to… |
| `0103df50` | EnsureBlock28Allocated | likely | If this+0x28 is null, allocates 0x10 bytes through 00d833a6 (throws bad_alloc on failure) and initializes them with DAT_01cb67b8, DAT_01cb67bc, 0 and 0, then s… |
| `00724660` | GetField_214 | speculative | Returns the dword field at this+0x214. |
| `00724670` | GetField_218 | speculative | Returns the dword field at this+0x218. |
| `00724680` | GetField_21c | speculative | Returns the dword field at this+0x21c. |
| `00724690` | GetField_220 | speculative | Returns the dword field at this+0x220. |
| `007246a0` | GetField_224 | speculative | Returns the dword field at this+0x224. |
| `007246b0` | GetField_228 | speculative | Returns the dword field at this+0x228. |
| `007246c0` | GetField_22c | speculative | Returns the dword field at this+0x22c. |
| `007246d0` | GetField_230 | speculative | Returns the dword field at this+0x230. |
| `007246e0` | GetField_c8 | speculative | Returns the dword field at this+0xc8. |
| `00724780` | SetField_214 | speculative | Stores param_2 into the dword field at this+0x214. |
| `00724790` | SetField_218 | speculative | Stores param_2 into the dword field at this+0x218. |
| `007247a0` | SetField_21c | speculative | Stores param_2 into the dword field at this+0x21c. |
| `007247b0` | SetField_220 | speculative | Stores param_2 into the dword field at this+0x220. |
| `007247c0` | SetField_224 | speculative | Stores param_2 into the dword field at this+0x224. |
| `007247d0` | SetField_228 | speculative | Stores param_2 into the dword field at this+0x228. |
| `007247e0` | SetField_22c | speculative | Stores param_2 into the dword field at this+0x22c. |
| `007247f0` | SetField_230 | speculative | Stores param_2 into the dword field at this+0x230. |
| `00724880` | ReturnArg | speculative | Returns its only argument unchanged (a 3-byte body: mov eax,[esp+4]; ret). |
| `00724890` | GetField_214_B | speculative | Returns the dword field at this+0x214; duplicate body of 00724660 for a second vtable. |
| `007248a0` | GetField_218_B | speculative | Returns the dword field at this+0x218; duplicate body of 00724670 for a second vtable. |
| `007249c0` | ReturnArg_B | speculative | Returns its only argument unchanged; duplicate 3-byte stub of 00724880. |
| `00a156f0` | Client_EvaluateCurrentItem | speculative | Returns 7 when FUN_0103dfe0 reports a blocking state, otherwise resolves the current record with FUN_00b0d810 and returns 00a15690 of it, or 0 when no record i… |
| `00a3df00` | LookupOrRequestItemDefinition 🌐 | speculative | Wrapper that asks FUN_00c96360 for a definition id and, when that returns -1 (not present), calls the request path 00a3ddd0 with the same id; it returns the va… |
| `00baec20` | ItemEquipRequirements_BuildText | speculative | Thiscall (param_2 = result object): reads the item's equip requirements at this+0x68 (gender), +0x74 (membership flag), +0x84 (level), +0xa4 (VIP rank), +0xa8 … |
| `0103dec0` | Field18CapAtField14 | speculative | Stores the stack argument at this+0x18, then if this+0x14 is below that argument stores this+0x14 into +0x18 instead, otherwise clamps +0x18 at 0 when the argu… |
| `0103df20` | Field1cSetCappedAt14 | speculative | Stores the argument at this+0x1c, then when the argument exceeds the +0x14 dword stores and returns the +0x14 value; otherwise, when the argument is negative, … |
| `0103dfe0` | IsBlock28BeforeCachedNow | speculative | Returns 1 when this+0x28 is non-null, its 64-bit value (dwords 0 and 1) is not the default pair DAT_01cb67b8/DAT_01cb67bc, and that value is strictly less (sig… |
| `0103e080` | Block28SetQword | speculative | Ensures the +0x28 block exists (0103df50), then stores the two stack arguments as dword 0 and dword 1 of that block (64-bit value, low dword first). Returns wi… |
| `0103e0a0` | Block28SetDword8 | speculative | Ensures the +0x28 block exists (0103df50), then stores the stack argument at block offset +8. Returns with ret 4. |
| `0103e0c0` | Block28SetDword12 | speculative | Ensures the +0x28 block exists (0103df50), then stores the stack argument at block offset +0xC. Returns with ret 4. |
| `0103e400` | Block28SetFromNowPlusArg | speculative | If the stack argument is non-zero, stores its 64-bit sum with the cached now value from 00702670 (low dword plus now, carried high dword); otherwise stores the… |
| `0103e500` | MergeCountFromSource | speculative | Returns 1 (AL) when this.+4 equals source.+4 and source.+8 is 0xFFFFFFFF or equals this.+8, and 0103e210 succeeds for the source; in that case it adds source +… |
| `0103fd10` | ZeroHeaderBody128 | speculative | __fastcall with ECX = this: zeroes the dwords at +4, +8 and +0xC, zeroes 128 bytes at +0x10 (memset via 00d836a0), zeroes the dword at +0, and returns this. |
| `0103fd90` | ZeroHeaderBody64 | speculative | __fastcall with ECX = this: zeroes the dwords at +4, +8 and +0xC, zeroes 64 bytes at +0x10 (memset via 00d836a0), zeroes the dword at +0, and returns this. |
| `0103fed0` | ZeroHeaderBody64 | speculative | __fastcall with ECX = this: zeroes the dwords at +4, +8 and +0xC, zeroes 64 bytes at +0x10 (memset via 00d836a0), zeroes the dword at +0, and returns this. |
| `0072e050` | — | unknown | Reads one row by a chain of field readers over a moving cursor: 007065d0 for most scalar fields, 00726440, 00726890, 00727820 and 007264c0 for other fields, 00… |
| `00a142c0` | — | unknown | Classifies an item definition record (id at +4, slot at +0x30, type field at +0x6c, byte +0x74, value +0x84, alias +0xac) into a 1-4 status: for type 0xc it ac… |
| `00a3c7e0` | — | unknown | Resolves an item-group data source by name: it hashes the name with the client's rolling hash ((hash*0x401) ^ (hash>>6) for each of the first 0x31 characters, … |
| `00a3cf00` | — | unknown | Loads the ItemGroupMappings file: builds an IString from the argument, appends "ItemGroupMappings.txt", opens it (FUN_00fbe910) and reads line by line (FUN_00f… |
| `00a3d2f0` | — | unknown | Cleanup body: calls FUN_00a3b460 to unlink every node of a doubly-linked list, then calls FUN_00a3d170 (the teardown of an IString-owning record); runs inside … |
| `00a3d350` | — | unknown | Loads the ItemGroups file: appends "ItemGroups.txt" to the argument string, opens it and on failure releases the temporary strings; on success it drains the re… |
| `00a3dab0` | — | unknown | Start-up step of the item-group system: allocates a 0x264-byte data source through FUN_00d833a6, constructs it with FUN_00c97230(client pointer from this+4, "B… |
| `00baeae0` | — | unknown | Thiscall (param_2 is a result object): takes the item stat value at this+8 with type at this+0xc. Type 0 keeps the value and uses the key 'ItemStatAdditiveValu… |
| `00c2f080` | — | unknown | For each linked candidate, calculates contributions from 0x410d4-byte records, applies a logarithmic transform for non-unit kinds and a multi-match multiplier,… |
| `00c30150` | — | unknown | When the profile flag is set, traverses source rows, requests configured field strings, lowercases and token-matches them, then inserts unseen row keys into th… |
| `00c30a70` | — | unknown | For each configured search term, obtains candidates through virtual slots, accumulates field-match ScoreCards, applies match/score thresholds, removes rejects,… |
| `00c95510` | — | unknown | Calls the cleanup routine for the 1024-bucket item-entry hash-list map and its nested tree storage. |
| `0103e210` | — | unknown | Compares the +0x28 blocks of this and source (sentinel pair DAT_01cb67b8/bc means empty). When the source is empty or sentinel and this block holds a real valu… |

### `ui.inventory`

**BaseInventoryDataSource** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a6d960` | BaseInventoryDataSource::ColumnLabel_PriceNoSale | confirmed | Virtual slot 1: returns the static string Price for argument 0x1b and NoSale for 0x1c. Other arguments are tail-jumped to 00cb09d0 with the same argument (e9 j… |
| `00a6d870` | BaseInventoryDataSource::BaseInventoryDataSource | likely | Constructor body: calls 00cb1bb0 with the constructor arguments, installs BaseInventoryDataSource vtables at offsets 0x0, 0x30, 0x34 and 0x254, stores param_2 … |
| `00a6e5f0` | BaseInventoryDataSource::GetRowValueKind1bOr1c | likely | Virtual slot 4 (+0x10): first calls 00cb1160 with (row, kind, out) and returns its value when nonzero. Otherwise, for kind 0x1b it looks up the row object thro… |
| `00a6d990` | BaseInventoryDataSource::QueryUnlessKind11 | speculative | Virtual slot 2 (17 bytes, fastcall with ECX=this): when the dword at this+0x2b0 equals 0x11 it returns 0; otherwise it tail-jumps to 00cb0c60 with the same thi… |
| `00a6dcf0` | BaseInventoryDataSource::SecondaryAdjustorToDtor | speculative | Adjustor thunk, 11 bytes: subtracts 0x254 from ECX and jumps to 00a6de20 (BaseInventoryDataSource scalar deleting destructor). It is slot 1 of the secondary Ba… |
| `00a6d9b0` | — | unknown | Virtual slot 15 (+0x3c) row predicate. Resolves the row id through 00b0d810 (must be nonzero), then tests the owner at this+0x2dc: field +0x1bc, the manager fl… |
| `00a6de20` | — | unknown | Scalar deleting destructor: calls 00cb1b10 (SEH-prolog-style helper) with the frame cookie, then frees this through _free (thunk 00d82463) when bit 0 of the fl… |
| `00cb0b30` | — | unknown | Sets a one-byte dirty flag and tail-dispatches through the base datasource vtable at slot 4. |

**InventoryCategoriesDataSource** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a6de90` | InventoryCategoriesDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the destructor body 00a6dd00, then frees this through _free when bit 0 of the flag argument is set; returns this. |
| `00a6e9d0` | InventoryCategoriesDataSource::InventoryCategoriesDataSource | confirmed | Constructor: calls 009974a0 with the BaseClient.InventoryCategoriesData name, stores param_2 at +0x38 (the keyed category table), installs the InventoryCategor… |
| `00a6e480` | InventoryCategoriesDataSource::RebuildCategoryTree | likely | Releases the object at +0x30 (010cdbd0, then its slot 0 with argument 1), creates the root CategoriesTreeNode through 00a6e170 with arguments 0, 0, 0, 1 when t… |
| `00a6dd00` | — | unknown | Destructor body: stores InventoryCategoriesDataSource::vftable at this; when the pointer at +0x30 is non-null it releases that object through 010cdbd0 and its … |

**InventoryCategoriesDataSource::CategoriesTreeNode** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a6d830` | InventoryCategoriesDataSource::CategoriesTreeNode::KeyNameF… | confirmed | Returns a pointer to the static string CategoryId when the argument is 0, otherwise to the static string UNKNOWN (20 bytes: cmp dword [esp+4],0; mov eax,Catego… |
| `00a6e170` | InventoryCategoriesDataSource::CategoriesTreeNode::Categori… | confirmed | Constructor: calls the base constructor 0099ef20, stores param_2 at +0x18, param_4 at +0x1c and param_5 at +0x20, installs the CategoriesTreeNode vtable at off… |
| `00a6e260` | InventoryCategoriesDataSource::CategoriesTreeNode::ReleaseN… | confirmed | Releases the IString<char> at +0x24: stores the IString vtable there and, when the length at +0x30 is positive, drops a reference on the buffer at +0x28 (lock-… |
| `00a6e2e0` | InventoryCategoriesDataSource::CategoriesTreeNode::ScalarDe… | confirmed | Scalar deleting destructor: calls 00a6e260 (name string release), then frees this through _free when bit 0 of the flag argument is set; returns this. |
| `00a6e430` | InventoryCategoriesDataSource::CategoriesTreeNode::FormatVa… | likely | Virtual slot 5 (+0x14): for index 0 formats the node field +0x18 and for index 1 the field +0x20 into the output buffer param_3 with format DAT_01770978 throug… |
| `00a6e230` | InventoryCategoriesDataSource::CategoriesTreeNode::NameFiel… | speculative | Returns this+0x24, the address of the node's name IString member (lea eax,[ecx+0x24]; ret; 4 bytes). |
| `00a6e240` | InventoryCategoriesDataSource::CategoriesTreeNode::GetValue… | speculative | Returns the dword at this+0x1c (mov eax,[ecx+0x1c]; ret; 4 bytes). That field is the fourth constructor argument stored by 00a6e170. |

**InventoryDataSource** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a6db10` | InventoryDataSource::ColumnLabel_Kinds1dTo1c | confirmed | Virtual slot 1 override: returns IsRental for argument 0x1d, RentalSecondsRemaining for 0x1e, Price for 0x1b and NoSale for 0x1c. Other arguments are forwarded… |
| `00a6deb0` | InventoryDataSource::InventoryDataSource | confirmed | Constructor body: calls the base constructor 00a6d870, installs InventoryDataSource vtables at offsets 0x0, 0x30, 0x34 and 0x254, embeds an SoeUtil::IString<ch… |
| `00a6df70` | InventoryDataSource::ReleaseTextMember2ec | confirmed | Releases the IString<char> member at +0x2ec: stores the SoeUtil::IString<char> vtable there and, when the length at +0x2f8 is positive, drops one reference on … |
| `00a6e0f0` | InventoryDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls 00a6df70 (text member release at +0x2ec), then frees this through _free when bit 0 of the flag argument is set; returns this. |
| `00a6e6b0` | InventoryDataSource::GetRowValueKind1dOr1e | likely | Virtual slot 4 override: returns 0 when the row index is at or beyond the count at +0x24c. Kind 0x1d looks the row up through 008becc0 and writes DAT_0180c498 … |
| `00a6db60` | InventoryDataSource::ForwardToSlot14 | speculative | Virtual slot 17 (+0x44) thunk, 7 bytes: loads the vtable pointer from ECX, reads the entry at +0x38 (virtual slot 14) and jumps to it with the caller's argumen… |
| `00a6df60` | InventoryDataSource::SecondaryAdjustorToDtor | speculative | Adjustor thunk, 11 bytes: subtracts 0x254 from ECX and jumps to 00a6e0f0 (InventoryDataSource scalar deleting destructor). |
| `00a6e910` | — | unknown | Virtual slot 14 (+0x38) of InventoryDataSource and StorageDataSource. When the byte at +0x2e8 is clear it calls 00cb1340 (SEH-prolog-style helper) and returns.… |

**InventoryNavigationDataSource** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a7a5e0` | InventoryNavigationDataSource::Slot1_ColumnName | confirmed | Virtual slot 1 of InventoryNavigationDataSource (vtable 01816c78): switch on the column index returning a column name: 0 ID, 1 Parent Row, 2 Node Type, 3 Name,… |
| `00a7bb70` | InventoryNavigationDataSource::GetEntryCount | confirmed | Virtual row-count getter of the datasource's +0x30 subobject. When the byte at this+0x264 (primary object +0x294) is non-zero it subtracts 0x30 from this and c… |
| `00a7bb90` | InventoryNavigationDataSource::GetEntryValue | confirmed | Virtual row-value getter of the datasource's +0x30 subobject. Rebuilds the index first when this+0x264 (primary +0x294) is set, then maps row index param_2 thr… |
| `00a7bfe0` | InventoryNavigationDataSource::RefreshNodeValues | confirmed | Refresh pass over the row nodes of the object at +0x254. If the rebuilt flag at this+0x40 is set it first calls 00a7b930, then takes the manager at this+0x1c a… |
| `00a7c6d0` | InventoryNavigationDataSource::DeletingDtorThunk254 | confirmed | Adjustor thunk occupying slot 1 of the class's +0x254 vtable group. The raw bytes are 81 e9 54 02 00 00 (sub ecx,0x254) followed by e9 05 01 00 00 (jmp 00a7c7e… |
| `00a7c6e0` | InventoryNavigationDataSource::InventoryNavigationDataSource | confirmed | Constructor. Calls FUN_00922890 with the literal Inventory.Navigation, runs the base construction FUN_0081b470, installs the four class vtable groups at +0, +0… |
| `00a7c7e0` | InventoryNavigationDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor (slot 0 of the class primary vtable): calls the class destructor 00a7c530 and then, when bit 0 of param_2 is set, releases the objec… |
| `00a7b930` | InventoryNavigationDataSource::RebuildEntryIndex | likely | Clears the visible-index count at +0x28c. When the filter byte +0x295 and int +0x298 are both zero, appends each index from the node array (+0x278 data, +0x27c… |
| `00a7bbf0` | — | unknown | First-time build of the category rows, run only while the node count at +0x27c is 0. It takes the category registry at manager(+0x270)+0x1bc, walks its buckets… |
| `00a7c530` | — | unknown | Reinstalls the four InventoryNavigationDataSource vtable groups (+0, +0x30, +0x34, +0x254). If the node-array count at +0x27c is positive, it passes the data p… |

**InventoryNavigationDataSource::CategoryNode** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a7b380` | InventoryNavigationDataSource::CategoryNode::Slot0_ReturnTh… | confirmed | Three bytes (8b c1 c3): moves ECX to EAX and returns, so it returns this unchanged. Slot 0 of this vtable is not a destructor here. |
| `00a7c220` | CategoryNode::GetColumnValue | confirmed | Column value getter of a category row node. Resolves the category definition through FUN_01045060(this+0x10). Column 3 writes the definition field +4 through t… |
| `00a7b2f0` | — | unknown | Constructor (138 bytes): stores the three arguments at +0x10, +0x14 and +0x18, sets +4 to 1, +8 and +0xc to -1, installs the CategoryNode vtable at [this], ins… |

**InventoryNavigationDataSource::ItemGroupNode** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a7a530` | InventoryNavigationDataSource::ItemGroupNode::Slot1_ReturnT… | confirmed | Three bytes (8b c1 c3): moves ECX to EAX and returns, so it returns its this pointer unchanged as the interface pointer. |
| `00a7ba90` | ItemGroupNode::MatchesFilter | confirmed | Looks up this node's id at +0x10 in the manager table (param_2+0x20c), using id%10 and the bucket chain; returns false if absent. It starts with a true result … |
| `00a7c310` | ItemGroupNode::GetColumnValue | likely | Column value getter of an item-group row node. Looks the node id at this+0x10 up in the manager's table at param_2+0x20c (bucket chain at +0x20 + (id % 10)*4) … |

**MerchantBuyInventoryDataSource** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cfd9a0` | MerchantBuyInventoryDataSource::IsRowEligible | likely | Rejects rows failing the base predicate, rows without a resolved usable item record, and certain duplicate linked records; returns one only when the checks pas… |

**StorageDataSource** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a6e000` | StorageDataSource::StorageDataSource | confirmed | Constructor: calls the InventoryDataSource constructor 00a6deb0 with the forwarded arguments, then installs StorageDataSource vtables at offsets 0x0, 0x30, 0x3… |
| `00a6db90` | StorageDataSource::RowPassesFilter | likely | Virtual slot 15 override for storage lists: resolves the row id through 00b0d810 (must be nonzero), requires 008a6920 on this+0x2e0 with the id at +0x70 to ret… |
| `00a6e090` | StorageDataSource::SecondaryAdjustorToDtor | speculative | Adjustor thunk, 11 bytes: subtracts 0x254 from ECX and jumps to 00a6e110 (StorageDataSource scalar deleting destructor). |
| `00a6e110` | — | unknown | Scalar deleting destructor: calls 00a6df70 (text member release at +0x2ec) with the frame cookie, then frees this through _free when bit 0 of the flag argument… |

**(no class)** (29)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a6ea40` | InventoryScreen_SelectCategoryEvent | confirmed | Checks that the UI handler HandlerInventoryScreen exists (00977740). When it does, it builds the event name HandlerInventoryScreen:SelectInventoryCategory (app… |
| `00a6ebf0` | InventoryScreen_SelectItemEvent | confirmed | Same flow as 00a6ea40 with the event suffix :SelectItem. It packs the value at *param_1 (the item id) into the argument array (007071f0) and dispatches Handler… |
| `00c09060` | RegisterInventoryScreenModule | confirmed | Registration wrapper: pushes nup=0, the 21-entry table 0x0182a4a8 and the 'InventoryScreen' module-name pointer, calls FUN_00e39e40 and returns 0. |
| `00a6e5d0` | CategoryReferenceData_RebuildAndNotify | likely | Calls 00a6e480 (category tree rebuild) on the same object, then calls its virtual slot 4 (+0x10) with no arguments (18 bytes). |
| `00a6e7b0` | RebuildIdListByVSlot15 | likely | Rebuilds the id array at this+0x278 (count at +0x27c; grows through 008c2cc0 when needed). It collects the entries returned by 00990d80 for param_2, and for ea… |
| `00a7af90` | EntryIndex_AppendRecursive | likely | Method (this in ECX) with an entry id and an entry record: if the record link at +8 is -1 (not yet indexed), it first recurses on the parent index at record+0x… |
| `00a7b4d0` | InventoryNavigationDataSource::BuildChildNodes | likely | With this, parent node, category key, and data-source argument, walks the data-source hash tables at +0x1b4 (20 buckets) and +0x14 (100 buckets). It collects m… |
| `00a7c1a0` | WriteSharedColumnValue | likely | Shared column handler for both row-node types: switch on the column index param_4 - column 0 writes the object field +0x10, column 1 writes +0xc and column 2 w… |
| `00bdc810` | ResolveNameThenUseItem | likely | Copies the C string argument into an SoeUtil::IString (0101cd40/0101cd50), and when the player pointer *(*(this+0x3e0)) is non-null resolves the name with FUN_… |
| `00c09080` | Inventory_FilterItemGroupByPage | likely | Lua 'FilterItemGroupByPage' handler: with one numeric argument (FUN_00e3a5f0 gate, read via FUN_00e3a770 + FUN_00d83c90) it calls FUN_00a3b370(page) on the inv… |
| `00c090c0` | Inventory_SelectItemGroup | likely | Lua 'SelectItemGroup' handler: with one numeric argument it calls FUN_00a3b380(group) on the inventory screen object; returns 0. |
| `00c091c0` | Inventory_GetGroupIdFromItemId | likely | Lua 'GetGroupIdFromItemId' handler: with one numeric argument it calls FUN_00a3b410(item id), pushes the returned id as a Lua number (FUN_00e3aa30) and returns… |
| `00c09280` | Inventory_CreateItemGroup | likely | Lua 'CreateItemGroup' handler: requires a string argument (FUN_00e3a630 gate, FUN_00e3a820 read), resolves the inventory screen object and calls FUN_00a3c7e0(s… |
| `00c092c0` | Inventory_SelectInventoryCategory | likely | Lua 'SelectInventoryCategory' handler: with one numeric argument it calls FUN_00a10790(category) on the inventory screen object; returns 0. |
| `00c09300` | Inventory_SelectAltInventoryCategory | likely | Lua 'SelectAltInventoryCategory' handler: with one numeric argument it calls FUN_00a107a0(category) on the inventory screen object; returns 0. |
| `00c09340` | Inventory_SelectInventoryGenderFilter | likely | Lua 'SelectInventoryGenderFilter' handler: with one numeric argument it calls FUN_00a107b0(filter) on the inventory screen object; returns 0. |
| `00c09380` | Inventory_HideItem | likely | Lua 'HideItem' handler: requires two numeric arguments (FUN_00e3a5f0 gates on -1 and -2; read with FUN_00e3a7b0) and calls FUN_0096c670(id, second) on the inve… |
| `00c09d30` | Inventory_RequestItemRequirementStatus | likely | Lua 'RequestItemRequirementStatus' handler: requires a string argument (FUN_00e3a630 gate), builds an InventoryPacketItemRequirementRequest (FUN_00c08fa0), sto… |
| `00a6daa0` | SetField2e0_NotifyPending | speculative | If the dword at this+0x2e0 differs from param_2 (35 bytes: mov eax,[esp+4]; cmp [ecx+0x2e0],eax; je; mov [ecx+0x2e0],eax; mov byte [ecx+0x2d0],1; call through … |
| `00a6dad0` | SetField2e4_NotifyPending | speculative | Same as 00a6daa0 for the dword at this+0x2e4 (param_1[0xb9]): stores param_2 when it differs, sets the byte flag at +0x2d0 to 1 and calls virtual slot 4 of thi… |
| `00a6dd70` | HashTree_ContainsRecursive | speculative | Recursive membership test over two chained hash tables. Returns 1 when param_2 is 0 or equals param_3. Otherwise it looks param_2 up with 00a6dc40, walks the 1… |
| `00a6e0a0` | Wrapper_ReleaseTextMember2ec | speculative | Builds an SEH frame (push -1, handler, fs:[0] link, cookie XOR) that saves ECX, then calls 00a6df70 with that same this. The cookie-XOR argument shown in the p… |
| `00a6e300` | KeyedNodeTable_BuildChildren | speculative | Recursive child builder. Looks up the node for key param_2+0x18 in the 20-bucket table at +0x1b4 of *(this+0x38), then walks the 100-bucket table (key at node+… |
| `00cb09d0` | InventoryColumnName | speculative | Returns a display name for each item column index, covering GUID, icon, description, quantity, rarity, equipped state, model, type, and inventory flags. |
| `00a76a60` | — | unknown | Method with this in ECX and three stack arguments (item pointer, optional wide value, optional int): sets the row count of the dual-string array to 24 (0x18) t… |
| `00c09220` | — | unknown | Lua 'GetItemDefinitionIndex' handler: with one numeric argument it calls FUN_00a3df00(item id), pushes the result as a Lua number (FUN_00e3aa30) and returns 1;… |
| `00c09720` | — | unknown | Lua 'ShowItemGroup' handler: with 3 arguments (numeric id plus two numbers/strings) it resolves the item by id (FUN_00726e30), selects the group through FUN_00… |
| `00c098b0` | — | unknown | Lua 'ShowItemId' handler: with 3 arguments it resolves the item through FUN_0095d070, looks up its definition (FUN_00b8c360), spawns the display object via FUN… |
| `00c09a80` | — | unknown | Lua 'SearchInventoryItems' handler: requires a string argument (FUN_00e3a630 gate), copies it into a stack buffer (FUN_0083d110), runs the search through FUN_0… |

### `ui.items`

**BaseItemDefinitionDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c92c10` | BaseItemDefinitionDataSource::ColumnName | confirmed | Maps indices zero through forty to item definition, display, model, purchase, ownership, equipment, stack, and tint-related field names. |
| `00c933f0` | BaseItemDefinitionDataSource::CleanupSlot3 | confirmed | Calls 009879a0 with this adjusted to the embedded subobject at +0x3654, restores ECX, and tail-jumps to 010c8e90. |
| `00c93990` | BaseItemDefinitionDataSource::RemoveCachedDefinition | confirmed | Searches the 1024-bucket item cache for the requested ID, removes a matching node, and invokes the update hook for that ID. |
| `00c93380` | — | unknown | Searches the item cache by key and either invalidates the matching cached definition or calls the update hook when no cached entry exists. |
| `00c942c0` | — | unknown | Finds the item-entry node in its 1024-bucket cache, removes the matching linked-list item, updates the associated ordered structure, and calls the ID update ho… |

**BaseItemFilterManager<ClientItemDefinition>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bb20c0` | BaseItemFilterManagerDtor | confirmed | Seven-byte destructor body (mov dword [ecx],0x018250cc; ret): writes BaseItemFilterManager<ClientItemDefinition>::vftable into this+0 and returns without touch… |
| `00bb20d0` | BaseItemFilterManagerScalarDeletingDtor | confirmed | Scalar deleting destructor at vtable slot 1: rewrites BaseItemFilterManager<ClientItemDefinition>::vftable into this+0 and, when bit 0 of the flag argument is … |

**ClientItemFilterManager** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bb2420` | ClientItemFilterManager::~ClientItemFilterManager | confirmed | Destructor body: rewrites ClientItemFilterManager::vftable into this+0, releases the refcounted data source at +0xb1c (asks it for its registry id through vtab… |
| `00bb2620` | ClientItemFilterManager::ScalarDeletingDtor | confirmed | Scalar deleting destructor at vtable slot 1: runs the destructor body 00bb2420 and, when bit 0 of the flag argument is set, releases this through the CRT free … |
| `00bb24c0` | ClientItemFilterManager::FilterMatch | likely | Vtable slot 0: returns false unless the client interface [[this+4]+0x3e0] and the filter entry for param_3 (bucket this+0xc4+(param_3 & 0x1f)*4, key +0x130, ch… |
| `00bb2350` | — | unknown | Constructor: installs BaseItemFilterManager<ClientItemDefinition>::vftable and then ClientItemFilterManager::vftable at +0, stores the argument at +4, runs the… |

**ItemAbilitiesDataSource** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a7e410` | ItemAbilitiesDataSource::GetColumnName | confirmed | Column-name table of the item abilities list: returns Ability ID, Ability Icon, Ability Name, Ability Description and Ability Effect Count for indices 0..4 and… |
| `00a7ea90` | ItemAbilitiesDataSource::GetColumnValue | confirmed | Row-value getter of the abilities list (vtable slot 4). Resolves the item definition by the id at this+0x240 in the item table (bucket id & 0x3ff, chain +0xb0 … |
| `00a7ede0` | ItemAbilitiesDataSource::~ItemAbilitiesDataSource | confirmed | Destructor body: rewrites the four ItemAbilitiesDataSource vtable pointers (+0, +0x30, +0x34 and +0x254) and then runs FUN_00835610 (observer base) and FUN_009… |
| `00a7ee70` | ItemAbilitiesDataSource::DeletingDtorThunk254 | confirmed | Adjustor thunk in slot 1 of the +0x254 vtable group: the raw bytes are 81 e9 54 02 00 00 (sub ecx,0x254) and e9 a5 00 00 00 (jmp 00a7ef20), so it re-bases the … |
| `00a7ee80` | ItemAbilitiesDataSource::ItemAbilitiesDataSource | confirmed | Constructor: registers BaseClient.ItemAbilities through FUN_00922890, runs the base construction FUN_0081b470, installs the four class vtables (+0, +0x30, +0x3… |
| `00a7ef20` | ItemAbilitiesDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the destructor body 00a7ede0 and frees the object with _free when bit 0 of param_2 is set, returning the object pointer. |
| `00a7e600` | ItemAbilitiesDataSource::GetRowCount | likely | Resolves the client singleton with FUN_00769710, then looks the item definition up by the id at this+0x240 in the item table at (*(singleton+4))+0x20c (bucket … |
| `00a7e660` | ItemAbilitiesDataSource::RefreshAfterAbilityDefinitionsCach… 🌐 | likely | Looks up the item definition for this+0x1c in the item table and gets the player's ability-modifier object. If either is absent it returns. It walks the defini… |

**ItemDefinitionDsUpdateDispatcher** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c92f70` | ItemDefinitionDsUpdateDispatcher::DtorBody | confirmed | Restores the Singleton<ItemDefinitionDsUpdateDispatcher> vtable during unwind cleanup. |
| `00c93090` | ItemDefinitionDsUpdateDispatcher::ScalarDeletingDtor | confirmed | Restores the singleton dispatcher vtable and conditionally frees the object when the deleting flag's low bit is set. |
| `00c93410` | ItemDefinitionDsUpdateDispatcher::Ctor | confirmed | Installs the singleton and dispatcher vtables and zeros the three dispatcher state fields. |
| `00c93470` | ItemDefinitionDsUpdateDispatcher::Dtor | confirmed | Restores the dispatcher and singleton vtables on the object. |
| `00c93c20` | ItemDefinitionDsUpdateDispatcher::ScalarDeletingDtor | confirmed | Restores the dispatcher vtable, then the singleton base vtable, and frees the object only when the deleting flag is set. |
| `00c93c90` | ItemDefinitionDsUpdateDispatcher::NotifyListeners | likely | Adjusts this by four bytes, passes the embedded listener list and callback 00c93300 to FUN_00c93740, then returns. |

**ItemRecord** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a84370` | ItemRecordCtor | confirmed | Constructor of an ItemRecord: stores ItemRecord::vftable and copies two dword values supplied through param_2 and param_3 into +4 and +8, then zeroes +0xc. |

**ItemSortDataSource** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bb2050` | ItemSortDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor at vtable slot 0 (decoded: push esi; mov esi,ecx; call 00bb1f50; test byte [esp+8],1; je; push esi; call 00d82463; ret): runs the de… |
| `00bb1dd0` | ItemSortDataSource::GetStaticTableByIndex | likely | Bare selector over the index argument (decoded bytes: mov eax,[esp+4]; sub eax,0; je; sub eax,1; je; two mov eax,imm32; ret 4): returns the static table at 0x0… |
| `00bb1e50` | ItemSortDataSource::GetItemCount | likely | Bare field getter (decoded bytes: mov eax,[ecx+0x224]; mov eax,[eax+0x30]; ret): returns the dword at model+0x30 where model is the object stored at this+0x224. |
| `00bb1e80` | ItemSortDataSource::RefreshList | likely | Calls vtable slot 3 of the model object at this+0x34 (reset), then walks the singly-linked list whose head is at [this+0x254]+0x28 and whose next pointer is at… |
| `00bb1ec0` | — | unknown | Vtable slot 4 override: returns 0 unless the index argument is below the count *(*(this+0x224)+0x30); it then resolves the entry through vtable slot 4 of [this… |
| `00bb1f50` | — | unknown | Destructor body: rewrites ItemSortDataSource::vftable into this+0, this+0x30 and this+0x34 (the three sub-object vtable slots) and then calls the base destruct… |
| `00bb1fd0` | — | unknown | Constructor: registers the data-source type through 00922890 with the name looked up at PTR_s_BaseClient.ItemSortTypes, stores the argument at this+0x254, inst… |

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a84450` | ItemRecordListAppend | likely | Appends one ItemRecord to the intrusive list owned by param_1: allocates a node through the owner's vtable slot [2], constructs the record with 00a84370(param_… |
| `00c20730` | ItemDefinitionSelect | likely | Reads an item identifier, validates it through the BaseClient.ItemDefinitionSelect data source, clears a temporary selection record, and calls 00c976c0. |
| `01088510` | ParseItemSortTypeRow | likely | Thiscall with a text cursor (param_2) and a length (param_3), returns 1. Parses two consecutive fields through 0072d900 into this+0x4 and this+0x8, advancing t… |

### `ui.tooltip`

**AbilityEffectsTooltipDataSource** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a73e00` | AbilityEffectsTooltipDataSource::ColumnTitle | confirmed | Virtual slot [1]. Switch on the int argument: 0 returns the literal Image ID, 1 returns the literal Description, any other value returns the literal <error>. P… |
| `00a74f60` | AbilityEffectsTooltipDataSource::DtorBody | confirmed | Stores the AbilityEffectsTooltipDataSource vtables at this, +0x30, +0x34 and +0x254, calls 00a74290 (the 10-slot entry array destructor body; receiver not show… |
| `00a75000` | AbilityEffectsTooltipDataSource::AdjustorToScalarDtor | confirmed | Secondary-vtable slot [1] (018162c8). Subtracts 0x254 from ECX and tail-jumps to 00a752e0 (jmp rel32), i.e. an adjustor thunk for the scalar deleting destructo… |
| `00a75230` | AbilityEffectsTooltipDataSource::Ctor | confirmed | Calls the base constructor 00922890 with its own stack argument (the function also references the literal BaseClient.Abilities at 0183523c), installs the Abili… |
| `00a752e0` | AbilityEffectsTooltipDataSource::ScalarDeletingDtor | confirmed | Virtual slot [0]. Calls the dtor body 00a74f60 on this, then calls _free on this when bit 0 of its stack argument is set; returns this (ret 4). |
| `00a73ee0` | AbilityEffectsTooltipDataSource::CastCheckSetFlagNotify | likely | Virtual slot [6]. Calls 00d8d382 (a dynamic cast helper) on the pointer at this+0x18 with the GuiDataSource and AbilitiesDataSource type descriptors. When the … |
| `00a75010` | AbilityEffectsTooltipDataSource::GetCellEntry | likely | Virtual slot [4] of the secondary vtable 01816304 (row, column, out). Returns 0 when row is negative or not below the count at +0x250. Otherwise it copies the … |
| `00a750e0` | AbilityEffectsTooltipDataSource::GetAfterOptionalRefresh | speculative | Virtual slot [2]. When the byte at this+0x240 is nonzero it calls 00a74af0 with ECX = this-0x30. Then returns the dword at this+0x250. |

**SoeUtil::Array<TooltipDataSource::Node,0,1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a76360` | TooltipDataSourceNodeArray_ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): stores the array vtable at [this], calls 00a75d50 with the count at [this+8] to destroy all elements, frees the buf… |
| `00a761e0` | TooltipDataSourceNodeArray_DtorBody | likely | Destructor body: stores the SoeUtil::Array<TooltipDataSource::Node,0,1> vtable at [this], calls 00a75d50 (ShrinkBy) with the element count at [this+8] so every… |

**TooltipDataSource** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a756b0` | TooltipDataSource::RowFieldOrError | confirmed | Virtual slot [1]. Takes an index; when the index is below the count at this+0x248 it returns the dword at (row buffer at this+0x244) + index*32 + 4; otherwise … |
| `00a76930` | TooltipDataSource::DtorBody | confirmed | Destructor body: restores TooltipDataSource::vftable at [this] and at the secondary base slots +0x30, +0x34 and +0x254, installs SoeUtil::Array<TooltipDataSour… |
| `00a79880` | TooltipDataSource::Ctor | confirmed | Constructor: calls base constructor 00922890 with param_2, then 0081b470, stores the TooltipDataSource vtable at the primary +0 and at +0x30, +0x34 and +0x254,… |
| `00a799b0` | TooltipDataSource::ScalarDeletingDtor | confirmed | Scalar deleting destructor (27 bytes, vtable slot 0 of the primary TooltipDataSource vtable): calls the destructor body 00a76930 on this, then if bit 0 of the … |
| `00a76a00` | TooltipDataSource::Thunk_Slot1_Adjust254 | likely | Adjustor thunk (11 bytes): subtracts 0x254 from ECX and jumps to 00a799b0. It is not a body; it forwards to 00a799b0 with this moved to the +0x254 subobject. |
| `00a76a10` | TooltipDataSource::Slot0_GetField248 | likely | Virtual slot 0 of the secondary TooltipDataSource vtable 01816410 (7 bytes): loads the 32-bit field at this+0x248 into EAX and returns it, with no arguments an… |
| `00a76160` | — | unknown | Virtual slot 4 of TooltipDataSource with a stack argument index: if index is below the count at this+0x248, calls 00836c70 with the address data(this+0x244) + … |
| `00a790f0` | — | unknown | Virtual slot 6 of the secondary TooltipDataSource vtable 018163d4 (called with this at obj+0x254): reads the tooltip kind at this+0x2c (object +0x280). Kind 5 … |

**(no class)** (31)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a76700` | Tooltip_FireOnTooltipRequest | confirmed | Method with this in ECX and one stack argument: builds an inline two-element Lua parameter array (0070ebe0), stores type 1 with the value of this+0x280 in elem… |
| `00a771b0` | StoreItemTooltip_BuildRows | confirmed | Method with this in ECX and one stack argument (a store item record): gates on the global client state (00769710 field +4 and +0x3e0), sets the row count to 12… |
| `00a77b40` | ItemGroupTooltip_BuildRowsById | confirmed | Method with this in ECX, a group id argument and a flag: gates on the global client state (00769710 field +4 and +0x20c), looks the id up in a 10-bucket chaine… |
| `00a77d40` | AbilityItemTooltip_BuildRows | confirmed | Method with this in ECX and two stack arguments (an ability-item record and a value): sets the row count to 11 through 00a76870 or 00a75d50, resets the state f… |
| `00a786d0` | AbilityTooltip_BuildRows | confirmed | Method with this in ECX and a record argument: when the global client state field at +4 has +0x174 nonzero, resets the state fields at +0x280 to +0x294, calls … |
| `00a78ca0` | ProxiedCharacter_BuildRows | confirmed | Method with this in ECX and three stack arguments (a wide name, a value and a wide text): when the global client state field +4 is nonzero and 00984960 resolve… |
| `00961730` | ClearTooltipStateAndHide | likely | Clears bit 0 at this+0x145, then calls FUN_00904bf0 if bit 1 remains set or FUN_00904e70 otherwise. |
| `00963dd0` | UpdateTooltipVisibility | likely | If bit 0 at +0x145 is set, clears it and hides through one of two tooltip helpers chosen by bit 1; then sets bit 0, shows tooltip rows when bit 1 and +0x70c ar… |
| `00a756e0` | ClearFields280To294 | likely | Zeroes the dword at this+0x280, the byte at +0x28c, and the dwords at +0x290 and +0x294, calls 008356b0 with argument 0, then zeroes +0x280 again. No return va… |
| `00a75da0` | CastToAbilityEffectsTooltipDataSource | likely | Returns param_1 after a runtime dynamic_cast (00d8d382) from GuiDataSource to AbilityEffectsTooltipDataSource (the two RTTI descriptors are passed in the call)… |
| `00a76210` | LookupAbilityEffectsTooltipDataSource | likely | Hashes its argument with 00705bc0, indexes a chained hash table of 1000 buckets at this+0xaea4 (node: value at +0, next at +4, hash at +8), and if a node with … |
| `00a77a80` | Tooltip_BuildItemRowsById | likely | Method with this in ECX and an id argument: gates on the global client state (00769710, field +4, field +0x3e0), resolves the id through 008becc0 and returns 0… |
| `00a78a40` | RecipeTooltip_BuildRows | likely | Method with this in ECX and a record id argument: gates on the global client state (00769710 field +4), resolves the record through 00a127d0 (an 11-byte adjust… |
| `00a78e80` | TrophyTooltip_BuildRows | likely | Method with this in ECX: gates on the global client state (00769710 field +4), resolves a trophy record through 008a6760 (guarded pointer accessor, +0x1be8 off… |
| `00a79620` | RecipeTooltip_BuildRowsKind13 | likely | Same body as 00a78a40 (RecipeTooltip_BuildRows) except that it sets the kind at +0x280 to 0xd instead of 6: gates on the global client state, resolves the reco… |
| `00a799d0` | Tooltip_ItemByIdOrRequest 🌐 | likely | Method with this in ECX, an item id, a record argument and two flags: when the global client state has a nonzero field +4 with +0x3e0 set, resets the state fie… |
| `00c1fdc0` | TooltipData_SetRowCount | likely | Accepts one argument, looks up the TooltipData source, and passes the argument with flag one to 00a76a20, which resizes its row storage and notifies the source. |
| `00c1fe10` | TooltipData_SetText | likely | Reads three arguments, converts the third into a wide string, looks up TooltipData, and calls its update helper with the first two values and a constant flag. |
| `00c20040` | TooltipData_PopulateFactoryNpcTooltip | likely | Requires zero arguments, resolves TooltipData, and calls 00a79250 to rebuild the source with factory-NPC fields when available. |
| `00c20220` | TooltipData_PopulatePetTooltip | likely | Reads an identifier, searches the global four-bucket structure at offsets 0x19ad4 and 0x120/0x124, and passes a matching record to 00a782d0 to populate Tooltip… |
| `00c202b0` | TooltipData_PopulatePetTrickTooltip | likely | Reads one argument, resolves TooltipData, passes the argument to 00a784e0, then pushes success. |
| `00c20420` | TooltipData_PopulateAbilityTooltip | likely | Reads one argument, resolves TooltipData, passes the value to 00a786d0, then pushes success. |
| `00c20480` | TooltipData_PopulateAbilityItemTooltip | likely | Reads a value, searches the client list through 00a14560, and passes the resulting item record and companion value to 00a77d40 when both are nonzero. |
| `01251db0` | ActionCodeToTooltip | likely | Non-virtual function of one int argument. Walks the constant table {0x4e9d, 0x532d, 0x5332, 0xffffffff, 0} and returns the parallel string table {'Reanimate', … |
| `012b4240` | TooltipMarkupFormat | likely | Cdecl, arguments: output std::string, a ValueData-style source at param_2 (type tag 3 means string; the string is copied to a temporary), and an int id. For ea… |
| `00a75100` | Setter274RefreshNotify | speculative | Takes one stack argument, stores it at this+0x274, sets the byte at this+0x270 to 1, calls 00a74af0 with ECX = this, then calls vtable slot +0x10 of this (ECX … |
| `00a75720` | ClearFields280ThenSetMode10 | speculative | Takes two stack arguments. When the global object's +4 sub-object is nonzero: zeroes the dwords at this+0x280, +0x290, +0x294 and the byte at +0x28c, calls 008… |
| `00a74af0` | — | unknown | Clears the list at this+0x280 through 00a740e0 with the count. Reads the global object through 00769710 and returns when its +4 sub-object's +0x174 field is ze… |
| `00a75790` | — | unknown | When the global object's +4 sub-object is nonzero: zeroes the same fields as 00a75720 (+0x280, +0x28c, +0x290, +0x294), calls 008356b0 with 0, increments the d… |
| `00a76520` | — | unknown | Method with this in ECX: looks up the data source named Tooltips.ActiveAbilityEffects (00a76210) and, when found, calls 00a75100 with this+0x8c; looks up Toolt… |
| `00c1ffe0` | — | unknown | Reads one recipe value, resolves its recipe component record through 00a127d0, and passes that record to 00a78a40 to populate RecipeTooltip rows when TooltipDa… |

## Source shards

[[S0150 0071f030-00725ff0|S0150]] (45), [[S0152|S0152]] (4), [[S0222 008b1e00-008b7730|S0222]] (3), [[S0256 009606e0-00965340|S0256]] (2), [[S0265 0098ec00-00995660|S0265]] (13), [[S0285 009fef30-00a059e0|S0285]] (1), [[S0288 00a13190-00a18a10|S0288]] (3), [[S0290 00a1de60-00a243e0|S0290]] (1), [[S0295 00a39b00-00a3f350|S0295]] (20), [[S0304 00a6c630-00a70790|S0304]] (39), [[S0305 00a707f0-00a75b20|S0305]] (14), [[S0306 00a75b70-00a7b4d0|S0306]] (30), [[S0307 00a7b930-00a81420|S0307]] (21), [[S0308 00a81470-00a87b30|S0308]] (2), [[S0339 00b29890-00b2eaf0|S0339]] (1), [[S0365 00bac0d0-00bb1110|S0365]] (2), [[S0366 00bb11b0-00bb7710|S0366]] (13), [[S0374 00bd9880-00be0090|S0374]] (17), [[S0380 00bf5ff0-00bf9410|S0380]] (6), [[S0383 00c05780-00c0db00|S0383]] (14), [[S0386 00c1f190-00c26aa0|S0386]] (12), [[S0388 00c2df40-00c34100|S0388]] (13), [[S0400 00c6df10-00c73b60|S0400]] (1), [[S0401 00c73bb0-00c7a5b0|S0401]] (1), [[S0405 00c89e70-00c8f7c0|S0405]] (1), [[S0406 00c8f830-00c95560|S0406]] (13), [[S0411 00cab620-00cb0bb0|S0411]] (2), [[S0426 00cfc4e0-00d01c70|S0426]] (1), [[S0569|S0569]] (23), [[S0571 01044480-0104a2b0|S0571]] (1), [[S0575|S0575]] (5), [[S0579|S0579]] (2), [[S0580 0107d0c0-010829c0|S0580]] (1), [[S0581|S0581]] (3), [[S0658 0124dfe0-01253ee0|S0658]] (1), [[S0677|S0677]] (1)
