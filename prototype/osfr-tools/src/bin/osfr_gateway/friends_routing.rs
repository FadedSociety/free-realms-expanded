//! Friends window requests: add (26/14), remove (26/15), answer a request (26/17) and ignore
//! (26/30), read as the client writes them, with the target found among live players or in the
//! account store so a request reaches someone who is away or was last seen before a restart.
//!
//! Client layouts (`lab/tools/ida-export/ida.py`): the command header is two u16 (`0090ac70`,
//! reader `00a9cb50`). 26/14 `008eb5c0` carries one length-prefixed string, the typed display
//! name (`0092a930`). 26/15 `00c1db80` and 26/30 `00c1dc10` carry a NameData record (three i32
//! then two strings, `00c22400`), 26/30 followed by the flag byte (1 ignore from `00c24b00`,
//! 0 unignore from `00c24c00`). 26/17 `00a0a500` carries the requester's guid, a u32 status and
//! NameData (`00a0f330`); the GAQ pop-up sends 0 to accept (`00a0f500`), 1 to decline
//! (`00a0f460`) and 2 when its timer ran out (`00a0f5a0`). The one parser of these commands is
//! `restoration/piece_087.rs`'s `decode_command`, included here; a body it refuses (a bad status,
//! a bad flag, an empty or trailing name) is dropped with no reply.
use super::*;
use osfr_tools::account_store::AccountStore;
use piece_087::{ConfirmStatus, FriendRequest, decode_command};
use std::sync::Arc;
use std::time::SystemTime;

#[path = "../../restoration/piece_087.rs"]
#[allow(dead_code)]
mod piece_087;

/// Where characters who are not logged in live: the account store names them, and each
/// character's social list sits beside their state file (`paths::social_path`). The store is
/// parsed once per change of its file into a small name index that the per-packet world clone
/// shares by `Arc`.
#[derive(Clone, Debug)]
pub(super) struct OfflineStore {
    accounts: PathBuf,
    state: PathBuf,
    index: Option<Arc<OfflineIndex>>,
}

#[derive(Debug)]
struct OfflineIndex {
    modified: SystemTime,
    /// Lower-cased display name -> guid; 0 marks a name several characters share, which
    /// `AccountStore::character_by_display_name` refuses too.
    by_name: BTreeMap<String, u64>,
    identities: BTreeMap<u64, SocialIdentity>,
}

impl OfflineStore {
    pub(super) fn new(accounts: PathBuf, state: PathBuf) -> Self {
        Self {
            accounts,
            state,
            index: None,
        }
    }

    /// The index of the store file as it is now; rebuilt only when the file changed.
    fn index(&mut self) -> Option<Arc<OfflineIndex>> {
        let modified = fs::metadata(&self.accounts).ok()?.modified().ok()?;
        if let Some(index) = &self.index
            && index.modified == modified
        {
            return Some(Arc::clone(index));
        }
        let store = AccountStore::load(&self.accounts).ok()?;
        let mut by_name = BTreeMap::new();
        let mut identities = BTreeMap::new();
        for record in store.characters() {
            let Ok(name) = SocialName::new(
                record.identity.first_name.clone(),
                record.identity.last_name.clone(),
            ) else {
                continue;
            };
            by_name
                .entry(name.display().to_lowercase())
                .and_modify(|guid| *guid = 0)
                .or_insert(record.guid);
            identities.insert(
                record.guid,
                SocialIdentity {
                    account_id: record.owner_account_id,
                    character_guid: record.guid,
                    name,
                },
            );
        }
        let index = Arc::new(OfflineIndex {
            modified,
            by_name,
            identities,
        });
        self.index = Some(Arc::clone(&index));
        Some(index)
    }

    fn social_path(&self, guid: u64) -> PathBuf {
        social_path(&character_state_path(&self.state, guid))
    }
}

enum Request {
    Add(String),
    Remove(SocialName),
    Answer {
        requester: u64,
        status: ConfirmStatus,
    },
    Ignore {
        name: SocialName,
        ignore: bool,
    },
}

/// The friend commands among the family-26 packets: `Ok(None)` for any other packet, `Err` for a
/// friend command whose body `decode_command` refuses.
fn parse(bytes: &[u8]) -> Result<Option<Request>, ()> {
    let Some(header) = bytes.get(..4) else {
        return Ok(None);
    };
    let family = u16::from_le_bytes([header[0], header[1]]);
    let sub = u16::from_le_bytes([header[2], header[3]]);
    if family != 26 || !matches!(sub, 14 | 15 | 17 | 30) {
        return Ok(None);
    }
    let social_name = |name: piece_087::NameData| {
        SocialName::new(name.first_name, name.last_name).map_err(|_| ())
    };
    let request = match decode_command(sub, &bytes[4..]).map_err(|_| ())? {
        FriendRequest::AddFriend { name } => Request::Add(name),
        FriendRequest::RemoveFriend { name } => Request::Remove(social_name(name)?),
        FriendRequest::ConfirmFriend { guid, status, .. } => Request::Answer {
            requester: guid,
            status,
        },
        FriendRequest::Ignore { name, ignore } => Request::Ignore {
            name: social_name(name)?,
            ignore,
        },
        FriendRequest::FriendsPositionRequest => return Ok(None),
    };
    Ok(Some(request))
}

/// `Some` when `inner` is one of the four friend requests (an unreadable one answers nothing,
/// never an error); `None` leaves the packet to the other routers.
pub(super) fn route(
    session: &Session,
    inner: &[u8],
    world: &mut SharedWorld,
) -> Option<Vec<Delivery>> {
    let request = match parse(inner) {
        Ok(Some(request)) => request,
        Ok(None) => return None,
        Err(()) => return Some(Vec::new()),
    };
    let me = session.character_guid;
    let messages = match request {
        Request::Add(display) => match resolve(world, &display) {
            Some(target) => world.social.request_friend(me, &target.name, &Found(&target)),
            // (R) No friend message exists for an unknown name; the client's family-22 text
            // "%s is not online." is the nearest of its own.
            None => return Some(vec![not_online(session.id, &display)]),
        },
        Request::Remove(name) => match resolve(world, &name.display()) {
            Some(target) => world.social.remove_friend(me, &target.name, &Found(&target)),
            None => return Some(Vec::new()),
        },
        Request::Answer { requester, status } => {
            // The requester may be away (or last seen before a restart): load their list so
            // the answer lands in it. A requester whose character is gone leaves a stale entry
            // that would reopen the pop-up at every login: drop it (the profile diff saves it).
            if world.social.profile(requester).is_none()
                && load_offline(world, |index| {
                    index.identities.contains_key(&requester).then_some(requester)
                })
                .is_none()
            {
                if let Some(mine) = world.social.profile_mut(me) {
                    mine.pending_from.remove(&requester);
                }
                return Some(Vec::new());
            }
            match status {
                ConfirmStatus::Accepted => world.social.respond_friend(me, requester, true),
                ConfirmStatus::Declined => world.social.respond_friend(me, requester, false),
                ConfirmStatus::TimedOut => world.social.timeout_friend_request(requester, me),
            }
        }
        Request::Ignore { name, ignore } => match resolve(world, &name.display()) {
            Some(target) if ignore => world.social.add_ignore(me, &target.name, &Found(&target)),
            Some(target) => world.social.remove_ignore(me, &target.name, &Found(&target)),
            None => return Some(Vec::new()),
        },
    };
    Some(
        messages
            .unwrap_or_default()
            .into_iter()
            .filter_map(|message| world.social_delivery(message))
            .collect(),
    )
}

/// The one character shown as `display` (case-insensitive, as `realm move` reads a name):
/// a live identity first, then the account store.
fn resolve(world: &mut SharedWorld, display: &str) -> Option<SocialIdentity> {
    let wanted = display.split_whitespace().collect::<Vec<_>>().join(" ");
    if wanted.is_empty() {
        return None;
    }
    if let Some(identity) = world
        .identities
        .values()
        .find(|identity| identity.name.display().eq_ignore_ascii_case(&wanted))
    {
        return Some(identity.clone());
    }
    let key = wanted.to_lowercase();
    load_offline(world, |index| {
        index.by_name.get(&key).copied().filter(|guid| *guid != 0)
    })
}

/// Register the social list of a character who is not logged in, the one `pick` names in the
/// account store's index, so the service can hold a request, an answer or a removal for them.
fn load_offline(
    world: &mut SharedWorld,
    pick: impl FnOnce(&OfflineIndex) -> Option<u64>,
) -> Option<SocialIdentity> {
    let offline = world.friend_store.as_mut()?;
    let index = offline.index()?;
    let guid = pick(&index)?;
    let identity = index.identities.get(&guid)?.clone();
    let path = offline.social_path(guid);
    world.social_profile(&identity, &path).ok()?;
    Some(identity)
}

/// A lookup that already found its character.
struct Found<'a>(&'a SocialIdentity);

impl CharacterLookup for Found<'_> {
    fn find_by_name(&self, _: &SocialName) -> Option<SocialIdentity> {
        Some(self.0.clone())
    }
}

/// Family 22 `PacketTargetClientNotOnline`, as `social_chat::ChatError::client_reply` sends it
/// for a tell: the client prints "%s is not online." (en_us 3023090394).
fn not_online(recipient: SessionId, name: &str) -> Delivery {
    let mut bytes = 22u16.to_le_bytes().to_vec();
    bytes.extend_from_slice(&(name.len() as u32).to_le_bytes());
    bytes.extend_from_slice(name.as_bytes());
    Delivery {
        recipient,
        bytes: wrap_application(&bytes),
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::test_support::{self, Scratch};
    use super::*;
    use osfr_tools::account::{AccountFlags, SlotPolicy, create_character};
    use osfr_tools::identity::NativeCreation;
    use osfr_tools::players::PlayerPresentation;

    const GUIDS: [u64; 2] = [401, 402];
    const NAMES: [(&str, &str); 2] = [("Alpha", "Fixture"), ("Beta", "Fixture")];

    struct Rig {
        sessions: [Session; 2],
        world: SharedWorld,
        scratch: Scratch,
    }

    fn options(scratch: &Scratch) -> Options {
        Options {
            world: Some(scratch.path().join("world")),
            ..test_support::options(scratch.state())
        }
    }

    fn session(options: &Options, id: u64, guid: u64, account: u64) -> Session {
        let mut session = Session::new(
            SessionId(id),
            "127.0.0.1:12345".parse().unwrap(),
            SessionRequest {
                id: id as u32,
                crc_length: 3,
                buffer_size: 512,
            },
            options,
            CRC_SEED,
            None,
            None,
        )
        .unwrap();
        session.character_guid = guid;
        session.account_guid = account;
        session
    }

    /// Put a character in the world as `join_player` does for the friends list: a registry
    /// row, the social list at `path`, logged in, and reachable for deliveries.
    fn enter(world: &mut SharedWorld, session: &Session, identity: &SocialIdentity, path: &Path) {
        let guid = identity.character_guid;
        world
            .players
            .join_with_presentation(
                session.id,
                identity.account_id,
                guid,
                ZoneInstance {
                    zone_id: 1,
                    instance_id: 1,
                },
                [guid as f32, 0.0, 0.0],
                0.0,
                PlayerPresentation {
                    character_guid: guid,
                    first_name: identity.name.first.clone(),
                    ..PlayerPresentation::default()
                },
            )
            .unwrap();
        world.social_profile(identity, path).unwrap();
        world.social.login(guid).unwrap();
        world.joined.insert(session.id);
        world.active_sessions.insert(guid, session.id);
    }

    fn rig() -> Rig {
        let scratch = Scratch::new("friends");
        let options = options(&scratch);
        let sessions: [Session; 2] = std::array::from_fn(|index| {
            session(&options, 41 + index as u64, GUIDS[index], GUIDS[index] + 10_000)
        });
        let mut world = SharedWorld::new();
        for (index, session) in sessions.iter().enumerate() {
            let guid = GUIDS[index];
            let (first, last) = NAMES[index];
            let identity = SocialIdentity {
                account_id: guid + 10_000,
                character_guid: guid,
                name: SocialName::new(first, last).unwrap(),
            };
            enter(
                &mut world,
                session,
                &identity,
                &scratch.path().join(format!("{guid}.social.json")),
            );
        }
        Rig {
            sessions,
            world,
            scratch,
        }
    }

    /// An account store with one account and one "<first> Fixture" character per name, saved
    /// in the scratch directory: `(store path, [(guid, account id)])`.
    fn store_with(scratch: &Scratch, firsts: &[&str]) -> (PathBuf, Vec<(u64, u64)>) {
        let mut store = AccountStore::new();
        let resources = test_support::creator_resources(scratch.path());
        let people = firsts
            .iter()
            .enumerate()
            .map(|(index, first)| {
                let account = store
                    .create_account(
                        [index as u8 + 9; 32],
                        AccountFlags::default(),
                        SlotPolicy { max_characters: 1 },
                    )
                    .unwrap();
                let mut choices = [0; 21];
                for (slot, value) in [(0, 1), (1, 1), (2, 2), (5, 1), (7, 201), (8, 189), (9, 199)] {
                    choices[slot] = value;
                }
                choices[20] = 1;
                let reply = create_character(
                    &mut store,
                    account,
                    NativeCreation {
                        first_name: (*first).into(),
                        last_name: "Fixture".into(),
                        temporary_first_name: String::new(),
                        temporary_last_name: String::new(),
                        choices,
                    },
                    &resources,
                );
                (reply.guid, account)
            })
            .collect();
        let accounts = scratch.path().join("accounts.json");
        store.save(&accounts).unwrap();
        (accounts, people)
    }

    fn identity(guid: u64, account: u64, first: &str) -> SocialIdentity {
        SocialIdentity {
            account_id: account,
            character_guid: guid,
            name: SocialName::new(first, "Fixture").unwrap(),
        }
    }

    fn name_data(first: &str, last: &str) -> Vec<u8> {
        let mut out = vec![0; 12];
        for part in [first, last] {
            out.extend_from_slice(&(part.len() as u32).to_le_bytes());
            out.extend_from_slice(part.as_bytes());
        }
        out
    }

    fn add_request(display: &str) -> Vec<u8> {
        let mut out = vec![26, 0, 14, 0];
        out.extend_from_slice(&(display.len() as u32).to_le_bytes());
        out.extend_from_slice(display.as_bytes());
        out
    }

    fn answer(requester: u64, status: u32, first: &str) -> Vec<u8> {
        let mut out = vec![26, 0, 17, 0];
        out.extend_from_slice(&requester.to_le_bytes());
        out.extend_from_slice(&status.to_le_bytes());
        out.extend(name_data(first, "Fixture"));
        out
    }

    /// Deliveries as `(recipient, family, sub, body)`, the body unwrapped.
    fn unwrap(deliveries: &[Delivery]) -> Vec<(SessionId, u16, u16, Vec<u8>)> {
        deliveries
            .iter()
            .map(|d| {
                let body = d.bytes[7..].to_vec(); // wrap_application: [5, 0, 1], u32 length, body
                let family = u16::from_le_bytes([body[0], body[1]]);
                let sub = if family == 74 || family == 112 {
                    body[2] as u16
                } else {
                    u16::from_le_bytes([body[2], body[3]])
                };
                (d.recipient, family, sub, body)
            })
            .collect()
    }

    fn route(rig: &mut Rig, who: usize, inner: &[u8]) -> Vec<(SessionId, u16, u16, Vec<u8>)> {
        let (a, b) = rig.sessions.split_at_mut(1);
        let session = if who == 0 { &mut a[0] } else { &mut b[0] };
        unwrap(&session.route_application(inner, &mut rig.world, 1_000))
    }

    #[test]
    fn a_typed_name_asks_the_target_and_the_answer_makes_two_friends() {
        if !test_support::bin_test_target("friends_") {
            return;
        }
        let mut rig = rig();
        let (alpha, beta) = (rig.sessions[0].id, rig.sessions[1].id);

        // Alpha types "beta fixture" (one string, any case): Alpha hears "waiting for Beta",
        // Beta gets the 26/16 pop-up naming Alpha's guid.
        let out = route(&mut rig, 0, &add_request("beta fixture"));
        let waiting = out
            .iter()
            .find(|(to, family, sub, _)| *to == alpha && *family == 74 && *sub == 8)
            .expect("74/8 to the requester");
        assert_eq!(waiting.3[3], 5, "FriendAddRequested");
        assert_eq!(&waiting.3[4..12], &402u64.to_le_bytes());
        let invite = out
            .iter()
            .find(|(to, family, sub, _)| *to == beta && *family == 26 && *sub == 16)
            .expect("26/16 to the target");
        assert_eq!(&invite.3[4..12], &401u64.to_le_bytes());
        assert_eq!(&invite.3[24..28], &5u32.to_le_bytes());
        assert_eq!(&invite.3[28..33], b"Alpha");

        // Beta's pop-up answers accept (status 0) with the guid from the pop-up.
        let out = route(&mut rig, 1, &answer(401, 0, "Alpha"));
        let adds: Vec<_> = out
            .iter()
            .filter(|(_, family, sub, _)| *family == 74 && *sub == 6)
            .map(|(to, ..)| *to)
            .collect();
        assert_eq!(adds, vec![alpha, beta]);
        assert!(out.iter().any(|(to, family, sub, body)| {
            *to == alpha && *family == 74 && *sub == 8 && body[3] == 1
        }));
        assert!(rig.world.social.profile(401).unwrap().friends.contains_key(&402));
        assert!(rig.world.social.profile(402).unwrap().friends.contains_key(&401));

        // Beta removes Alpha by the row's NameData: both rows go, both lists are empty.
        let mut remove = vec![26, 0, 15, 0];
        remove.extend(name_data("Alpha", "Fixture"));
        let out = route(&mut rig, 1, &remove);
        assert_eq!(
            out.iter().map(|(to, f, s, _)| (*to, *f, *s)).collect::<Vec<_>>(),
            vec![(beta, 74, 7), (alpha, 74, 7)]
        );
        assert!(rig.world.social.profile(401).unwrap().friends.is_empty());
        assert!(rig.world.social.profile(402).unwrap().friends.is_empty());
    }

    #[test]
    fn decline_timeout_ignore_and_unknown_names_answer_with_the_clients_texts() {
        if !test_support::bin_test_target("friends_") {
            return;
        }
        let mut rig = rig();
        let (alpha, beta) = (rig.sessions[0].id, rig.sessions[1].id);

        route(&mut rig, 0, &add_request("Beta Fixture"));
        let out = route(&mut rig, 1, &answer(401, 1, "Alpha"));
        assert_eq!(out.len(), 1);
        assert_eq!((out[0].0, out[0].3[3]), (alpha, 2), "FriendAddRequestDeclined");

        route(&mut rig, 0, &add_request("Beta Fixture"));
        let out = route(&mut rig, 1, &answer(401, 2, "Alpha"));
        assert_eq!((out[0].0, out[0].3[3]), (alpha, 3), "FriendAddRequestTimedOut");
        // An unknown status, a stale answer and a short body are dropped, not fatal.
        assert!(route(&mut rig, 1, &answer(401, 7, "Alpha")).is_empty());
        assert!(route(&mut rig, 1, &answer(401, 0, "Alpha")).is_empty());
        assert!(route(&mut rig, 1, &[26, 0, 17, 0, 1, 2]).is_empty());

        // Beta ignores Alpha (NameData then flag 1): 112/2 to Beta; Alpha's next request is
        // told "did not respond" and nothing is stored. Unignore sends 112/3.
        let mut ignore = vec![26, 0, 30, 0];
        ignore.extend(name_data("Alpha", "Fixture"));
        ignore.push(1);
        let out = route(&mut rig, 1, &ignore);
        assert_eq!(
            out.iter().map(|(to, f, s, _)| (*to, *f, *s)).collect::<Vec<_>>(),
            vec![(beta, 112, 2)]
        );
        let out = route(&mut rig, 0, &add_request("Beta Fixture"));
        assert_eq!((out.len(), out[0].3[3]), (1, 3));
        assert!(rig.world.social.profile(402).unwrap().pending_from.is_empty());
        let mut unignore = ignore.clone();
        *unignore.last_mut().unwrap() = 0;
        let out = route(&mut rig, 1, &unignore);
        assert_eq!((out[0].0, out[0].1, out[0].2), (beta, 112, 3));
        let mut bad_flag = ignore.clone();
        *bad_flag.last_mut().unwrap() = 2;
        assert!(route(&mut rig, 1, &bad_flag).is_empty());

        // A name nobody has gets the client's "%s is not online." (family 22) with the text
        // as typed; a blank name and a request with a trailing byte are dropped.
        let out = route(&mut rig, 0, &add_request("Nobody Here"));
        assert_eq!((out[0].0, out[0].1), (alpha, 22));
        assert_eq!(&out[0].3[2..6], &11u32.to_le_bytes());
        assert_eq!(&out[0].3[6..], b"Nobody Here");
        assert!(route(&mut rig, 0, &add_request("   ")).is_empty());
        let mut long = add_request("Beta Fixture");
        long.push(0);
        assert!(route(&mut rig, 0, &long).is_empty());
        // Other command sub-opcodes are not ours.
        assert!(parse(&[26, 0, 21, 0]).unwrap().is_none());
        assert!(parse(&[40, 0, 3, 0]).unwrap().is_none());
    }

    /// A name nobody online has is looked up in the account store (parsed once, reused until
    /// the file changes); the character's social list is loaded from beside their state file,
    /// the request waits there (and is saved), and the pop-up opens when they log in.
    #[test]
    fn a_request_to_a_player_who_is_away_waits_in_the_account_stores_character() {
        if !test_support::bin_test_target("friends_") {
            return;
        }
        let mut rig = rig();
        let alpha = rig.sessions[0].id;
        let (accounts, people) = store_with(&rig.scratch, &["Gamma"]);
        let (gamma, account) = people[0];
        let state = rig.scratch.state();
        rig.world.friend_store = Some(OfflineStore::new(accounts.clone(), state.clone()));

        let out = route(&mut rig, 0, &add_request("gamma fixture"));
        assert_eq!(out.len(), 1);
        assert_eq!((out[0].0, out[0].1, out[0].3[3]), (alpha, 74, 5));
        assert_eq!(&out[0].3[4..12], &gamma.to_le_bytes());
        let gamma_list = rig.world.social.profile(gamma).expect("loaded on demand");
        assert!(gamma_list.pending_from.contains_key(&401));
        let path = social_path(&character_state_path(&state, gamma));
        assert_eq!(rig.world.social_paths.get(&gamma), Some(&path));
        persist_social_changes(&SharedWorld::new(), &rig.world).unwrap();
        let saved = SocialLists::load(&path, account, gamma).unwrap();
        assert!(saved.pending_from.contains_key(&401));

        let at_login = rig.world.social.login(gamma).unwrap();
        assert!(at_login.iter().any(|m| {
            m.recipient_guid == gamma
                && m.bytes.starts_with(&[26, 0, 16, 0])
                && m.bytes[4..12] == 401u64.to_le_bytes()
        }));

        // The index is reused while the file is unchanged, and rebuilt when it changes.
        let first = Arc::as_ptr(rig.world.friend_store.as_mut().unwrap().index().as_ref().unwrap());
        assert_eq!(route(&mut rig, 0, &add_request("Delta Fixture"))[0].1, 22);
        let again = Arc::as_ptr(rig.world.friend_store.as_mut().unwrap().index().as_ref().unwrap());
        assert_eq!(first, again, "unchanged file, same index");
        let mut store = AccountStore::load(&accounts).unwrap();
        let delta = store
            .create_account([77; 32], AccountFlags::default(), SlotPolicy { max_characters: 1 })
            .unwrap();
        let mut choices = [0; 21];
        for (slot, value) in [(0, 1), (1, 1), (2, 2), (5, 1), (7, 201), (8, 189), (9, 199)] {
            choices[slot] = value;
        }
        choices[20] = 1;
        create_character(
            &mut store,
            delta,
            NativeCreation {
                first_name: "Delta".into(),
                last_name: "Fixture".into(),
                temporary_first_name: String::new(),
                temporary_last_name: String::new(),
                choices,
            },
            &test_support::creator_resources(rig.scratch.path()),
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
        store.save(&accounts).unwrap();
        assert_eq!(route(&mut rig, 0, &add_request("Delta Fixture"))[0].3[3], 5);
    }

    /// After a restart only the target is online: their pop-up (saved as `pending_from`) is
    /// answered against the requester's list loaded from the store, so both lists change and
    /// both are saved. An entry whose requester no longer exists is dropped, not reopened.
    #[test]
    fn an_answer_after_a_restart_loads_the_requesters_list_and_drops_stale_entries() {
        if !test_support::bin_test_target("friends_") {
            return;
        }
        let scratch = Scratch::new("friends-restart");
        let options = options(&scratch);
        let state = scratch.state();
        let (accounts, people) = store_with(&scratch, &["Alpha", "Gamma"]);
        let ((alpha, alpha_account), (gamma, gamma_account)) = (people[0], people[1]);
        let path_of = |guid| social_path(&character_state_path(&state, guid));

        // Before the restart: Alpha, online, asks Gamma, who is away.
        let mut before = SharedWorld::new();
        before.friend_store = Some(OfflineStore::new(accounts.clone(), state.clone()));
        let mut alpha_session = session(&options, 51, alpha, alpha_account);
        enter(
            &mut before,
            &alpha_session,
            &identity(alpha, alpha_account, "Alpha"),
            &path_of(alpha),
        );
        let out = unwrap(&alpha_session.route_application(
            &add_request("Gamma Fixture"),
            &mut before,
            1_000,
        ));
        assert_eq!((out.len(), out[0].3[3]), (1, 5));
        persist_social_changes(&SharedWorld::new(), &before).unwrap();

        // After the restart: a fresh world, Gamma logs in alone and gets the pop-up.
        let mut after = SharedWorld::new();
        after.friend_store = Some(OfflineStore::new(accounts, state.clone()));
        let mut gamma_session = session(&options, 52, gamma, gamma_account);
        enter(
            &mut after,
            &gamma_session,
            &identity(gamma, gamma_account, "Gamma"),
            &path_of(gamma),
        );
        assert!(after.social.profile(gamma).unwrap().pending_from.contains_key(&alpha));
        assert!(after.social.profile(alpha).is_none(), "Alpha is not online");
        let out = unwrap(&gamma_session.route_application(&answer(alpha, 0, "Alpha"), &mut after, 1_000));
        assert_eq!(
            out.iter().map(|(to, f, s, _)| (*to, *f, *s)).collect::<Vec<_>>(),
            vec![(gamma_session.id, 74, 6)],
            "Gamma's row; Alpha's 74/6 and 74/8 wait for Alpha's login"
        );
        assert!(after.social.profile(alpha).unwrap().friends.contains_key(&gamma));
        assert!(after.social.profile(gamma).unwrap().friends.contains_key(&alpha));
        assert!(after.social.profile(gamma).unwrap().pending_from.is_empty());
        persist_social_changes(&SharedWorld::new(), &after).unwrap();
        let alpha_saved = SocialLists::load(&path_of(alpha), alpha_account, alpha).unwrap();
        assert!(alpha_saved.friends.contains_key(&gamma));
        let gamma_saved = SocialLists::load(&path_of(gamma), gamma_account, gamma).unwrap();
        assert!(gamma_saved.friends.contains_key(&alpha));

        // A request from a character the store no longer has: the stale entry goes, quietly.
        after
            .social
            .profile_mut(gamma)
            .unwrap()
            .pending_from
            .insert(999, SocialName::new("Gone", "Fixture").unwrap());
        let out = gamma_session.route_application(&answer(999, 0, "Gone"), &mut after, 1_000);
        assert!(out.is_empty());
        assert!(after.social.profile(gamma).unwrap().pending_from.is_empty());
        persist_social_changes(&before, &after).unwrap();
        let gamma_saved = SocialLists::load(&path_of(gamma), gamma_account, gamma).unwrap();
        assert!(gamma_saved.pending_from.is_empty());
    }
}
