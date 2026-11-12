//! The encounter's start card (41/114 EncounterDetailsResponse) and the goal rows it lists.
//!
//! The client builds its goal panel from the card's goal list (`HandleMiniGameStart` and
//! `HandlerMiniGameStatus_Populate` -> `00c42c60` -> one 0x110 record per row, `00c42870`), then
//! 45/1, 45/2 and 45/3 find each record by its key (`009ad510`, `009ad5a0`, `009b6700`). A row
//! the card does not list never shows, so every goal a fight shows must be on its card. The
//! in-game Goals panel on the side is another list (`BaseClient.ObjectiveHelper`), filled by
//! 47/1, 47/2 and 47/3 (`BaseUiPacketDispatch` `00a91bf0`); [`goal_opened`] and its two siblings
//! send each goal to both.
use super::*;

/// One row of the card's goal list. Wire order (reader `008fd770`, 103 bytes with its empty
/// bundle): key, name id, description id, a flag, a reward bundle (`008e7930`), four counters
/// (state, count, total, kind; `00c42870` copies them), a members-only flag, then the category
/// `GetObjectiveCategoryPrefix` (`00a872f0`) prefixes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct GoalRow {
    pub key: i32,
    pub name: i32,
    pub category: i32,
    /// The progress target; above 1 the record reads "count/total" (`00c42440`).
    pub total: i32,
}
/// The category whose prefix is "Bonus" (`ObjectiveCategoryPrefixBonus`). The main goal keeps 0,
/// no prefix, as the video's "Goals (2)" panel shows it (5_U-uAzDuR4/00002).
const BONUS_CATEGORY: i32 = 4;
/// The opening cave's single row names Darkthorne's defence; natively accepted as sent.
const CAVE_GOAL_NAME: i32 = 4419610;

impl EncounterEngine {
    /// The rows a card lists: every goal of the encounter's fight file that shows a row, each
    /// with the total the fight starts it at, else the encounter's one goal, named by its own
    /// text, with `total` (the roster).
    pub(super) fn card_goals(&self, id: u32, objective_id: i32, total: usize) -> Vec<GoalRow> {
        let mut rows: Vec<GoalRow> = Vec::new();
        if let Some(fight) = self.fight_state.fights.get(&id) {
            let run = director::begin(fight, &director::Party::default());
            for (goal, at_start) in fight.objectives.iter().zip(&run.goals) {
                if goal.text_id > 0 && !rows.iter().any(|row| row.key == goal.text_id) {
                    rows.push(goal_row(goal.text_id, goal.bonus, at_start.of as i32));
                }
            }
        }
        if rows.is_empty() {
            rows.push(goal_row(objective_id, false, total as i32));
        }
        rows
    }

    /// The card of the encounter now running (a zone-ready refresh when `launch`).
    pub(super) fn details(&self, d: &Dungeon, a: &ActiveEncounter, launch: bool) -> Vec<u8> {
        let goals = match d.guide {
            Some(_) => vec![GoalRow {
                key: d.objective_id,
                name: CAVE_GOAL_NAME,
                category: 0,
                total: d.enemies.len() as i32,
            }],
            None => self.card_goals(d.id, d.objective_id, d.enemies.len()),
        };
        details_packet(
            d.id,
            a.instance_id,
            [d.title_id, d.icon_id, d.description_id, 1],
            launch,
            &prize_bundle(&wheel_rows(&d.prizes, self.player_profile), d.wheel_coins),
            &goals,
        )
    }
}

/// 47/1 and 45/1: a goal opens, as a row of the Goals panel (`00a8d1f0`) and as the total and
/// InProgress state of the card's record (`009ad510`).
pub(super) fn goal_opened(row: &GoalRow) -> [Vec<u8>; 2] {
    let goal = crate::minigame::Goal {
        id: row.key as u32,
        name_id: row.name as u32,
        description_id: row.name as u32,
        category: row.category as u32,
        bonus: row.category == BONUS_CATEGORY,
        stars: 0,
        coins: 0,
        total: row.total.max(0) as u32,
        failure_condition: false,
        members_only: false,
    };
    [
        crate::minigame::goal_row_packet(&goal),
        objective(1, row.key, row.total),
    ]
}

/// 47/2 and 45/2: a goal's count moved (each list keeps its own copy).
pub(super) fn goal_counted(key: i32, count: i32) -> [Vec<u8>; 2] {
    crate::minigame::goal_progress_packets(key as u32, count.max(0) as u32)
}

/// 47/3 and 45/3: the goal is met; 45/3 carries the award the client adds to its score.
pub(super) fn goal_met(key: i32) -> [Vec<u8>; 2] {
    let [panel, _] = crate::minigame::goal_done_packets(key as u32);
    [panel, objective_complete(key)]
}

/// A goal named by its own text: a fight goal, or a legacy row's single goal.
pub(super) fn goal_row(key: i32, bonus: bool, total: i32) -> GoalRow {
    GoalRow {
        key,
        name: key,
        category: if bonus { BONUS_CATEGORY } else { 0 },
        total,
    }
}

/// 41/114 EncounterDetailsResponse. `launch = false` opens the minigame start
/// screen (HandleMiniGameStart); `true` only refreshes a running encounter.
/// `text` is [title, icon, description, difficulty] (MiniGameInfo NameId, IconId,
/// DescriptionId, Difficulty per the community layout and the native reader
/// 009bdd70); the cave's difficulty was a constant 1. `preview` is the third reward bundle:
/// the prizes the start screen lists and the loot wheel spins.
pub(super) fn details_packet(
    id: u32,
    instance: i32,
    text: [i32; 4],
    launch: bool,
    preview: &[u8],
    goals: &[GoalRow],
) -> Vec<u8> {
    let [title_id, icon_id, description_id, difficulty] = text;
    let mut b = encounter_header(114, id, instance);
    for v in [0_i32, 0, 0, 0, 8, 0] {
        b.extend(v.to_le_bytes());
    }
    b.extend([1, 0, 1]);
    for v in [0_i32, 0] {
        b.extend(v.to_le_bytes());
    }
    // Original Lua MiniGameTypes.COMBAT_TYPE is 4. SAMPLE_TYPE (1)
    // disables native combat target selection despite combat rules being on.
    // MiniGameInfo.ProfileType is the job TYPE the game needs, not the
    // player's job id: the start screen's Switch Jobs picker lists owned
    // jobs of this type whose rank meets the difficulty (00c8ff30), and
    // only enables at all when it is non-zero. Combat jobs are type 2
    // (profile definitions: Brawler, Medic, Wizard, Ninja, ...).
    for v in [
        title_id,
        icon_id,
        description_id,
        difficulty,
        COMBAT_PROFILE_TYPE,
        4,
    ] {
        b.extend(v.to_le_bytes());
    }
    b.push(0);
    // Reward bundles: base and member empty; the third ("preview") lists
    // the prizes on the start screen and hides its placeholder art.
    b.extend(prize_bundle(&[], 0));
    b.extend(prize_bundle(&[], 0));
    b.extend(preview);
    // The goal list (`009bc380`). Each row's counters start NotStarted (state 0) at count 0;
    // 45/1 and 45/2 move them in the fight.
    b.extend((goals.len() as i32).to_le_bytes());
    for row in goals {
        for v in [row.key, row.name, row.name] {
            b.extend(v.to_le_bytes());
        }
        b.push(0);
        b.extend(prize_bundle(&[], 0));
        for v in [0, 0, row.total, 0] {
            b.extend(v.to_le_bytes());
        }
        b.push(0);
        b.extend(row.category.to_le_bytes());
    }
    // MiniGameInfo's tail (`009bdd70` after the list): five flags, an empty string, then the
    // remaining ids and flags; the record's two closing flags (`00a29120`); and the details'
    // own launch flag, word and empty list (`00aa32d0`). `launch` is the flag
    // `ClientCommandProcessor__DispatchEncounterPacket` reads to refresh instead of start.
    b.extend([1; 5]);
    b.extend(0_i32.to_le_bytes());
    b.extend(1_i32.to_le_bytes());
    b.push(1);
    b.extend(0_i32.to_le_bytes());
    b.extend([0; 4]);
    b.extend(0_i32.to_le_bytes());
    b.extend([0, 1, u8::from(launch)]);
    b.extend([0; 8]);
    b
}

/// The largest quantity one battle prize row may preview. The content validator
/// enforces it; the stack preflight additionally caps each row at the item's
/// own `max_stack`.
pub const MAX_PRIZE_QUANTITY: u32 = 100;
/// The most prizes one job's wheel holds (the wiki's battle wheels hold up to seven, plus coins).
pub(super) const MAX_WHEEL: usize = 8;

/// A prize shown on the battle start screen and won at random on victory
/// (the battle's "prize wheel"; the owner: prizes spin like a loot box). Item
/// definition, name and icon ids come from the client item catalogue. `job` is the combat
/// job (profile id) whose wheel holds it, as each battle's wiki page lists one wheel per job;
/// 0 is the wheel of any other job.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Prize {
    pub item_id: i32,
    pub name_id: i32,
    pub icon_id: i32,
    #[serde(default = "one")]
    pub quantity: u32,
    #[serde(default)]
    pub job: i32,
}

/// The wheel a player in `job` spins: that job's prizes, else the any-job ones.
pub(super) fn wheel_rows(prizes: &[Prize], job: i32) -> Vec<Prize> {
    let own: Vec<Prize> = prizes.iter().filter(|p| p.job == job).cloned().collect();
    if own.is_empty() {
        prizes.iter().filter(|p| p.job == 0).cloned().collect()
    } else {
        own
    }
}

/// Every job's wheel is well formed: ids set, quantities in range, at most [`MAX_WHEEL`] each.
pub(super) fn wheel_ok(prizes: &[Prize]) -> bool {
    let jobs: BTreeSet<i32> = prizes.iter().map(|p| p.job).collect();
    jobs.iter().all(|&job| {
        job >= 0 && prizes.iter().filter(|p| p.job == job).count() <= MAX_WHEEL
    }) && prizes.iter().all(|p| {
        let quantity = (1..=MAX_PRIZE_QUANTITY).contains(&p.quantity);
        p.item_id > 0 && p.name_id > 0 && p.icon_id > 0 && quantity
    })
}

/// Where the wheel stops: one of the job's prizes, or the coin slot the client adds after
/// them when the preview carries coins (`ScoreScreen:PopulateLootWheel`).
pub(super) enum Landing {
    Item(Prize),
    Coins(u32),
}

/// (R) One slot of the job's wheel, uniform over the prizes and the coin slot.
pub(super) fn spin(prizes: &[Prize], job: i32, coins: u32, now_ms: u64) -> Option<Landing> {
    let rows = wheel_rows(prizes, job);
    let slots = rows.len() + usize::from(coins > 0);
    if slots == 0 {
        return None;
    }
    let slot = (now_ms % slots as u64) as usize;
    Some(match rows.get(slot) {
        Some(prize) => Landing::Item(prize.clone()),
        None => Landing::Coins(coins),
    })
}

/// 39/45 MiniGameLootWheelSetItemToLandOn: the wheel lands on the preview entry of this item,
/// or with no entry and coins, on the coin slot (native `StopMiniGameLootWheelAt` 009b6da0);
/// the client answers 39/46 when the spin stops, then shows the scores.
pub(super) fn landing_bundle(landing: &Landing) -> Vec<u8> {
    match landing {
        Landing::Item(prize) => prize_bundle(std::slice::from_ref(prize), 0),
        Landing::Coins(coins) => prize_bundle(&[], *coins),
    }
}

/// A reward bundle (native reader 008e7930) from the one writer every wheel uses
/// (`minigame::reward_bundle`: coins at `+0x50`, the coin multiplier `+0x78` at 1.0, as the
/// client's own constructor 0091c110 starts it; it was 0.0 here, so a coin slot showed 0).
/// Each prize is a type-1 Item entry (base reader 0103c460): hidden, icon, tint, name,
/// quantity, item id, param, empty string, text colour, members-only. Empty, it is 69 bytes.
pub(super) fn prize_bundle(prizes: &[Prize], coins: u32) -> Vec<u8> {
    let rows: Vec<crate::rewards::RewardBundleRow> = (prizes.iter())
        .map(|p| {
            let mut body = vec![0]; // hidden
            for v in [p.icon_id, 0, p.name_id, p.quantity as i32, p.item_id, 0, 0, 0] {
                body.extend(v.to_le_bytes());
            }
            body.push(0); // members only
            crate::rewards::RewardBundleRow { type_id: 1, body }
        })
        .collect();
    crate::minigame::reward_bundle(coins, &rows)
}
