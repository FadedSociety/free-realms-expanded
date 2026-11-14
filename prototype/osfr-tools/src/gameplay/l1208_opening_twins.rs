//! L1208: an opening stage and a regional quest record can be the same client quest (one
//! title id: Fighting off the Pack 910007 = Fighting the Pack 3000009). Finishing the stage
//! settles the record so Cleetus and Pappy Farnum never offer it again; a save that finished
//! stages before this settles them at login. Built on the shipped content: `data/content`
//! (quest records) and `data/release/opening.json` (the stages the release ships).
use super::*;
use crate::opening::OpeningStage;
use crate::quest_engine::{QuestProgress, QuestState, QuestStatus};

fn shipped() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn release() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/release")
}

/// The shipped opening with every stage before `stage_id` finished.
fn opening_through(stage_id: u32) -> OpeningEngine {
    let fresh = OpeningEngine::load(&release(), None).unwrap();
    let mut state = fresh.state().clone();
    for stage in fresh.stages().iter().filter(|s| s.id < stage_id) {
        let progress = state.stages.get_mut(&stage.id).unwrap();
        progress.accepted = true;
        progress.completed = true;
        progress.rewarded = true;
        progress.counts = stage.objectives.iter().map(|g| g.required).collect();
    }
    OpeningEngine::load(&release(), Some(state)).unwrap()
}

/// Every shipped quest record id whose client title is `title_id` (read from the data).
fn records_titled(title_id: u32) -> Vec<u32> {
    let mut ids = Vec::new();
    for entry in std::fs::read_dir(shipped().join("quests")).unwrap() {
        let text = std::fs::read_to_string(entry.unwrap().path()).unwrap();
        let file: serde_json::Value = serde_json::from_str(&text).unwrap();
        let records = file["records"].as_array().into_iter().flatten();
        ids.extend(
            records
                .filter(|r| r["title_string_id"].as_u64() == Some(u64::from(title_id)))
                .filter_map(|r| r["id"].as_u64().and_then(|id| u32::try_from(id).ok())),
        );
    }
    ids
}

fn game_with(opening: OpeningEngine, saved: Option<QuestState>) -> Gameplay {
    let mut game = Gameplay::load(&shipped(), None).unwrap();
    game.opening = Some(opening);
    game.saved_npc_quests = saved;
    game.bind_npc_quest_catalogue(&shipped(), 1).unwrap();
    game
}

fn status(game: &Gameplay, quest_id: u32) -> Option<QuestStatus> {
    let engine = game.npc_quests.as_ref().unwrap();
    engine.progress(quest_id).map(|p| p.status.clone())
}

/// The first shipped stage that has an enabled regional twin, and that twin.
fn first_twinned_stage(game: &Gameplay) -> (OpeningStage, u32) {
    let engine = game.npc_quests.as_ref().unwrap();
    let opening = OpeningEngine::load(&release(), None).unwrap();
    (opening.stages().iter())
        .find_map(|stage| {
            let mut twins = records_titled(stage.title_id).into_iter();
            let twin = twins.find(|id| engine.is_enabled(*id))?;
            Some((stage.clone(), twin))
        })
        .expect("the shipped opening has a stage with an enabled regional twin")
}

#[test]
fn a_finished_stage_settles_its_regional_twin_once_without_paying() {
    let mut game = game_with(OpeningEngine::load(&release(), None).unwrap(), None);
    let (stage, twin) = first_twinned_stage(&game);
    assert_eq!(status(&game, twin), None, "a fresh character has not met the twin");
    // The stage's goals are met; the Talk to its turn-in guide completes it.
    let mut opening = opening_through(stage.id);
    let mut state = opening.state().clone();
    let progress = state.stages.get_mut(&stage.id).unwrap();
    progress.accepted = true;
    progress.counts = stage.objectives.iter().map(|g| g.required).collect();
    opening = OpeningEngine::load(&release(), Some(state)).unwrap();
    let guide = (opening.actors().iter()).find(|a| a.guid == stage.turnin_guid).unwrap();
    opening.set_location_without_events(guide.zone_id, guide.position).unwrap();
    let result = opening.on_event(OpeningEvent::Talk { actor_id: stage.turnin_guid }).unwrap();
    assert_eq!(result.completed_title_ids, [stage.title_id], "{:?}", result.messages);
    // Only the settle part of the outcome: the stage's own reward is not under test here.
    let completed_title_ids = result.completed_title_ids;
    let settle = OpeningOutcome { completed_title_ids, ..Default::default() };
    let mut out = Outcome::default();
    game.apply_opening(settle.clone(), &mut out).unwrap();
    assert!(out.changed);
    assert_eq!(status(&game, twin), Some(QuestStatus::Completed));
    assert!(out.replies.is_empty(), "the twin pays and says nothing; the stage paid");
    // The settled record is a valid completion: a reload of the saved state accepts it.
    let fresh = OpeningEngine::load(&release(), None).unwrap();
    let reloaded = game_with(fresh, game.npc_quest_state());
    assert_eq!(status(&reloaded, twin), Some(QuestStatus::Completed));
    // A second report changes nothing.
    let mut again = Outcome::default();
    game.apply_opening(settle, &mut again).unwrap();
    assert!(!again.changed);
}

#[test]
fn a_save_that_finished_the_opening_settles_every_enabled_twin_at_login() {
    let game = game_with(opening_through(u32::MAX), None);
    let engine = game.npc_quests.as_ref().unwrap();
    let opening = OpeningEngine::load(&release(), None).unwrap();
    let mut settled = 0;
    for stage in opening.stages() {
        for id in records_titled(stage.title_id) {
            if engine.is_enabled(id) {
                let done = Some(QuestStatus::Completed);
                assert_eq!(status(&game, id), done, "{id} twins {}", stage.id);
                settled += 1;
            } else {
                assert_eq!(status(&game, id), None, "disabled twin {id} stays untouched");
            }
        }
    }
    assert!(settled > 0, "the shipped content has enabled twins");
    let completed = (engine.state().quests.values())
        .filter(|p| p.status == QuestStatus::Completed)
        .count();
    assert_eq!(completed, settled, "only twins were settled");
}

#[test]
fn a_twin_the_player_already_took_stays_in_the_journal() {
    let probe = game_with(OpeningEngine::load(&release(), None).unwrap(), None);
    let (stage, twin) = first_twinned_stage(&probe);
    // The record's goal count, from the copy a finished stage settles.
    let settled = game_with(opening_through(stage.id + 1), None);
    let goals = settled.npc_quests.as_ref().unwrap().progress(twin).unwrap().counts.len();
    let mut quests = BTreeMap::new();
    let active = QuestProgress { status: QuestStatus::Active, counts: vec![0; goals] };
    quests.insert(twin, active.clone());
    let saved = QuestState {
        version: 1,
        character_id: 1,
        quests,
        pending_offer: None,
        pending_turnin: None,
        pending_turnin_actor: None,
        selected_quest_id: None,
        auto_switch_selected: false,
        repeats: Default::default(),
    };
    let game = game_with(opening_through(stage.id + 1), Some(saved));
    assert_eq!(game.npc_quests.as_ref().unwrap().progress(twin), Some(&active));
}
