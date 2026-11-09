//! T006: family-129 guild routing.
//!
//! The store, policy and wire encoders live in `osfr_tools::guilds`; this file only holds the
//! world/session glue and the one server-wide guild file. Client readers: 129/22 `0x00b1e730`
//! -> `0x00b1e160`, 129/17 `0x00b1c840`, 129/24 `0x00b1be30` -> `0x00b1d210`, 129/27
//! `0x00b1d5b0`, 129/28 `0x00b1d1b0`, 129/29 `0x00b1ace0`, 129/32 `0x00b1bd30`.
//!
//! (R) A guild request may run at any location; it is not an activity, never moves the
//! character and is charged to the shared account action budget before it reaches this file.

use super::*;
use osfr_tools::guilds::{
    GuildCaller, GuildMessage, GuildOutcome, GuildStore, can_create_message, decode_request,
    player_status_bytes,
};
use osfr_tools::social_chat::NameData;

/// T006 (R): one JSON guild file beside the character save, owned by the staged world.
/// The store is shared copy-on-write: a world clone costs one refcount bump. Malformed
/// bodies and requests the service answers with silence are refused before the first
/// mutation, so they never clone; a request that can change or answer clones once.
#[derive(Clone, Debug)]
pub(super) struct GuildWorld {
    service: Arc<osfr_tools::guilds::GuildService>,
    path: PathBuf,
}

impl Default for GuildWorld {
    fn default() -> Self {
        Self {
            service: Arc::new(osfr_tools::guilds::GuildService::default()),
            path: PathBuf::new(),
        }
    }
}

impl GuildWorld {
    pub(super) fn load(state: &Path, invite_window_ms: u64) -> io::Result<Self> {
        let path = state.with_file_name("guilds.json");
        Ok(Self {
            service: Arc::new(osfr_tools::guilds::GuildService::load(
                &path,
                invite_window_ms,
            )?),
            path,
        })
    }

    pub(super) fn store(&self) -> &GuildStore {
        self.service.store()
    }

    /// Persist a staged change; an empty path keeps a unit-test world in memory.
    pub(super) fn save_changed(&mut self) -> io::Result<()> {
        if self.service.is_dirty() && !self.path.as_os_str().is_empty() {
            Arc::make_mut(&mut self.service).save(&self.path)?;
        }
        Ok(())
    }
}

fn member_name(identity: Option<&SocialIdentity>) -> NameData {
    identity
        .map(|identity| NameData::new(identity.name.first.clone(), identity.name.last.clone()))
        .unwrap_or_default()
}

/// Resolve a 129/3 name invite against the online identities. Duplicate display names resolve
/// to the lowest guid so the result never depends on map order.
fn resolve_target(identities: &BTreeMap<u64, SocialIdentity>, raw: &str) -> Option<u64> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    identities
        .values()
        .filter(|identity| member_name(Some(identity)).matches_display(raw))
        .map(|identity| identity.character_guid)
        .min()
}

impl SharedWorld {
    /// One guild reply to the live session of its recipient character.
    fn guild_delivery(&self, message: &GuildMessage) -> Option<Delivery> {
        let session = *self.active_sessions.get(&message.recipient_guid)?;
        if !self.joined.contains(&session) {
            return None;
        }
        Some(Delivery {
            recipient: session,
            bytes: wrap_application(&message.bytes),
        })
    }

    fn guild_messages(&self, outcome: &GuildOutcome) -> Vec<Delivery> {
        outcome
            .messages
            .iter()
            .filter_map(|message| self.guild_delivery(message))
            .collect()
    }

    /// Roster, create eligibility and logged-in member names for one world entry.
    pub(super) fn guild_login(&mut self, session: &Session) -> Vec<Delivery> {
        let eligible = session
            .gateway
            .gameplay()
            .is_some_and(Gameplay::guild_eligible);
        let name = member_name(session.social_identity.as_ref());
        let character_guid = session.character_guid;
        // A guildless entry is the common case; only the member path refreshes a name
        // snapshot, so a guildless entry has nothing to mutate.
        if self.guilds.store().guild_guid_for(character_guid).is_none() {
            let mut outcome = GuildOutcome::default();
            outcome
                .messages
                .push(can_create_message(character_guid, eligible));
            return self.guild_messages(&outcome);
        }
        let outcome =
            Arc::make_mut(&mut self.guilds.service).login(character_guid, &name, eligible);
        let mut deliveries = self.guild_messages(&outcome);
        // The registry's AddPc for this entry went out without a guild (the join ran first);
        // mirror the guild now and tell the players already in view through 129/31.
        if let Some(guild_guid) = self.guilds.store().guild_guid_for(character_guid) {
            self.players.set_guild(session.id, guild_guid);
            deliveries.extend(self.guild_tag_deliveries(
                session.id,
                character_guid,
                guild_guid,
                true,
            ));
        }
        deliveries
    }

    /// 129/31 to the player's current viewers and the player: the AddPc guild list only
    /// reaches viewers who meet the player later. A clear names the guild that was left: the
    /// client removes the tag by that guild's key (`00b1ce80`).
    fn guild_tag_deliveries(
        &self,
        session_id: SessionId,
        character_guid: u64,
        guild_guid: u64,
        in_guild: bool,
    ) -> Vec<Delivery> {
        let Some(player) = self.players.get(session_id) else {
            return Vec::new();
        };
        let bytes = wrap_application(&player_status_bytes(character_guid, guild_guid, in_guild));
        let mut recipients = self.players.nearby_sessions(player.zone, player.position);
        recipients.insert(session_id);
        recipients
            .into_iter()
            .filter(|recipient| self.joined.contains(recipient))
            .map(|recipient| Delivery {
                recipient,
                bytes: bytes.clone(),
            })
            .collect()
    }

    /// Mirror the guild store into the player registry for every joined character whose
    /// guild changed (create, join, leave, kick, disband), telling their viewers. Guild requests
    /// are rare, so a scan of the joined characters is fine.
    pub(super) fn sync_guild_tags(&mut self) -> Vec<Delivery> {
        let changes: Vec<(SessionId, u64, u64, u64)> = self
            .active_sessions
            .iter()
            .filter_map(|(character_guid, session_id)| {
                let wanted = self
                    .guilds
                    .store()
                    .guild_guid_for(*character_guid)
                    .unwrap_or(0);
                let shown = self.players.guild_of(*session_id);
                (self.players.contains(*session_id) && shown != wanted).then_some((
                    *session_id,
                    *character_guid,
                    shown,
                    wanted,
                ))
            })
            .collect();
        let mut deliveries = Vec::new();
        for (session_id, character_guid, shown, wanted) in changes {
            self.players.set_guild(session_id, wanted);
            // A member who left (or was kicked, or whose guild disbanded) is cleared under the
            // guild they were shown with; a joiner is bound to the new guild.
            let (guild_guid, in_guild) = if wanted != 0 {
                (wanted, true)
            } else {
                (shown, false)
            };
            deliveries.extend(self.guild_tag_deliveries(
                session_id,
                character_guid,
                guild_guid,
                in_guild,
            ));
        }
        deliveries
    }

    /// Presence notices for the members still connected after a disconnect.
    pub(super) fn guild_logout(&mut self, character_guid: u64) -> Vec<Delivery> {
        if self.guilds.store().guild_guid_for(character_guid).is_none() {
            return Vec::new();
        }
        let outcome = Arc::make_mut(&mut self.guilds.service).logout(character_guid);
        self.guild_messages(&outcome)
    }
}

impl Session {
    /// Family 129 (guilds). The actor is this session's authenticated character; the body never
    /// names it. Malformed bodies and refusals produce no packet and change nothing (R).
    pub(super) fn route_guild(
        &mut self,
        inner: &[u8],
        world: &mut SharedWorld,
        now_ms: u64,
    ) -> Vec<Delivery> {
        if self.character_guid == 0 || !world.joined.contains(&self.id) {
            return Vec::new();
        }
        // Decode before touching the staged store: a malformed family-129 body must not
        // make the copy-on-write service clone. A silent refusal is skipped the same way.
        let Ok(request) = decode_request(inner) else {
            return Vec::new();
        };
        if !world
            .guilds
            .service
            .responds_to(self.character_guid, &request)
        {
            return Vec::new();
        }
        let name = member_name(self.social_identity.as_ref());
        let eligible = self
            .gateway
            .gameplay()
            .is_some_and(Gameplay::guild_eligible);
        let identities = &world.identities;
        let resolve = |raw: &str| resolve_target(identities, raw);
        let caller = GuildCaller {
            character_guid: self.character_guid,
            name: &name,
            eligible,
            // (R) The client declines an invite from an ignored player (`00b1e7e0`); a
            // server-side ignore lookup for forged accepts is a follow-up.
            ignored_pair: false,
        };
        let outcome = Arc::make_mut(&mut world.guilds.service)
            .handle_decoded(&caller, &resolve, request, now_ms);
        let mut deliveries = world.guild_messages(&outcome);
        deliveries.extend(world.sync_guild_tags());
        deliveries
    }
}
