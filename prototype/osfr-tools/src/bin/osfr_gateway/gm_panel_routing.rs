//! L072: the referee panel's `!gm` lines, answered here because only the gateway sees every
//! player. Admin only (unknown realm commands are admin-only in `can_run_realm_command`); a
//! panel command is never relayed as chat, whoever types it. The panel's other commands are
//! realm commands by then (`gm_panel::alias`) and run in the session's gameplay.
use super::*;
use osfr_tools::{
    account_store::can_run_realm_command,
    chat,
    gm_panel::{self, ListKind, Request},
};

/// `Some` (possibly empty) when `inner` is a panel command; `None` leaves it to ordinary chat.
pub(super) fn route(session: &Session, inner: &[u8], world: &SharedWorld) -> Option<Vec<Delivery>> {
    let text = social_chat::parse_client_chat(inner).ok()?.message;
    if gm_panel::alias(&text).is_some() {
        return Some(Vec::new());
    }
    let request = gm_panel::parse(&text)?;
    if !can_run_realm_command(session.gateway.role(), "gm") {
        return Some(Vec::new());
    }
    let lines = match request {
        Request::Open => vec![gm_panel::line(&["open"])],
        Request::List {
            kind: ListKind::Players,
            page,
            search,
        } => gm_panel::list(ListKind::Players, page, &search, &players(world)),
        Request::List {
            kind: ListKind::Items,
            page,
            search,
        } => {
            let rows = gm_panel::item_rows(
                session.gateway.item_catalogue(),
                osfr_tools::world_text::locale_text,
            );
            gm_panel::list(ListKind::Items, page, &search, &rows)
        }
        // (R) NPCs and houses need their name catalogs (a later slice); empty until then.
        Request::List { kind, page, search } => gm_panel::list(kind, page, &search, &[]),
        Request::Other => vec!["The referee panel cannot do that here yet.".to_owned()],
    };
    Some(
        lines
            .iter()
            .map(|line| Delivery {
                recipient: session.id,
                bytes: delivery::wrap_application(&chat::system(line)),
            })
            .collect(),
    )
}

/// Every player in the world: guid, name, and where they are.
fn players(world: &SharedWorld) -> Vec<Vec<String>> {
    world
        .active_sessions
        .iter()
        .filter_map(|(guid, id)| {
            let name = &world.identities.get(guid)?.name;
            let zone = world
                .players
                .get(*id)
                .map(|player| format!("zone {}", player.zone.zone_id))
                .unwrap_or_default();
            Some(vec![
                guid.to_string(),
                format!("{} {}", name.first, name.last).trim().to_owned(),
                zone,
            ])
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::tests::test_support::{self, Scene, Scratch};
    use super::*;
    use osfr_tools::account_store::AccountRole;

    /// The system lines a session got back.
    fn system_lines(replies: &[Vec<u8>]) -> Vec<String> {
        replies
            .iter()
            .filter(|message| message.get(7..11) == Some(&[15, 0, 3, 0]))
            .map(|message| {
                let length = u32::from_le_bytes(message[11..15].try_into().unwrap()) as usize;
                String::from_utf8(message[15..15 + length].to_vec()).unwrap()
            })
            .collect()
    }

    /// L072: an admin's `!gm` opens the panel and `!gm players` lists who is on; a player gets
    /// nothing, and no panel command is relayed as chat to anyone.
    #[test]
    fn gm_lines_answer_admins_only_and_never_reach_chat() {
        if !test_support::bin_test_target("gm") {
            return;
        }
        let scratch = Scratch::new("l072-gm");
        let options = test_support::options(scratch.state());
        if !test_support::fixtures_present(&options) {
            return;
        }
        let mut scene = Scene::new(&options, 72);
        test_support::enter_world(&mut scene, &options);
        let _ = scene.drain(300);
        let guid = scene.session.character_guid;
        scene.world.active_sessions.insert(guid, scene.session.id);
        scene.world.identities.insert(
            guid,
            SocialIdentity {
                account_id: guid,
                character_guid: guid,
                name: SocialName::new("Referee", "Rae").unwrap(),
            },
        );
        let typed = |text: &str| osfr_tools::chat::local_request(text);

        scene.session.gateway.set_role(AccountRole::Player);
        // A relayed line would come back to its sender as chat carrying the typed text.
        let relayed = |replies: &[Vec<u8>], text: &str| {
            replies
                .iter()
                .any(|reply| reply.windows(text.len()).any(|w| w == text.as_bytes()))
        };
        for text in ["!gm", "!gm players 1", "!godmode on"] {
            let replies = scene.exchange(&options, &typed(text), 400);
            let lines = system_lines(&replies);
            assert!(
                lines.iter().all(|line| !line.contains(gm_panel::TOKEN)),
                "{text}: {lines:?}"
            );
            assert!(!relayed(&replies, text), "{text} relayed as chat");
        }

        scene.session.gateway.set_role(AccountRole::Admin);
        scene.session.gateway.set_dev_tools(true);
        let replies = scene.exchange(&options, &typed("!gm"), 500);
        assert!(!relayed(&replies, "!gm"));
        let open = system_lines(&replies);
        assert!(
            open.contains(&format!("{}|open", gm_panel::TOKEN)),
            "{open:?}"
        );
        let god = system_lines(&scene.exchange(&options, &typed("!godmode on"), 550));
        assert!(
            god.iter().any(|line| line.starts_with("God mode")),
            "{god:?}"
        );
        let listed = system_lines(&scene.exchange(&options, &typed("!gm players 1 rae"), 600));
        let token = gm_panel::TOKEN;
        let items = system_lines(&scene.exchange(&options, &typed("!gm items 1"), 650));
        let header = items[0].split('|').collect::<Vec<_>>();
        assert_eq!(&header[1..4], ["items", "", "1"], "{header:?}");
        let total: usize = header[5].parse().unwrap();
        assert!(total > 0, "the startup catalogue has items");
        assert_eq!(
            items.len(),
            2 + total.min(gm_panel::PAGE),
            "header, one page, end"
        );
        assert!(items[1].starts_with(&format!("{}|item|", gm_panel::TOKEN)));
        assert_eq!(
            listed,
            [
                format!("{token}|players|rae|1|1|1"),
                format!(
                    "{token}|player|{guid}|Referee Rae|zone {}",
                    scene
                        .world
                        .players
                        .get(scene.session.id)
                        .unwrap()
                        .zone
                        .zone_id
                ),
                format!("{token}|end"),
            ]
        );
    }
}
