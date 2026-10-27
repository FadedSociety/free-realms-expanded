//! T378: every enabled quest giver and talk target carries a recovered greeting.
//!
//! Content only: `NpcInteraction::dialogue_context` already copies `greeting_string_id`
//! into `DialogueContext.body_string_id`, and `gameplay/npc.rs::dialogue_packet_with`
//! writes it into the first 26/3 payload dword, so this slice adds no packet, handler,
//! selector, table or flag.
//!
//! Client evidence re-checked in IDA for T378: 26/3 handler `0x00aa12e0` calls payload
//! reader `0x00a9ef10`, which stores the first payload dword at object `+16`
//! (`*(this + 16) = *v3`), then calls display routine `0x009f5ae0`, which passes
//! `*(a2 + 16)` to the `Global.Text.<id>` lookup `0x0097c1e0` immediately before
//! `:SetDialogText`. That is the first dword after the four-byte family/subtype header,
//! matching `encode_dialogue_26_3`'s `initial_dwords[0]`; `patch_npc_actor` overwrites
//! only bytes 12..20 with the runtime actor GUID. The full byte order is pinned by
//! `tests/npc_interaction.rs::dialogue_26_3_synthetic_fixture_round_trips_exact_native_field_order`.
//!
//! Locale ids, JenkinsLookup2 hashes, `en_us_data.dat` lines, speaker proofs, the
//! reconstructed-field flags and every blank row's reason are in
//! `notes/Autopilot/T378.md`.
//!
//! The enabled quest set is derived from the production split loader
//! (`QuestEngine::from_split_content`, the call `Gameplay::bind_npc_quest_catalogue`
//! makes from the same directory). `QuestEngine` has no public all-definitions
//! iterator, so the test enumerates the same `quests/*.json` records the loader reads
//! and validates every enabled flag and giver through `QuestEngine::is_enabled` and
//! `QuestEngine::giver_npc_id`; a `talk_to_npc` objective contributes its `npc_id`.
//!
//! The packet builders, the locale line convention and the 26/3 body offset live in
//! the shared `tests/common/npc_greetings.rs` harness.

#[path = "common/npc_greetings.rs"]
mod npc_greetings;

use npc_greetings::{
    GREETING_HEADER, MENU_HEADER, character_at, content_dir, gameplay_dir, gameplay_talk_body,
    greeting_bindings, interact, lab_report, locale_id_to_text, locale_rows, npc_position,
    npc_record, npc_records, read_json, select, service, service_talk_body,
};
use osfr_tools::{
    gameplay::Gameplay,
    npc_interaction::{NpcAction, NpcInteractionError, NpcSelection},
    quest_engine::{CharacterEligibility, QuestEngine},
    world::NpcVisibility,
    world_text::locale_hash,
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

const COBBLESTONE_GUIDE: &str = "notes/Build Guides/Content/NPCs/NPCs - Cobblestone Village.md";
const CRACKED_CLAW_GUIDE: &str = "notes/Build Guides/Content/Dungeons/Cracked Claw Caverns.md";
const POSTMAN_GUIDE: &str = "notes/Build Guides/Content/Jobs/Postman.md";
const MINER_GUIDE: &str = "notes/Build Guides/Content/Jobs/Miner.md";
const ADVENTURER_GUIDE: &str = "notes/Build Guides/Content/Jobs/Adventurer.md";
const BRAWLER_GUIDE: &str = "notes/Build Guides/Content/Jobs/Brawler.md";
const SANCTUARY_GUIDE: &str = "notes/Build Guides/Content/NPCs/NPCs - Sanctuary.md";
const DERBY_GUIDE: &str = "notes/Build Guides/Content/Jobs/Demolition Derby Driver.md";
const KART_GUIDE: &str = "notes/Build Guides/Content/Jobs/Kart Driver.md";
const SUNSTONE_GUIDE: &str = "notes/Build Guides/Content/NPCs/NPCs - Sunstone Valley.md";

/// How the cited guide row proves the speaker. A `GuideQuote` row names the NPC and
/// contains the line's hash. A `QuestOffer` row names the NPC as the giver of the
/// named quest; the production quest record must bind the same giver and intro, and
/// the record's own `source` note must name the locale id. A `SelfReference` row
/// names the NPC and the quest, and the line must claim that quest's object in the
/// first person ("my spectral spectacles"); this is only used where the client has
/// no quest record for the event quest.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Proof {
    GuideQuote,
    QuestOffer {
        title: &'static str,
    },
    SelfReference {
        object: &'static str,
        title: &'static str,
    },
}

#[derive(Clone, Copy)]
struct Binding {
    npc_id: u64,
    name: &'static str,
    file: &'static str,
    greeting: u32,
    hash: u32,
    dat_line: u32,
    guide: &'static str,
    guide_line: u32,
    guide_row: &'static str,
    proof: Proof,
    quest: Option<(&'static str, u32)>,
}

/// Every row this slice binds. The greeting is always the clicked row's own
/// `greeting_string_id`; no row borrows a neighbour's line. T369 added Crash
/// Davis and Dalton Narrowbrow when it enabled the derby chapter's two quests.
const BINDINGS: &[Binding] = &[
    Binding {
        npc_id: 9000000042,
        name: "Carlos Brazenfist",
        file: "cobblestone_village.json",
        greeting: 9800,
        hash: 2441571970,
        dat_line: 64292,
        guide: COBBLESTONE_GUIDE,
        guide_line: 46,
        guide_row: "| **Carlos Brazenfist**",
        proof: Proof::GuideQuote,
        quest: None,
    },
    Binding {
        npc_id: 9000000047,
        name: "Chloe",
        file: "blackspore_swamp.json",
        greeting: 78852,
        hash: 1599642277,
        dat_line: 42353,
        guide: CRACKED_CLAW_GUIDE,
        guide_line: 98,
        guide_row: "| **Ninja: Strike from the Shadows** |",
        proof: Proof::GuideQuote,
        quest: Some(("ninja.json", 3110005)),
    },
    Binding {
        npc_id: 9000000172,
        name: "Postchug Runalug",
        file: "wugachug.json",
        greeting: 130231,
        hash: 346429626,
        dat_line: 9230,
        guide: POSTMAN_GUIDE,
        guide_line: 109,
        guide_row: "| 20 | **Postman: Fair Warning** |",
        proof: Proof::GuideQuote,
        quest: Some(("postman.json", 3130023)),
    },
    Binding {
        npc_id: 9000000203,
        name: "Silverray",
        file: "blackspore_swamp.json",
        greeting: 77746,
        hash: 3485775830,
        dat_line: 92308,
        guide: MINER_GUIDE,
        guide_line: 135,
        guide_row: "| 20 | Miner: Minor Mix-Up |",
        proof: Proof::GuideQuote,
        quest: Some(("miner.json", 3100021)),
    },
    Binding {
        npc_id: 9000000217,
        name: "Therin",
        file: "snowhill.json",
        greeting: 413741,
        hash: 1726071120,
        dat_line: 45544,
        guide: MINER_GUIDE,
        guide_line: 96,
        guide_row: "| 1 | So, You Want To Be A Miner?",
        proof: Proof::GuideQuote,
        quest: None,
    },
    Binding {
        npc_id: 9000000289,
        name: "Crash Davis",
        file: "wilds.json",
        greeting: 393376,
        hash: 2000949526,
        dat_line: 52695,
        guide: DERBY_GUIDE,
        guide_line: 71,
        guide_row: "| 2 | **Demo Derby: License to Drive** |",
        proof: Proof::GuideQuote,
        quest: Some(("demo_derby_driver.json", 3060005)),
    },
    Binding {
        npc_id: 9000000295,
        name: "Dalton Narrowbrow",
        file: "wilds.json",
        greeting: 395495,
        hash: 3058718898,
        dat_line: 80791,
        guide: DERBY_GUIDE,
        guide_line: 70,
        guide_row: "| 1 | **Demo Derby: Rules of the Road** |",
        proof: Proof::GuideQuote,
        quest: Some(("demo_derby_driver.json", 3060004)),
    },
    Binding {
        npc_id: 9000000330,
        name: "Geoffrey",
        file: "sanctuary.json",
        greeting: 90621,
        hash: 3236560243,
        dat_line: 85513,
        guide: ADVENTURER_GUIDE,
        guide_line: 124,
        guide_row: "| E1 | **Chapter 1: The Missing Sisters** |",
        proof: Proof::GuideQuote,
        quest: Some(("adventurer.json", 3000029)),
    },
    Binding {
        npc_id: 9000000428,
        name: "Petra",
        file: "blackspore_swamp.json",
        greeting: 130793,
        hash: 3927787240,
        dat_line: 103883,
        guide: BRAWLER_GUIDE,
        guide_line: 123,
        guide_row: "| M1 | Brawler: Prey on the Cray |",
        proof: Proof::GuideQuote,
        quest: None,
    },
    Binding {
        npc_id: 9000000446,
        name: "Sabastian Cabet",
        file: "blackspore_swamp.json",
        greeting: 130856,
        hash: 954454844,
        dat_line: 25260,
        guide: BRAWLER_GUIDE,
        guide_line: 95,
        guide_row: "| C6 | Brawler: Given to Guard |",
        proof: Proof::QuestOffer {
            title: "Brawler: Given to Guard",
        },
        quest: Some(("brawler.json", 3030009)),
    },
    Binding {
        npc_id: 9500000488,
        name: "Greyli Dustywings",
        file: "sanctuary.json",
        greeting: 415317,
        hash: 1498450452,
        dat_line: 39685,
        guide: SANCTUARY_GUIDE,
        guide_line: 45,
        guide_row: "| Greyli Dustywings (name",
        proof: Proof::SelfReference {
            object: "spectral spectacles",
            title: "Spectral Spectacles",
        },
        quest: None,
    },
    Binding {
        npc_id: 9500000613,
        name: "Petal Oceanmist",
        file: "seaside.json",
        greeting: 394665,
        hash: 1191567603,
        dat_line: 31544,
        guide: KART_GUIDE,
        guide_line: 77,
        guide_row: "| 17 | **Kart Driver: Seaside Board of Tourism Sponsorship**",
        proof: Proof::QuestOffer {
            title: "Kart Driver: Seaside Board of Tourism Sponsorship",
        },
        quest: Some(("kart_driver.json", 3080019)),
    },
    Binding {
        npc_id: 9500000650,
        name: "Arch Druid Camellia",
        file: "shrouded_glade.json",
        greeting: 440465,
        hash: 3546861059,
        dat_line: 93910,
        guide: SUNSTONE_GUIDE,
        guide_line: 190,
        guide_row: "| 1 | **Aiding our Allies**",
        proof: Proof::GuideQuote,
        quest: None,
    },
    Binding {
        npc_id: 9500000731,
        name: "Burt 'Wrecking' Crewe",
        file: "snowhill.json",
        greeting: 139675,
        hash: 2539856207,
        dat_line: 66905,
        guide: DERBY_GUIDE,
        guide_line: 166,
        guide_row: "| **The Burden of Fame** | Raina Rush |",
        proof: Proof::GuideQuote,
        quest: None,
    },
    Binding {
        npc_id: 9500000759,
        name: "Mustang Alli",
        file: "snowhill.json",
        greeting: 117327,
        hash: 3470571122,
        dat_line: 91900,
        guide: KART_GUIDE,
        guide_line: 50,
        guide_row: "| **Racing Tutorial** | Mustang Alli",
        proof: Proof::QuestOffer {
            title: "Racing Tutorial",
        },
        quest: Some(("kart_driver.json", 3080002)),
    },
    Binding {
        npc_id: 9500000941,
        name: "Shakey",
        file: "wilds.json",
        greeting: 94519,
        hash: 2762924238,
        dat_line: 72786,
        guide: COBBLESTONE_GUIDE,
        guide_line: 112,
        guide_row: "| **Call The Crew!** |",
        proof: Proof::GuideQuote,
        quest: None,
    },
];

/// Enabled quest givers and talk targets whose client text has no recoverable
/// first-person line of their own. Each stays unbound; the test fails a new
/// unbound id that is not listed here.
const BLANK_WITH_REASON: &[(u64, &str)] = &[
    (
        9000000023,
        "Beth Callahan (T743 talk target of 3090027): the client names her only in third person (387592 'Speak to Beth Callahan', the objective text, and Doctor Callahan's offer speech 387576 that sends the player to her); quest-lines/objective-specs-fixed.json q3090027 records no first-person line, so none is recovered.",
    ),
    (
        9000000069,
        "Derp Axemaker (T729 giver of 3100012 and 3100013): the only first-person lines of his enabled quests are their offer intros 387452 and 77415, and quest-lines/dialogs.json leaves both speakers unattributed; binding a QuestOffer greeting moves the Snowhill bound-row pins in the region greeting tests, which this quest-data change does not touch, so he stays unbound. His disabled quests carry other first-person intros, but a disabled quest offers no proof and is not a greeting source.",
    ),
    (
        9000000070,
        "Diggy (T466 Touring Seaside talk target): NPCs - Seaside.md:45 quotes only third-person cooking objectives and other speakers' referrals (428610, 21340, 384277, 100280); locale 93435 ('finest seafood') has no speaker in any guide row, and 93429 is the quest-step goal text, so no first-person line of his own is proven.",
    ),
    (
        9000000107,
        "Honey (T500 talk target of 3000310): NPCs - Sanctuary.md:47 quotes only a farm location and harvesting objectives (841702301, 2418103113, 623626341) and the interview mention (2478429130); her reply 390820 is the quest-step dialogue of 3000310 goal 2, not a greeting. Her chef quest intros are disabled quests.",
    ),
    (
        9000000140,
        "Manager Landon (T409): his client lines are third person (402947, 402841, 416471, 402687); the first-person substitute plea 418984 is the Snowy Scrimmage offer intro, whose speaker the Soccer Star guide leaves unnamed (1591815111), so no uncontested greeting of his own is recovered.",
    ),
    (
        9000000146,
        "Merri De'Flor (T727 talk target of 3060011): NPCs - Lakeshore.md:27 quotes only third-person lines about her (1113274275, 1119011320, 9582519, 446677410) and the restaurant minigame prompts; the sponsor lines 393379 (Old Betty's offer, dialogs.json q3060011) and 394288 (no attributed speaker) are not hers, so no first-person line of her own is recovered.",
    ),
    (
        9000000164,
        "Osgood (T723 giver of Wizard: Reach Level 10, Helping a Friend and Reach Level 15; talk target of both gates): NPCs - Sanctuary.md:60 quotes only his role, the wand build and his level-10 and level-15 blocks; his first-person lines 387662, 387665, 387666, 387669 and 37310 are the offers and end texts of those three quests, whose speaker the guide leaves unnamed, and a quest line is not a standing greeting, so none is bound.",
    ),
    (
        9000000173,
        "Postmaster Lane: the split Trail of Mail intro 141828 is quoted by the Postman guide under Enve LeStamp's earlier Sasparilla row, so the speaker is contested; no uncontested first-person line exists.",
    ),
    (
        9000000190,
        "Sandbur (T723 talk target of Wizard: Helping a Friend): NPCs - Sanctuary.md:61 gives him no quote of his own (a role, a location and the quests he receives items for); 37312 is the quest's objective text 'Speak with Sandbur in Sanctuary's Inner Circle.', not a line he speaks, so no first-person greeting is recovered.",
    ),
    (
        9000000191,
        "Sasparilla Brewmaster Wugawump (T502 turn-in of 3000400): NPCs - Wugachug.md:38 quotes only the third-person brewery and party talk (1459611394, 3773925023) and the referral to him; his first-person line is the Join the Party end text 385981, which is the quest-step dialogue of 3000400 goal 1, and the intro 385978 belongs to Boranug, that quest's giver, so no uncontested greeting of his own is proven.",
    ),
    (
        9000000227,
        "Valerie: only third-person tutorial and objective text; the untied Welcome window 3453817544 is not attributed to her, so no first-person line is recovered.",
    ),
    (
        9000000495,
        "Tucker Thunderjolt (T727 talk target of 3060013): NPCs - Wilds.md:184 gives only a role ('Crash Davis's teammate'), the location 'Outside of Thunder Falls' (1929354201, 2909963659) and the sponsor referral 880667942; 394016 is the quest objective and 402845 the journal-style completion line, and no first-person line of his own is recovered.",
    ),
    (
        9500000164,
        "Linx Redline (T732 giver of 3080007): NPCs - Briarwood.md:122/133 and Kart Driver.md row 5 quote only third-person race objectives and the T.R. hand-off; his own line 101178 is the quest offer and 101170 the progress reminder, not a greeting.",
    ),
    (
        9500000209,
        "Sergeant Cargug (T502 turn-in of 3000392): NPCs - Briarwood.md:148 quotes only third-person role and Nettleseed location text (3637816292, 1789311537); 36445 is the objective and 390591 is the end text of 3000392 goal 1, so no first-person line that is not a quest step exists.",
    ),
    (
        9500000213,
        "Smoggo (T502 giver of 3000392): NPCs - Briarwood.md:114 quotes only a deep-Briarwood location and the new-camp line (3534154508) and 307 names 'Briarwood's Not So Bad' in a title-only row with no giver, so a QuestOffer proof for his intro 390588 cannot be made; 387480 is the objective and 390261 is the end text of 3000391 goal 1.",
    ),
    (
        9500000288,
        "Blorple (T502 giver of 3000390): NPCs - Merry Vale.md:169 names him as the giver of Misplaced Mugwopp but quotes only the third-person referral 2817473271; NPCs - Greenwood Forest.md:63 quotes only a location (2490543637, 3536286642). His first-person intro 46210 is a QuestOffer candidate, but npc_greetings_job_hubs.rs pins every unlisted greenwood_forest.json row unbound; 46618 is the objective and 46252 is the end text of 3000390 goal 1.",
    ),
    (
        9500000289,
        "Bollylog: only third-person arrow-delivery objectives and one unattributed recruit greeting; no individual first-person line exists.",
    ),
    (
        9500000290,
        "Brugadug: only third-person arrow-delivery objectives; the three recipient replies cannot be assigned to one recruit.",
    ),
    (
        9500000309,
        "Hrugagu: only third-person arrow-delivery objectives; the three recipient replies cannot be assigned to one recruit.",
    ),
    (
        9500000443,
        "Barin Battlerock (T500 talk target of 3000310): NPCs - Sanctuary.md:76 gives only a role and a video location, 'Interview target in Looking for Lavender'; his reply 390822 is the quest-step dialogue of 3000310 goal 3, not a greeting, and 390802 is the objective.",
    ),
    (
        9500000467,
        "Dani (T500 talk target of 3000310): NPCs - Sanctuary.md:77 quotes only a Pixiewood location (3686406336) and card-quest and pet-trick objectives (4174833278); her reply 390817 is the quest-step dialogue of 3000310 goal 1, not a greeting, and 390798 is the objective.",
    ),
    (
        9500000561,
        "Brendan: only the objective string 573579885 and third-person lotion references; no recovered first-person speech.",
    ),
    (
        9500000562,
        "Brenna (T466 Testing the Tour talk target): NPCs - Seaside.md:28 quotes only Cole's referrals to her (383202, 103055) and objectives (140239, 93454); her first-person quiz lines 93464, 93466 and 93474 are the quest-step dialogue of 3000323 goal 1, not a greeting.",
    ),
    (
        9500000567,
        "Cole (T466 giver of 3000321-3000323, turn-in of 3000320): NPCs - Seaside.md:27 quotes only lines about him (93369, 93334, 93324, 93424, 116733); his first-person lines are the tour intros 93398, 93419, 93449 and end texts 93335, 93401, 93422, 93451. A QuestOffer proof could bind an intro, but tests/npc_greetings_seaside_briarwood.rs and notes/Autopilot/T317.md pin his Seaside row unbound.",
    ),
    (
        9500000580,
        "Emuzz (T466 Feminine Touch talk target): NPCs - Seaside.md:104 quotes only another speaker's survey and flyer objectives (101404, 95036, 139578); her first-person reply 101414 is the quest-step dialogue of 3000360 goal 2, not a greeting.",
    ),
    (
        9500000588,
        "Heddy Heathcliff (T503 giver and turn-in of 3000361): NPCs - Seaside.md row 50 quotes only third-person role, location and quest summaries (4042850801, 570276910, 4186195979); his first-person lines are the quest intro 400433 and the end text 400436, and a QuestOffer proof for 400433 cannot be made because that row says the quest titles are not found, so no uncontested greeting of his own is recovered.",
    ),
    (
        9500000633,
        "Tazinna (T466 Feminine Touch talk target): NPCs - Seaside.md:105 quotes only third-person survey lines (101402, 139578); her first-person reply 101412 is the quest-step dialogue of 3000360 goal 1, not a greeting.",
    ),
    (
        9500000645,
        "Yancy Gilbert (T466 giver of 3000360): NPCs - Seaside.md:106 quotes only third-person novel and flyer objectives (101401, 101410, 100067) and the player's line 101415; his first-person lines are the Feminine Touch intro 101408 and end text 101411. A QuestOffer proof could bind 101408, but tests/npc_greetings_seaside_briarwood.rs and notes/Autopilot/T317.md pin his Seaside row unbound.",
    ),
    (
        9500000690,
        "Headmaster Merk (T502 turn-in of 3000380): NPCs - Shrouded Glade.md:122 quotes the third-person academy role and 203 names him only as Aiding our Allies' hand-in; 440471 is the objective and 440468 is the end text of 3000380 goal 1. The intro 440465 is Camellia's, and he gives no enabled quest, so no first-person greeting of his own is proven.",
    ),
    (
        9500000697,
        "Kosha: only the objective string 1664864679; no recovered first-person speech.",
    ),
    (
        9500000715,
        "Suni Sorrowsmile (T500 talk target of 3000311): NPCs - Shrouded Glade.md:59 quotes only the third-person 'Suni can be found in the Shrouded Glade' (3882523674) and objectives; her reply 390855 is the quest-step dialogue of 3000311 goal 1, not a greeting, and 390850 is the objective.",
    ),
    (
        9500000735,
        "Chad Fastflutter: only Raina's referral lines and objective strings; no first-person line of his own.",
    ),
    (
        9500000872,
        "Blammo (T732 giver and talk target of 3080009): NPCs - Wilds.md:148 and Kart Driver.md row 7 quote only the third-person Exploder's League role; his line 394486 is the quest offer, not a greeting.",
    ),
    (
        9500000874,
        "Blazing Star (T498 giver of 3000332): NPCs - Wilds.md:77 quotes no line of hers ('See Flax', standing spot not stated); her first-person text is the quest intro 389950, a QuestOffer proof, but tests/npc_greetings_lakeshore_wilds.rs pins every unlisted Wilds row unbound.",
    ),
    (
        9500000882,
        "Bulb (T498 talk target of 3000333): NPCs - Wilds.md:71 quotes only third-person objectives (1264997128, 394541242, 4285047538); his reply 38163 is the quest-step dialogue of 3000333 goal 1, not a greeting.",
    ),
    (
        9500000898,
        "Flax (T498 turn-in of 3000333): NPCs - Wilds.md:70 quotes only third-person fish-chain objectives (314189566, 1332727062, 394541242); his line 2134 is the Making Amends end text, not a greeting.",
    ),
    (
        9500000900,
        "Funnel Bindweed (T498 talk target of 3000332): NPCs - Wilds.md:73 quotes only drink-ingredient objectives (1396874048, 811508545, 2607838681) and a porch location; his reply 389970 is the quest-step dialogue of 3000332 goal 2, not a greeting.",
    ),
    (
        9500000925,
        "Nog (T498 talk target of 3000340): NPCs - Wilds.md:118 quotes only location and stew objectives (781547105, 2141422511, 3164390028); 38530 is the quest-step dialogue of 3000340 goal 4, and the other locale lines naming him (21056, 23400, 38516) are objectives.",
    ),
    (
        9500000926,
        "Oxalis Shamrock (T498 giver of 3000333, turn-in of 3000332): NPCs - Wilds.md:67 quotes only chef-errand objectives (43249685, 2645354306, 2735689201); his first-person lines are the intro 38142 (a QuestOffer proof, but tests/npc_greetings_lakeshore_wilds.rs pins every unlisted Wilds row unbound), the step reply 390019 and the end text 389953.",
    ),
    (
        9500000950,
        "T.R. Walker Jr. (T732 talk target and end-screen actor of 3080007): Kart Driver.md row 5 quotes only the third-person challenge objective; his line 101186 is the post-race concession, not a greeting.",
    ),
    (
        9500000955,
        "Vergosh (T498 giver of 3000340 and 3000341): NPCs - Wilds.md:117 quotes only a camp location and quarry objectives (2064782438, 3047426870, 1990199215); his first-person lines are the intros 38384 and 70102 (QuestOffer proofs, but tests/npc_greetings_lakeshore_wilds.rs pins every unlisted Wilds row unbound) and the end text 9585.",
    ),
    (
        9500000965,
        "Zinnea Magnolio (T498 talk target of 3000332): NPCs - Wilds.md:72 quotes only third-person flower objectives (331882309, 4075335134) and a suspect line (2063005555); her reply 389967 is the quest-step dialogue of 3000332 goal 1, not a greeting.",
    ),
    (
        9500000967,
        "Boranug (T502 giver of 3000400): NPCs - Wugachug.md:36 quotes only the third-person greeter role and the contest score (401856720), and 148 names 'Welcome to Wugachug!', not this quest; the Join the Party title appears in no guide page, so the QuestOffer proof for his intro 385978 cannot be made. 385984 is the objective; no other first-person line of his is recovered.",
    ),
    (
        9500001007,
        "Burglz (T498 turn-in of 3000341, a new row): the guide row for Burgl (NPCs - Wilds.md:119) quotes only objectives (2090604938, 946401124, 1449038490); his first-person line 70105 is the Question at the Quarry end text, not a greeting, and 441432 is the Seaside banker's greeting and must not be reused.",
    ),
    (
        9500001008,
        "Hasti (T500 talk target of 3000301, a new row bound to extra spawn 100000003016): NPCs - Merry Vale.md:166 lists him only as a racetrack ambassador target with no row; his reply 140679 is the quest-step dialogue of 3000301 goal 1, not a greeting, and 46621 is the objective.",
    ),
    (
        9500001009,
        "Shannon (T501 talk target of 3000371, a new row bound to extra spawn 100000002187): the Merry Vale guide has no Shannon row; locale 40518 is only her name and the adjacent 40519 is the Scary Stories quest line, not hers; her reply 74214 is the quest-step dialogue of 3000371 goal 3, so no first-person line of her own is proven.",
    ),
    (
        9500001010,
        "Sugaboo (T501 talk target of 3000376, a new row bound to extra spawn 100000002279): the Merry Vale guide has no Sugaboo row; locale 40355 is only her name and 40356 is the quest_dialog challenge line tagged Sugaboo that no guide row or production quest record binds, while her reply 387057 is the quest-step dialogue of 3000376 goal 1, so no greeting is proven.",
    ),
];

/// Any key that marks a third-person objective rather than a speaker's line.
const THIRD_PERSON_PREFIXES: [&str; 18] = [
    "talk to",
    "speak to",
    "speak with",
    "find ",
    "return to",
    "visit ",
    "collect ",
    "deliver ",
    "go to",
    "head to",
    "report to",
    "achieve ",
    "complete ",
    "bring ",
    "give ",
    "defeat ",
    "click ",
    "interact ",
];

fn quest_files() -> Vec<PathBuf> {
    let dir = content_dir().join("quests");
    let mut paths: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|error| panic!("{}: {error}", dir.display()))
        .map(|entry| entry.expect("quest content entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect();
    paths.sort();
    paths
}

fn quest_records() -> Vec<(String, Value)> {
    let mut records = Vec::new();
    for path in quest_files() {
        let file = path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("quest file name")
            .to_owned();
        let document = read_json(&path);
        for record in document["records"]
            .as_array()
            .unwrap_or_else(|| panic!("{}: records array", path.display()))
        {
            records.push((file.clone(), record.clone()));
        }
    }
    records
}

fn quest_record(file: &str, quest_id: u32) -> Value {
    read_json(&content_dir().join("quests").join(file))["records"]
        .as_array()
        .unwrap_or_else(|| panic!("{file}: records array"))
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(quest_id)))
        .unwrap_or_else(|| panic!("{file}: quest {quest_id}"))
        .clone()
}

/// The enabled givers and `talk_to_npc` targets, derived from the production
/// split loader and the same records it reads.
fn enabled_quest_targets() -> BTreeSet<u64> {
    let engine =
        QuestEngine::from_split_content(&content_dir(), 1, CharacterEligibility::default(), None)
            .expect("the production split quest loader accepts data/content");
    let mut targets = BTreeSet::new();
    let mut enabled = 0usize;
    let mut seen = BTreeSet::new();
    for (file, record) in quest_records() {
        let id =
            u32::try_from(record["id"].as_u64().expect("quest id")).expect("quest id fits u32");
        assert!(seen.insert(id), "{file}: duplicate quest id {id}");
        let record_enabled = record
            .get("enabled")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        assert_eq!(
            engine.is_enabled(id),
            record_enabled,
            "{file} quest {id}: the production loader's enabled flag"
        );
        if !record_enabled {
            continue;
        }
        enabled += 1;
        let giver = record["giver_npc_id"]
            .as_u64()
            .unwrap_or_else(|| panic!("{file} quest {id}: giver_npc_id"));
        assert_eq!(
            engine.giver_npc_id(id),
            Some(giver),
            "{file} quest {id}: the production loader's giver"
        );
        targets.insert(giver);
        for goal in record["objectives"].as_array().into_iter().flatten() {
            if goal["kind"].as_str() == Some("talk_to_npc") {
                targets.insert(
                    goal["npc_id"]
                        .as_u64()
                        .unwrap_or_else(|| panic!("{file} quest {id}: talk_to_npc without npc_id")),
                );
            }
        }
    }
    assert!(enabled > 0, "the enabled quest catalogue is not empty");
    targets
}

/// Every NPC row in `data/content/npcs`, keyed by content id.
fn npc_index() -> BTreeMap<u64, (String, Value)> {
    let dir = content_dir().join("npcs");
    let mut files: Vec<String> = fs::read_dir(&dir)
        .unwrap_or_else(|error| panic!("{}: {error}", dir.display()))
        .map(|entry| {
            entry
                .expect("NPC content entry")
                .file_name()
                .to_str()
                .expect("NPC file name")
                .to_owned()
        })
        .filter(|file| file.ends_with(".json"))
        .collect();
    files.sort();
    let mut index = BTreeMap::new();
    for file in files {
        for record in npc_records(&file) {
            let npc_id = record["npc_id"]
                .as_u64()
                .unwrap_or_else(|| panic!("{file}: row without npc_id"));
            assert!(
                index.insert(npc_id, (file.clone(), record)).is_none(),
                "duplicate NPC id {npc_id}"
            );
        }
    }
    index
}

/// The cited guide table row, found by its stable leading cells so unrelated note
/// edits above it cannot break the test. Exactly one row must match.
fn guide_row(guide: &str, marker: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(guide);
    let text =
        fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let mut rows = text
        .lines()
        .filter(|line| line.starts_with('|') && line.contains(marker));
    let row = rows
        .next()
        .unwrap_or_else(|| panic!("{}: no table row contains {marker:?}", path.display()))
        .to_owned();
    assert!(
        rows.next().is_none(),
        "{}: more than one table row contains {marker:?}",
        path.display()
    );
    row
}

/// True for text that reads as a third-person objective or narration about the
/// NPC, which a greeting must not be.
fn looks_third_person(text: &str, name: &str) -> bool {
    let plain = text
        .replace("<BR>", " ")
        .replace("<br>", " ")
        .split('<')
        .map(|part| part.split_once('>').map_or(part, |(_, rest)| rest))
        .collect::<String>()
        .trim()
        .to_lowercase();
    if THIRD_PERSON_PREFIXES
        .iter()
        .any(|prefix| plain.starts_with(prefix))
    {
        return true;
    }
    if plain == name.to_lowercase() {
        return true;
    }
    let Some(rest) = plain.strip_prefix(&name.to_lowercase()) else {
        return false;
    };
    rest.trim_start()
        .split_whitespace()
        .next()
        .is_some_and(|verb| {
            matches!(
                verb,
                "would"
                    | "wants"
                    | "has"
                    | "is"
                    | "was"
                    | "asks"
                    | "needs"
                    | "says"
                    | "believes"
                    | "thinks"
                    | "claims"
                    | "can"
                    | "will"
            )
        })
}

/// The speaker proof: the cited guide row must name the NPC and carry the line's
/// hash, or (for a quest offer) name the giver while the production quest record
/// binds the same giver and intro, or (for a self-referenced event quest) name the
/// quest the line claims in the first person. A known third-person line fails.
fn speaker_proof(binding: &Binding, text: &str) -> Result<(), String> {
    if text.trim().is_empty() {
        return Err("the locale text is empty".to_owned());
    }
    if looks_third_person(text, binding.name) {
        return Err(format!(
            "the locale line is third-person narration: {text:.80}"
        ));
    }
    let row = guide_row(binding.guide, binding.guide_row);
    if !row.contains(binding.name) {
        return Err(format!(
            "{} row {:?} does not name {}",
            binding.guide, binding.guide_row, binding.name
        ));
    }
    match binding.proof {
        Proof::GuideQuote => {
            if !row.contains(&binding.hash.to_string()) {
                return Err(format!(
                    "{} row {:?} does not quote hash {}",
                    binding.guide, binding.guide_row, binding.hash
                ));
            }
        }
        Proof::QuestOffer { title } => {
            if !row.contains(title) {
                return Err(format!(
                    "{} row {:?} does not name quest {title:?}",
                    binding.guide, binding.guide_row
                ));
            }
            let (file, quest_id) = binding
                .quest
                .ok_or_else(|| "a quest-offer binding must name its quest record".to_owned())?;
            let quest = quest_record(file, quest_id);
            if quest["giver_npc_id"].as_u64() != Some(binding.npc_id) {
                return Err(format!(
                    "{file} quest {quest_id} is not giver-owned by {}",
                    binding.npc_id
                ));
            }
            if quest["intro_text_string_id"].as_u64() != Some(u64::from(binding.greeting)) {
                return Err(format!(
                    "{file} quest {quest_id} does not use {} as its intro",
                    binding.greeting
                ));
            }
            let evidence = quest["source"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|source| source["evidence"].as_str())
                .collect::<Vec<_>>()
                .join(" ");
            if !evidence.contains(binding.guide)
                || !evidence.contains(&binding.greeting.to_string())
            {
                return Err(format!(
                    "{file} quest {quest_id} source note does not cite {} and {}",
                    binding.guide, binding.greeting
                ));
            }
        }
        Proof::SelfReference { object, title } => {
            if !row.contains(title) {
                return Err(format!(
                    "{} row {:?} does not name quest {title:?}",
                    binding.guide, binding.guide_row
                ));
            }
            if !text.to_lowercase().contains(&format!("my {object}")) {
                return Err(format!(
                    "the locale line does not claim {object:?} in the first person"
                ));
            }
        }
    }
    Ok(())
}

fn locale_text(hash: u32) -> String {
    locale_rows(&BTreeSet::from([hash]))
        .remove(&hash)
        .unwrap_or_else(|| panic!("hash {hash} is not a stock client locale key"))
        .1
}

/// The real 26/8 -> 26/10 -> Talk path, returning the 26/3 packet and the menu
/// packet the router answered with. The menu packet must be the service's own
/// 26/9 packet (the router only patches the request target into bytes 4..12,
/// and the request target is the content key here).
fn in_world_talk(
    game: &mut Gameplay,
    file: &str,
    npc_id: u64,
    tick: u64,
) -> (Vec<u8>, Vec<u8>, u64) {
    let record = npc_record(file, npc_id);
    // The client clicks the runtime actor: a legacy_data row uses its NPC key,
    // an extra spawn uses its spawn GUID. The router maps both back to the
    // content key before reading the greeting row.
    let runtime_guid = if record["placement"].as_str() == Some("legacy_data") {
        npc_id
    } else {
        extra_spawn_guid(npc_id)
            .unwrap_or_else(|| panic!("NPC {npc_id} in {file}: no runtime extra spawn"))
    };
    let character = character_at(npc_position(&record, npc_id));
    let menu = game
        .handle_in_world(&interact(runtime_guid), &character, tick)
        .unwrap_or_else(|error| panic!("NPC {npc_id} ({runtime_guid}) interact: {error}"));
    let menu_packet = menu
        .replies
        .iter()
        .find(|packet| packet.starts_with(&MENU_HEADER))
        .unwrap_or_else(|| panic!("NPC {npc_id} must answer 26/8 with a 26/9 menu"))
        .clone();
    let outcome = game
        .handle_in_world(&select(runtime_guid, 1), &character, tick + 1)
        .unwrap_or_else(|error| panic!("NPC {npc_id} Talk: {error}"));
    let dialogue = outcome
        .replies
        .into_iter()
        .find(|packet| packet.starts_with(&GREETING_HEADER))
        .unwrap_or_else(|| panic!("NPC {npc_id} Talk must answer 26/3"));
    (dialogue, menu_packet, runtime_guid)
}

fn body_dword(packet: &[u8]) -> u32 {
    u32::from_le_bytes(packet[4..8].try_into().unwrap())
}

fn binding_for(npc_id: u64) -> Binding {
    *BINDINGS
        .iter()
        .find(|binding| binding.npc_id == npc_id)
        .unwrap_or_else(|| panic!("NPC {npc_id} is not a T378 binding"))
}

#[test]
fn every_enabled_quest_giver_and_talk_target_is_bound_or_explained() {
    let targets = enabled_quest_targets();
    let index = npc_index();
    let visibility = NpcVisibility::load(&content_dir()).expect("NPC visibility loads");
    let id_to_text = locale_id_to_text();
    let mut bound = 0usize;
    let mut bound_hashes = BTreeSet::new();

    for npc_id in &targets {
        let (file, record) = index
            .get(npc_id)
            .unwrap_or_else(|| panic!("enabled quest target {npc_id} has no NPC row"));
        assert!(
            record["position"]
                .as_array()
                .is_some_and(|position| position.len() == 3),
            "NPC {npc_id} in {file}: finite 3-component position"
        );
        assert_spawned(&visibility, *npc_id, file, record);
        let greeting = record["greeting_string_id"].as_u64().unwrap_or(0);
        if greeting > 0 {
            bound += 1;
            let greeting = u32::try_from(greeting).expect("greeting id fits u32");
            let text = id_to_text.get(&u64::from(greeting)).unwrap_or_else(|| {
                panic!("NPC {npc_id}: greeting {greeting} does not resolve in the stock locale")
            });
            assert!(
                !text.trim().is_empty(),
                "NPC {npc_id}: greeting {greeting} is empty"
            );
            bound_hashes.insert(locale_hash(i32::try_from(greeting).expect("id fits i32")));
        } else {
            let reason = BLANK_WITH_REASON
                .iter()
                .find(|(blank, _)| *blank == *npc_id)
                .unwrap_or_else(|| {
                    panic!(
                        "NPC {npc_id} ({}) is an enabled quest giver/talk target with no greeting and no BLANK_WITH_REASON entry",
                        record["name"]
                    )
                });
            assert!(
                reason.1.len() > 40,
                "NPC {npc_id}: the blank reason is too short to be exact"
            );
        }
    }

    // No stale blank entries and no silently dropped target.
    for &(npc_id, _) in BLANK_WITH_REASON {
        assert!(
            targets.contains(&npc_id),
            "NPC {npc_id} is in BLANK_WITH_REASON but is not an enabled quest giver/talk target"
        );
        let (file, record) = &index[&npc_id];
        assert!(
            record.get("greeting_string_id").is_none(),
            "NPC {npc_id} in {file}: blank rows must not gain a greeting"
        );
        assert!(
            record["field_sources"].get("greeting_string_id").is_none(),
            "NPC {npc_id} in {file}: blank rows must not gain greeting sources"
        );
    }

    assert!(
        bound >= BINDINGS.len(),
        "the enabled catalogue has {bound} bound rows, fewer than this slice's {}",
        BINDINGS.len()
    );
    assert!(
        BINDINGS
            .iter()
            .all(|binding| targets.contains(&binding.npc_id)),
        "every T378 binding must be an enabled quest giver or talk target"
    );

    // Resolve every bound id through the stock locale (the pre-existing bindings
    // included), so a zero/unknown id cannot pass as "bound".
    let locale = locale_rows(&bound_hashes);
    for npc_id in &targets {
        let (_, record) = &index[npc_id];
        let Some(greeting) = record["greeting_string_id"].as_u64().filter(|id| *id > 0) else {
            continue;
        };
        let greeting = u32::try_from(greeting).expect("greeting id fits u32");
        let hash = locale_hash(i32::try_from(greeting).expect("id fits i32"));
        let (_, text) = locale
            .get(&hash)
            .unwrap_or_else(|| panic!("NPC {npc_id}: hash {hash} is not in the stock locale"));
        assert!(
            !text.trim().is_empty(),
            "NPC {npc_id}: greeting {greeting} resolves to empty text"
        );
    }
}

#[test]
fn each_bound_greeting_is_the_npcs_own_first_person_line() {
    let id_to_text = locale_id_to_text();
    let hash_to_ids = read_json(&lab_report("locale-id-map.json"))["hash_to_ids"].clone();
    let wanted: BTreeSet<u32> = BINDINGS.iter().map(|binding| binding.hash).collect();
    let locale = locale_rows(&wanted);
    let all_bindings = greeting_bindings();
    let mut greeting_ids = BTreeSet::new();
    let mut edited = BTreeSet::new();

    for binding in BINDINGS {
        assert!(
            binding.greeting > 0,
            "NPC {}: zero greeting",
            binding.npc_id
        );
        assert!(
            greeting_ids.insert(binding.greeting),
            "NPC {}: greeting {} reused",
            binding.npc_id,
            binding.greeting
        );
        assert!(
            edited.insert(binding.npc_id),
            "NPC {}: edited twice",
            binding.npc_id
        );

        let record = npc_record(binding.file, binding.npc_id);
        assert_eq!(
            record["name"].as_str(),
            Some(binding.name),
            "NPC {}: npc_id resolves the expected row",
            binding.npc_id
        );
        assert_eq!(
            record["greeting_string_id"].as_u64(),
            Some(u64::from(binding.greeting)),
            "NPC {}: greeting binding",
            binding.npc_id
        );
        assert!(
            record["name"].as_str().is_some_and(|name| !name.is_empty()),
            "NPC {}: name",
            binding.npc_id
        );
        if let Some(name_id) = record["name_string_id"].as_u64() {
            assert_ne!(
                u64::from(binding.greeting),
                name_id,
                "NPC {}: a name id is not a dialogue body",
                binding.npc_id
            );
        }

        // No pre-existing row's greeting id is reused by a new speaker.
        for ((file, npc_id), greeting) in &all_bindings {
            if *npc_id == binding.npc_id && file == binding.file {
                continue;
            }
            assert_ne!(
                *greeting, binding.greeting,
                "NPC {}: greeting {} already belongs to {file} NPC {npc_id}",
                binding.npc_id, binding.greeting
            );
        }

        // The row's own source note names the locale row, the guide row and (for
        // quest offers) the quest record, and marks the fixed-default policy (R).
        let sources = record["field_sources"]["greeting_string_id"]
            .as_array()
            .unwrap_or_else(|| panic!("NPC {}: greeting sources", binding.npc_id));
        assert_eq!(
            sources.len(),
            2,
            "NPC {}: exactly one client and one reconstructed source",
            binding.npc_id
        );
        let client = sources
            .iter()
            .find(|source| source["kind"] == "client")
            .unwrap_or_else(|| panic!("NPC {}: client source", binding.npc_id));
        let reconstructed = sources
            .iter()
            .find(|source| source["kind"] == "reconstructed")
            .unwrap_or_else(|| panic!("NPC {}: reconstructed source", binding.npc_id));
        let evidence = client["evidence"].as_str().unwrap_or_default();
        for needle in [
            "T378".to_owned(),
            format!("Global.Text.{}", binding.greeting),
            format!("JenkinsLookup2 hash {}", binding.hash),
            format!("line {}", binding.dat_line),
            binding.guide.to_owned(),
            format!("line {}", binding.guide_line),
        ] {
            assert!(
                evidence.contains(&needle),
                "NPC {}: client source must name {needle:?}",
                binding.npc_id
            );
        }
        if let Some((quest_file, quest_id)) = binding.quest {
            assert!(
                evidence.contains(&format!("data/content/quests/{quest_file}"))
                    && evidence.contains(&format!("quest {quest_id}")),
                "NPC {}: client source must name its quest record",
                binding.npc_id
            );
        }
        let policy = reconstructed["evidence"].as_str().unwrap_or_default();
        assert!(
            policy.starts_with("(R)"),
            "NPC {}: reconstructed evidence must start with (R)",
            binding.npc_id
        );
        assert!(
            policy.contains("fixed default Talk greeting"),
            "NPC {}: fixed-default Talk policy",
            binding.npc_id
        );

        // The id is a Global.Text key: hash it, find the bound id behind the hash,
        // and check the physical stock-locale line and the id-to-text report agree.
        assert_eq!(
            locale_hash(i32::try_from(binding.greeting).expect("stock locale id fits i32")),
            binding.hash,
            "NPC {}: greeting id must hash to the cited Global.Text key",
            binding.npc_id
        );
        let bound = hash_to_ids
            .get(&binding.hash.to_string())
            .and_then(Value::as_array)
            .unwrap_or_else(|| {
                panic!(
                    "NPC {}: hash {} is not in locale-id-map.json",
                    binding.npc_id, binding.hash
                )
            });
        assert!(
            bound
                .iter()
                .any(|id| id.as_u64() == Some(u64::from(binding.greeting))),
            "NPC {}: hash {} must resolve to {}",
            binding.npc_id,
            binding.hash,
            binding.greeting
        );
        let reported = id_to_text
            .get(&u64::from(binding.greeting))
            .unwrap_or_else(|| {
                panic!(
                    "NPC {}: id {} is not in locale-id-to-text.json",
                    binding.npc_id, binding.greeting
                )
            });
        let (line, text) = locale.get(&binding.hash).unwrap_or_else(|| {
            panic!(
                "NPC {}: hash {} is not a key in the stock client locale",
                binding.npc_id, binding.hash
            )
        });
        assert_eq!(
            *line, binding.dat_line as usize,
            "NPC {}: en_us_data.dat line",
            binding.npc_id
        );
        assert_eq!(
            text, reported,
            "NPC {}: stock locale text must equal the id-to-text report",
            binding.npc_id
        );
        if let Err(error) = speaker_proof(binding, text) {
            panic!("NPC {}: {error}", binding.npc_id);
        }
    }

    // The line is the row's own speech: the cited guide row must name the speaker,
    // and a valid line for a different speaker must fail.
    let wrong = Binding {
        guide: CRACKED_CLAW_GUIDE,
        guide_row: "| **Ninja: Strike from the Shadows** |",
        ..binding_for(9000000042)
    };
    let error = speaker_proof(&wrong, &locale_text(wrong.hash))
        .expect_err("Carlos's line must not pass as Chloe's");
    assert!(
        error.contains("does not name") || error.contains("does not quote"),
        "{error}"
    );

    // A known third-person objective fails even though its guide row quotes it.
    let objective = Binding {
        npc_id: 9000000428,
        name: "Petra",
        file: "blackspore_swamp.json",
        greeting: 72497,
        hash: 616697640,
        dat_line: 16340,
        guide: BRAWLER_GUIDE,
        guide_line: 131,
        guide_row: "| M4 | Brawler: Splendid Silver! |",
        proof: Proof::GuideQuote,
        quest: None,
    };
    let error = speaker_proof(&objective, &locale_text(objective.hash))
        .expect_err("a third-person objective is not a greeting");
    assert!(error.contains("third-person"), "{error}");
}

#[test]
fn clicking_a_bound_quest_npc_returns_its_greeting_body() {
    // Service path: the context the router copies into the 26/3 body.
    let mut interaction = service();
    for binding in BINDINGS {
        assert_eq!(
            service_talk_body(&mut interaction, binding.npc_id),
            Some(binding.greeting),
            "NPC {}: service Talk body",
            binding.npc_id
        );
    }

    // Dispatch path: gateway -> Gameplay -> gameplay/npc.rs -> dialogue_packet_with.
    let mut game = Gameplay::load(&gameplay_dir(), None).expect("gameplay content loads");
    for binding in BINDINGS {
        let (dialogue, menu_packet, runtime_guid) = in_world_talk(
            &mut game,
            binding.file,
            binding.npc_id,
            binding.npc_id % 1_000,
        );
        assert_eq!(
            dialogue[..4],
            GREETING_HEADER,
            "NPC {}: 26/3 family/subtype",
            binding.npc_id
        );
        assert_eq!(
            body_dword(&dialogue),
            binding.greeting,
            "NPC {}: 26/3 body is the first dword after the four-byte header",
            binding.npc_id
        );
        assert_eq!(
            &dialogue[12..20],
            runtime_guid.to_le_bytes(),
            "NPC {}: patch_npc_actor writes only bytes 12..20",
            binding.npc_id
        );
        // No menu option is lost: the real 26/9 is the service's own packet with
        // the clicked runtime GUID patched in, and the greeting id is never one of
        // its option labels.
        let mut fresh = service();
        let (zone, position) = fresh
            .location(binding.npc_id)
            .unwrap_or_else(|| panic!("NPC {}: location", binding.npc_id));
        let menu = fresh
            .interact(binding.npc_id, zone, position, None)
            .unwrap_or_else(|error| panic!("NPC {} menu: {error:?}", binding.npc_id));
        assert_eq!(
            &menu_packet[..4],
            &MENU_HEADER,
            "NPC {}: 26/9 family/subtype",
            binding.npc_id
        );
        assert_eq!(
            &menu_packet[4..12],
            &runtime_guid.to_le_bytes(),
            "NPC {}: the router patches the clicked runtime GUID",
            binding.npc_id
        );
        assert_eq!(
            &menu_packet[12..],
            &menu.packet[12..],
            "NPC {}: the menu options and name are the service's own",
            binding.npc_id
        );
        assert_eq!(
            menu.options.first().map(|option| option.action.clone()),
            Some(NpcAction::Talk),
            "NPC {}: Talk must stay the first menu option",
            binding.npc_id
        );
        assert_eq!(
            menu.options.first().map(|option| option.id),
            Some(1),
            "NPC {}: Talk is choice 1",
            binding.npc_id
        );
        assert!(
            menu.options
                .iter()
                .all(|option| u64::from(option.string_id) != u64::from(binding.greeting)),
            "NPC {}: the greeting body must not appear as a menu label",
            binding.npc_id
        );
    }
}

#[test]
fn an_unbound_row_still_sends_the_existing_empty_body() {
    // A blank enabled-quest target (Hrugagu) and a non-quest ambient control must
    // both send body 0 and never inherit a neighbour's line.
    let mut interaction = service();
    let visibility = NpcVisibility::load(&content_dir()).expect("NPC visibility loads");
    let mut game = Gameplay::load(&gameplay_dir(), None).expect("gameplay content loads");
    for (file, npc_id) in [
        ("greenwood_forest.json", 9500000309),
        ("sanctuary.json", 9500000443),
    ] {
        let record = npc_record(file, npc_id);
        assert_spawned(&visibility, npc_id, file, &record);
        assert!(
            record.get("greeting_string_id").is_none(),
            "NPC {npc_id}: the control row must stay unbound"
        );
        assert!(
            record["field_sources"].get("greeting_string_id").is_none(),
            "NPC {npc_id}: the control row has no greeting sources"
        );
        assert_eq!(
            service_talk_body(&mut interaction, npc_id),
            None,
            "NPC {npc_id}: an NPC without a greeting has no body"
        );
        let (dialogue, _, _) = in_world_talk(&mut game, file, npc_id, npc_id % 1_000);
        assert_eq!(
            body_dword(&dialogue),
            0,
            "NPC {npc_id}: the router still encodes a zero body"
        );
        assert!(
            BINDINGS
                .iter()
                .all(|binding| u64::from(binding.greeting) != 0),
            "zero is not a recovered greeting id"
        );
    }

    // A refused choice changes nothing: select Talk directly on the still-open
    // menu (no second `interact`), so this fails if `select` consumes the menu.
    let (zone, position) = interaction.location(9500000309).unwrap();
    let menu = interaction
        .interact(9500000309, zone, position, None)
        .unwrap();
    let before = menu.options.clone();
    let talk = menu
        .options
        .iter()
        .find(|option| option.action == NpcAction::Talk)
        .expect("the blank NPC offers Talk")
        .id;
    assert!(matches!(
        interaction.select(&select(9500000309, i32::MAX as u32), zone, position, None),
        Err(NpcInteractionError::InvalidOption)
    ));
    assert!(matches!(
        interaction.select(&[26, 0, 10, 0, 1, 2, 3], zone, position, None),
        Err(NpcInteractionError::InvalidPacket)
    ));
    assert!(matches!(
        interaction.select(
            &select(9_999_999_999, u32::try_from(talk).unwrap()),
            zone,
            position,
            None
        ),
        Err(NpcInteractionError::UnknownNpc)
    ));
    match interaction.select(
        &select(9500000309, u32::try_from(talk).unwrap()),
        zone,
        position,
        None,
    ) {
        Ok(NpcSelection::Talk(context)) => assert_eq!(
            context.body_string_id, None,
            "refusals must not consume the menu or inject a body"
        ),
        _ => panic!("the refusals must leave the menu usable"),
    }
    // The refused selections did not change the open menu's option list.
    let reopened = interaction
        .interact(9500000309, zone, position, None)
        .unwrap();
    assert_eq!(
        reopened.options, before,
        "a refusal must leave the menu exactly as it was"
    );
}

#[test]
fn two_players_see_the_same_greeting() {
    let npc_id = 9000000330; // Geoffrey, a Chapter 1 giver.
    let binding = binding_for(npc_id);
    let mut first = Gameplay::load(&gameplay_dir(), None).expect("first gameplay content");
    let mut second = Gameplay::load(&gameplay_dir(), None).expect("second gameplay content");

    let (dialogue_a, _, _) = in_world_talk(&mut first, binding.file, npc_id, 100);
    let (dialogue_b, _, _) = in_world_talk(&mut second, binding.file, npc_id, 200);
    assert_eq!(
        body_dword(&dialogue_a),
        binding.greeting,
        "player A: the static row greeting"
    );
    assert_eq!(
        body_dword(&dialogue_b),
        binding.greeting,
        "player B: the same static row greeting"
    );
    assert_eq!(
        body_dword(&dialogue_a),
        body_dword(&dialogue_b),
        "two sessions must see the same body"
    );

    // Closing A's dialogue (26/4) cannot blank B's next Talk: dialogue state is
    // per session and the body is read from the immutable content row.
    let character = character_at(npc_position(&npc_record(binding.file, npc_id), npc_id));
    first
        .handle_in_world(&[26, 0, 4, 0], &character, 300)
        .expect("26/4 close");
    let (dialogue_b2, _, _) = in_world_talk(&mut second, binding.file, npc_id, 400);
    assert_eq!(
        body_dword(&dialogue_b2),
        binding.greeting,
        "player A's close must not change player B's body"
    );

    // A fresh service load (a reconnect) returns the same body, with no write.
    assert_eq!(
        service_talk_body(&mut service(), npc_id),
        Some(binding.greeting),
        "a fresh loader sends the same static body"
    );

    // The shared harness path returns the same dword on the first session.
    assert_eq!(
        gameplay_talk_body(&mut first, binding.file, npc_id),
        binding.greeting,
        "the shared 26/8 -> 26/10 harness sees the same body"
    );
}

#[test]
fn the_greetings_add_no_quest_or_menu_change() {
    // The 26/9 option list is built from roles, active turn-ins and the trainer,
    // vendor, race, derby and fishing gates; `greeting_string_id` is not an input.
    // Two fresh loads must build the same list and packet, and the in-world 26/9
    // must be that same packet. Only the 26/3 Talk body carries the greeting.
    let mut first = service();
    let mut second = service();
    let mut game = Gameplay::load(&gameplay_dir(), None).expect("gameplay content loads");
    for (file, npc_id, min_options) in [
        ("sanctuary.json", 9000000330u64, 1usize), // Geoffrey, quest giver
        ("cobblestone_village.json", 9000000042, 1), // Carlos, quest giver and trainer role
        ("blackspore_swamp.json", 9000000047, 1),  // Chloe, quest giver and trainer role
    ] {
        let binding = binding_for(npc_id);
        let (zone, position) = first
            .location(npc_id)
            .unwrap_or_else(|| panic!("NPC {npc_id}: location"));
        let menu = first
            .interact(npc_id, zone, position, None)
            .unwrap_or_else(|error| panic!("NPC {npc_id} menu: {error:?}"));
        let (zone2, position2) = second
            .location(npc_id)
            .unwrap_or_else(|| panic!("NPC {npc_id}: second location"));
        let twin = second
            .interact(npc_id, zone2, position2, None)
            .unwrap_or_else(|error| panic!("NPC {npc_id} second menu: {error:?}"));
        assert_eq!(
            menu, twin,
            "NPC {npc_id}: a fresh load must build the same 26/9 menu"
        );
        assert_eq!(
            menu.options.first().map(|option| option.action.clone()),
            Some(NpcAction::Talk),
            "NPC {npc_id}: Talk remains choice 1"
        );
        assert!(
            menu.options.len() >= min_options,
            "NPC {npc_id}: Talk remains the menu's first option"
        );
        assert!(
            menu.options
                .iter()
                .all(|option| u64::from(option.string_id) != u64::from(binding.greeting)),
            "NPC {npc_id}: the greeting id is not a menu option"
        );

        let (dialogue, menu_packet, runtime_guid) =
            in_world_talk(&mut game, file, npc_id, 700 + npc_id % 100);
        assert_eq!(
            &menu_packet[4..12],
            &runtime_guid.to_le_bytes(),
            "NPC {npc_id}: the real 26/9 carries the clicked runtime GUID"
        );
        assert_eq!(
            &menu_packet[12..],
            &menu.packet[12..],
            "NPC {npc_id}: the real 26/9 options equal the service menu"
        );
        assert_eq!(
            body_dword(&dialogue),
            binding.greeting,
            "NPC {npc_id}: only the Talk body carries the greeting"
        );
    }
}

/// The runtime GUID of a renderable `npc_spawns_extra` row for `content_key`,
/// applying the production loader's own skip rules (`capture_match` rows, the
/// missing knocker base model, non-finite positions and a zero content key).
fn extra_spawn_guid(npc_id: u64) -> Option<u64> {
    let document = read_json(&content_dir().join("npc_spawns_extra.json"));
    document["records"].as_array()?.iter().find_map(|record| {
        if record["content_key"].as_u64() != Some(npc_id)
            || !record["capture_match"].is_null()
            || record["model_file_name"]
                .as_str()
                .is_some_and(|name| name.eq_ignore_ascii_case("knocker_m_boss.adr"))
        {
            return None;
        }
        let position = record["position"].as_array()?;
        if position.len() != 3
            || !position
                .iter()
                .all(|value| value.as_f64().is_some_and(f64::is_finite))
        {
            return None;
        }
        record["spawn_guid"].as_u64().filter(|guid| *guid != 0)
    })
}

/// The row is drawable through the production visibility loader: a spawned
/// `legacy_data` row is keyed by its own NPC id, and an extra spawn is keyed by
/// its runtime `spawn_guid` and maps back to the content key.
fn assert_spawned(visibility: &NpcVisibility, npc_id: u64, file: &str, record: &Value) {
    if record["placement"].as_str() == Some("legacy_data") {
        assert_eq!(
            visibility.content_key(npc_id),
            Some(npc_id),
            "NPC {npc_id} in {file}: a legacy_data row must spawn as its NPC key"
        );
        return;
    }
    let spawn_guid = extra_spawn_guid(npc_id).unwrap_or_else(|| {
        panic!("NPC {npc_id} in {file}: no renderable npc_spawns_extra row with its content key")
    });
    assert_eq!(
        visibility.content_key(spawn_guid),
        Some(npc_id),
        "NPC {npc_id} in {file}: the extra spawn {spawn_guid} must key back to the NPC"
    );
}
