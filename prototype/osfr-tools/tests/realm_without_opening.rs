//! rc-0136 (enemy tuning): a realm that runs without the opening content (the shipped
//! `data/content` alone) took every chat line through `Gameplay::handle`, which knows only the
//! ordinary commands, so the developer routes answered nothing: `realm god`, `realm sim`,
//! `realm here` and the rest went silent through a gateway login. They now run on both paths
//! (`gameplay/developer_chat.rs`). Needs the private fixture like the other gateway suites.
#![allow(dead_code)]
#[path = "common/field_battle_fixture.rs"]
mod field_battle_fixture;
use field_battle_fixture::{Fixture, chat_text, content_dir, send};
use osfr_tools::chat;

#[test]
fn developer_commands_answer_through_a_login_with_and_without_the_opening() {
    if !field_battle_fixture::available() {
        return;
    }
    for shipped in [false, true] {
        let mut fixture = Fixture::new();
        if shipped {
            fixture.content = content_dir();
        }
        let mut gateway = fixture.gateway([-793.0, -2.0, 1016.0], 1);
        for (n, command) in ["realm god", "realm here", "realm sim 96"].iter().enumerate() {
            let out = send(&mut gateway, &chat::local_request(command), 1_000 + n as u64 * 100);
            let text = chat_text(&out);
            assert!(
                !text.trim().is_empty(),
                "{command} answered nothing (shipped content alone: {shipped})"
            );
        }
        // The engine-less realm says why it cannot simulate, instead of nothing.
        let sim = send(&mut gateway, &chat::local_request("realm sim 96 1"), 2_000);
        let text = chat_text(&sim);
        assert!(
            text.contains("Battle 96") || text.contains("No encounter engine"),
            "{text}"
        );
    }
}
