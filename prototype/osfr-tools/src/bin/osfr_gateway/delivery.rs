//! Application-delivery framing for the gateway: one place that wraps an inner packet in
//! the 5/1 tunnel the client decodes, builds a chat refusal, fans a server notice out,
//! filters shared-field packets per recipient, or keeps the public look later AddPc
//! deliveries use.
use super::*;

/// One trade service message per recipient, wrapped in the application tunnel. Moved out of the
/// binary root unchanged; the trade lifecycle is `w163_trade_routing`'s subject.
pub(super) fn trade_deliveries(messages: Vec<TradeMessage>) -> Vec<Delivery> {
    messages
        .into_iter()
        .map(|message| Delivery {
            recipient: message.recipient,
            bytes: wrap_application(&message.bytes),
        })
        .collect()
}

/// (W163) Keep the trade service's copy of this character's inventory current. A character
/// whose inventory does not validate is not tradable: its trade ends as a plain cancel (the
/// player is still online, so nobody reads "left the game", T885), then it is unregistered.
pub(super) fn refresh_trade_player(
    world: &mut SharedWorld,
    session: SessionId,
    gateway: &Gateway,
) -> Vec<Delivery> {
    let Some(player) = world.players.get(session).cloned() else {
        return Vec::new();
    };
    let guid = player.character_guid;
    match gateway.trade_inventory() {
        Some(inventory) => {
            if world.trades.inventory(guid).is_some() {
                // A snapshot that fails the service's checks keeps the previous one (R).
                let _ = world.trades.sync_inventory(guid, inventory);
            } else {
                let _ = world.trades.register_player(&player, inventory);
            }
            // A battle or dungeon run ends a trade and blocks a new one.
            trade_deliveries(world.trades.set_away(guid, gateway.in_encounter()))
        }
        None => {
            let mut ended = world.trades.cancel_for(guid);
            ended.extend(world.trades.unregister_player(guid));
            trade_deliveries(ended)
        }
    }
}

/// Queue one delivery: on the current session, or on the recipient's own session when it is
/// still connected (a recipient that left is skipped).
pub(super) fn queue_delivery(
    current: &mut Session,
    sessions: &mut SessionMap,
    world: &SharedWorld,
    delivery: Delivery,
) -> Result<(), Box<dyn std::error::Error>> {
    if delivery.recipient == current.id {
        current.sender.queue_message(&delivery.bytes)?;
    } else if let Some(key) = world.session_keys.get(&delivery.recipient) {
        if let Some(session) = sessions.get_mut(key) {
            session.sender.queue_message(&delivery.bytes)?;
        }
    }
    Ok(())
}

/// (R) Inspect shows only unlocked jobs with ranks and the active job's worn items; no look
/// (no profiles loaded) means no Inspect entry. The same snapshot feeds AddPc for later viewers.
pub(super) fn refresh_public_look(world: &mut SharedWorld, guid: u64, gateway: &Gateway) {
    match gateway.public_look() {
        Some(look) => {
            world.public_looks.insert(guid, look);
        }
        None => {
            world.public_looks.remove(&guid);
        }
    }
}

/// The character and whether it was added (true) or removed (false), for the registry's AddPc
/// (35/1) and RemovePlayer (35/3) tunnel frames; `None` for every other delivery.
pub(super) fn rider_event(frame: &[u8]) -> Option<(u64, bool)> {
    let inner = frame.get(7..)?;
    match inner.get(..4)? {
        [35, 0, 1, 0] => Some((u64::from_le_bytes(inner.get(4..12)?.try_into().ok()?), true)),
        // 35/3 carries a u16 submode before the GUID.
        [35, 0, 3, 0] => Some((
            u64::from_le_bytes(inner.get(6..14)?.try_into().ok()?),
            false,
        )),
        _ => None,
    }
}

/// Wrap one inner application packet in the 5/1 tunnel.
pub(super) fn wrap_application(inner: &[u8]) -> Vec<u8> {
    let mut out = vec![5, 0, 1];
    out.extend_from_slice(&(inner.len() as u32).to_le_bytes());
    out.extend_from_slice(inner);
    out
}

/// The 156/3 reply to the requesting session, or none when the Fotomat module refuses. The
/// caller returns these deliveries unchanged; a refusal stores nothing and sends nothing.
pub(super) fn fotomat_reply(
    session: &Session,
    inner: &[u8],
    world: &SharedWorld,
) -> Option<Vec<Delivery>> {
    let reply = osfr_tools::fotomat::request(inner, session.id, &world.players)?;
    Some(vec![Delivery {
        recipient: session.id,
        bytes: wrap_application(&reply),
    }])
}

/// T008/F290: one opcode-164 jump body to the sessions whose directed view
/// contains the sender (the `ride_fanout` visibility rule). Called after the
/// commit succeeds, like the T199 broadcast; the sender never receives its own
/// jump. Each character has a relay budget: over budget the 164 is dropped
/// silently, while the accepted pose still fans out through the 125 path. The
/// budget uses the server's monotonic wall clock (`Instant::now()`), the
/// lead-accepted F290 behaviour; F309 briefly anchored it to the dispatch clock
/// and F323 reverted that unsigned clock change.
pub(super) fn jump_fanout(
    world: &SharedWorld,
    sender: SessionId,
    jump: Option<Vec<u8>>,
) -> Vec<Delivery> {
    let Some(jump) = jump else {
        return Vec::new();
    };
    let Some((zone, position, character)) = world
        .players
        .get(sender)
        .filter(|_| world.joined.contains(&sender))
        .map(|player| (player.zone, player.position, player.character_guid))
    else {
        return Vec::new();
    };
    let viewers = world
        .players
        .nearby_sessions(zone, position)
        .into_iter()
        .filter(|recipient| {
            *recipient != sender && world.players.knows_character(*recipient, character)
        })
        .collect::<Vec<_>>();
    if viewers.is_empty()
        || !world
            .security
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .jump_relays
            .allow(character, Instant::now())
    {
        return Vec::new();
    }
    let bytes = wrap_application(&jump);
    viewers
        .into_iter()
        .map(|recipient| Delivery {
            recipient,
            bytes: bytes.clone(),
        })
        .collect()
}

/// Queue the notices staged with a successful transaction: the T199 broadcast,
/// then the T008/F290 jump relay. The caller passes only a committed outcome, so
/// a refused or failed transaction queues neither.
pub(super) fn post_commit(
    session: &mut Session,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    broadcast: Option<String>,
    jump: Option<Vec<u8>>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(text) = broadcast {
        for delivery in broadcast_deliveries(&text, world) {
            queue_delivery(session, sessions, world, delivery)?;
        }
    }
    for delivery in jump_fanout(world, session.id, jump) {
        queue_delivery(session, sessions, world, delivery)?;
    }
    Ok(())
}

/// Drop a character's relay budget when its session ends.
pub(super) fn forget_jump_relay(world: &SharedWorld, character: u64) {
    world
        .security
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .jump_relays
        .remove(&character);
}

/// One client-refusal reply, or nothing when the error has no client form.
pub(super) fn chat_error_delivery(
    error: osfr_tools::social_chat::ChatError,
    recipient: SessionId,
) -> Vec<Delivery> {
    error
        .client_reply()
        .map(|bytes| {
            vec![Delivery {
                recipient,
                bytes: wrap_application(&bytes),
            }]
        })
        .unwrap_or_default()
}

/// T199: one wrapped 15/3 system line for every session that has entered the world
/// (`chat_sessions`), including the sender. Encoded once; each recipient gets its own
/// copy. The caller queues these only after `commit_outcome` succeeds; a recipient that
/// leaves before queueing is skipped by `queue_delivery`.
pub(super) fn broadcast_deliveries(text: &str, world: &SharedWorld) -> Vec<Delivery> {
    let bytes = wrap_application(&osfr_tools::world_notice::encode(text));
    world
        .chat_sessions
        .keys()
        .map(|recipient| Delivery {
            recipient: *recipient,
            bytes: bytes.clone(),
        })
        .collect()
}

/// The inner packets that travel the shared-field fan-out, filtered by `field_packets_for`.
pub(super) fn field_packet(packet: &[u8]) -> bool {
    packet.starts_with(&[125, 0])
        || packet.get(..4).is_some_and(|h| {
            matches!(
                h,
                [35, 0, 5, 0] | [35, 0, 8, 0] | [35, 0, 16, 0] | [32, 0, 7, 0]
            )
        })
}

/// The field packets `recipient` may see: only in its own ready instance, and only about
/// characters or NPCs it already knows.
pub(super) fn field_packets_for(
    recipient: &Session,
    membership: &World,
    players: &PlayerRegistry,
    instance: i32,
    packets: &[Vec<u8>],
    known_npcs: &BTreeSet<u64>,
) -> Vec<Delivery> {
    if !field_recipient_matches(
        recipient.gateway.is_world_loaded(),
        membership
            .player_location(recipient.character_guid)
            .map(|l| (l.instance_id, l.ready)),
        instance,
    ) {
        return Vec::new();
    }
    let known = |guid| {
        guid == recipient.character_guid
            || known_npcs.contains(&guid)
            || players.knows_character(recipient.id, guid)
    };
    packets
        .iter()
        .filter(|packet| {
            let offset = if packet.starts_with(&[125, 0]) { 2 } else { 4 };
            let Some(guid) = packet
                .get(offset..offset + 8)
                .and_then(|b| b.try_into().ok())
                .map(u64::from_le_bytes)
            else {
                return false;
            };
            if !known(guid) {
                return false;
            }
            if packet.starts_with(&[32, 0, 7, 0]) {
                let Some(target) = packet
                    .get(20..28)
                    .and_then(|b| b.try_into().ok())
                    .map(u64::from_le_bytes)
                else {
                    return false;
                };
                return known(target);
            }
            true
        })
        .map(|packet| Delivery {
            recipient: recipient.id,
            bytes: wrap_application(packet),
        })
        .collect()
}

// (R) A server-owned membership, never the attacker's directed PC view,
// determines which ready instance may observe a field event.
pub(super) fn field_recipient_matches(
    loaded: bool,
    location: Option<(i32, bool)>,
    instance: i32,
) -> bool {
    #[cfg(test)]
    if std::env::var_os("OSFR_W127_CROSS_INSTANCE_CANARY").is_some() {
        return loaded; // Deliberate test-only broadcast mutation.
    }
    loaded && location.is_some_and(|(id, ready)| id == instance && ready)
}

/// Live appearance updates name a player and are safe only for viewers who
/// already received that player's AddPc.
pub(super) fn live_outfit_packet(packet: &[u8]) -> bool {
    // 35/7 reader 0x0091af50; 35/6 is T027's existing equipment-change prefix; 35/39 is a style
    // card's look change (reader 0x0091aff0).
    matches!(packet, [35, 0, 6 | 7 | 39, 0, ..])
}

impl Server {
    /// (R) A full reliable queue refuses that recipient only: the shared field
    /// and every other viewer keep ticking, and the next tick retries.
    pub(super) fn refuse_field_delivery(session: &Session, error: Box<dyn std::error::Error>) {
        event(json!({
            "event":"field_delivery_refused",
            "character":session.character_guid,
            "reason":error.to_string()
        }));
    }

    pub(super) fn queue_field_deliveries(&mut self, deliveries: Vec<Delivery>) {
        for delivery in deliveries {
            if let Some(session) = self
                .world
                .session_keys
                .get(&delivery.recipient)
                .and_then(|key| self.sessions.get_mut(key))
            {
                if let Err(error) = session.queue_shared_message(&delivery.bytes) {
                    Self::refuse_field_delivery(session, error);
                }
            }
        }
    }
}

impl SharedWorld {
    /// W162: `packets` for other players in interest range of `session`. Live appearance updates
    /// also require the recipient's directed view to contain the actor. The registry position is
    /// read before a zone transfer removes the player.
    pub(super) fn ride_fanout(&self, session: SessionId, packets: &[Vec<u8>]) -> Vec<Delivery> {
        let is_joined = self.joined.contains(&session);
        let Some(player) = self.players.get(session).filter(|_| is_joined) else {
            return Vec::new();
        };
        let mut deliveries = Vec::new();
        for other in self.players.nearby_sessions(player.zone, player.position) {
            if other == session {
                continue;
            }
            let knows_character = self.players.knows_character(other, player.character_guid);
            for packet in packets {
                if knows_character || !live_outfit_packet(packet) {
                    deliveries.push(Delivery {
                        recipient: other,
                        bytes: wrap_application(packet),
                    });
                }
            }
        }
        deliveries
    }
}
