//! T552: the schedules fixed at load from the placed content actors: conversation
//! groups, idle loops and look-at actors, and the `due` rule of each.

use super::*;

#[derive(Clone, Debug)]
pub(super) struct ConversationGroup {
    pub(super) members: Vec<ConversationMember>,
    cycle_ms: u64,
    phase_ms: u64,
}

#[derive(Clone, Debug)]
pub(super) struct ConversationMember {
    pub(super) guid: u64,
    model_id: u32,
    animation_ids: Vec<i32>,
    /// Turn offset in each cycle: the sum of the gaps of the members before this one
    /// in the stable GUID order.
    offset_ms: u64,
}

#[derive(Clone, Debug)]
pub(super) struct IdleSchedule {
    pub(super) model_id: u32,
    animation_ids: Vec<i32>,
    interval_ms: u64,
    phase_ms: u64,
}

#[derive(Clone, Debug)]
pub(super) struct LookAtSchedule {
    pub(super) model_id: u32,
    pub(super) radius: f32,
    interval_ms: u64,
    phase_ms: u64,
    pub(super) position: [f32; 3],
}

/// A talker waiting to be grouped (its row's conversation fields).
struct Talker {
    guid: u64,
    model_id: u32,
    position: [f32; 3],
    animation_ids: Vec<i32>,
    min_gap_ms: u64,
    max_gap_ms: u64,
}

/// The connected components, of at least two, of the graph that links two talkers
/// within `radius_sq` of each other in XZ: a talker joins a group through any chain
/// of neighbours, not only the group's first member. `talkers` must be sorted by
/// GUID; each component lists indices in that order.
fn components(talkers: &[Talker], radius_sq: f32) -> Vec<Vec<usize>> {
    fn root(parent: &mut [usize], mut node: usize) -> usize {
        while parent[node] != node {
            parent[node] = parent[parent[node]];
            node = parent[node];
        }
        node
    }
    let mut parent: Vec<usize> = (0..talkers.len()).collect();
    for (a, left) in talkers.iter().enumerate() {
        for (b, right) in talkers.iter().enumerate().skip(a + 1) {
            let dx = left.position[0] - right.position[0];
            let dz = left.position[2] - right.position[2];
            if dx * dx + dz * dz <= radius_sq {
                let (left_root, right_root) = (root(&mut parent, a), root(&mut parent, b));
                parent[left_root.max(right_root)] = left_root.min(right_root);
            }
        }
    }
    let mut by_root: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for node in 0..talkers.len() {
        by_root
            .entry(root(&mut parent, node))
            .or_default()
            .push(node);
    }
    by_root
        .into_values()
        .filter(|group| group.len() > 1)
        .collect()
}

impl NpcAmbient {
    /// Fix the conversation groups, idle actors and look-at actors from the
    /// placed content actors. Every position is finite-checked; an excluded actor
    /// is absent from `actors`. Season is not applied here: each firing is gated by
    /// the live model lookup.
    /// `(R)` Grouping is the configured XZ radius (6 m in the shipped table); gaps
    /// come from each member's row with a GUID-stable hash, and the n-th member of
    /// the stable GUID order speaks after the gaps of the n members before it.
    pub(super) fn place_actors(&mut self, actors: &[(u64, u32, [f32; 3])]) {
        let mut talkers = Vec::new();
        for &(guid, model_id, position) in actors {
            if !position.iter().all(|value| value.is_finite()) {
                continue;
            }
            let Some(row) = self.rows.get(&model_id) else {
                continue;
            };
            if !row.conversation_animation_ids.is_empty() {
                talkers.push(Talker {
                    guid,
                    model_id,
                    position,
                    animation_ids: row.conversation_animation_ids.clone(),
                    min_gap_ms: row.conversation_min_gap_ms,
                    max_gap_ms: row.conversation_max_gap_ms,
                });
            }
            if !row.idle_animation_ids.is_empty() {
                self.idles.insert(
                    guid,
                    IdleSchedule {
                        model_id,
                        animation_ids: row.idle_animation_ids.clone(),
                        interval_ms: row.idle_interval_ms,
                        phase_ms: mix64(guid ^ IDLE_SALT) % row.idle_interval_ms,
                    },
                );
            }
            if row.look_at_radius > 0.0 {
                self.look_at.insert(
                    guid,
                    LookAtSchedule {
                        model_id,
                        radius: row.look_at_radius,
                        interval_ms: row.look_at_interval_ms,
                        phase_ms: mix64(guid ^ LOOK_AT_SALT) % row.look_at_interval_ms,
                        position,
                    },
                );
            }
        }
        talkers.sort_by_key(|talker| talker.guid);

        self.conversation_groups = components(&talkers, self.conversation_radius_sq)
            .into_iter()
            .map(|members| {
                let gaps: Vec<u64> = members
                    .iter()
                    .map(|&index| {
                        let talker = &talkers[index];
                        let span = talker.max_gap_ms - talker.min_gap_ms + 1;
                        talker.min_gap_ms + mix64(talker.guid ^ CONVERSATION_GAP_SALT) % span
                    })
                    .collect();
                let cycle_ms = gaps.iter().copied().fold(0_u64, u64::saturating_add).max(1);
                let mut offset_ms = 0_u64;
                ConversationGroup {
                    phase_ms: mix64(talkers[members[0]].guid ^ CONVERSATION_PHASE_SALT) % cycle_ms,
                    cycle_ms,
                    members: members
                        .iter()
                        .zip(&gaps)
                        .map(|(&index, &gap)| {
                            let talker = &talkers[index];
                            let member = ConversationMember {
                                guid: talker.guid,
                                model_id: talker.model_id,
                                animation_ids: talker.animation_ids.clone(),
                                offset_ms,
                            };
                            offset_ms = offset_ms.saturating_add(gap);
                            member
                        })
                        .collect(),
                }
            })
            .collect();
        self.talking = self
            .conversation_groups
            .iter()
            .flat_map(|group| group.members.iter().map(|member| member.guid))
            .collect();
    }
}

impl ConversationGroup {
    /// The latest firing in `(from_ms, to_ms]`, as `(guid, model id, animation
    /// id)`. Equal firing times (zero gaps) break toward the later member, so the
    /// speaker is still deterministic.
    pub(super) fn due(&self, from_ms: u64, to_ms: u64) -> Option<(u64, u32, i32)> {
        let mut best: Option<(u64, usize, u64)> = None;
        for (index, member) in self.members.iter().enumerate() {
            let start = self.phase_ms.checked_add(member.offset_ms)?;
            if to_ms < start {
                continue;
            }
            let cycle = (to_ms - start) / self.cycle_ms;
            let at = start.checked_add(cycle.checked_mul(self.cycle_ms)?)?;
            if at <= from_ms {
                continue;
            }
            if best.is_none_or(|(best_at, best_index, _)| (at, index) > (best_at, best_index)) {
                best = Some((at, index, cycle));
            }
        }
        let (_, index, cycle) = best?;
        let member = &self.members[index];
        let index = mix64(member.guid ^ CONVERSATION_ID_SALT ^ cycle.wrapping_mul(FIRING_SALT))
            % member.animation_ids.len() as u64;
        Some((
            member.guid,
            member.model_id,
            member.animation_ids[index as usize],
        ))
    }
}

impl IdleSchedule {
    /// The latest idle firing in `(from_ms, to_ms]`, cycling from index 0 in a
    /// GUID-stable phase.
    pub(super) fn due(&self, from_ms: u64, to_ms: u64) -> Option<i32> {
        let last = self
            .phase_ms
            .checked_add(to_ms.checked_sub(self.phase_ms)? / self.interval_ms * self.interval_ms)?;
        if last <= from_ms {
            return None;
        }
        let firing = (last - self.phase_ms) / self.interval_ms;
        let index = firing % self.animation_ids.len() as u64;
        Some(self.animation_ids[index as usize])
    }
}

impl LookAtSchedule {
    /// Whether a look-at firing falls in `(from_ms, to_ms]`.
    pub(super) fn due(&self, from_ms: u64, to_ms: u64) -> bool {
        let Some(elapsed) = to_ms.checked_sub(self.phase_ms) else {
            return false;
        };
        self.phase_ms
            .checked_add(elapsed / self.interval_ms * self.interval_ms)
            .is_some_and(|last| last > from_ms)
    }
}
