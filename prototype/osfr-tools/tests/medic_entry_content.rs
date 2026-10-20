//! F236/T197 focused content target for the Medic entry chapter.
//!
//! The production packet flow is in `medic_entry_chain.rs`; this target runs the
//! same authored-content check against the split quest and NPC JSON, exactly as
//! `archer_entry_content.rs` does for the Archer chapter.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

#[path = "support/medic_entry.rs"]
mod medic_entry;

#[test]
fn enabled_medic_chapter_records_are_complete_and_others_stay_disabled() {
    medic_entry::assert_enabled_medic_chapter_records();
}
