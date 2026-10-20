//! T194/F217: focused content target for the Archer entry chapter. The assertions live
//! in the shared support module so the protected chain target and this target run the
//! same checks without a second copy.

#[path = "support/archer_entry.rs"]
mod archer_entry;

#[test]
fn enabled_archer_chapter_records_are_complete_and_others_stay_disabled() {
    archer_entry::assert_enabled_archer_chapter_records();
}
