# Native startup builders — run client-baseline-20261013T215401Z

Lead native check, 2026-10-13 (Denver). Release built from source after W19 startup integration (gateway builds 45 of 46 startup messages from typed state + `lab/content/startup-snapshot/startup.json`; index 07 self data still on the patch path; template fallback OFF) and W09 inventory fixes.

Save: disposable copy `lab/reports/startup-native-20261013T213340Z/` of the kart-weapons-probe-v20 character (never the canonical save).

Observed in the original client:
- Native login and character select (Go!) work.
- Welcome screen (opcode 93, built) renders; "Popular Items: undefined" also appears in the previous run 20261010T064034Z, so not a regression.
- World entry: HUD, minimap, friends panel, health 1000/1000 and energy 100/100 (38/1, 38/13 built), ability bar, NPCs present.
- Inventory opens with 359 coins and full item list; **text search "hammer" filters to 4 items and the client stays responsive** (previous crash path; W09 withholds owned items without definitions). Static evidence + this run; not proof for every search string.
- Developer `realm goto Mae` teleports to Mae Farnum with the interaction prompt; position saves.
- Normal Exit Game -> Yes closes the client; launcher exit 0; 1,557 datagrams captured.

Not checked: full capture CRC/reassembly analysis, appearance/equipment parity with the previous run (character select showed shirt/trousers; in-world model looked lighter-dressed — compare when self-data W18 lands), multiplayer, quests. Screenshots: `lead-02` … `lead-10` in the run folder.
