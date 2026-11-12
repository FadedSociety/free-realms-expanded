//! rc-0136 recheck: a Talk that offers nothing still says something. In the owner's run on
//! build `3a3280a3` Rizzle (A Beautiful Friendship active), Vladimir (Going Batty done) and
//! other quest NPCs answered `npc_select outcome: dialogue, reason: no_eligible_quest` with a
//! 26/3 whose body id was 0: an empty text box (`ShowDialogFromPacket` `0x009f5ae0` looks the
//! first payload dword up as `Global.Text.<id>`).
//!
//! `Gameplay::talk_to_npc` now picks, in order: a completed talk step's own line, the greeting,
//! the quest line (`QuestEngine::talk_line`: an active quest's open goal line, else the closing
//! line of the last quest handed in here), then the stock "Hello!" (`Global.Text.139503`).
//! Every body must be a stock client line: the launcher's `locale-additions.json` rows are on
//! no install until the launcher writes them.
//!
//! Every quest NPC is clicked through a real `Gateway` session (26/8, the menu's Talk row
//! through 26/10), moved beside each one by the GM panel's relocation
//! (`Gateway::stage_admin_relocation`): with every enabled quest done and with every enabled
//! quest active (every event held on, so event NPCs are drawn), and out of season (a fresh save,
//! every live and calendar event off), where a giver whose only quests belong to an event has
//! nothing to offer.

#[path = "common/npc_greetings.rs"]
mod npc_greetings;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/placement.rs"]
mod placement;
#[path = "support/quest_records.rs"]
mod quest_records;

use npc_quest_chain::{
    CHARACTER, Fixture, TALK_LABEL, content_dir, gateway_send, menu_option_opt, npc_interact,
    npc_location, npc_select,
};
use osfr_tools::{
    event_calendar::{self, EventCalendar, Mode},
    gameplay::GameplayState,
    gateway::Gateway,
    npc_interaction::{NpcInteraction, decode_dialogue_26_3},
    quest_engine::{CharacterEligibility, QuestEngine, QuestProgress, QuestStatus},
    world::NpcVisibility,
    world_text::locale_hash,
};
use std::collections::{BTreeMap, BTreeSet};

/// The stock "Hello!" an NPC with no greeting and no quest line says.
const HELLO: u32 = 139_503;
const RIZZLE: u64 = 9_500_000_848;
/// A Beautiful Friendship's first goal, "Find the Stolen Sasparilla".
const FIND_THE_SASPARILLA: u32 = 439_836;
const VLADIMIR: u64 = 9_500_000_542;

/// The shipped catalogue and NPC table.
struct Shipped {
    engine: QuestEngine,
    /// Enabled quest ids, ascending (the save's order).
    enabled: Vec<u32>,
    /// Every enabled quest's giver, turn-in NPC and talk targets.
    npcs: BTreeSet<u64>,
    greetings: BTreeMap<u64, u32>,
    /// Content NPC key -> the runtime actor the client clicks (the 35/2 draw set).
    actors: BTreeMap<u64, u64>,
    table: NpcInteraction,
}

fn shipped() -> Shipped {
    let root = content_dir();
    let engine =
        QuestEngine::from_split_content(&root, CHARACTER, CharacterEligibility::default(), None)
            .expect("the split quest catalogue loads");
    let records = quest_records::read_quests(&root);
    let records: Vec<_> = (records.into_iter())
        .filter(|quest| engine.is_enabled(quest.id))
        .collect();
    let mut enabled: Vec<u32> = records.iter().map(|quest| quest.id).collect();
    enabled.sort_unstable();
    let mut npcs = BTreeSet::new();
    for quest in &records {
        npcs.insert(quest.giver_npc_id);
        npcs.extend(engine.turnin_npc_id(quest.id));
        npcs.extend(
            quest
                .objectives
                .iter()
                .filter_map(|goal| goal["npc_id"].as_u64()),
        );
    }
    let visibility = NpcVisibility::load(&root).expect("the NPC draw set loads");
    let mut actors = BTreeMap::new();
    for drawn in placement::drawn_npcs(&visibility) {
        if let Some(key) = drawn.content_key {
            actors.entry(key).or_insert(drawn.actor);
        }
    }
    let greetings = npc_greetings::greeting_bindings()
        .into_iter()
        .map(|((_, npc), greeting)| (npc, greeting))
        .collect();
    Shipped {
        engine,
        enabled,
        npcs,
        greetings,
        actors,
        table: NpcInteraction::load(&root.join("npcs"), &root.join("stores.json"))
            .expect("the NPC table loads"),
    }
}

/// `base` with every enabled quest at `status`. Empty counters: `repair_counters` fills a done
/// quest's and zeroes an active one's.
fn with_every_quest(base: &GameplayState, enabled: &[u32], status: QuestStatus) -> GameplayState {
    let mut state = base.clone();
    let quests = &mut state.npc_quests.as_mut().unwrap().quests;
    for &id in enabled {
        let progress = QuestProgress {
            status: status.clone(),
            counts: Vec::new(),
        };
        quests.insert(id, progress);
    }
    state
}

/// One click and Talk on `actor`: every reply, or `None` when its menu has no Talk row.
fn talk(gateway: &mut Gateway, actor: u64, now: u64) -> Option<Vec<Vec<u8>>> {
    let opened = gateway_send(gateway, &npc_interact(actor), now);
    let menu = opened.iter().find(|p| p.starts_with(&[26, 0, 9, 0]))?;
    let row = menu_option_opt(menu, TALK_LABEL)?;
    Some(gateway_send(gateway, &npc_select(actor, row), now + 1))
}

struct Sweep {
    /// The body of every Talk that sent one dialogue and no offer (49/1) or turn-in (49/13).
    quiet: BTreeMap<u64, u32>,
    /// Every 26/3 body any Talk sent.
    sent: BTreeSet<u32>,
    /// NPCs this session cannot click: no drawn actor, no place, or no Talk row.
    skipped: Vec<u64>,
}

/// Move beside every quest NPC with the GM panel's relocation and Talk to it.
fn sweep(shipped: &Shipped, gateway: &mut Gateway) -> Sweep {
    let mut out = Sweep {
        quiet: BTreeMap::new(),
        sent: BTreeSet::new(),
        skipped: Vec::new(),
    };
    let mut now = 100_000;
    for &npc in &shipped.npcs {
        now += 1_000;
        // A regional `zone_id` is a content label inside the continent the session stands in.
        let place = shipped.table.location(npc);
        let replies = match (shipped.actors.get(&npc), place) {
            (Some(&actor), Some((_, position)))
                if gateway.stage_admin_relocation(position, None, now).is_ok() =>
            {
                talk(gateway, actor, now)
            }
            _ => None,
        };
        let Some(replies) = replies else {
            out.skipped.push(npc);
            continue;
        };
        let bodies: Vec<u32> = (replies.iter().filter(|p| p.starts_with(&[26, 0, 3, 0])))
            .map(|p| {
                decode_dialogue_26_3(p)
                    .expect("26/3 decodes")
                    .initial_dwords[0]
            })
            .collect();
        let offer = (replies.iter())
            .any(|p| p.starts_with(&[49, 0, 1, 0]) || p.starts_with(&[49, 0, 13, 0]));
        if let ([body], false) = (bodies.as_slice(), offer) {
            out.quiet.insert(npc, *body);
        }
        out.sent.extend(bodies);
    }
    out
}

#[test]
fn every_quest_npc_says_a_stock_line_when_it_offers_nothing() {
    let shipped = shipped();
    let fixture = Fixture::for_character(CHARACTER).with_bulk_quests();
    let at = npc_location(RIZZLE).1;
    let base = fixture
        .session(CHARACTER, at, None)
        .gameplay()
        .unwrap()
        .state();
    let mut failures = Vec::new();
    let mut sent = BTreeSet::new();
    // Every live and calendar event held on (`on`) or off (`off`), as an admin's overrides do.
    let calendar = EventCalendar::load(&content_dir()).expect("the event calendar loads");
    let on = fixture.open_live_events(shipped.engine.live_events().keys().copied());
    let off = fixture.path().join("live-off");
    std::fs::create_dir_all(&off).unwrap();
    for key in calendar.keys() {
        event_calendar::write_override(&on, key, Some(Mode::On)).unwrap();
        event_calendar::write_override(&off, key, Some(Mode::Off)).unwrap();
    }
    for status in [QuestStatus::Completed, QuestStatus::Active] {
        let state = with_every_quest(&base, &shipped.enabled, status.clone());
        let mut gateway = fixture.session(CHARACTER, at, Some(state));
        let game = gateway.gameplay_mut().unwrap();
        game.set_world_dir(Some(on.clone())).unwrap();
        let pass = sweep(&shipped, &mut gateway);
        let clicked = shipped.npcs.len() - pass.skipped.len();
        assert!(
            clicked * 10 >= shipped.npcs.len() * 9,
            "{status:?}: only {clicked} of {} quest NPCs could be clicked: {:?}",
            shipped.npcs.len(),
            pass.skipped
        );
        assert!(
            pass.quiet.len() * 10 >= clicked * 9,
            "{status:?}: most Talks offered something"
        );
        if status == QuestStatus::Completed {
            // Nothing is active, so no step line: the greeting, the closing line, or Hello.
            for (&npc, &body) in &pass.quiet {
                let closing = (shipped.enabled.iter())
                    .filter(|id| shipped.engine.turnin_npc_id(**id) == Some(npc))
                    .filter_map(|id| shipped.engine.end_text_string_id(*id))
                    .last();
                let want = (shipped.greetings.get(&npc).copied())
                    .or(closing)
                    .unwrap_or(HELLO);
                if body != want {
                    failures.push(format!("{status:?}: {npc} said {body}, not {want}"));
                }
            }
            // The owner's Vladimir after Going Batty: no greeting and no closing line yet.
            assert_eq!(pass.quiet.get(&VLADIMIR), Some(&HELLO));
        } else {
            // The owner's Rizzle with A Beautiful Friendship active: its open goal.
            assert_eq!(pass.quiet.get(&RIZZLE), Some(&FIND_THE_SASPARILLA));
        }
        sent.extend(pass.sent);
    }
    // Out of season, a giver whose only quests belong to an event offers nothing and still
    // speaks (QF: Camellia, Candi Ivy, Counselor Wes and others were empty boxes).
    let mut gateway = fixture.session(CHARACTER, at, None);
    let game = gateway.gameplay_mut().unwrap();
    game.set_world_dir(Some(off)).unwrap();
    let pass = sweep(&shipped, &mut gateway);
    let gated: BTreeSet<u32> = (calendar.gated_quests().into_iter())
        .chain(shipped.engine.live_events().keys().copied())
        .collect();
    let event_only: Vec<u64> = (shipped.npcs.iter().copied())
        .filter(|npc| !pass.skipped.contains(npc))
        .filter(|npc| {
            let mut given = (shipped.engine.quest_ids_for_giver(*npc))
                .filter(|id| shipped.engine.is_enabled(*id))
                .peekable();
            given.peek().is_some() && given.all(|id| gated.contains(&id))
        })
        .collect();
    assert!(!event_only.is_empty(), "no event-only giver was clicked");
    for npc in event_only {
        if !pass.quiet.contains_key(&npc) {
            failures.push(format!("out of season: event-only giver {npc} sent no lone 26/3"));
        }
    }
    sent.extend(pass.sent);
    // Every body sent is a non-empty line of the stock 2014 client locale.
    let hashes: BTreeMap<u32, u32> = sent
        .iter()
        .map(|id| (locale_hash(*id as i32), *id))
        .collect();
    let rows = npc_greetings::locale_rows(&hashes.keys().copied().collect());
    for (hash, id) in &hashes {
        match rows.get(hash) {
            Some((_, text)) if !text.trim().is_empty() => {}
            _ => failures.push(format!("body {id} is no stock client line")),
        }
    }
    assert_eq!(rows[&locale_hash(HELLO as i32)].1, "Hello!");
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
