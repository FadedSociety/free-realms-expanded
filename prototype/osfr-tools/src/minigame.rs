//! Shared, persistent lifecycle for one player's active minigame run.
//!
//! This owns launch identity and terminal sequencing. Per-game rules, score
//! authority and material staging stay with the Activity implementation.
//! The run counter and reward snapshot must be saved together before packets
//! are sent by the eventual gateway integration.

use crate::rewards::{RewardBundleRow, RewardGrant, RewardState, apply_grant};
use serde::{Deserialize, Serialize};
use std::io;

const FAMILY_MINIGAME: u16 = 39;
const FAMILY_COMMAND: u16 = 26;
const FAMILY_ENCOUNTER: u16 = 41;
const HEADER_LEN: usize = 15;
const INFO_OBJECTIVE_BYTES: usize = 0xe8;
const EMPTY_REWARD_BUNDLES_BYTES: usize = 3 * 69;

/// Generic lifecycle phase. Terminal state is retained until the real result
/// close arrives; `39/7` is only the automatic removal acknowledgement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunState {
    Offered,
    Starting,
    Playing,
    Won,
    Lost,
    Closed,
    Cancelled,
}

/// Transport-specific start handling. The encounter's ids and player guid
/// belong to its owning service and are never read from a minigame score.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunMode {
    Flash {
        movie: String,
    },
    Encounter {
        encounter_id: i32,
        player_guid: u64,
    },
    /// T765: a type-16 Trading Card Game run. The client hosts the card DLL itself
    /// (`ClientTradingCardGameManager`): 39/5 is answered with 39/17 alone, and its 39/37
    /// (`009bf590`, sent at manager state 5 by `00ad4f50`) asks for the launch ticket.
    CardClient,
}

/// Fields needed to emit the shared 39/16 info record. The reward bundles are
/// the three exact serialized client bundles; objectives are packed 0xe8-byte
/// records supplied by the owning activity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunSpec {
    pub game_id: i32,
    pub name_id: i32,
    pub icon_id: i32,
    pub description_id: i32,
    pub difficulty: i32,
    pub profile_type: i32,
    pub type_id: i32,
    pub members_only: bool,
    pub reward_bundles: Vec<u8>,
    pub objectives: Vec<u8>,
    pub flags: [u8; 5],
    pub preload_file: String,
    pub mode: RunMode,
}

impl RunSpec {
    /// Minimal empty-reward/goal offer. Difficulty and flags are server
    /// defaults (R); adapters should replace them with their content values.
    pub fn empty(game_id: i32, type_id: i32, mode: RunMode) -> Self {
        Self {
            game_id,
            name_id: 0,
            icon_id: 0,
            description_id: 0,
            difficulty: 1,
            profile_type: 0,
            type_id,
            members_only: false,
            reward_bundles: vec![0; EMPTY_REWARD_BUNDLES_BYTES],
            objectives: Vec::new(),
            flags: [1; 5],
            preload_file: String::new(),
            mode,
        }
    }

    fn validate(&self) -> io::Result<()> {
        if self.game_id <= 0
            || self.type_id < 0
            || !(1..=5).contains(&self.difficulty)
            || self.profile_type < 0
            || self.reward_bundles.len() > 1024 * 1024
            || self.reward_bundles.len() < EMPTY_REWARD_BUNDLES_BYTES
            || self.objectives.len() % INFO_OBJECTIVE_BYTES != 0
            || self.objectives.len() / INFO_OBJECTIVE_BYTES > 4096
            || self.flags.iter().any(|flag| *flag > 1)
            || self.flags[4] != 1
            || self.preload_file.len() > 1024
            || self
                .preload_file
                .bytes()
                .any(|byte| byte == 0 || byte < 0x20)
        {
            return Err(invalid("invalid minigame offer"));
        }
        match &self.mode {
            RunMode::Flash { movie } => validate_asset_name(movie)?,
            RunMode::Encounter {
                encounter_id,
                player_guid,
            } if *encounter_id <= 0 || *player_guid == 0 => {
                return Err(invalid("invalid encounter launch identity"));
            }
            RunMode::Encounter { .. } | RunMode::CardClient => {}
        }
        Ok(())
    }
}

/// Per-activity terminal facts. Amounts and eligibility remain activity-owned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActivityFinish {
    pub won: bool,
    pub stars: u32,
    pub elapsed_seconds: u32,
}

/// Plug point for Cooking, Smelting, Forging and Mining. These modules can
/// report authoritative score and completion without owning wire lifecycle.
pub trait Activity {
    fn score(&self) -> u32;
    fn finish(&self) -> Option<ActivityFinish>;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Run {
    instance_id: i32,
    spec: RunSpec,
    state: RunState,
    score: u32,
    wheel_entry: Option<WheelEntry>,
    wheel_stopped: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct WheelEntry {
    type_id: u32,
    body: Vec<u8>,
}

/// Small journal sufficient to reject stale packets and preserve the next id.
/// The caller persists its JSON snapshot with economy state before sending.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MinigameRuns {
    version: u8,
    next_instance_id: i32,
    current: Option<Run>,
    last: Option<Run>,
}

/// Staged reply. `replies` keep the native lifecycle order; reward messages
/// are separate so the caller can atomically persist both snapshots first.
pub struct RunOutcome {
    pub accepted: bool,
    pub changed: bool,
    pub state: Option<RunState>,
    pub replies: Vec<Vec<u8>>,
    pub reward_packets: Vec<Vec<u8>>,
    pub reward_state: Option<RewardState>,
    /// T765: a card run's client asked for its launch ticket (39/37); the owner mints it and
    /// answers with `card_start_packet`.
    pub card_ticket: Option<CardTicketRequest>,
}

/// T765: the run a card launch ticket is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardTicketRequest {
    pub instance_id: i32,
    pub game_id: i32,
}

impl Default for RunOutcome {
    fn default() -> Self {
        Self {
            accepted: false,
            changed: false,
            state: None,
            replies: Vec::new(),
            reward_packets: Vec::new(),
            reward_state: None,
            card_ticket: None,
        }
    }
}

impl MinigameRuns {
    /// Start with the next unused positive id. The allocation namespace is R.
    pub fn new(next_instance_id: i32) -> io::Result<Self> {
        if next_instance_id <= 0 {
            return Err(invalid("minigame instance counter must be positive"));
        }
        Ok(Self {
            version: 1,
            next_instance_id,
            current: None,
            last: None,
        })
    }

    pub fn next_instance_id(&self) -> i32 {
        self.next_instance_id
    }

    pub fn current_instance_id(&self) -> Option<i32> {
        self.current.as_ref().map(|run| run.instance_id)
    }

    /// Whether `packet` is a lifecycle packet (family 39) naming the current or the last run.
    pub fn names_run(&self, packet: &[u8]) -> bool {
        packet.starts_with(&FAMILY_MINIGAME.to_le_bytes())
            && packet.len() >= 7
            && self.state(read_i32(packet, 3)).is_some()
    }

    pub fn state(&self, instance_id: i32) -> Option<RunState> {
        self.current
            .as_ref()
            .filter(|run| run.instance_id == instance_id)
            .or_else(|| {
                self.last
                    .as_ref()
                    .filter(|run| run.instance_id == instance_id)
            })
            .map(|run| run.state)
    }

    /// Serialize only this ledger. The outer save must commit it atomically
    /// with `RewardState`, activity state and any consumed materials.
    pub fn snapshot(&self) -> io::Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(invalid)
    }

    pub fn restore(bytes: &[u8]) -> io::Result<Self> {
        let restored: Self = serde_json::from_slice(bytes).map_err(invalid)?;
        restored.validate()?;
        Ok(restored)
    }

    pub fn validate(&self) -> io::Result<()> {
        if self.version != 1 || self.next_instance_id <= 0 {
            return Err(invalid("unsupported minigame ledger snapshot"));
        }
        for run in [&self.current, &self.last].into_iter().flatten() {
            run.spec.validate()?;
            if run.instance_id <= 0 || run.instance_id >= self.next_instance_id {
                return Err(invalid("minigame run exceeds persisted instance counter"));
            }
        }
        if self
            .current
            .as_ref()
            .is_some_and(|run| matches!(run.state, RunState::Closed | RunState::Cancelled))
            || self
                .last
                .as_ref()
                .is_some_and(|run| !matches!(run.state, RunState::Closed | RunState::Cancelled))
            || self
                .current
                .as_ref()
                .zip(self.last.as_ref())
                .is_some_and(|(a, b)| a.instance_id == b.instance_id)
        {
            return Err(invalid("invalid minigame ledger state"));
        }
        Ok(())
    }

    /// Create one offer. A live or terminal-unclosed run blocks another run.
    pub fn offer(&mut self, spec: RunSpec) -> io::Result<RunOutcome> {
        spec.validate()?;
        if self.current.is_some() {
            return Ok(RunOutcome::default());
        }
        let id = self.next_instance_id;
        let next = id
            .checked_add(1)
            .ok_or_else(|| invalid("minigame instance counter exhausted"))?;
        let mut replies = vec![info_packet(id, &spec)?];
        // An encounter run (fishing 21, soccer 12) has no preload list, so the client's own ready
        // step (manager state 3 in `009bf5e0`, which `009beb70` reaches only for types 1, 2, 17,
        // 22 or through a preload) never runs and the start screen stays on "Loading...". 41/107
        // is the server's ready: dispatcher `00aa36c0` case 0x6b calls `009b0cc0`
        // (`HandlerMiniGameStart:setReady`, the Go button), as kart's `loaded` sends it.
        if let RunMode::Encounter { encounter_id, .. } = spec.mode {
            replies.push(encounter_packet(107, encounter_id, id));
        }
        self.next_instance_id = next;
        self.current = Some(Run {
            instance_id: id,
            spec,
            state: RunState::Offered,
            score: 0,
            wheel_entry: None,
            wheel_stopped: false,
        });
        Ok(RunOutcome {
            accepted: true,
            changed: true,
            state: Some(RunState::Offered),
            replies,
            ..Default::default()
        })
    }

    /// Consume a client packet for the current instance. Only the shared
    /// start, leave, acknowledgement and close controls are handled here.
    pub fn handle_client(&mut self, packet: &[u8]) -> io::Result<RunOutcome> {
        if packet.len() >= 3 && packet[..2] == FAMILY_MINIGAME.to_le_bytes() {
            let sub = packet[2];
            if !matches!(sub, 5 | 6 | 7 | 37 | 46) {
                return Ok(RunOutcome::default());
            }
            if packet.len() != HEADER_LEN {
                return Err(invalid("malformed minigame lifecycle packet"));
            }
            let id = read_i32(packet, 3);
            let group = read_i32(packet, 7);
            let game = read_i32(packet, 11);
            if group != -1 || game != -1 {
                return Ok(RunOutcome::default());
            }
            if sub == 7 {
                let known = self
                    .current
                    .as_ref()
                    .or(self.last.as_ref())
                    .is_some_and(|run| run.instance_id == id);
                return Ok(if known {
                    RunOutcome {
                        accepted: true,
                        state: self.state(id),
                        ..Default::default()
                    }
                } else {
                    RunOutcome::default()
                });
            }
            if sub == 46 {
                let Some(run) = self.current.as_mut() else {
                    return Ok(RunOutcome::default());
                };
                if run.instance_id != id
                    || !matches!(run.state, RunState::Won | RunState::Lost)
                    || run.wheel_entry.is_none()
                    || run.wheel_stopped
                {
                    return Ok(RunOutcome::default());
                }
                run.wheel_stopped = true;
                return Ok(RunOutcome {
                    accepted: true,
                    changed: true,
                    state: Some(run.state),
                    ..Default::default()
                });
            }
            let Some(run) = self.current.as_ref() else {
                return Ok(RunOutcome::default());
            };
            if run.instance_id != id {
                return Ok(RunOutcome::default());
            }
            return match sub {
                5 => self.start(id),
                // T765: only a started card run asks for a ticket; anything else ignores it.
                37 if run.spec.mode == RunMode::CardClient
                    && matches!(run.state, RunState::Starting | RunState::Playing) =>
                {
                    Ok(RunOutcome {
                        accepted: true,
                        state: Some(run.state),
                        card_ticket: Some(CardTicketRequest {
                            instance_id: id,
                            game_id: run.spec.game_id,
                        }),
                        ..Default::default()
                    })
                }
                6 if matches!(
                    run.state,
                    RunState::Offered | RunState::Starting | RunState::Playing
                ) =>
                {
                    let reply = state_remove_packet(id, run.spec.game_id);
                    self.retire(RunState::Cancelled);
                    Ok(RunOutcome {
                        accepted: true,
                        changed: true,
                        state: Some(RunState::Cancelled),
                        replies: vec![reply],
                        ..Default::default()
                    })
                }
                _ => Ok(RunOutcome::default()),
            };
        }

        if packet == [26, 0, 42, 0] {
            let Some(state) = self.current.as_ref().map(|run| run.state) else {
                return Ok(RunOutcome::default());
            };
            if matches!(state, RunState::Won | RunState::Lost) {
                self.retire(RunState::Closed);
                return Ok(RunOutcome {
                    accepted: true,
                    changed: true,
                    state: Some(RunState::Closed),
                    ..Default::default()
                });
            }
            if state == RunState::Offered {
                self.retire(RunState::Cancelled);
                return Ok(RunOutcome {
                    accepted: true,
                    changed: true,
                    state: Some(RunState::Cancelled),
                    ..Default::default()
                });
            }
            return Ok(RunOutcome::default());
        }

        if packet.len() == 12
            && u16::from_le_bytes([packet[0], packet[1]]) == FAMILY_ENCOUNTER
            && u16::from_le_bytes([packet[2], packet[3]]) == 109
        {
            let encounter_id = read_i32(packet, 4);
            let instance_id = read_i32(packet, 8);
            let Some(run) = self.current.as_ref() else {
                return Ok(RunOutcome::default());
            };
            let RunMode::Encounter {
                encounter_id: expected_encounter,
                ..
            } = &run.spec.mode
            else {
                return Ok(RunOutcome::default());
            };
            if encounter_id != 0 || instance_id != run.instance_id || *expected_encounter <= 0 {
                return Ok(RunOutcome::default());
            }
            if matches!(run.state, RunState::Won | RunState::Lost) {
                self.retire(RunState::Closed);
                return Ok(RunOutcome {
                    accepted: true,
                    changed: true,
                    state: Some(RunState::Closed),
                    ..Default::default()
                });
            }
            if matches!(
                run.state,
                RunState::Offered | RunState::Starting | RunState::Playing
            ) {
                self.retire(RunState::Cancelled);
                return Ok(RunOutcome {
                    accepted: true,
                    changed: true,
                    state: Some(RunState::Cancelled),
                    ..Default::default()
                });
            }
        }
        Ok(RunOutcome::default())
    }

    /// Cancel only a pre-win run after session loss. (R) No response, grant or
    /// activity material mutation is performed by this shared lifecycle.
    pub fn disconnect(&mut self) -> RunOutcome {
        if self.current.as_ref().is_some_and(|run| {
            matches!(
                run.state,
                RunState::Offered | RunState::Starting | RunState::Playing
            )
        }) {
            self.retire(RunState::Cancelled);
            return RunOutcome {
                accepted: true,
                changed: true,
                state: Some(RunState::Cancelled),
                ..Default::default()
            };
        }
        RunOutcome::default()
    }

    /// Store the owning activity's current authoritative score while playing.
    pub fn score<A: Activity>(&mut self, instance_id: i32, activity: &A) -> RunOutcome {
        let Some(run) = self.current.as_mut() else {
            return RunOutcome::default();
        };
        if run.instance_id != instance_id || run.state != RunState::Playing {
            return RunOutcome::default();
        }
        let score = activity.score();
        if i32::try_from(score).is_err() || run.score == score {
            return RunOutcome::default();
        }
        run.score = score;
        RunOutcome {
            accepted: true,
            changed: true,
            state: Some(RunState::Playing),
            ..Default::default()
        }
    }

    /// Send the server-selected prize entry before the client spins. The
    /// later 39/46 packet only reports the animation stop and never pays.
    pub fn choose_wheel_prize(
        &mut self,
        instance_id: i32,
        entry: RewardBundleRow,
    ) -> io::Result<RunOutcome> {
        let Some(run) = self.current.as_ref() else {
            return Ok(RunOutcome::default());
        };
        if run.instance_id != instance_id
            || run.state != RunState::Playing
            || run.wheel_entry.is_some()
        {
            return Ok(RunOutcome::default());
        }
        let packet = wheel_prize_packet(instance_id, run.spec.game_id, &entry);
        let run = self.current.as_mut().unwrap();
        run.wheel_entry = Some(WheelEntry {
            type_id: entry.type_id,
            body: entry.body,
        });
        Ok(RunOutcome {
            accepted: true,
            changed: true,
            state: Some(RunState::Playing),
            replies: vec![packet],
            ..Default::default()
        })
    }

    /// Settle one activity result. Win grants use a stable run-derived key and
    /// losses cannot carry one. A win must carry its grant: an activity whose
    /// win legitimately pays nothing (a content zero award or a reached daily
    /// cap) calls [`Self::finish_win_without_reward`] instead. Both returned
    /// snapshots require one save.
    pub fn finish<A: Activity>(
        &mut self,
        instance_id: i32,
        activity: &A,
        settlement: Option<(&RewardState, &RewardGrant)>,
    ) -> io::Result<RunOutcome> {
        self.finish_inner(instance_id, activity, settlement, false)
    }

    /// Settle a won run that pays nothing. The named entry point keeps
    /// [`Self::finish`] strict, so a caller cannot lose a win's grant by
    /// forgetting it; `unpaid_win` records the explicit zero-award decision.
    pub fn finish_win_without_reward<A: Activity>(
        &mut self,
        instance_id: i32,
        activity: &A,
    ) -> io::Result<RunOutcome> {
        self.finish_inner(instance_id, activity, None, true)
    }

    fn finish_inner<A: Activity>(
        &mut self,
        instance_id: i32,
        activity: &A,
        settlement: Option<(&RewardState, &RewardGrant)>,
        unpaid_win: bool,
    ) -> io::Result<RunOutcome> {
        let Some(run) = self.current.as_ref() else {
            return Ok(RunOutcome::default());
        };
        if run.instance_id != instance_id || run.state != RunState::Playing {
            return Ok(RunOutcome::default());
        }
        let Some(result) = activity.finish() else {
            return Ok(RunOutcome::default());
        };
        if !result.won && settlement.is_some() {
            return Err(invalid("a loss cannot carry a minigame grant"));
        }
        if result.won && settlement.is_none() && !unpaid_win {
            return Err(invalid("a won minigame must carry its grant"));
        }
        let score = activity.score();
        let score_i32 = i32::try_from(score).map_err(invalid)?;
        let mut replies = vec![payload_packet(
            instance_id,
            run.spec.game_id,
            if result.won {
                "OnGameWonMsg"
            } else {
                "OnGameLostMsg"
            },
        )?];
        replies.extend(result_packets(
            instance_id,
            run.spec.game_id,
            result.won,
            result.stars,
            result.elapsed_seconds,
            score_i32,
        )?);

        let (reward_state, reward_packets) = if let Some((state, grant)) = settlement {
            let key = format!("minigame-run:{instance_id}");
            let (next, packets) = apply_grant(state, grant, &key)?;
            (Some(next), packets)
        } else {
            (None, Vec::new())
        };

        let terminal = if result.won {
            RunState::Won
        } else {
            RunState::Lost
        };
        self.current.as_mut().unwrap().state = terminal;
        self.current.as_mut().unwrap().score = score;
        Ok(RunOutcome {
            accepted: true,
            changed: true,
            state: Some(terminal),
            replies,
            reward_packets,
            reward_state,
            card_ticket: None,
        })
    }

    fn start(&mut self, instance_id: i32) -> io::Result<RunOutcome> {
        let Some(run) = self.current.as_ref() else {
            return Ok(RunOutcome::default());
        };
        if run.instance_id != instance_id || run.state != RunState::Offered {
            return Ok(RunOutcome::default());
        }
        let replies = match &run.spec.mode {
            RunMode::Flash { movie } => vec![
                start_ack_packet(run.instance_id, run.spec.game_id),
                flash_launch_packet(movie),
            ],
            RunMode::Encounter {
                encounter_id,
                player_guid,
            } => vec![
                encounter_started_packet(*encounter_id, run.instance_id, *player_guid),
                // The 39/16 join never flags its state as an encounter (`009bfdb0` passes 0
                // to `009beb70`), so 41/2's `009b9500` skips it. 39/17 (`009b9550`) is what
                // starts it and shows the game HUD (`009b6aa0`). (R) 41/2 stays as before: on
                // such a state it only sets client byte +0x780 (`008a9200`) and a loader flag
                // (`008a8a30`).
                start_ack_packet(run.instance_id, run.spec.game_id),
            ],
            // The client's card manager starts on 39/17 and asks for its ticket with 39/37.
            RunMode::CardClient => vec![start_ack_packet(run.instance_id, run.spec.game_id)],
        };
        self.current.as_mut().unwrap().state = RunState::Starting;
        Ok(RunOutcome {
            accepted: true,
            changed: true,
            state: Some(RunState::Starting),
            replies,
            ..Default::default()
        })
    }

    /// Activity readiness completes the start handshake. It never grants.
    pub fn started(&mut self, instance_id: i32) -> RunOutcome {
        let Some(run) = self.current.as_mut() else {
            return RunOutcome::default();
        };
        if run.instance_id != instance_id || run.state != RunState::Starting {
            return RunOutcome::default();
        }
        run.state = RunState::Playing;
        RunOutcome {
            accepted: true,
            changed: true,
            state: Some(RunState::Playing),
            ..Default::default()
        }
    }

    fn retire(&mut self, state: RunState) {
        if let Some(mut run) = self.current.take() {
            run.state = state;
            self.last = Some(run);
        }
    }
}

/// 39/16 info record: reader `009bfc50` / body reader `009bdd70`.
fn info_packet(instance_id: i32, spec: &RunSpec) -> io::Result<Vec<u8>> {
    let mut bytes = family39_header(16, instance_id, spec.game_id);
    for value in [
        spec.name_id,
        spec.icon_id,
        spec.description_id,
        spec.difficulty,
        spec.profile_type,
        spec.type_id,
    ] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.push(u8::from(spec.members_only));
    bytes.extend(&spec.reward_bundles);
    let goal_count =
        i32::try_from(spec.objectives.len() / INFO_OBJECTIVE_BYTES).map_err(invalid)?;
    bytes.extend(goal_count.to_le_bytes());
    bytes.extend(&spec.objectives);
    bytes.extend(spec.flags);
    bytes.extend(
        i32::try_from(spec.preload_file.len())
            .map_err(invalid)?
            .to_le_bytes(),
    );
    bytes.extend(spec.preload_file.as_bytes());
    // Remaining ids/flags are partly unnamed by the client. Values match the
    // accepted cooking/smelting/forging writers; treating them as defaults is (R).
    bytes.extend(1_i32.to_le_bytes());
    bytes.push(1);
    bytes.extend(spec.game_id.to_le_bytes());
    bytes.extend([0; 4]);
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    Ok(bytes)
}

/// Server acknowledgement 39/17; client reader `009b9550`.
fn start_ack_packet(instance_id: i32, game_id: i32) -> Vec<u8> {
    family39_header(17, instance_id, game_id)
}

/// T765: 39/0x28 `TradingCardStartGamePacket` (reader `009b7f30`): the run's header, then the
/// 64-bit launch ticket as two i32 halves, low then high. `DispatchMiniGamePacket`
/// (`009bf890`) finds the run by instance id and, for type 16, hands both halves to the card
/// manager (slot 25 `00ad32e0`: +0x2ba98, +0x2ba9c), which passes them to the DLL as
/// `--ticket=%llu`.
pub fn card_start_packet(request: CardTicketRequest, ticket: u64) -> Vec<u8> {
    let mut bytes = family39_header(0x28, request.instance_id, request.game_id);
    bytes.extend((ticket as u32).to_le_bytes());
    bytes.extend(((ticket >> 32) as u32).to_le_bytes());
    bytes
}

/// Client Flash launch 26/12; handler `00aa0a10` and `StartFlashMiniGameUi` `009bd650`.
fn flash_launch_packet(movie: &str) -> Vec<u8> {
    let mut bytes = [FAMILY_COMMAND.to_le_bytes(), 12_u16.to_le_bytes()].concat();
    for value in ["MiniGameFlash", movie] {
        bytes.extend((value.len() as i32).to_le_bytes());
        bytes.extend(value.as_bytes());
    }
    bytes.push(0);
    bytes
}

/// Encounter started packet 41/2; client dispatcher `00aa36c0`, body reader `00a9f760`.
fn encounter_started_packet(encounter_id: i32, instance_id: i32, player_guid: u64) -> Vec<u8> {
    let mut bytes = encounter_packet(2, encounter_id, instance_id);
    bytes.extend(player_guid.to_le_bytes());
    bytes.push(0);
    bytes
}

/// Encounter header `u16 41 | u16 sub | i32 encounter | i32 instance` (reader `008d6690`); alone
/// it is the whole 41/107 body (`00a9f990`).
fn encounter_packet(sub: u16, encounter_id: i32, instance_id: i32) -> Vec<u8> {
    let mut bytes = [FAMILY_ENCOUNTER.to_le_bytes(), sub.to_le_bytes()].concat();
    bytes.extend(encounter_id.to_le_bytes());
    bytes.extend(instance_id.to_le_bytes());
    bytes
}

/// Movie command 39/14: NUL-inclusive length, read by `009b89e0`.
fn payload_packet(instance_id: i32, game_id: i32, message: &str) -> io::Result<Vec<u8>> {
    let mut bytes = family39_header(14, instance_id, game_id);
    let length = i32::try_from(message.len() + 1).map_err(invalid)?;
    bytes.extend(length.to_le_bytes());
    bytes.extend(message.as_bytes());
    bytes.push(0);
    Ok(bytes)
}

/// Ordered terminal sequence: 39/18 (`009ad480`), 39/47 (`009bb030` /
/// row reader `009b2b30`), then 39/19 (`009bf190`).
fn result_packets(
    instance_id: i32,
    game_id: i32,
    won: bool,
    stars: u32,
    elapsed_seconds: u32,
    score: i32,
) -> io::Result<Vec<Vec<u8>>> {
    let stars = i32::try_from(stars).map_err(invalid)?;
    let elapsed = i32::try_from(elapsed_seconds).map_err(invalid)?;
    let mut game_over = family39_header(18, instance_id, game_id);
    game_over.push(u8::from(won));
    for value in [stars, elapsed, score] {
        game_over.extend(value.to_le_bytes());
    }

    let mut scores = family39_header(47, instance_id, game_id);
    scores.extend(1_i32.to_le_bytes());
    let name = b"scoreTotalScore";
    scores.extend((name.len() as i32).to_le_bytes());
    scores.extend(name);
    for value in [-1_i32, 4, -1, -1, score] {
        scores.extend(value.to_le_bytes());
    }
    Ok(vec![
        game_over,
        scores,
        state_remove_packet(instance_id, game_id),
    ])
}

/// The client's `39/19` result-screen reader is `009bf190`; its automatic
/// `39/7` acknowledgement is not the player's result Close. Every minigame
/// terminal path (results, cancel, lobby timeout) reuses this one header writer.
pub(crate) fn state_remove_packet(instance_id: i32, game_id: i32) -> Vec<u8> {
    family39_header(19, instance_id, game_id)
}

/// Prize-wheel landing entry 39/45; the client compares it to RewardPreview
/// in `StopMiniGameLootWheelAt` (`009b6da0`). The caller supplies the exact
/// entry body decoded by the applicable client reward-entry reader.
fn wheel_prize_packet(instance_id: i32, game_id: i32, entry: &RewardBundleRow) -> Vec<u8> {
    let mut bytes = family39_header(45, instance_id, game_id);
    bytes.extend(entry.type_id.to_le_bytes());
    bytes.extend(&entry.body);
    bytes
}

fn family39_header(sub: u8, instance_id: i32, game_id: i32) -> Vec<u8> {
    let mut bytes = FAMILY_MINIGAME.to_le_bytes().to_vec();
    bytes.push(sub);
    for value in [instance_id, -1, game_id] {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}

fn validate_asset_name(name: &str) -> io::Result<()> {
    if name.is_empty()
        || name.len() > 128
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_.-".contains(&byte))
    {
        return Err(invalid("invalid minigame movie name"));
    }
    Ok(())
}

fn read_i32(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn invalid(error: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}
