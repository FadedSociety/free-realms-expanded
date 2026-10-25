//! The complete greeting set of the two regions no other greeting suite owns:
//! `shrouded_glade.json` and `wugachug.json`.
//!
//! The T380 changed-content guard (`npc_greetings_merry_vale_stillwater`) checks
//! a foreign NPC file on a commit only when the commit also touches one of its two
//! owned files. A greeting committed to one of these two files on its own would
//! therefore have no greeting suite; this pin fails instead.
//!
//! The field is real client behaviour: the 26/3 payload reader `0x00a9ef10`
//! stores the first payload dword at object `+16`, and display `0x009f5ae0`
//! passes `*(a2 + 16)` to the `Global.Text` lookup `0x0097c1e0` before
//! `:SetDialogText`. Both rows' 26/3 bodies are already driven through the
//! production dispatch by
//! `npc_greetings_enabled_quests::clicking_a_bound_quest_npc_returns_its_greeting_body`;
//! this suite pins the content, not a second packet path.
//!
//! Adding a greeting here is allowed, but it must bring its speaker proof (the
//! T378 rules) and update this pin in the same change.

#[path = "common/npc_greetings.rs"]
mod npc_greetings;

use npc_greetings::{assert_drawable, greeting_bindings, npc_record};
use std::collections::BTreeMap;

/// Every `(file, row)` -> greeting in the two files. Camellia and Postchug are
/// the only recovered bindings; every other drawable row stays blank.
const PINNED: [(&str, u64, u32); 2] = [
    ("shrouded_glade.json", 9_500_000_650, 440_465),
    ("wugachug.json", 9_000_000_172, 130_231),
];

#[test]
fn no_other_row_in_the_two_regions_gets_a_greeting() {
    let wanted: BTreeMap<(String, u64), u32> = PINNED
        .iter()
        .map(|(file, npc_id, greeting)| ((file.to_string(), *npc_id), *greeting))
        .collect();
    let actual: BTreeMap<(String, u64), u32> = greeting_bindings()
        .into_iter()
        .filter(|((file, _), _)| PINNED.iter().any(|(pinned, _, _)| *pinned == file.as_str()))
        .collect();
    assert_eq!(
        actual, wanted,
        "a greeting was added, removed or moved in shrouded_glade.json/wugachug.json; \
         add the speaker proof and update this pin"
    );

    for (file, npc_id, greeting) in PINNED {
        let record = npc_record(file, npc_id);
        assert_eq!(
            record["greeting_string_id"].as_u64(),
            Some(u64::from(greeting)),
            "{file}: row {npc_id} greeting"
        );
        assert_drawable(&record, npc_id);
    }
}
