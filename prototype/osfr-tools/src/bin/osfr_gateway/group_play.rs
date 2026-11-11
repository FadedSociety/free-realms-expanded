//! PB2 group play on the shared gateway, once per server tick (from `group_arena::tick`):
//! 1. every shared field learns who is grouped in it and where, so a field kill credits the
//!    group members in reach (`SharedFieldState::set_parties`);
//! 2. each player's active job reaches their group's rows (40/9, `SocialService::set_member_job`);
//! 3. the shared quest credits a session earned (`Gameplay::take_group_credits`) go to every
//!    eligible member: same group, zone and instance, and the whole instance in a battle or
//!    dungeon or `group_play.json`'s reach in the open world. Each member's own quests count it
//!    (`receive_group_credit`), on a staged copy saved before it is installed, as `heal_group` is.
use super::*;
use osfr_tools::{group_play::GroupPlay, quest_engine::QuestEvent, world::InstanceKind};
use std::sync::{Arc, Mutex, OnceLock};

pub(super) fn tick(server: &mut Server, now_ms: u64) {
    let rules = rules(server);
    publish_parties(server, &rules);
    sync_jobs(server, &rules);
    share_credits(server, &rules, now_ms);
}

/// The content directory's `group_play.json`, read once per directory. A bad file is reported
/// once and the shipped values apply, so a typo never stops the world loop.
fn rules(server: &Server) -> Arc<GroupPlay> {
    static CACHE: OnceLock<Mutex<BTreeMap<PathBuf, Arc<GroupPlay>>>> = OnceLock::new();
    let Some(dir) = server.options.content.clone() else {
        return Arc::new(GroupPlay::shipped());
    };
    let cache = CACHE.get_or_init(Default::default);
    let mut cache = cache
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let rules = cache.entry(dir).or_insert_with_key(|dir| {
        Arc::new(GroupPlay::load(dir).unwrap_or_else(|error| {
            event(json!({"event":"group_play_refused","reason":error.to_string()}));
            GroupPlay::shipped()
        }))
    });
    Arc::clone(rules)
}

fn publish_parties(server: &mut Server, rules: &GroupPlay) {
    let mut by_instance: BTreeMap<i32, BTreeMap<u64, (u32, [f32; 3])>> = BTreeMap::new();
    for session in server.sessions.values() {
        let guid = session.character_guid;
        let group = server.world.social.group_of(guid);
        if let (Some(group), Some(at)) = (group, server.membership.player_location(guid)) {
            let spot = (group, session.gateway.state().position);
            by_instance
                .entry(at.instance_id)
                .or_default()
                .insert(guid, spot);
        }
    }
    for instance in server.membership.field_instances() {
        let parties = by_instance.remove(&instance).unwrap_or_default();
        if let Some(field) = server.membership.field_mut(instance) {
            field.set_parties(parties, rules.credit_range_m);
        }
    }
}

fn sync_jobs(server: &mut Server, rules: &GroupPlay) {
    let mut sends = Vec::new();
    for session in server.sessions.values() {
        let guid = session.character_guid;
        let Some(look) = server.world.public_looks.get(&guid) else {
            continue;
        };
        let active = look.active_profile;
        let rank = (look.profiles.iter().find(|(id, _)| *id == active)).map_or(1, |(_, r)| *r);
        let messages = server
            .world
            .social
            .set_member_job(guid, rules.member_job(active, rank));
        sends.extend(messages.unwrap_or_default());
    }
    for message in sends {
        if let Some(key) = group_arena::key_of(server, message.recipient_guid) {
            if let Some(session) = server.sessions.get_mut(&key) {
                let _ = session.queue_shared_message(&wrap_application(&message.bytes));
            }
        }
    }
}

fn share_credits(server: &mut Server, rules: &GroupPlay, now_ms: u64) {
    let keys: Vec<SessionKey> = server.sessions.keys().copied().collect();
    for key in keys {
        let Some(session) = server.sessions.get_mut(&key) else {
            continue;
        };
        let credits =
            (session.gateway.gameplay_mut()).map_or_else(Vec::new, |g| g.take_group_credits());
        if credits.is_empty() {
            continue;
        }
        let (source, from) = (session.character_guid, session.gateway.state().clone());
        for member in eligible(server, rules, source, &from) {
            for credit in &credits {
                give(server, member, credit.clone(), now_ms);
            }
        }
    }
}

/// The members a credit `source` earned at `from` reaches, each once.
pub(crate) fn eligible(
    server: &Server,
    rules: &GroupPlay,
    source: u64,
    from: &CharacterState,
) -> Vec<SessionKey> {
    let social = &server.world.social;
    let (Some(group), Some(here)) = (
        social.group_of(source),
        server
            .membership
            .player_location(source)
            .map(|at| at.instance_id),
    ) else {
        return Vec::new();
    };
    let kind = server
        .membership
        .instance(here)
        .map(|instance| &instance.kind);
    let whole = matches!(
        kind,
        Some(InstanceKind::Dungeon { .. } | InstanceKind::Encounter { .. })
    );
    let members = social.group_members(group).unwrap_or_default();
    (members.iter().copied().filter(|guid| *guid != source))
        .filter_map(|guid| {
            let key = group_arena::key_of(server, guid)?;
            let at = server.sessions.get(&key)?.gateway.state();
            let placed = server.membership.player_location(guid)?.instance_id == here;
            (placed
                && at.zone_id == from.zone_id
                && rules.in_reach(from.position, at.position, whole))
            .then_some(key)
        })
        .collect()
}

/// One member's share: counted on a staged copy, saved, then installed. A refusal or a failed
/// save changes nothing for that member.
fn give(server: &mut Server, key: SessionKey, credit: QuestEvent, now_ms: u64) {
    let Some(session) = server.sessions.get_mut(&key) else {
        return;
    };
    let mut staged = session.gateway.clone();
    let Some(game) = staged.gameplay_mut() else {
        return;
    };
    let out = match game.receive_group_credit(credit) {
        Ok(out) => out,
        Err(error) => {
            event(
                json!({"event":"group_credit_refused","at_ms":now_ms,"reason":error.to_string()}),
            );
            return;
        }
    };
    if out.changed {
        if let Err(error) = save(&session.state_path, &staged.snapshot()) {
            event(
                json!({"event":"group_credit_refused","at_ms":now_ms,"reason":error.to_string()}),
            );
            return;
        }
        session.gateway = staged;
    }
    for reply in out.replies {
        let _ = session.queue_shared_message(&wrap_application(&reply));
    }
}
