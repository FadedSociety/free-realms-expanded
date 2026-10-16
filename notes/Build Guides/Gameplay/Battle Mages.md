---
type: build-guide
section: Gameplay
updated: 2026-10-12
---

# Battle Mages

Part of [[Build Guides]] › Gameplay. Related: [[Minigame Framework]].

<!-- NARRATIVE START -->
## How it works

**How to read this.** Confidence words (`confirmed`, `likely`, `speculative`, `unknown`) are the native-map labels for a function. Every label used here is accepted, and many Battle Mages labels are `unknown` or `speculative`. Where this page leans on one of those it says "read from pseudocode in this pass", which means the behaviour was taken from the Ghidra pseudocode in `map.sqlite` (and, where noted, from the raw bytes of `FreeRealms.exe`) by this write-up. It is not a labelled fact. Source tags: *client* = the shipped executable (shard findings S0330, S0352, S0353, S0418, S0419, S0579, S0247, labels, pseudocode, data bytes), *client locale* = the shipped text table, *client resource* = a file in the shipped `Resources` folder or the extracted asset catalog, *client Lua* = the decompiled `ScriptsBase.lua`, *video* = a reviewed frame record, *wiki* = the fan wiki. Precedence is client, then video, then wiki. **(R)** marks a reconstruction: our own server policy, or an inference the client alone cannot settle. Shard text that was a server hypothesis is repeated here only as a hypothesis. No original server code exists. OSFR-held packet captures are not used. This page does not use OpenSourceFreeRealms or FreeRealms-Legacy code or data, any community packet catalogue, or vault notes that depend on them. No player names, chat or player-chosen names are recorded. Related: [[Minigame Framework]] (launch, HUD and results shared by every game; Battle Mages is one of its native engines), [[Trading Card Game]] (a different, card-based game with its own client DLL; nothing in Battle Mages touches it), [[Fishing]], [[Soccer]], [[Kart Racing]], [[Demo Derby]], [[Ship Combat (Pirates)]] (sibling native activities that use the same switch-on pattern), [[Gateway and Transport]] (the tunnel that carries these messages), [[Combat and Abilities]].

### Overview

Battle Mages is a native client activity: its own packet family, a processor object, a player controller class and two console-style commands. It is not a Flash movie and not a card game. The only evidence of what it plays like is the code. The client has no Lua panel, no locale text, no sound set and no minigame-table row for it (checked below). Everything here about gameplay is therefore what the code shape implies, and the page says "unknown" where the code is silent.

- **What the client shows it to be.** A real-time multiplayer activity with a third-person character controller. Other players appear as "proxied players" and projectiles appear as "proxied projectiles", both created, moved and destroyed by server messages. The local player can ask to attack (a "RequestAttack" message). All of that is client code: `BattleMagesProxiedPlayer`, `BattleMagesProxiedProjectile`, `ControllerBattleMages`, `BattleMagesPacketRequestAttack` (see Main classes). The word "mage" and the idea of spells come from the name only.
- **How many players.** The processor sizes its roster array to 8 (`00b71200`, unknown label, read from pseudocode) and the create-player handler rejects a slot number of 8 or more (`00b71bd0`, unknown label, read from pseudocode; shard S0352 says the same). So at most eight roster slots. What a slot means (a team, a spawn point, a colour) is unknown.
- **What the player controls.** Movement and turning from input, a camera with a distance and a pitch range, and three ability-slot actions (action ids `0x13` to `0x15`, the same three ids the fishing controller uses). The first two slots send the attack request; the third takes a different local branch that is not understood (`00af9590`, speculative; `00afab40`, unknown).
- **What the player sees of others.** Other players and projectiles drawn from server data. A projectile is an object driven by a 4x4-style transform and later vectors; the client builds it from the physics pool (`00cd5710` unknown label, `00fdbd40` likely, read from pseudocode). Hits, damage, health, scores, teams, win or lose, results and rewards: no message in this family carries any of them. The only game-wide value is one 32-bit number in `UpdateGameState` whose meaning is unknown.
- **How a player gets in.** Not through the minigame tables. The mode is switched on from a status bit in the normal player update, the same mechanism the kart, derby, soccer, fishing and ship activities use. See Entering and leaving.
- **How a player gets out.** The only recovered way to switch the mode off is the zoning teardown (`BeginPlayerZoning`, `0093d180`, confirmed). Nothing else turns it off in the code that was read. See Open questions.
- **Developer commands.** Two console-style commands, `bm_reloaddata` and `bm_camera`, with built-in usage text (client).
- **Other uses of the words.** The locale has "Battle Mage" (two entries) and "Battle Mage Gear", shaped like job names and "[job] Gear" (client locale); nothing in the code links them to this activity. The locale also has "Bixie Battlemage" enemy text; that is an unrelated monster (see Video and wiki evidence).
- **Was it live.** The code is in the shipped client, but no shipped text, table row, script or non-test asset for it was found (the only asset names are `test_bm_*flag_bbe`, see Data and formats), and no video or wiki page mentions it. Whether the activity was ever live is unknown.

### How the client implements it

#### Main classes

| Class or object | Role | Evidence |
|---|---|---|
| `BattleMagesProcessor` (ctor `00b71200`, vtable `01822bc4`; destructor `00b71320`) | One per client, built in the client bring-up routine `009162f0` (unknown label) and stored at client `+0x124`. Slot 4 `00b728a0` is the receive dispatcher. Memory layout under Data and formats. | `00b728a0` confirmed, `00b71320` confirmed, `00b71200` unknown label; ctor and storage read from pseudocode in this pass |
| `BaseBattleMagesPacket` and twelve subclasses | Message objects. The base constructors `00b6f250` and `00b6f2b0` write the family value `0x85` (133) at `+4` and the sub-opcode at `+8` (both labels `unknown`; the stores are three lines of pseudocode, read in this pass). Subclass constructors are listed under Network. | confirmed constructors, see Network |
| `ControllerBattleMages` (ctor `00afa420`, vtable `0181c3d4`) | The player controller used while the mode is on. Its vtable slot 0 `00af8df0` returns `0x11` (17), the controller type id. The controller factory `008b8ff0` builds it in its case `0x11`. Holds the camera block pointer at `+0x13c`. | ctor unknown label; slot 0 has no label, read from pseudocode (`return 0x11`); factory unknown label |
| `BattleMagesProxiedPlayer` (ctor `00cd4e70`) | One roster entry for another (or the local) player: guid pair at `+8` and `+0xc`, a dword at `+0x10` that starts at 0, a flag byte at `+0x14`, the client pointer at `+0x1c`. | confirmed |
| `BattleMagesProxiedProjectile` (ctor `00cd52d0`, destructor body `00cd5280`) | One projectile: id at `+0x10`, effect/physics object at `+0x14`, handle pair at `+0x28`/`+0x2c`, a 16-float block at `+0x30..+0x6f`, two dwords at `+0x70`/`+0x74`, an "updated" byte at `+0x78`. | ctor unknown label, destructor confirmed |
| `BattleMagesCommandProcessor` (ctor `00cd5b90`, dtor `00cd5c20`, `ProcessCommand` `00cd69e0`) | Registers the two commands and routes them. | confirmed |
| Camera block (8 floats and a string, in the processor at `+0x54` and in `BattleMagesPacketCameraConfig` at `+0x10`; defaults set by `0107b840`) | The camera tuning the server can send. | `0107b840` likely, layout read from pseudocode and data bytes |
| Proxy lists | `SoeUtil::List<BattleMagesProxiedPlayer,-1>` at processor `+0x2c`, `SoeUtil::List<BattleMagesProxiedProjectile,-1>` at `+0x3c`, and an 8-entry `SoeUtil::Array` of character guids at `+0x18` (vtables in the constructor `00b71200`). Player nodes are `0x28` bytes, projectile nodes `0x90` bytes (allocators `00b701e0`, `00b70250`, confirmed). | confirmed allocators; roster array read from pseudocode |

#### Entering and leaving

1. **The trigger.** In `BaseClient::DispatchBasePlayerUpdate` (`0092f460`, confirmed), the branch for player-update subtype `0x14` reads the status flag dword of the updated character (the dword at `+0x6f8`, the same word the fishing and kart guides describe). When bit 21 is set, the Battle Mages processor exists (client `+0x124`) and its active byte at processor `+0x28` is 0, the code calls `00b72f60(1)` (label `unknown`, read from pseudocode in this pass). The nesting is deep and the neighbouring bits arm the other native activities, so the exact conditions are my reading, not a label. Whether it filters to the local character is not resolved; the Fishing guide describes the same branch as the local-player branch.
2. **Turning on (`00b72f60` with a new non-zero state).** It stores the state in processor `+0x28`, then: registers the two commands through the command processor (`00cd5b30`, speculative); switches the client's controller to type `0x11` (`008c5d90(0x11)`, likely; this is what builds `ControllerBattleMages`); calls `008a7e00` (likely, a wrapper that calls `009cf7d0` with 0; purpose unknown); hands the camera block at processor `+0x54` to the new controller (`00af9230`, speculative); clears a byte in a client sub-object (`+0x1a8` then `+0x61`; meaning unknown); sends **RegisterPlayer** with the local character's guid (`00b72dd0`, likely); calls `008a68f0` with 0 (likely, a wrapper of `009d0f30`; purpose unknown); and writes the state to a byte at client `+0x878`. (read from raw bytes and pseudocode in this pass)
3. **Turning off (state 0).** It sends the same RegisterPlayer again (see step 5), sets the sub-object byte to 1, and, if the current controller's type is `0x11` (`008a6e20`, likely), clears its camera block (`00af9230(0)`). It then calls `008d2830` (unknown) and `00cd5b70` (unknown label; the command processor pointer goes in as its argument and it forwards to `00986c10`, which the command processor destructor `00cd5c20`, confirmed, also calls, so this is probably the command unregistration) and calls `008a68f0` with 1.
4. **Only one caller turns it off.** `00b72f60` has two callers: the player-update branch above (which only passes 1 in what was read) and `BeginPlayerZoning` (`0093d180`, confirmed), which passes 0 with the other activity teardowns. So zoning is the only recovered exit (client).
5. **RegisterPlayer is sent on both enter and leave.** `00b72dd0` takes an entity, a middle argument and a flag. The raw bytes never read the middle argument, and the enter call passes `(entity, 1, 1)` while the leave call passes `(entity, 0, 1)`. The two messages are therefore identical in the recovered code (read from raw bytes in this pass). Whether the original server told them apart some other way is unknown.
6. **The loading barrier.** `WaitForWorldReady` (`00936d00`, confirmed) has a Battle Mages step. When the processor is active it loops while the byte at processor `+0x29` is non-zero, calling the per-frame update `00b70ad0` and checking the gateway on each pass; a lost gateway aborts with "Disconnect from gateway waiting for battlemages" and the loading screen text "waiting for battlemages processor" (client strings). In the mapped Battle Mages ranges that byte is written only by the constructor (to 0), so with a normal server the barrier passes at once (searched in this pass; the byte could be set elsewhere).
7. **Per-frame update.** `PlayClient::Update` (`0090c970`, confirmed) calls `00b70ad0` (`WalkFlaggedLinkedChains`, likely). When the active byte is set it calls one thunk per projectile node and one per player node (`00cd5ae0` and `00cd4f50`, both speculative wrappers of `00cd5550` and `00cd4ed0`). `00cd5550` (unknown label) composes the projectile's matrices with the transform of the character named by its handle pair while the node's `+0x78` byte is 0, and then hands the result to the object at node `+0x24`; `00cd4ed0` is `ResolveHandlePairAndNotify` (speculative). Since the first UpdateProxiedProjectile sets `+0x78` to 1, a projectile probably follows its anchor character until the server first moves it (inference). What each pass achieves beyond that is not established.

#### Roster and proxied players

- **Create (sub 4).** `00b71bd0` (unknown label) builds a CreateProxiedPlayer object, reads the body (`00b70ee0`, likely), and accepts it only if the guid differs from the "none" guid constants `DAT_01be9940`/`DAT_01be9944`, the slot is below 8, no node with that guid exists yet (`00b70720`, likely) and a node can be appended (`00b702b0`, likely). It then writes the guid into roster array element `slot`, updates the new node, and compares the guid with the local character's guid (taken through client `+0xc8`, then an object at `+0x54`, vtable slot `+0x10`). If they match, the processor stores the node pointer at `+0xc` and the guid's high half at `+0x14`, and calls `00cd51b0` (likely), which builds and sends **UpdatePlayerState**. From the offsets it reads, the `this` of `00cd51b0` is a proxied-player node, but the call site's ECX is not visible in the pseudocode. A duplicate guid, a none guid or a slot of 8 or more makes the whole message fail (read from pseudocode in this pass).
- **What UpdatePlayerState carries.** `00cd51b0` builds a sub-7 object from the node: the node's guid pair (`+8`, `+0xc`) and its dword at `+0x10`, which starts at 0 (`00cd4e70`). It sends only if the gateway at client `+0x280` exists. What that dword means is unknown (read from pseudocode in this pass).
- **Kick (sub 5).** `00b71d90` (speculative) can remove a roster node by guid (`00b70b20`, likely), but sub-opcode 5 is not in the dispatcher's jump table and `00b71d90` has no recorded caller. So the original server could not have removed a player this way unless a path outside the mapped code exists. Two orphan fragments `00b71cc8` (likely) and `00b71e80` (speculative) have no callers either (S0353).
- **UpdateData (sub 2).** The dispatcher reads a guid pair and a 4-float vector, then calls the proxied-character lookup `00984960` (likely) and returns 0 (read from pseudocode). The pseudocode shows no further use of the vector or of the lookup result. Meaning unknown.

#### Projectiles

- **Create (sub 0xb).** `00b72540` (unknown label) reads the body (`00b718e0`, likely), requires that the buffer is consumed exactly, ignores the message if the id already exists in the projectile list, otherwise appends a node (`00b70350`, likely) and runs the spawn helper `00cd5710` (unknown label) with three floats and a dword from the tail of the body. If the helper reports failure the node is removed again (`00b707e0`, likely).
- **What the spawn helper does.** It resolves the node's handle pair to a character (`00984960`), looks up a record by the final dword (`00726e30`), turns a name from it into a string, asks the object whose pointer sits at node `+0x24` to create something (a slot-1 call; the object's type is not shown, and if it returns nothing the helper stops without creating the effect object), multiplies the node's 16 floats with matrices built from the character (`00cd5350`, speculative), and gets an effect object from the physics pool (`00fdbd40`, likely) which it stores at `+0x14`. It then applies three setters to that object (`00fe0bb0`, `00fe0a00`, `00fe08a0`) and finishes with `00cd5550`. (read from pseudocode in this pass; the roles are inferred from labels and shapes)
- **Update (sub 0xd).** `00b72700` (unknown label) finds the node by id, sets its byte at `+0x78` to 1, and passes three of the vectors to thunks that jump into rigid-body style setters (`00cd5250` to `00fe2850` "RigidBodySetTransformClearVelocity", `00cd5260` to `00fe08a0` and `00cd5270` to `00fe0990`, all speculative). Which received vector goes to which setter, and what the other three are, is not resolved.
- **Destroy (sub 0xc).** `00b71ef0` (likely) finds the node by id and removes it (`00b707e0`). It returns 1 when found.
- **Authority.** The client never creates or moves a projectile on its own in the recovered code. Its only attack output is the request message, so the server owns projectile existence and motion (client; the server side is a reconstruction).

#### Controller, input and camera

- **Controller type.** `ControllerBattleMages` is controller type `0x11` (`00af8df0`, read from pseudocode and bytes). It shares a template with the ship, fishing, soccer and derby controllers: slot 9 stub, slot 50 input step, slot 51 per-frame update (shard S0330).
- **Input step (slot 50, `00afab40`, unknown label).** It walks the controller's action list. Action ids `0x13` to `0x15` set the controller's slot index to `id - 0x13` and call `00af9590`. Ids `0x22` to `0x24` call `009c9650` with 0 to 2 (unknown). Id `0x26` toggles a flag at `+0xad`. Id `0x2a` calls `00af8fc0` when the byte at `+0x4d` is 0 (unknown label; it sets or clears that byte and makes several calls whose meaning is not established). Directional entries (kind 2: ids 1, 2, 7, 8) set forward, reverse and turn values, and the step ends by calling `00afa5e0` with a clamped time (label text accepted; I did not re-read every branch).
- **Ability slots (`00af9590`, speculative).** For a slot index below 2 and when the processor exists, it calls the attack sender `00b72e80` with the index. For an index of 2 or more it calls `008a7eb0(index+1, 0)`, and when a flag in the owner's sub-object is set it runs a probe (`00fd4290`, flags `0x103`) and `00a35860`. The meaning is not established. The label leaves open whether the index is an action-bar slot or a spell; what the slots do is unknown.
- **Movement (`00afa5e0`, speculative).** It turns toward the input with a clamped step, rotates the controller orientation and calls `00963840` (`QueuePositionAndDirectionUpdate`, confirmed), the ordinary movement queue. So walking is not a Battle Mages message in the recovered code; position goes out through the normal movement path (client; whether the original server also used UpdatePlayerState for position is unknown).
- **Camera follow (slot 51, `00af9690`, unknown label).** A 3.2 KB per-frame function. It scales time, integrates an orientation at `+0x100..+0x10c`, clamps the values at `+0xf0` and `+0xf4` between limits taken from the camera block, runs several scene queries (`00fd4290`, flags `0x80000103` and `0x80000003`; their meaning is not established) and writes a smoothed position into `+0x110..+0x11c`, which a getter (`00af8ea0`, speculative) copies out. The shape matches a chase camera. The probe meaning is not established.
- **Camera block.** The block holds eight floats. The roles below come from the `/bm_camera` command, the controller getters `00af8f00` to `00af8f60` and the constructor defaults (all speculative labels; defaults read from the executable's data in this pass).

| Offset | Role | Default | Evidence |
|---|---|---|---|
| `+0x00` | Divisor used by the ratio blend functions `00af91a0` and `00af9490` | 2.0 | speculative |
| `+0x04` | Unknown | 2.0 | `0107b840` likely |
| `+0x08` | Starting camera distance (passed to controller virtual `+0x5c`) | 12.0 | `00af9030`, `/bm_camera distance` |
| `+0x0c` | Minimum camera distance | 1.5 | `00af8f00`, matches the controller's own default |
| `+0x10` | Maximum camera distance | 12.0 | `00af8f20` |
| `+0x14` | Starting pitch (virtual `+0x68`, radians) | 0.5236 rad (30 degrees) | `00af9030`; `0107b840` |
| `+0x18` | Pitch limit (lower) | -0.98175 rad (-56.25 degrees) | `00af8f40`; `0107b840`, and the controller's own fallback `01b58684` |
| `+0x1c` | Pitch limit (upper) | +0.98175 rad (+56.25 degrees) | `00af8f60`; `0107b840`, and the controller's own fallback `01b58688` |

`0107b840` (likely) stores the pitch defaults as degrees (30, -56.25, +56.25) times 0.0174533 (data at `01ba2de8`). The constant `017ed828` is 57.2958, the radians-to-degrees factor, so `/bm_camera` prints pitch in degrees. All constants read from the executable's data in this pass. A string follows the eight floats (an `SoeUtil::IString` at block `+0x20`); the debug command's text calls it a camera configuration name (see below).

#### Developer commands

- **Registration.** The command processor registers `bm_reloaddata` with id 1 and `bm_camera` with id 2 (`00cd5b90`, confirmed; `00cd5b30`, speculative, does it again on entry).
- **Dispatch.** `ProcessCommand` (`00cd69e0`, confirmed) handles type 1 by sending a Command message with command id 1 (via `00cd62c0`, speculative, and `00cd6110`, likely) and type 2 by running the `/bm_camera` parser `00cd62f0` (likely). Both are gated on a client virtual predicate at vtable offset `+0x28` whose meaning is unresolved; if it returns false the command does nothing (`00cd62c0` and `00cd62f0` pseudocode). Whether shipped players could ever run these is therefore unknown.
- **`/bm_camera`.** Usage text in the client: `debug [on | off]`, `load [name]` (loads a camera configuration, or reloads the current one), `distance [dist] [min] [max]` and `pitch [pitch] [min] [max]` (pass `-` to leave the current value). Messages: "There is no active Battle Mages controller, ignoring command." when no controller of type `0x11` is found (`00b6f780`, likely), and "Camera configuration '%s' reloaded." (client strings). `distance` and `pitch` change the controller locally (virtuals `+0x5c`, `+0x60`, `+0x68`, `+0x6c`). `debug` sets a byte at controller `+0x140`. `load` with a name builds a Command message with command id 3 and the name as its string (`00cd6230`, likely) and sends it. `load` with no name, when the controller already holds a block, reapplies it locally (`00af9030`) instead of sending. (read from pseudocode in this pass)

### Data and formats

**Client resources.** There is no Battle Mages data file in the shipped `Resources` folder, no row in `MiniGameData.txt` (783 rows; type ids are listed in [[Minigame Framework]]) and no Lua code or panel for it (`ScriptsBase.lua` has no match for "battlemage" or "bm_"; the 251 matches for "mage" there are other words). The camera configuration and the effect records the spawn helper looks up must come from the server or from packed assets, but no asset name in the extracted pack catalogue says so. (client resource, client Lua)

**Asset names.** The pack catalogue (`manifests/rust-asset-catalog.tsv`) has twelve files whose names start `test_bm_`: `test_bm_redflag_bbe`, `test_bm_greenflag_bbe` and `test_bm_blueflag_bbe`, each with `.adr`, `.dsk`, `.cdt` and `.dma` versions. The `bm` matches the command prefix and the "test" prefix suggests development assets. Nothing links them to a flag mechanic. (client resource; interpretation unproven)

**Locale.** "Battle Mage" (ids 2035897657 and 2294936351) and "Battle Mage Gear" (4286859379) exist. They follow the pattern of "Wizard", "Warrior" and "Warrior Gear" (client locale). No locale text mentions a Battle Mages arena, round, score or prize.

**The processor in memory** (from the constructor `00b71200`, read from pseudocode):

| Offset | Content |
|---|---|
| `+0x00` / `+0x04` / `+0x08` | vtable; asset manager pointer; client pointer |
| `+0x0c` / `+0x10` / `+0x14` | local player's node (set by sub 4); game-state dword (set by sub 6); set to the local guid's high half by sub 4, starts at -1 |
| `+0x18` | `SoeUtil::Array` of eight 8-byte character guids (vtable, data, size, capacity) |
| `+0x28` / `+0x29` | active byte (set by `00b72f60`); loading byte the world-ready barrier waits on |
| `+0x2c` / `+0x3c` | player list and projectile list (head at `+0x30` and `+0x40`; node links at `+0x20`/`+0x24` and `+0x80`/`+0x84`) |
| `+0x4c` / `+0x50` | `BattleMagesCommandProcessor`; a dword copied from the camera message |
| `+0x54` | camera block (eight floats and a string) |

**Wire primitives.** All reads are raw little-endian copies from the received byte span. A value read past the end becomes 0 and sets an overflow byte in the reader. A float that is NaN sets a second flag. A string is a 32-bit length then the bytes (`00911c30` writer, `00b71000` reader). A handle (guid) is two 32-bit words (`008fb810` writes eight bytes). A "vec4" is four 32-bit floats (`008e2410`). (client; `00b70e40` likely, `00b71000` likely, `008e2410` read from pseudocode)

### Network

**Envelope.** Battle Mages messages are tunnelled inside the client packet (`ClientGateway::PacketTunneledClientPacket`, constructor `0089adc0`, confirmed; sender `00925640`, unknown label, argument flag 1). Sends happen only when the transport pointer at client `+0x280` exists. Every body starts with two 16-bit words written by `00b72160` (speculative): the family value (`0x85`, 133) and the sub-opcode (reader `00b70e40`, likely). The sub-opcode reader sign-extends the words and the dispatcher does not compare the family value itself, so routing by family must happen before `00b728a0`. That routing code was not traced for this page (see Open questions). Body byte order on the wire is unverified because no datagram has been read; the client reads and writes native little-endian. (client)

**Sub-opcode map.** Constructors were found for sub-opcodes 1 to 9 and 0xb to 0xd. Sub 0 is the empty header object `00b6f250`. No constructor was found for sub 0xa or for 0xe and above. Subs 1, 3, 7 and 9 are only built by senders, subs 2, 4, 6, 8, 0xb, 0xc and 0xd are routed to handlers, and sub 5 has a handler but no route.

**Server to client (handled by `00b728a0`, confirmed; sub-opcodes 2, 4, 6, 8, 0xb, 0xc and 0xd only; any other value falls through and returns 0).**

| Sub | Class (ctor) | Body after the 4-byte header | Client action |
|---|---|---|---|
| 2 | UpdateData (`00b6fb80`, confirmed) | u32, u32 (a handle), vec4 (`00b71570`, likely) | Proxied-character lookup; dispatcher returns 0. Meaning unknown. |
| 4 | CreateProxiedPlayer (`00b6fd30`, confirmed) | u32, u32 (guid), u32 slot, u8 flag (`00b70ee0`, likely) | Roster add as described above. Message is 17 bytes. |
| 6 | UpdateGameState (`00b6f520`, likely) | u32 (`00b71490`, likely) | Stored at processor `+0x10`. Exact length required. |
| 8 | CameraConfig (`00b70c20`, confirmed) | u32, 8 x f32, u32 length, bytes (`00b72090`, `00b71000`, likely) | Copies the u32 to processor `+0x50` and the floats to `+0x54..+0x70`, then applies the block only if the active controller, or its fallback object at `+0x78`, has type `0x11` (`00b72200`, unknown label). Always returns success once parsed. |
| 0xb | CreateProxiedProjectile (`00b705b0`, confirmed) | u32 id, u32, u32 (handle), 4 x vec4, 3 x f32, 2 x u32 (`00b718e0`, likely). 100 bytes in all. | Spawns as described above; exact length required. |
| 0xc | DestroyProxiedProjectile (`00b6f670`, confirmed) | u32 id (`00b71530`, likely) | Removes the node. |
| 0xd | UpdateProxiedProjectile (`00b70100`, confirmed) | u32 id, 6 x vec4 (`00b71a70`, likely). 104 bytes in all. | Forwards three vectors; exact length required. |
| 5 | KickProxiedPlayer (`00b6fe40`, likely) | u32, u32 (`00b714e0`, speculative) | Handler exists, but the dispatcher does not route sub 5. |

In `00b71af0` (unknown label, sub 6), `00b72540` (sub 0xb) and `00b72700` (sub 0xd) the check is "no overflow and the cursor reached the end", so a body with trailing bytes is rejected. The NaN flag is set by float reads but the handlers read here test only the overflow byte (read from pseudocode in this pass).

**Client to server (senders).**

| Sub | Class (ctor) | Body after the header | Sender |
|---|---|---|---|
| 1 | Command (`00cd5cc0`, unknown label) | u32 command id, 3 x u32, u32 length, bytes (serializer `00cd5e90`, likely). 24 bytes plus the string. | `00cd6110` (id from caller; id 1 for `/bm_reloaddata`), `00cd6190` (four supplied values, no caller found), `00cd6230` (id 3 plus a name, `/bm_camera load`); wrapper `00cd5fe0` (likely) |
| 3 | RegisterPlayer (`00b6fc50`, confirmed) | u64 guid (`00b72b60`, unknown label). 12 bytes. | `00b72dd0` (likely), on entering and on leaving |
| 7 | UpdatePlayerState (`00cd4d90`, confirmed) | u64 guid, u32 (serializer `00cd5000`, speculative). 16 bytes. | `00cd51b0` (likely) through `00cd5080` (unknown label) |
| 9 | RequestAttack (`00b6ffb0`, likely) | u64 guid, u32 slot index, u32 (from the local player object `+0x390`), f32 (the float at player `+0x394`, negated). 24 bytes. | `00b72e80` (likely) through `00b72c80` (unknown label), called from `00af9590` for slot indexes 0 and 1 |

Notes on the sender layouts, read from pseudocode and raw bytes in this pass:

- The pseudocode for `00b72e80` shows no argument, but the function ends in `ret 4` and stores its stack argument into packet `+0x18` (checked in the executable's bytes), so the slot index is sent. The constructor default for that field is `0xffffffff`.
- The constant `DAT_017ea5a4` that `00b72e80` subtracts from is negative zero (`0x80000000`), so the last field is just the negation of the float at `+0x394`. The shard hypothesis that this is a "relative coordinate or mirrored axis" is therefore a negation and nothing more. The shard hypothesis that this is a periodic keepalive is contradicted by the call chain: the only caller is the ability-slot function, and the class is named RequestAttack.
- What the dword at player `+0x390` and the float at `+0x394` are is unknown. `00b72e80` reads them from the object at client `+0xe8`, which the label calls the local player object.
- A Command message carries the client's request only. What the server did with command ids 1 and 3 is unknown.

### What the Rust server must do

Items marked (R) are reconstructed policy, not original behaviour. No Battle Mages game rule is recoverable, so this list covers what the client needs to enter, hold and leave the mode, and marks where we must design rules ourselves.

1. **Decide whether to build it.** The client code is complete, but no text, UI, asset or data row presents it as a player-facing game (see Overview). Build it after the live activities. (R)
2. **Frame and route.** Unwrap the tunnelled client packet, read two little-endian 16-bit values (family 133, then the sub-opcode), and dispatch. Answer through the same tunnel. Use exact lengths from the tables above: the client rejects a body that is too short and, for subs 6, 0xb and 0xd, one that is too long. See [[Gateway and Transport]] for the tunnel.
3. **Enter.** After the player exists in the zone, set bit 21 of the character status flags in the normal player update (subtype `0x14`, see Entering and leaving), with the correct world, zone and any minigame state set up first ([[Minigame Framework]]). The client then switches controller, registers its commands and sends RegisterPlayer. Which arena, map or zone the activity runs in is unknown (R).
4. **Register and roster.** On RegisterPlayer (3), bind the guid to the sending session and ignore the payload guid if it is not the sender's own character (R). Assign a free slot from 0 to 7. For every roster member send CreateProxiedPlayer (4) to every participant, including each participant's own entry, with a non-none guid, a unique slot and a flag byte (value and meaning unknown; send 0 (R)). The client answers its own entry with UpdatePlayerState (7); store it and do not rely on its dword (meaning unknown). Treat a second RegisterPlayer from a registered player as a leave only if you have no better signal (R), because the client sends the same message both ways.
5. **Game state.** Send UpdateGameState (6) with one u32. The meaning is unknown; keep a documented placeholder enum and change it only with new evidence (R).
6. **Camera.** Sending CameraConfig (8) is optional. If you send it, send it after the controller exists (the client only applies it while the controller type is `0x11`), with eight finite floats following the table above and a string you choose. If you do not, the client uses the defaults in its processor constructor (distance 1.5 to 12.0 starting at 12.0, pitch from -56.25 to +56.25 degrees starting at 30 degrees). (R)
7. **Attacks and projectiles.** On RequestAttack (9), treat the guid and slot index as claims and validate them against the session (R). Spawn with CreateProxiedProjectile (0xb) using a unique id per live projectile, a handle that resolves to a character the client knows, a transform and the tail fields (their meaning is unresolved, see Open questions), stream UpdateProxiedProjectile (0xd) with six vectors, and send DestroyProxiedProjectile (0xc) at the end. All motion, collision and damage is server design (R); the client sends nothing back when a projectile hits.
8. **Damage, health, score and results.** No Battle Mages message carries them. Use the generic combat, minigame-result and reward paths from [[Combat and Abilities]], [[Minigame Framework]] and [[Rewards and Collections]] if the design needs them (R).
9. **Leave.** Ending the activity by clearing bit 21 is not enough, because the only recovered turn-off is the zoning teardown. End a session by moving the player through a zone transition (R), and clean up the roster and projectile nodes when a participant leaves. Do not send sub 5 (Kick); the client does not route it.
10. **Commands.** Log Command (1) messages. Do not rely on them. If you want `/bm_camera load` to work, answer command id 3 with a CameraConfig for the requested name (R, speculative). Id 1 (`bm_reloaddata`) most likely asked the server to reload its data; no client evidence says more.
11. **No capture, no original server.** There is no original server code to compare against. Every behaviour above is built from what the client reads and writes. Re-check the open questions below with a native run before treating any of it as settled. (R)

### Video and wiki evidence

- **Video.** No reviewed frame names Battle Mages. `vid-grep` for "Battle Mage" and "battlemage" returned nothing. A few frames from one source video carry a job label read as "Mage" or "Mage?"; which job that is cannot be told from the record, and it is not evidence for this activity. (video)
- **Wiki.** There is no wiki page for Battle Mages. The wiki "Mini-Games" page lists Archaeology, Board Games, Crafting, Harvesting, Mining, Recipes, Racing, Demolition Derby, Kart Racing, Tower Defense, Pirate's Plunder, Hack n' Slash, Soccer, TCG and Feeling the Beat, and no Battle Mages. The wiki "Jobs" page lists Wizard (members only) and no Battle Mage job. (wiki)
- **Unrelated "Bixie Battlemage".** The wiki page "Bixie Battlemage" is a Difficulty 1 enemy bixie NPC in Queensfields and Highroad Junction (also called "Bixie Magus" in Highroad Junction). The client locale has matching enemy text: an ability called "Battlemage Particle Blast", "A direct damage blast from the staff of the bixie battlemage.", a buff that raises nearby soldiers' and pikemen's weapon damage by 25%, a summon of "2 bixie soldiers and a battlemage", and the quest "Spray Bixie Battlemages" (ids 595559788, 2728125950, 3704142569, 3205007484, 129024187, 3746549926). These share only the word. (client locale, wiki)
- **Weak lead.** The client locale has a housing-style "Capture The Flag Trophy" (ids 313654616 and 1631681041). The only other flag-like evidence is the `test_bm_*flag` assets. No code or text connects the two to Battle Mages. (client locale, client resource)

### Open questions and contradictions

- **What the game is.** Rules, win and lose, teams, scoring, health, damage values, prizes and what an attack does are all unknown. No message carries them, the client has no text for them, and no video or wiki page describes them. The name implies mages and spells; that is an inference.
- **Was it ever live.** There is no minigame-table row, no Lua, no locale text and only test-prefixed assets. The shipped code may be a prototype. Unknown.
- **How it was entered.** Only the player-update status bit 21 arms it. Which world object or activity request made the server set the bit, and which arena was used, is unknown.
- **How it ended.** The recovered exit is zoning. The bit-clear direction is not seen in `0092f460`, the Kick handler is unreachable and no leave message exists. The original server may have ended the mode only by moving the player. Unknown.
- **Routing from the tunnel to the processor.** No function that hands a received tunnelled body to `BattleMagesProcessor` slot 4 was traced for this page. The family value `0x85` is only seen in the packet constructors. How the client chooses this processor, and whether it compares the family, is unresolved.
- **Identical RegisterPlayer on enter and leave** (raw bytes). The pseudocode label calls `00b72dd0`'s flag a real flag, but its middle argument is unused. Whether the original server could distinguish enter from leave is unknown.
- **Field meanings.** The slot (a roster index below 8), the flag byte in CreateProxiedPlayer, the dword in UpdateGameState, the dword stored at processor `+0x50` from CameraConfig, the dword in UpdatePlayerState, the two trailing dwords and three floats at the end of CreateProxiedProjectile, the three unused vectors of UpdateProxiedProjectile, the dword and float read from the player object in RequestAttack, and the first two camera-block floats are all unnamed.
- **UpdateData (sub 2).** Parsed, then only a lookup is visible. It may drive something the pseudocode lost, or nothing.
- **Command ids.** Ids 1 and 3 are sent. Id 2 (the registration id of `bm_camera`) is not used as a message id in the code read. The `00cd6190` variant with four values has no caller.
- **Debug predicate.** The virtual at client vtable `+0x28` that gates `bm_reloaddata` and `bm_camera` is not identified.
- **Shard contradictions.** S0353 calls `00b72e80` a periodic state or keepalive report and reads `DAT_017ea5a4` as a possible coordinate offset. The call chain (only caller is the ability slot function) and the constant (negative zero) do not support that; this page treats it as a RequestAttack sender. S0330 suggests the movement function may send a position; the call goes to the ordinary movement queue (`00963840`, confirmed) instead.
- **Decompiler caveat.** The pseudocode for `00b72e80` hid a stack argument. I checked its raw bytes. Other "no argument" readings in this range may need the same check.
- **Locale names.** "Battle Mage" and "Battle Mage Gear" in the client locale may belong to a class or item category, or to this activity. The code does not use them.
- **Assets.** The `test_bm_*flag` assets and the "Capture The Flag Trophy" text may or may not be related. Nothing ties them to code.
- **Unresolved callee roles.** `008a7e00`, `008d2830`, `008a68f0`, `00cd5b70`, `00af8fc0`, `009c9650`, `008a7eb0`, `00a35860` and the probe flags are not understood.

### Key functions

| Address | Name | Confidence |
|---|---|---|
| `00b728a0` | BattleMagesProcessor::DispatchPacket (receive switch) | confirmed |
| `00b71200` | BattleMagesProcessor constructor | unknown |
| `00b71320` | BattleMagesProcessor destructor | confirmed |
| `00b72f60` | Mode switch (enter and leave) | unknown |
| `0092f460` | BaseClient::DispatchBasePlayerUpdate (arms the mode) | confirmed |
| `0093d180` | BeginPlayerZoning (turns the mode off) | confirmed |
| `00936d00` | WaitForWorldReady (Battle Mages barrier) | confirmed |
| `00b70ad0` | Per-frame walk of players and projectiles | likely |
| `00b6f250`, `00b6f2b0` | Family `0x85` base constructors | unknown |
| `00b6fb80`, `00b6fc50`, `00b6fd30`, `00b70c20`, `00b705b0`, `00b6f670`, `00b70100`, `00cd4d90` | Subclass constructors for subs 2, 3, 4, 8, 0xb, 0xc, 0xd, 7 | confirmed |
| `00b6fe40`, `00b6f520`, `00b6ffb0` | Subclass constructors for subs 5, 6, 9 | likely |
| `00cd5cc0` | Command packet constructor (sub 1) | unknown |
| `00b70e40` | Two-u16 header reader | likely |
| `00b70ee0`, `00b71000`, `00b71490`, `00b71530`, `00b71570`, `00b718e0`, `00b71a70`, `00b72020`, `00b72090` | Body readers | likely |
| `00b71bd0`, `00b71af0`, `00b72200`, `00b72540`, `00b72700` | Handlers for subs 4, 6, 8, 0xb, 0xd | unknown |
| `00b71ef0` | Handler for sub 0xc | likely |
| `00b71d90`, `00b71cc8`, `00b71e80` | Unreachable kick handler and orphan fragments | speculative, likely, speculative |
| `00b72160`, `00b72450`, `00cd5000` | Header and body writers | speculative, likely, speculative |
| `00b72b60`, `00b72c80`, `00cd5080` | Tunnel wrappers and senders | unknown |
| `00b72dd0`, `00b72e80`, `00cd51b0` | RegisterPlayer, RequestAttack, UpdatePlayerState senders | likely |
| `00cd5e90`, `00cd5fe0`, `00cd6110`, `00cd6190`, `00cd6230` | Command serializer, wrapper and builders | likely |
| `00cd5710` | Projectile spawn helper | unknown |
| `00cd5350`, `00cd5550`, `00cd5250`, `00cd5260`, `00cd5270` | Matrix compose, re-apply, rigid-body thunks | speculative, unknown, speculative |
| `00cd5b90`, `00cd69e0`, `00cd5c20` | Command processor constructor, ProcessCommand, destructor | confirmed |
| `00cd5b30`, `00cd62c0` | Register commands, command gate | speculative |
| `00cd62f0`, `00b6f780` | `/bm_camera` parser, controller type-17 lookup | likely |
| `00afa420`, `00afab40`, `00af9690` | ControllerBattleMages constructor, input step, per-frame update | unknown |
| `00af9590`, `00afa5e0`, `00af9230`, `00af9030`, `00af8ea0`, `00af8f00`, `00af8f20`, `00af8f40`, `00af8f60` | Controller slot dispatch, movement, camera block handling | speculative |
| `00af8df0` | Controller type getter (returns `0x11`) | unlabelled |
| `008b8ff0`, `008c5d90`, `009162f0` | Controller factory, controller switch, client bring-up | unknown, likely, unknown |
| `0107b840` | Camera block defaults | likely |
| `00963840` | QueuePositionAndDirectionUpdate (movement path) | confirmed |
<!-- NARRATIVE END -->

## Mapped functions (generated)

88 accepted functions: 36 confirmed, 13 likely, 25 speculative, 14 unknown. Network-linked (`wire`): 18.

Subsystems: `minigame.battlemages` (48), `net.battlemages` (40)

### `minigame.battlemages`

**BattleMagesCommandProcessor** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cd5b90` | BattleMagesCommandProcessor::BattleMagesCommandProcessor | confirmed | Stores CommandProcessor::CommandEventHandler::vftable and then BattleMagesCommandProcessor::vftable at this; stores arg2 at +0x4; calls 00987000 with bm_reload… |
| `00cd5c20` | BattleMagesCommandProcessor::~BattleMagesCommandProcessor | confirmed | Stores BattleMagesCommandProcessor::vftable at this, calls 00986c10 with this, then stores CommandProcessor::CommandEventHandler::vftable at this; returns void. |
| `00cd69e0` | BattleMagesCommandProcessor::ProcessCommand | confirmed | Branches on command type values 1 and 2 and dispatches the matching command processing path. |
| `00cd5ca0` | BattleMagesCommandProcessor::CallDtorBodyThenFreeIfFlag | likely | Virtual slot 1 of BattleMagesCommandProcessor: calls destructor body 00cd5c20 on this, frees this when bit 0 of the flag argument is set, and returns this. |

**BattleMagesProxiedPlayer** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cd4d50` | BattleMagesProxiedPlayer::~BattleMagesProxiedPlayer | confirmed | Stores BattleMagesProxiedPlayer::vftable at [ECX] (mov dword [ecx],0x0183972c; ret). Nothing else. |
| `00cd4e70` | BattleMagesProxiedPlayer::BattleMagesProxiedPlayer | confirmed | Stores BattleMagesProxiedPlayer::vftable at this; stores arg2 at +0x8 and arg3 at +0xc; stores arg4 (byte) at +0x14; zeroes +0x10, +0x18 and +0x1c; then loads … |
| `00cd4eb0` | BattleMagesProxiedPlayer::SetVtableThenFreeIfFlag | confirmed | Virtual slot 0: stores BattleMagesProxiedPlayer::vftable at this, frees this when bit 0 of the flag argument is set, and returns this. The destructor body 00cd… |

**BattleMagesProxiedProjectile** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cd5280` | BattleMagesProxiedProjectile::~BattleMagesProxiedProjectile | confirmed | Stores BattleMagesProxiedProjectile::vftable at this. When the dword at +0x14 is non-zero it clears its +0x1c, sets its +0x20 to -1, calls 00769710, releases t… |
| `00cd5330` | BattleMagesProxiedProjectile::ScalarDeletingDtor | confirmed | Calls destructor body 00cd5280 on this, then frees this when bit 0 of the flag argument is set; returns this. |
| `00cd52d0` | — | unknown | Stores arg2 at +0x10 and BattleMagesProxiedProjectile::vftable at this; zeroes +0x14, +0x18 and +0x1c; stores arg6 at +0x20 and the dword at arg6+4 at +0x24; s… |

**ControllerBattleMages** (20)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00af8da0` | ControllerBattleMages::Dtor | confirmed | Fastcall destructor body (ECX = this): stores ControllerBattleMages::vftable at +0, sets the SEH state to -1, calls base Controller destructor 0094fab0 and ret… |
| `00af9140` | ControllerBattleMages::ScalarDeletingDtor | confirmed | Thiscall scalar deleting destructor body: stores ControllerBattleMages::vftable at +0, calls base destructor 0094fab0, then frees this with _free when bit 0 of… |
| `00af8e00` | ControllerBattleMages::SetF124AndFlag120 | speculative | Virtual at ControllerBattleMages vtable slot 18: stores the dword argument at this+0x124 and sets the byte at this+0x120 to 1. |
| `00af8e20` | ControllerBattleMages::ClearFlag120 | speculative | Virtual at ControllerBattleMages vtable slot 19: sets the byte at this+0x120 to 0. |
| `00af8e30` | ControllerBattleMages::GetF0 | speculative | Getter at ControllerBattleMages vtable slot 27: returns the float at this+0xf0 as float10. |
| `00af8e40` | ControllerBattleMages::VcallA4AfterBaseCall | speculative | Fastcall virtual at ControllerBattleMages vtable slot 9. Calls 0094e210 with no arguments, then virtual call +0xa4 on this with the global constant at 0x01814c… |
| `00af8e70` | ControllerBattleMages::GetF124OrCall | speculative | Virtual at ControllerBattleMages vtable slot 16: when the byte at this+0x120 is non-zero returns the float at this+0x124; otherwise returns the float result of… |
| `00af8ea0` | ControllerBattleMages::CopyVec4_110 | speculative | Thiscall virtual at ControllerBattleMages vtable slot 32: copies the four dwords at this+0x110..+0x11c to the output pointer and returns 1. |
| `00af8f80` | ControllerBattleMages::AtanF100F108OrZero | speculative | Virtual at ControllerBattleMages vtable slot 29: when the squared length of (+0x100, +0x108) exceeds the float at 0x017ebbb8 returns fpatan of those two floats… |
| `00af91a0` | ControllerBattleMages::ComputeRatioBlend | speculative | Virtual at ControllerBattleMages vtable slot 24: q = this+0xe0 / the block's first float at +0x13c (default 0x0181c3bc); r = min(q, 1.0); returns the float10 (… |
| `00af92a0` | ControllerBattleMages::SetVec4FromAxisAngle | speculative | Thiscall virtual at ControllerBattleMages vtable slot 28: calls FUN_00770e70 with a local constant vector (0, 1.0, 0, 0) and the float argument, scales the ret… |
| `00af9350` | ControllerBattleMages::ComputeNegXformVec4 | speculative | Virtual at ControllerBattleMages vtable slot 33: builds a normalised 3-vector from cross-product terms of this+0x100..+0x10c, calls FUN_00770e70 with the value… |
| `00af9490` | ControllerBattleMages::SolveBlendToEc | speculative | Virtual at ControllerBattleMages vtable slot 23: q = this+0xe0 / block[0] (default 0x0181c3bc), r = min(q, 1.0), computes (param - block[3] * q) / (r * block[4… |
| `00af9540` | ControllerBattleMages::SetF4ClampedOrF0 | speculative | Virtual at ControllerBattleMages vtable slot 26: when the pointer at this+0x13c is non-null, clamps the argument into [block+0x18, block+0x1c] and stores it at… |
| `00af9590` | ControllerBattleMages::DispatchSlotIndex | speculative | Thiscall with an int index. For index < 2 and owner(+0x74)+0x124 non-zero, calls 00b72e80(index) and returns. For index >= 2 calls 008a7eb0(index+1, 0); then, … |
| `00afa5e0` | ControllerBattleMages::StepMoveFromInput | speculative | Thiscall with a pointer to a four-float input (read from param_2: two floats, a third float and a flag byte at +0xc). Updates a turn value at +0x128 toward the… |
| `00af8fc0` | — | unknown | Thiscall with a byte argument: when non-zero calls virtual +0x5c with 0, 008a8cb0(0x20), 00a46010(2,1), 00a46030 and sets the byte at this+0x4d to 1; when zero… |
| `00af9690` | — | unknown | Thiscall virtual at ControllerBattleMages vtable slot 51, with a time argument scaled by the float at 0x017ebf0c. Gets the local client object through 0094cf60… |
| `00afa420` | — | unknown | Thiscall constructor with three stack arguments forwarded to base constructor 0094f6c0 (Controller). Stores ControllerBattleMages::vftable at +0, initialises f… |
| `00afab40` | — | unknown | Thiscall virtual at ControllerBattleMages vtable slot 50. Calls virtual +0x40 on this and stores its float result at client+0x3ec (client = object from 0094cf6… |

**(no class)** (18)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cd6110` | BuildBattleMagesCommand 🌐 | likely | Copies command data into the Battle Mages packet object and sets its command fields before handing it to the shared send routine. |
| `00cd6190` | SetBattleMagesCommandFields 🌐 | likely | Writes supplied values into the command object's Battle Mages payload fields. |
| `00cd6230` | BuildBattleMagesCommandFromInput 🌐 | likely | Reads the supplied input value, applies the local command conversion, and invokes the Battle Mages command construction path. |
| `0107b840` | InitFloatsAndIString20 | likely | Fastcall (this in ECX): stores DAT_017ea718 at +0 and +4, DAT_0181bc48 at +8 and +0x10, and _DAT_017ed7e4 at +0xc; stores three floats at +0x14, +0x18 and +0x1… |
| `00af8f00` | ReadBlock13cF0c | speculative | Fastcall: when the pointer at this+0x13c is non-null returns the float at that block+0xc, otherwise returns the float constant at 0x0181c3c8. |
| `00af8f20` | ReadBlock13cF10 | speculative | Fastcall: when the pointer at this+0x13c is non-null returns the float at that block+0x10, otherwise returns the float constant at 0x0181c3cc. |
| `00af8f40` | ReadBlock13cF18 | speculative | Fastcall: when the pointer at this+0x13c is non-null returns the float at that block+0x18, otherwise returns the float constant at 0x01b58684. |
| `00af8f60` | ReadBlock13cF1c | speculative | Fastcall: when the pointer at this+0x13c is non-null returns the float at that block+0x1c, otherwise returns the float constant at 0x01b58688. |
| `00af9030` | ApplyBlock13cViaVcalls | speculative | Fastcall: when the pointer at this+0x13c is non-null calls virtual +0x5c with dword +0x8 of that block and virtual +0x68 with dword +0x14 of that block. |
| `00af9230` | SetBlock13cAndApply | speculative | Thiscall setter: stores param_2 at this+0x13c; when it is non-zero calls virtual +0x5c with dword +0x8 of that block and virtual +0x68 with dword +0x14 of that… |
| `00cd5250` | Thunk_Member14_00fe2850 | speculative | Jump thunk: loads ECX from [ECX+0x14] (mov ecx,[ecx+0x14]) and jumps to 00fe2850 (jmp, not call). |
| `00cd5260` | Thunk_Member14_00fe08a0 | speculative | Jump thunk: loads ECX from [ECX+0x14] (mov ecx,[ecx+0x14]) and jumps to 00fe08a0 (jmp, not call). |
| `00cd5270` | Thunk_Member14_00fe0990 | speculative | Jump thunk: loads ECX from [ECX+0x14] (mov ecx,[ecx+0x14]) and jumps to 00fe0990 (jmp, not call). |
| `00cd5b30` | RegisterReloadAndCameraCommands | speculative | Fastcall (ECX = this). Calls 00987000 with the name bm_reloaddata and id 1, then with bm_camera and id 2, passing this as the second argument. |
| `00cd62c0` | CanIssueBattleMagesCommand | speculative | Calls a virtual predicate on the controller and returns one when it is true, otherwise zero. |
| `00b72f60` | — 🌐 | unknown | Stores the new state byte at this+0x28 and, when it changed, toggles the battle-mages activity: on state 0 calls FUN_00b72dd0(controller,0,1), sets a client fl… |
| `00cd5710` | — | unknown | Thiscall with arg2..arg5. Resolves the handle pair at +0x28 and +0x2c (00769710, 00984960); when found, looks up a record with 00726e30 and calls slot 1 of the… |
| `00cd5b70` | — | unknown | Pushes its incoming ECX as a stack argument, loads ECX from dword 0x01be1090 plus 1, calls 00986c10 and returns (bytes 51 8b 0d .. 83 c1 01 e8 .. c3). 00986c10… |

### `net.battlemages`

**BaseBattleMagesPacket** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b6f320` | BaseBattleMagesPacket::~BaseBattleMagesPacket | confirmed | Non-deleting destructor body of the family base class: overwrites +0 with BaseBattleMagesPacket::vftable and then with BasePacket::vftable (the inlined base de… |
| `00b6f370` | BaseBattleMagesPacket::ScalarDeletingDestructor | confirmed | Virtual deleting destructor at vtable slot 0: writes BaseBattleMagesPacket::vftable, then BasePacket::vftable (inlined base destructor), and calls _free(this) … |
| `00b6f3e0` | BaseBattleMagesPacket::~BaseBattleMagesPacket | confirmed | Copy of the family base non-deleting destructor: writes BaseBattleMagesPacket::vftable then BasePacket::vftable into +0 and returns; byte-identical to the body… |
| `00b6f430` | BaseBattleMagesPacket::~BaseBattleMagesPacket | confirmed | Copy of the family base non-deleting destructor: writes BaseBattleMagesPacket::vftable then BasePacket::vftable into +0 and returns, with no other field or cal… |
| `00b6f480` | BaseBattleMagesPacket::~BaseBattleMagesPacket | confirmed | Copy of the family base non-deleting destructor: writes BaseBattleMagesPacket::vftable then BasePacket::vftable into +0 and returns, with no other field or cal… |
| `00b6f620` | BaseBattleMagesPacket::~BaseBattleMagesPacket | confirmed | Copy of the family base non-deleting destructor: writes BaseBattleMagesPacket::vftable then BasePacket::vftable into +0 and returns, with no other field or cal… |
| `00b6f6d0` | BaseBattleMagesPacket::~BaseBattleMagesPacket | confirmed | Copy of the family base non-deleting destructor: writes BaseBattleMagesPacket::vftable then BasePacket::vftable into +0 and returns, with no other field or cal… |
| `00b6f720` | BaseBattleMagesPacket::~BaseBattleMagesPacket | confirmed | Copy of the family base non-deleting destructor: writes BaseBattleMagesPacket::vftable then BasePacket::vftable into +0 and returns, with no other field or cal… |
| `00b6f250` | — 🌐 | unknown | Default constructor: stores BasePacket::vftable at +0, writes the family opcode 0x85 (133) into +4, stores BaseBattleMagesPacket::vftable at +0 and zeroes the … |
| `00b6f2b0` | — 🌐 | unknown | Shared constructor taking the sub-opcode as an argument: writes BasePacket::vftable at +0, opcode 0x85 (133) into +4, BaseBattleMagesPacket::vftable at +0 and … |
| `00b6f4d0` | — | unknown | Copy of the family base non-deleting destructor: writes BaseBattleMagesPacket::vftable then BasePacket::vftable into +0 and returns, with no other field or cal… |
| `00b6f580` | — | unknown | Copy of the family base non-deleting destructor: writes BaseBattleMagesPacket::vftable then BasePacket::vftable into +0 and returns, with no other field or cal… |
| `00b6f5d0` | — | unknown | Copy of the family base non-deleting destructor: writes BaseBattleMagesPacket::vftable then BasePacket::vftable into +0 and returns, with no other field or cal… |

**BattleMagesPacketCameraConfig** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b70c20` | BattleMagesPacketCameraConfig::BattleMagesPacketCameraConfig 🌐 | confirmed | Constructor for sub-opcode 8: calls the shared base constructor with 8, stores BattleMagesPacketCameraConfig::vftable at +0, zeroes the field at +0xc and const… |
| `00b70c90` | BattleMagesPacketCameraConfig::ScalarDeletingDestructor | confirmed | Virtual deleting destructor at the camera-config packet's vtable slot 0: calls the destructor body 00b70a70 (releases the camera's SoeUtil::IString<char> and r… |
| `00b70a70` | BattleMagesPacketCameraConfig::~BattleMagesPacketCameraConf… | likely | Destructor body of the camera-config sub-packet: calls 00b70a00 with this+0x10, which releases the SoeUtil::IString<char> embedded at this+0x30, then overwrite… |
| `00b71000` | BattleMagesPacketCameraConfig::ReadBody | likely | Reads eight 32-bit floats into destination offsets +0..+0x1c, setting reader+0x11 if any is NaN. Then reads a signed length; if nonnegative and within the rema… |

**BattleMagesPacketCreateProxiedPlayer** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b6fd30` | BattleMagesPacketCreateProxiedPlayer::BattleMagesPacketCrea… 🌐 | confirmed | Constructor for sub-opcode 4: calls the shared base constructor with 4, stores BattleMagesPacketCreateProxiedPlayer::vftable at +0, initialises +0x10/+0x14 fro… |
| `00b6fdd0` | BattleMagesPacketCreateProxiedPlayer::ScalarDeletingDestruc… | confirmed | Virtual deleting destructor at the create-proxied-player packet's vtable slot 0: writes BaseBattleMagesPacket::vftable, then BasePacket::vftable, and calls _fr… |
| `00b70ee0` | BattleMagesPacketCreateProxiedPlayer::ReadBody | likely | Body reader for the sub-opcode 4 packet: first reads the two header words with 00b70e40, then copies an 8-byte handle from the stream into packet+0x10/+0x14, a… |

**BattleMagesPacketCreateProxiedProjectile** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b705b0` | BattleMagesPacketCreateProxiedProjectile::BattleMagesPacket… 🌐 | confirmed | Constructor for sub-opcode 11: calls the shared base constructor with 0xb, stores BattleMagesPacketCreateProxiedProjectile::vftable at +0, zeroes +0xc, initial… |
| `00b70620` | BattleMagesPacketCreateProxiedProjectile::ScalarDeletingDes… | confirmed | Virtual deleting destructor at the create-proxied-projectile packet's vtable slot 0: writes BaseBattleMagesPacket::vftable, then BasePacket::vftable, and calls… |

**BattleMagesPacketDestroyProxiedProjectile** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b6f670` | BattleMagesPacketDestroyProxiedProjectile::BattleMagesPacke… 🌐 | confirmed | Constructor for sub-opcode 12: calls the shared base constructor with 0xc, stores BattleMagesPacketDestroyProxiedProjectile::vftable at +0 and zeroes the field… |
| `00b70090` | BattleMagesPacketDestroyProxiedProjectile::ScalarDeletingDe… | confirmed | Virtual deleting destructor at the destroy-proxied-projectile packet's vtable slot 0: writes BaseBattleMagesPacket::vftable, then BasePacket::vftable, and call… |

**BattleMagesPacketKickProxiedPlayer** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b6fed0` | BattleMagesPacketKickProxiedPlayer::ScalarDeletingDestructor | confirmed | Virtual deleting destructor at the kick-proxied-player packet's vtable slot 0: writes BaseBattleMagesPacket::vftable, then BasePacket::vftable, and calls _free… |
| `00b6fe40` | BattleMagesPacketKickProxiedPlayer::BattleMagesPacketKickPr… 🌐 | likely | Constructor for sub-opcode 5: calls the shared base constructor with 5, stores BattleMagesPacketKickProxiedPlayer::vftable at +0, and initialises +0x10/+0x14 f… |

**BattleMagesPacketRegisterPlayer** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b6fc50` | BattleMagesPacketRegisterPlayer::BattleMagesPacketRegisterP… 🌐 | confirmed | Constructor for sub-opcode 3: calls the shared base constructor with 3, stores BattleMagesPacketRegisterPlayer::vftable at +0, and copies the two dwords of the… |
| `00b6fcc0` | BattleMagesPacketRegisterPlayer::ScalarDeletingDestructor | confirmed | Virtual deleting destructor at the RegisterPlayer packet's vtable slot 0: writes BaseBattleMagesPacket::vftable, then BasePacket::vftable, and calls _free(this… |

**BattleMagesPacketRequestAttack** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b70020` | BattleMagesPacketRequestAttack::ScalarDeletingDestructor | confirmed | Virtual deleting destructor at the RequestAttack packet's vtable slot 0: writes BaseBattleMagesPacket::vftable, then BasePacket::vftable, and calls _free(this)… |
| `00b6ffb0` | BattleMagesPacketRequestAttack::BattleMagesPacketRequestAtt… 🌐 | likely | Constructor for sub-opcode 9: calls the common header constructor, installs the RequestAttack vtable, writes the invalid handle constant to +0x10/+0x14 and ini… |

**BattleMagesPacketUpdateData** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b6fb80` | BattleMagesPacketUpdateData::BattleMagesPacketUpdateData 🌐 | confirmed | Constructor for sub-opcode 2: calls the shared base constructor with 2 and stores BattleMagesPacketUpdateData::vftable at +0; it initialises no other field. |
| `00b6fbe0` | BattleMagesPacketUpdateData::ScalarDeletingDestructor | confirmed | Virtual deleting destructor at the UpdateData packet's vtable slot 0: writes BaseBattleMagesPacket::vftable, then BasePacket::vftable, and calls _free(this) wh… |

**BattleMagesPacketUpdateGameState** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b6ff40` | BattleMagesPacketUpdateGameState::ScalarDeletingDestructor | confirmed | Virtual deleting destructor at the UpdateGameState packet's vtable slot 0: writes BaseBattleMagesPacket::vftable, then BasePacket::vftable, and calls _free(thi… |
| `00b6f520` | BattleMagesPacketUpdateGameState::BattleMagesPacketUpdateGa… 🌐 | likely | Constructor for sub-opcode 6: calls the shared base constructor with 6, stores BattleMagesPacketUpdateGameState::vftable at +0 and zeroes the 32-bit field at +… |

**BattleMagesPacketUpdateProxiedProjectile** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b70100` | BattleMagesPacketUpdateProxiedProjectile::BattleMagesPacket… 🌐 | confirmed | Constructor for sub-opcode 13: calls the shared base constructor with 0xd, stores BattleMagesPacketUpdateProxiedProjectile::vftable at +0 and zeroes the field … |
| `00b70160` | BattleMagesPacketUpdateProxiedProjectile::ScalarDeletingDes… | confirmed | Virtual deleting destructor at the update-proxied-projectile packet's vtable slot 0: writes BaseBattleMagesPacket::vftable, then BasePacket::vftable, and calls… |

**BattleMagesProcessor** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b71320` | BattleMagesProcessor::~BattleMagesProcessor | confirmed | Destructor body: rewrites +0 with BattleMagesProcessor::vftable, releases the BattleMagesCommandProcessor at +0x4c through its vtable slot 1 with flag 1 and ze… |
| `00b71200` | — | unknown | Constructor: takes the manager pointer at +4 from [FUN_00769710()+0x1c], installs AssetManagement::AssetRequestInterface::AssetRequester::vftable then BattleMa… |

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cd5e90` | SerializeBattleMagesFields 🌐 | likely | Writes a two-word inner header, copies four 32-bit fields from this+0x0c through this+0x18 into the bounded writer, then serializes the length-prefixed byte fi… |
| `00cd5fe0` | SendBattleMagesCommand 🌐 | likely | Initializes a packet writer, serializes this command, constructs a tunneled client packet, submits it through the generic gateway sender, then destroys the tem… |

## Source shards

[[S0330|S0330]] (26), [[S0352 00b6cd80-00b71320|S0352]] (38), [[S0353 00b71490-00b77eb0|S0353]] (1), [[S0418 00cd1210-00cd5dd0|S0418]] (15), [[S0419 00cd5e90-00cdb1a0|S0419]] (7), [[S0579|S0579]] (1)
