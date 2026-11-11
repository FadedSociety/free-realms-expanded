//! The encounter's start card (41/114 EncounterDetailsResponse) and the goal rows it lists.
//!
//! The client builds its goal panel from the card's goal list (`HandleMiniGameStart` and
//! `HandlerMiniGameStatus_Populate` -> `00c42c60` -> one 0x110 record per row, `00c42870`), then
//! 45/1, 45/2 and 45/3 find each record by its key (`009ad510`, `009ad5a0`, `009b6700`). A row
//! the card does not list never shows, so every goal a fight shows must be on its card.
use super::*;

/// One row of the card's goal list: key, name text and category. Wire order (reader
/// `008fd770`): key, name id, description id, a flag, an empty reward bundle, four counters, a
/// members-only flag, then the category `GetObjectiveCategoryPrefix` (`00a872f0`) prefixes.
pub(super) type GoalRow = [i32; 3];
/// The category whose prefix is "Bonus" (`ObjectiveCategoryPrefixBonus`). The main goal keeps 0,
/// no prefix, as the video's "Goals (2)" panel shows it (5_U-uAzDuR4/00002).
const BONUS_CATEGORY: i32 = 4;
/// The opening cave's single row names Darkthorne's defence; natively accepted as sent.
const CAVE_GOAL_NAME: i32 = 4419610;

impl EncounterEngine {
    /// The rows a card lists: every goal of the encounter's fight file that shows a row, else
    /// the encounter's one goal, named by its own text.
    pub(super) fn card_goals(&self, id: u32, objective_id: i32) -> Vec<GoalRow> {
        let mut rows: Vec<GoalRow> = Vec::new();
        let fight = self.fight_state.fights.get(&id);
        for goal in fight.iter().flat_map(|f| &f.objectives) {
            if goal.text_id > 0 && !rows.iter().any(|row| row[0] == goal.text_id) {
                let category = if goal.bonus { BONUS_CATEGORY } else { 0 };
                rows.push([goal.text_id, goal.text_id, category]);
            }
        }
        if rows.is_empty() {
            rows.push([objective_id, objective_id, 0]);
        }
        rows
    }

    /// The card of the encounter now running (a zone-ready refresh when `launch`).
    pub(super) fn details(&self, d: &Dungeon, a: &ActiveEncounter, launch: bool) -> Vec<u8> {
        let goals = match d.guide {
            Some(_) => vec![[d.objective_id, CAVE_GOAL_NAME, 0]],
            None => self.card_goals(d.id, d.objective_id),
        };
        details_packet(
            d.id,
            a.instance_id,
            [d.title_id, d.icon_id, d.description_id, 1],
            d.enemies.len(),
            launch,
            &d.prizes,
            &goals,
        )
    }
}

/// 41/114 EncounterDetailsResponse. `launch = false` opens the minigame start
/// screen (HandleMiniGameStart); `true` only refreshes a running encounter.
/// `text` is [title, icon, description, difficulty] (MiniGameInfo NameId, IconId,
/// DescriptionId, Difficulty per the community layout and the native reader
/// 009bdd70); the cave's difficulty was a constant 1.
pub(super) fn details_packet(
    id: u32,
    instance: i32,
    text: [i32; 4],
    enemy_count: usize,
    launch: bool,
    prizes: &[Prize],
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
    b.extend([0; 138]);
    b.extend(prize_bundle(prizes));
    b.extend((goals.len() as i32).to_le_bytes());
    for [key, name, category] in goals {
        // Key, name, description (the name again), flag, empty reward bundle and the four
        // counters 45/1 and 45/2 fill later, the members-only flag, then the category.
        for v in [*key, *name, *name] {
            b.extend(v.to_le_bytes());
        }
        b.push(0);
        b.extend([0; 65]);
        b.extend(category.to_le_bytes());
    }
    for v in [0_i32, 0, enemy_count as i32, 0] {
        b.extend(v.to_le_bytes());
    }
    b.push(0);
    b.extend(0_i32.to_le_bytes());
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
