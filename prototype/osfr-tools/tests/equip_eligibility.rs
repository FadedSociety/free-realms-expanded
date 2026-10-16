//! W138: equip eligibility (42/2, 42/3, 42/8) for players with known facts.
//! Each refusal has a canary: the same request succeeds once the one missing
//! fact is supplied, so the refusal is the rule and not an unrelated failure.
use osfr_tools::{
    chat,
    inventory::{
        EquipContext, EquippedItem, Inventory, InventoryItem, InventoryState, RentalState,
    },
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

const PLAIN: i32 = 200;
const MEMBERS: i32 = 201;
const GIRLS: i32 = 202;
const RANK_5: i32 = 203;
const STAR_RUNE: i32 = 204;
const RENTAL: i32 = 205;
const BOYS: i32 = 206;
const GARBAGE: i32 = 207;

/// A 147-byte definition row with the recovered layout: 11 dwords and two
/// flags, empty model and texture strings, gender usage, type, category,
/// members flag, then minimum profile rank at +26 after the strings.
fn row(id: i32, slot: i32, gender: i32, members: bool, min_rank: i32) -> Vec<u8> {
    let mut bytes = vec![0u8; 147];
    bytes[..4].copy_from_slice(&id.to_le_bytes());
    bytes[32..36].copy_from_slice(&1i32.to_le_bytes()); // class
    bytes[40..44].copy_from_slice(&slot.to_le_bytes());
    bytes[54..58].copy_from_slice(&gender.to_le_bytes());
    bytes[66] = u8::from(members);
    bytes[80..84].copy_from_slice(&min_rank.to_le_bytes());
    bytes
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn item(id: i32, definition_id: i32) -> InventoryItem {
    InventoryItem {
        id,
        definition_id,
        tint: 0,
        count: 1,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: i64::from(i32::MAX),
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    }
}
fn serialize_item(item: &InventoryItem) -> Vec<u8> {
    let mut bytes = Vec::new();
    for value in [
        item.definition_id,
        item.tint,
        item.id,
        item.count as i32,
        item.consumed_count as i32,
        i32::try_from(item.last_cast_time).unwrap(),
    ] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.push(u8::from(item.rental.is_some()));
    if let Some(rental) = &item.rental {
        bytes.extend(rental.expiration.to_le_bytes());
        bytes.extend(rental.upsells.to_le_bytes());
        bytes.extend(rental.bundle_id.to_le_bytes());
    }
    bytes.extend(item.ability_count.to_le_bytes());
    bytes.push(u8::from(item.activate_enabled));
    bytes
}
fn request(opcode: u16, fields: &[i32]) -> Vec<u8> {
    let mut bytes = [42u16.to_le_bytes(), opcode.to_le_bytes()].concat();
    for field in fields {
        bytes.extend(field.to_le_bytes());
    }
    bytes
}
fn equip_by_id(id: i32, profile: i32, slot: i32) -> Vec<u8> {
    request(3, &[id, profile, slot])
}
fn equip_by_record(definition: i32, profile: i32, slot: i32) -> Vec<u8> {
    request(8, &[definition, 0, profile, slot])
}

struct Fixture {
    path: PathBuf,
}
impl Fixture {
    /// Profiles 1 (slots 3 and 14), 2 (slot 3) and 3 (slot 3). Profile 3
    /// wears item 10 so a refused unequip has something to keep.
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-equip-eligibility-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        let mut rental = item(15, RENTAL);
        rental.rental = Some(RentalState {
            expiration: 1000,
            upsells: 0,
            bundle_id: 0,
        });
        let state = InventoryState {
            next_instance_id: None,
            coins: 0,
            items: vec![
                item(10, PLAIN),
                item(11, MEMBERS),
                item(12, GIRLS),
                item(13, RANK_5),
                item(14, STAR_RUNE),
                rental,
                item(16, BOYS),
                item(17, GARBAGE),
            ],
            equipment: [(1, 3, 0), (1, 14, 0), (2, 3, 0), (3, 3, 10)]
                .map(|(profile_id, slot, item_id)| EquippedItem {
                    profile_id,
                    slot,
                    item_id,
                    category: slot,
                })
                .to_vec(),
        };
        // Spans follow the default state: coins, three equipment blocks, items.
        let mut offset = 13;
        let mut spans = Vec::new();
        let mut span = |kind: &str, profile_id: i32, bytes: Vec<u8>| {
            spans.push(json!({"kind":kind,"offset":offset,"length":bytes.len(),"profile_id":profile_id,"sha256":digest(&bytes)}));
            offset += bytes.len();
        };
        span("coins", 0, 0i32.to_le_bytes().to_vec());
        for profile in 1..=3 {
            let rows: Vec<_> = state
                .equipment
                .iter()
                .filter(|e| e.profile_id == profile)
                .collect();
            let mut bytes = (rows.len() as i32).to_le_bytes().to_vec();
            for e in rows {
                for value in [e.slot, e.item_id, e.category] {
                    bytes.extend(value.to_le_bytes());
                }
            }
            span("equipment", profile, bytes);
        }
        let mut bytes = (state.items.len() as i32).to_le_bytes().to_vec();
        for item in &state.items {
            bytes.extend(serialize_item(item));
        }
        span("items", 0, bytes);

        let mut garbage = row(GARBAGE, 3, 2, true, 99);
        garbage[46..50].copy_from_slice(&i32::MAX.to_le_bytes()); // absurd string length
        let rows = [
            row(PLAIN, 3, 0, false, 1),
            row(MEMBERS, 3, 0, true, 1),
            row(GIRLS, 3, 2, false, 1),
            row(RANK_5, 3, 0, false, 5),
            row(STAR_RUNE, 14, 0, false, 1),
            row(RENTAL, 3, 0, false, 1),
            row(BOYS, 3, 1, false, 1),
            garbage,
        ];
        let definitions: Vec<_> = rows.iter().map(|bytes| {
            let id = i32::from_le_bytes(bytes[..4].try_into().unwrap());
            let slot = i32::from_le_bytes(bytes[40..44].try_into().unwrap());
            let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
            json!({"id":id,"slot":slot,"class_id":1,"default_tint":0,"max_stack":1,"model":"m.adr","texture":"t","tint_alias":"a","effect":0,"wield_type":0,"serialized_hex":hex,"sha256":digest(bytes)})
        }).collect();
        let profiles: Vec<_> = [(1, json!([3, 14])), (2, json!([3])), (3, json!([3]))]
            .into_iter()
            .map(|(id, slots)| json!({"id":id,"allowed_classes":[1],"slots":slots}))
            .collect();
        let content = json!({"schema_version":1,"player_guid":1,"default_state":state,"profiles":profiles,"spans":spans,"definitions":definitions});
        fs::write(
            path.join("inventory.json"),
            serde_json::to_vec(&content).unwrap(),
        )
        .unwrap();
        Self { path }
    }
    fn inventory(&self) -> Inventory {
        Inventory::load(&self.path, None).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.path.join("inventory.json"));
        let _ = fs::remove_dir(&self.path);
    }
}

/// A boy, not a member, rank 3 on profile 1 and rank 12 on profile 2;
/// profile 3 is not unlocked. The rental ran out at 1000.
fn player() -> EquipContext {
    EquipContext {
        gender: 1,
        member: false,
        profile_ranks: [(1, 3), (2, 12)].into(),
        rental_now: Some(2000),
    }
}

fn worn(inventory: &Inventory, profile: i32, slot: i32) -> i32 {
    inventory
        .state()
        .equipment
        .iter()
        .find(|e| e.profile_id == profile && e.slot == slot)
        .unwrap()
        .item_id
}

#[test]
fn eligible_gear_still_equips_on_active_and_inactive_profiles() {
    let f = Fixture::new();
    let mut inventory = f.inventory();
    let mut player = player();
    player.member = true;
    // Rank is the target profile's own: rank 5 gear fits profile 2 (rank 12).
    for (id, definition, profile, slot) in [
        (10, PLAIN, 1, 3),
        (11, MEMBERS, 2, 3),
        (13, RANK_5, 2, 3),
        (16, BOYS, 1, 3),
    ] {
        let outcome = inventory
            .handle_with(&equip_by_id(id, profile, slot), &player)
            .unwrap();
        assert!(outcome.changed, "item {id}");
        assert_eq!(&outcome.replies[0][..4], &[38, 0, 5, 0]);
        assert_eq!(outcome.replies[1][..4], [35, 0, 6, 0]);
        assert_eq!(worn(&inventory, profile, slot), id);
        // The same item by record reaches the same decision.
        let again = inventory
            .handle_with(&equip_by_record(definition, profile, slot), &player)
            .unwrap();
        assert!(!again.changed);
        assert_eq!(&again.replies[0][..4], &[38, 0, 5, 0]);
    }
    // An unexpired rental and a slot rank that is met are allowed.
    player.rental_now = Some(500);
    assert!(
        inventory
            .handle_with(&equip_by_id(15, 2, 3), &player)
            .unwrap()
            .changed
    );
    player.profile_ranks.insert(1, 10);
    assert!(
        inventory
            .handle_with(&equip_by_id(14, 1, 14), &player)
            .unwrap()
            .changed
    );
}

#[test]
fn each_eligibility_refusal_changes_nothing_explains_itself_and_has_a_canary() {
    type Fix = fn(&mut EquipContext);
    let cases: [(&str, i32, i32, i32, &str, Fix); 6] = [
        (
            "members only",
            11,
            MEMBERS,
            1,
            "This item is for members only.",
            |p| p.member = true,
        ),
        (
            "girls only",
            12,
            GIRLS,
            1,
            "This item is only for girls.",
            |p| p.gender = 2,
        ),
        (
            "item rank",
            13,
            RANK_5,
            1,
            "This item needs class rank 5.",
            |p| {
                p.profile_ranks.insert(1, 5);
            },
        ),
        (
            "expired rental",
            15,
            RENTAL,
            1,
            "This rental has expired.",
            |p| {
                p.rental_now = Some(999);
            },
        ),
        (
            "locked profile",
            16,
            BOYS,
            3,
            "That class is not unlocked.",
            |p| {
                p.profile_ranks.insert(3, 1);
            },
        ),
        (
            "rental expiry skipped without a clock",
            15,
            RENTAL,
            1,
            "This rental has expired.",
            |p| {
                p.rental_now = None;
            },
        ),
    ];
    for (name, id, definition, profile, message, fix) in cases {
        for request in [
            equip_by_id(id, profile, 3),
            equip_by_record(definition, profile, 3),
        ] {
            let f = Fixture::new();
            let mut inventory = f.inventory();
            let before = inventory.state().clone();
            let mut player = player();
            let outcome = inventory.handle_with(&request, &player).unwrap();
            assert!(!outcome.changed, "{name}");
            assert_eq!(outcome.replies, vec![chat::system(message)], "{name}");
            assert_eq!(inventory.state(), &before, "{name}");
            // Canary: supply the one missing fact and the request succeeds.
            fix(&mut player);
            let ok = inventory.handle_with(&request, &player).unwrap();
            assert!(ok.changed, "{name} canary");
            assert_eq!(worn(&inventory, profile, 3), id, "{name} canary");
        }
    }
    // Boys-only gear for a girl names the other side too.
    let f = Fixture::new();
    let mut inventory = f.inventory();
    let mut girl = player();
    girl.gender = 2;
    let outcome = inventory
        .handle_with(&equip_by_id(16, 1, 3), &girl)
        .unwrap();
    assert_eq!(
        outcome.replies,
        vec![chat::system("This item is only for boys.")]
    );
}

#[test]
fn unknown_gender_does_not_refuse_gendered_gear() {
    // Gameplay reports 0 until the character identity is applied.
    let f = Fixture::new();
    let mut inventory = f.inventory();
    let mut unknown = player();
    unknown.gender = 0;
    let outcome = inventory
        .handle_with(&equip_by_id(12, 1, 3), &unknown)
        .unwrap();
    assert!(outcome.changed);
    assert_eq!(worn(&inventory, 1, 3), 12);
}

#[test]
fn star_rune_slot_needs_rank_ten_even_for_unrestricted_gear() {
    let f = Fixture::new();
    let mut inventory = f.inventory();
    let mut player = player();
    let before = inventory.state().clone();
    let refused = inventory
        .handle_with(&equip_by_id(14, 1, 14), &player)
        .unwrap();
    assert_eq!(
        refused.replies,
        vec![chat::system("This item needs class rank 10.")]
    );
    assert!(!refused.changed);
    assert_eq!(inventory.state(), &before);
    player.profile_ranks.insert(1, 10);
    assert!(
        inventory
            .handle_with(&equip_by_id(14, 1, 14), &player)
            .unwrap()
            .changed
    );
}

#[test]
fn unequip_needs_an_unlocked_profile_and_keeps_inactive_outfits() {
    let f = Fixture::new();
    let mut inventory = f.inventory();
    let mut player = player();
    let before = inventory.state().clone();
    let refused = inventory
        .handle_with(&request(2, &[3, 3]), &player)
        .unwrap();
    assert!(!refused.changed);
    assert_eq!(
        refused.replies,
        vec![chat::system("That class is not unlocked.")]
    );
    assert_eq!(inventory.state(), &before);
    // Canary: unlocked, the same request empties the slot with 38/6.
    player.profile_ranks.insert(3, 1);
    let ok = inventory
        .handle_with(&request(2, &[3, 3]), &player)
        .unwrap();
    assert!(ok.changed);
    assert_eq!(ok.replies, vec![vec![38, 0, 6, 0, 3, 0, 0, 0, 3, 0, 0, 0]]);
    assert_eq!(worn(&inventory, 3, 3), 0);
    // Required slots are not banned: the original server rule is unknown.
    assert!(inventory.handle_with(&request(2, &[3, 2]), &player).is_ok());
}

#[test]
fn forged_requests_still_fail_and_leave_state_unchanged() {
    let f = Fixture::new();
    let mut inventory = f.inventory();
    let mut player = player();
    player.member = true;
    let before = inventory.state().clone();
    for forged in [
        equip_by_id(999, 1, 3),     // not owned
        equip_by_id(10, 99, 3),     // unknown profile
        equip_by_id(10, 1, 14),     // slot differs from the definition
        equip_by_id(14, 2, 14),     // slot is not on this profile
        equip_by_record(999, 1, 3), // no such record
        request(2, &[9, 1]),        // unknown equipment slot
        request(3, &[10, 1]),       // truncated
    ] {
        assert!(inventory.handle_with(&forged, &player).is_err());
        assert_eq!(inventory.state(), &before);
    }
}

#[test]
fn malformed_definition_rows_do_not_panic_or_widen_other_items() {
    let f = Fixture::new();
    let mut inventory = f.inventory();
    // The garbage row's string length overruns its bytes: no requirement is
    // read from it (fail open, operator content only), and nothing panics.
    assert!(
        inventory
            .handle_with(&equip_by_id(17, 1, 3), &player())
            .unwrap()
            .changed
    );
    // Its neighbours keep their own restrictions.
    assert!(
        !inventory
            .handle_with(&equip_by_id(11, 1, 3), &player())
            .unwrap()
            .changed
    );
}

#[test]
fn permissive_handle_is_unchanged_for_callers_without_player_facts() {
    let f = Fixture::new();
    let mut inventory = f.inventory();
    assert!(inventory.handle(&equip_by_id(11, 1, 3)).unwrap().changed);
    assert!(inventory.handle(&equip_by_id(12, 2, 3)).unwrap().changed);
}
