//! L052: the staff jobs Referee (58) and Enforcer (20). The gateway grants them from the account
//! role at login (`Gateway::bind_account_role`), lists them in the self data, and a 26/13 wears
//! one over the real job: display only, never saved, never owned, refused for players.
//! The private lab reference and content are read; every file written lives in a scratch dir.
//! The three outfit definitions are synthetic catalogue rows (no client data is copied).

#[path = "common/item_catalogue.rs"]
#[allow(dead_code)]
mod item_catalogue;

use osfr_tools::{
    account::{AccountFlags, SlotPolicy},
    account_store::{AccountRole, AccountStore},
    game_store::{self, Snapshot},
    gateway::{Gateway, GatewayOutcome},
    profiles::Profiles,
    startup_selfdata::{JobProfile, parse_self},
};
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

const REFEREE: i32 = 58;
const ENFORCER: i32 = 20;
/// Accounts 1 to 3 in the scratch store.
const PLAYER: u64 = 1;
const MODERATOR: u64 = 2;
const ADMIN: u64 = 3;
/// Synthetic definitions for the staff outfit (`data/content/staff_jobs.json` default items):
/// id, slot (Chest, Legs, Feet), model.
const OUTFIT: [(i32, i32, &str); 3] = [
    (13682, 3, "l052_chest.adr"),
    (421, 4, "l052_legs.adr"),
    (429, 5, "l052_feet.adr"),
];
const DEFAULT_TINT: i32 = 7;

fn lab() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab/references")
}

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

fn select(job: i32) -> Vec<u8> {
    let mut inner = vec![26, 0, 13, 0];
    inner.extend(job.to_le_bytes());
    wrap(&inner)
}

fn dword(bytes: &[u8], at: usize) -> i32 {
    i32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

fn said(outcome: &GatewayOutcome, text: &str) -> bool {
    outcome
        .replies
        .iter()
        .any(|reply| String::from_utf8_lossy(reply).contains(text))
}

/// The inner 38/21 activations among `replies` (each reply is a 5/0/1 tunnel).
fn activations(replies: &[Vec<u8>]) -> Vec<Vec<u8>> {
    replies
        .iter()
        .filter(|reply| reply.len() > 11 && reply[7..11] == [38, 0, 21, 0])
        .map(|reply| reply[7..].to_vec())
        .collect()
}

/// The attachment list the client gets for the staff outfit (count, then per piece model,
/// texture, tint alias, tint, effect, slot), as the catalogue rows above define it.
fn staff_outfit() -> Vec<u8> {
    let mut bytes = 3i32.to_le_bytes().to_vec();
    for (_, slot, model) in OUTFIT {
        for text in [model, "l052", ""] {
            bytes.extend((text.len() as u32).to_le_bytes());
            bytes.extend(text.as_bytes());
        }
        for value in [DEFAULT_TINT, 0, slot] {
            bytes.extend(value.to_le_bytes());
        }
    }
    bytes
}

struct Fixture {
    root: PathBuf,
    reference: PathBuf,
    content: PathBuf,
    store: PathBuf,
}

impl Fixture {
    /// The lab gameplay content plus a catalogue holding the three outfit definitions, and an
    /// account store with a player, a moderator and an admin.
    fn new(tag: &str) -> Self {
        Self::with_outfit(tag, &OUTFIT)
    }

    fn with_outfit(tag: &str, outfit: &[(i32, i32, &str)]) -> Self {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("l052-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let content = root.join("content");
        fs::create_dir_all(&content).unwrap();
        let source = lab().join("rust-opening-cooking-original-assets");
        for entry in fs::read_dir(&source).unwrap() {
            let path = entry.unwrap().path();
            if path.is_file() {
                fs::copy(&path, content.join(path.file_name().unwrap())).unwrap();
            }
        }
        let rows: Vec<Value> = outfit
            .iter()
            .map(|(id, slot, model)| {
                let mut row = item_catalogue::row(*id, id.to_le_bytes().to_vec());
                row["slot"] = json!(slot);
                row["class_id"] = json!(if *slot == 3 { 26 } else { 1 });
                row["default_tint"] = json!(DEFAULT_TINT);
                row["model"] = json!(model);
                row["texture"] = json!("l052");
                row
            })
            .collect();
        item_catalogue::write(&content, &rows);
        let store = root.join("accounts.json");
        let mut accounts = AccountStore::new();
        for (hash, role) in [
            (1, AccountRole::Player),
            (2, AccountRole::Moderator),
            (3, AccountRole::Admin),
        ] {
            let id = accounts
                .create_account(
                    [hash; 32],
                    AccountFlags::default(),
                    SlotPolicy { max_characters: 1 },
                )
                .unwrap();
            accounts.set_account_role_offline(id, role).unwrap();
        }
        accounts.save(&store).unwrap();
        Self {
            root,
            reference: lab().join("rust-gateway-reference"),
            content,
            store,
        }
    }

    fn set_role(&self, account: u64, role: AccountRole) {
        let mut accounts = AccountStore::load(&self.store).unwrap();
        accounts.set_account_role_offline(account, role).unwrap();
        accounts.save(&self.store).unwrap();
    }

    /// A login in production order: the host binds the verified owner's role, then the
    /// gateway takes the login and the world entry. Returns the world-entry replies.
    fn log_in(&self, account: u64) -> (Gateway, Vec<Vec<u8>>) {
        let initial = Gateway::reference_state(&self.reference).unwrap();
        self.log_in_from(
            account,
            Snapshot {
                character: initial,
                gameplay: None,
                pets: None,
            },
        )
    }

    /// The same login from a saved character (`game_store::load_or_create`).
    fn log_in_from(&self, account: u64, saved: Snapshot) -> (Gateway, Vec<Vec<u8>>) {
        let mut gateway = Gateway::load_gameplay(
            &self.reference,
            saved.character,
            &self.content,
            saved.gameplay,
        )
        .unwrap();
        // As `load_gateway_from` does whenever an account store is configured.
        gateway.set_role_commands(true);
        gateway.bind_account_role(&self.store, account);
        let manifest: Value =
            serde_json::from_slice(&fs::read(self.reference.join("manifest.json")).unwrap())
                .unwrap();
        let mut login = 1u16.to_le_bytes().to_vec();
        for (index, value) in ["l052", manifest["client_version"].as_str().unwrap(), ""]
            .into_iter()
            .enumerate()
        {
            login.extend((value.len() as u32).to_le_bytes());
            login.extend(value.as_bytes());
            if index == 0 {
                login.extend(1u64.to_le_bytes());
            }
        }
        let mut replies = gateway.handle(&login).unwrap().replies;
        for inner in [[13, 0], [10, 0]] {
            replies.extend(gateway.handle(&wrap(&inner)).unwrap().replies);
        }
        (gateway, replies)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// The job rows of the self data (opcode 12) among `replies`.
fn self_jobs(replies: &[Vec<u8>]) -> Vec<JobProfile> {
    let packets: Vec<_> = replies
        .iter()
        .filter(|reply| reply.len() > 9 && reply[..3] == [5, 0, 1] && reply[7..9] == [12, 0])
        .collect();
    assert_eq!(packets.len(), 1, "one self-data message at world entry");
    parse_self(packets[0]).jobs.profiles
}

fn ids(rows: &[JobProfile]) -> Vec<i32> {
    rows.iter().map(|row| row.map_key as i32).collect()
}

/// What a save or a reconnect would carry, and what other players see.
fn observable(gateway: &Gateway) -> (Value, i32, Vec<u8>) {
    let look = gateway.player_presentation(false).unwrap();
    (
        serde_json::to_value(gateway.snapshot()).unwrap(),
        look.active_profile,
        look.attachments_wire,
    )
}

#[test]
fn staff_jobs_come_from_the_login_role_and_go_with_a_demotion() {
    let fixture = Fixture::new("role");
    let (_, replies) = fixture.log_in(PLAYER);
    let player = ids(&self_jobs(&replies));
    assert_eq!(player.len(), 16, "the 16 player jobs are unchanged");
    assert!(!player.contains(&REFEREE) && !player.contains(&ENFORCER));
    for account in [MODERATOR, ADMIN] {
        let (_, replies) = fixture.log_in(account);
        let staff = ids(&self_jobs(&replies));
        assert_eq!(staff[..16], player[..], "{account}");
        assert_eq!(staff[16..], [REFEREE, ENFORCER], "{account}");
    }
    // A moderator saves (the atomic character save) while wearing the Referee job, is demoted
    // and logs in again from that save: both jobs are gone, the real job is back, and the save
    // never held a staff job or item, so there is nothing to take back.
    let (mut moderator, _) = fixture.log_in(MODERATOR);
    let real = observable(&moderator);
    moderator.handle(&select(REFEREE)).unwrap();
    let path = fixture.root.join("character.json");
    game_store::save(&path, &moderator.snapshot()).unwrap();
    fixture.set_role(MODERATOR, AccountRole::Player);
    let initial = Gateway::reference_state(&fixture.reference).unwrap();
    let saved = game_store::load_or_create(&path, initial).unwrap();
    let (demoted, replies) = fixture.log_in_from(MODERATOR, saved);
    assert_eq!(ids(&self_jobs(&replies)), player);
    assert_eq!(observable(&demoted), real);
}

#[test]
fn a_spoofed_staff_job_select_is_refused_and_changes_nothing() {
    let fixture = Fixture::new("spoof");
    let (mut player, _) = fixture.log_in(PLAYER);
    let before = observable(&player);
    for job in [REFEREE, ENFORCER] {
        let outcome = player.handle(&select(job)).unwrap();
        assert!(said(&outcome, "that class has not been unlocked"), "{job}");
        assert!(activations(&outcome.replies).is_empty() && outcome.nearby.is_empty());
        assert!(!outcome.gameplay_changed);
        assert_eq!(observable(&player), before, "{job}");
    }
    // The grant is decided at login (R, as the L048 referee flag): after a demotion, the next
    // login refuses the select like a player's.
    fixture.set_role(MODERATOR, AccountRole::Player);
    let (mut demoted, _) = fixture.log_in(MODERATOR);
    let before = observable(&demoted);
    let outcome = demoted.handle(&select(REFEREE)).unwrap();
    assert!(said(&outcome, "that class has not been unlocked"));
    assert_eq!(observable(&demoted), before);
}

#[test]
fn a_moderator_wears_a_staff_job_without_saving_owning_or_levelling_it() {
    let fixture = Fixture::new("wear");
    let (mut moderator, _) = fixture.log_in(MODERATOR);
    // Brawler, unlocked in the fixture and dressed by its starter outfit, so it can be reselected.
    let real_job = 43;
    let outcome = moderator.handle(&select(real_job)).unwrap();
    assert_eq!(dword(&activations(&outcome.replies)[0], 8), real_job);
    let (save_before, _, real_outfit) = observable(&moderator);
    for job in [REFEREE, ENFORCER] {
        let outcome = moderator.handle(&select(job)).unwrap();
        let activation = activations(&outcome.replies);
        assert_eq!(activation.len(), 1, "{job}");
        assert_eq!(dword(&activation[0], 8), job, "38/21 names the staff job");
        assert!(outcome.gameplay_changed, "the public look is refreshed");
        // Neighbours get the outfit now (35/7), later viewers through AddPc.
        let mut swap = vec![35, 0, 7, 0];
        swap.extend(1u64.to_le_bytes());
        swap.extend(staff_outfit());
        swap.extend(0i32.to_le_bytes());
        assert_eq!(outcome.nearby, [swap], "{job}");
        let (save, shown, outfit) = observable(&moderator);
        assert_eq!((shown, outfit), (job, staff_outfit()), "{job}");
        // No item, job, unlock or level is granted or saved: the save stays the same, so the
        // outfit is nothing the player owns, trades or sells.
        assert_eq!(save, save_before, "{job}");
    }
    // Taking the same real job back is an ordinary select; every real switch ends the staff job.
    let outcome = moderator.handle(&select(real_job)).unwrap();
    assert_eq!(dword(&activations(&outcome.replies)[0], 8), real_job);
    assert_eq!(
        outcome.nearby.len(),
        1,
        "neighbours see the real outfit again"
    );
    assert!(outcome.gameplay_changed);
    let (save, shown, outfit) = observable(&moderator);
    assert_eq!((shown, outfit), (real_job, real_outfit));
    assert_eq!(save, save_before);
}

#[test]
fn a_staff_job_the_catalogue_cannot_dress_is_refused_and_changes_nothing() {
    // A piece the catalogue lacks, and an outfit with no Legs piece (the client's
    // ProcessProfileSwap rejects a group without Chest and Legs).
    let missing: &[_] = &[OUTFIT[0], OUTFIT[2]];
    let no_legs: &[_] = &[OUTFIT[0], (421, 6, "l052_hands.adr"), OUTFIT[2]];
    for (tag, outfit) in [("missing", missing), ("no-legs", no_legs)] {
        let fixture = Fixture::with_outfit(tag, outfit);
        let (mut moderator, _) = fixture.log_in(MODERATOR);
        let before = observable(&moderator);
        let outcome = moderator.handle(&select(REFEREE)).unwrap();
        assert!(said(&outcome, "that job's outfit is unavailable"), "{tag}");
        assert!(activations(&outcome.replies).is_empty() && outcome.nearby.is_empty());
        assert!(!outcome.gameplay_changed);
        assert_eq!(observable(&moderator), before, "{tag}");
    }
}

#[test]
fn experience_paid_while_a_staff_job_is_worn_goes_to_the_real_job() {
    let fixture = Fixture::new("xp");
    let (mut admin, _) = fixture.log_in(ADMIN);
    let (_, real_job, _) = observable(&admin);
    admin.handle(&select(REFEREE)).unwrap();
    // `realm level` is an admin payout through the job XP path (`Profiles::active`).
    let outcome = admin
        .handle(&wrap(&osfr_tools::chat::local_request("realm level 3")))
        .unwrap();
    assert!(said(&outcome, &format!("Job {real_job} is now level 3.")));
    let (save, shown, _) = observable(&admin);
    assert_eq!(shown, REFEREE, "the staff job stays worn");
    let profiles = &save["gameplay"]["profiles"];
    assert_eq!(profiles["active"], json!(real_job));
    assert_eq!(profiles["experience"], json!({real_job.to_string(): 200}));
    let unlocked = profiles["unlocked"].as_array().unwrap();
    assert!(!unlocked.contains(&json!(REFEREE)) && !unlocked.contains(&json!(ENFORCER)));
}

#[test]
fn the_staff_record_has_the_captured_job_layout_in_38_21_and_the_self_data() {
    let fixture = Fixture::new("wire");
    let (mut moderator, replies) = fixture.log_in(MODERATOR);
    let rows = self_jobs(&replies);
    for job in [REFEREE, ENFORCER] {
        let activation = activations(&moderator.handle(&select(job)).unwrap().replies).remove(0);
        let length = dword(&activation, 4) as usize;
        let record = &activation[8..8 + length];
        let name = if job == REFEREE { 23268 } else { 2301 };
        let header: Vec<i32> = (0..8).map(|index| dword(record, index * 4)).collect();
        assert_eq!(header, [job, name, 2302, 9, 258, 4, 19, 21]);
        assert_eq!(record[32], 0, "members only");
        assert_eq!(dword(record, 33), 0, "(R) tenth dword");
        let classes = dword(record, 37) as usize;
        assert_eq!(classes, 23);
        let keys: Vec<i32> = (0..classes)
            .map(|row| dword(record, 41 + row * 12))
            .collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted, "class map in key order");
        assert!((0..classes).all(|row| dword(record, 45 + row * 12) == keys[row]));
        let at = 41 + classes * 12;
        assert_eq!(record[at..at + 14], [0; 14], "(R) five unnamed fields");
        let progression: Vec<i32> = (0..5)
            .map(|index| dword(record, at + 14 + index * 4))
            .collect();
        assert_eq!(progression, [20, 0, 0, 0, 0], "rank 20");
        let equipment = at + 34;
        let tail: Vec<i32> = (0..12)
            .map(|index| dword(record, equipment + index * 4))
            .collect();
        assert_eq!(
            tail,
            [0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            "no equipment, empty abilities"
        );
        assert_eq!(record.len(), equipment + 48);
        let rest = &activation[8 + length..];
        assert_eq!(rest[..rest.len() - 12], staff_outfit()[..]);
        let trailer: Vec<i32> = (0..3)
            .map(|index| dword(rest, rest.len() - 12 + index * 4))
            .collect();
        assert_eq!(trailer, [3001, 4005, 0]);
        // The self-data row is the same record, read back by the client-layout parser.
        let row = rows.iter().find(|row| row.map_key == job as u32).unwrap();
        assert_eq!(
            row.opaque_before_equipment_without_key,
            record[4..equipment]
        );
        assert_eq!(row.after_equipment_value, 0);
        assert_eq!(row.abilities.len(), 8);
        assert!(row.abilities.iter().all(|ability| ability.kind == 0));
        assert!(row.trailing_records.is_empty());
    }
}

#[test]
fn staff_jobs_stay_out_of_trainers_and_survive_a_reward_reload() {
    let fixture = Fixture::new("isolation");
    let mut profiles = Profiles::load(&fixture.content, None).unwrap();
    // A trainer row can only name content jobs, never a staff job.
    let trainers = fixture.root.join("trainers");
    fs::create_dir_all(&trainers).unwrap();
    fs::write(
        trainers.join("job_trainers.json"),
        json!({"schema_version": 1, "trainers": [{
            "npc": 1, "profile": REFEREE, "job": "Referee", "label_string_id": 1,
            "source": "synthetic L052 row"
        }]})
        .to_string(),
    )
    .unwrap();
    profiles.set_staff_jobs(true);
    assert!(profiles.load_trainers(&trainers).is_err());
    // A reward adopts a new progression snapshot; the session's staff jobs stay.
    let state = profiles.state().clone();
    let current = Some(profiles);
    let adopted = Profiles::adopt(&current, &fixture.content, state).unwrap();
    assert!(adopted.is_staff_job(REFEREE) && adopted.is_staff_job(ENFORCER));
    assert!(
        !Profiles::adopt(&None, &fixture.content, adopted.state().clone())
            .unwrap()
            .is_staff_job(REFEREE)
    );
}
