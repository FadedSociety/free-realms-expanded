//! Field-kill coin and job-XP rewards: the bounded level-band table and the
//! one payout each credited player receives for a server-authoritative kill.
//!
//! The table is separate from `field_enemies.json` so that actor rows keep
//! their existing schema and limits; a missing table is an empty table for old
//! test packages, while a malformed or out-of-bounds table refuses to load.
use super::*;

const FILE: &str = "field_kill_rewards.json";
const MAX_BYTES: usize = 1024 * 1024;
const MAX_BANDS: usize = 20;
const MAX_OVERRIDES: usize = 64;
const MAX_REWARD: u32 = 1000;
const MAX_LEVEL: u32 = 20;

/// (R) Per-kill coins and active-job XP, looked up by definition then level.
#[derive(Debug, Default)]
pub(super) struct FieldKillRewards {
    /// Sorted by `min_level`; bands never overlap.
    bands: Vec<LevelReward>,
    /// Sorted by `definition_id`; unique.
    overrides: Vec<DefinitionReward>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LevelReward {
    min_level: u32,
    max_level: u32,
    /// Zero means this part of the payout is empty.
    coins: u32,
    job_xp: u32,
    source: Vec<crate::content::SourceTag>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DefinitionReward {
    definition_id: i32,
    coins: u32,
    job_xp: u32,
    source: Vec<crate::content::SourceTag>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RewardFile {
    schema_version: u32,
    level_rewards: Vec<LevelReward>,
    #[serde(default)]
    definition_overrides: Vec<DefinitionReward>,
}

impl FieldKillRewards {
    /// Load `field_kill_rewards.json` from a content directory. A missing file
    /// is an empty table (old test packages); a bad file is an error.
    pub(super) fn load(directory: impl AsRef<Path>) -> io::Result<Self> {
        let path = directory.as_ref().join(FILE);
        // Refuse an oversized package before allocating it. `parse` keeps the
        // same bound for bytes handed in directly.
        match fs::metadata(&path) {
            Ok(metadata) if metadata.len() > MAX_BYTES as u64 => {
                return Err(invalid("field kill reward content too large"));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(error),
        }
        Self::parse(&fs::read(path)?)
    }

    fn parse(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() > MAX_BYTES {
            return Err(invalid("field kill reward content too large"));
        }
        let file: RewardFile =
            serde_json::from_slice(bytes).map_err(|e| invalid(&e.to_string()))?;
        if file.schema_version != 1
            || file.level_rewards.len() > MAX_BANDS
            || file.definition_overrides.len() > MAX_OVERRIDES
        {
            return Err(invalid("unsupported field kill reward content"));
        }
        let mut bands = file.level_rewards;
        for band in &bands {
            if band.min_level < 1
                || band.max_level > MAX_LEVEL
                || band.min_level > band.max_level
                || band.coins > MAX_REWARD
                || band.job_xp > MAX_REWARD
            {
                return Err(invalid("invalid field kill reward band"));
            }
            validate_source(&band.source)?;
        }
        bands.sort_by_key(|band| band.min_level);
        if bands
            .windows(2)
            .any(|pair| pair[0].max_level >= pair[1].min_level)
        {
            return Err(invalid("overlapping field kill reward bands"));
        }
        let mut overrides = file.definition_overrides;
        for row in &overrides {
            if row.definition_id <= 0 || row.coins > MAX_REWARD || row.job_xp > MAX_REWARD {
                return Err(invalid("invalid field kill reward override"));
            }
            validate_source(&row.source)?;
        }
        overrides.sort_by_key(|row| row.definition_id);
        if overrides
            .windows(2)
            .any(|pair| pair[0].definition_id == pair[1].definition_id)
        {
            return Err(invalid("duplicate field kill reward override"));
        }
        Ok(Self { bands, overrides })
    }

    /// The override first, then the band containing `level`. A level outside
    /// 1..=20, or one no band covers, pays nothing rather than guessing.
    pub(super) fn reward_for(&self, definition_id: i32, level: u32) -> Option<(u32, u32)> {
        if !(1..=MAX_LEVEL).contains(&level) {
            return None;
        }
        if let Ok(index) = self
            .overrides
            .binary_search_by_key(&definition_id, |row| row.definition_id)
        {
            let row = &self.overrides[index];
            return Some((row.coins, row.job_xp));
        }
        self.bands
            .iter()
            .find(|band| (band.min_level..=band.max_level).contains(&level))
            .map(|band| (band.coins, band.job_xp))
    }
}

/// Every row carries provenance, and a reconstructed row starts with `(R)`.
fn validate_source(source: &[crate::content::SourceTag]) -> io::Result<()> {
    if source.is_empty() {
        return Err(invalid("field kill reward row needs provenance"));
    }
    for tag in source {
        let evidence = tag.evidence();
        if evidence.trim().is_empty() || tag.is_reconstructed() && !evidence.starts_with("(R)") {
            return Err(invalid("invalid field kill reward provenance"));
        }
    }
    Ok(())
}

impl EncounterEngine {
    /// Pay one field kill: this engine's player at once, every other credited
    /// participant on their next shared tick. The pair is carried with the
    /// credit, so a later content reload can neither change nor drop it.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn credit_field_kill(
        &mut self,
        table: Option<&str>,
        reward: Option<(u32, u32)>,
        enemy: u64,
        at: [f32; 3],
        now_ms: u64,
        hitters: BTreeMap<u64, u64>,
        engaged: Option<u64>,
        out: &mut EncounterOutcome,
    ) {
        if table.is_none() && reward.is_none() {
            return;
        }
        let kill = (enemy, now_ms);
        let Some(killer) = self.shared_sender else {
            if let Some(table) = table {
                self.drop_power_ups(table, kill, at, now_ms, 1, out);
            }
            pay_field_reward(reward, out);
            return;
        };
        let window = self
            .power_ups
            .as_ref()
            .map_or(30_000, |p| p.participation_window_ms());
        let mut who: BTreeSet<u64> = hitters
            .into_iter()
            .filter(|&(_, last)| now_ms.saturating_sub(last) <= window)
            .map(|(player, _)| player)
            .collect();
        who.extend(engaged.filter(|&g| g != 0));
        who.insert(killer);
        let participants = who.len();
        if let Some(table) = table {
            self.drop_power_ups(table, kill, at, now_ms, participants, out);
        }
        pay_field_reward(reward, out);
        for player in who.into_iter().filter(|&p| p != killer) {
            let waiting = self.ledger.credits.entry(player).or_default();
            if waiting.len() >= MAX_CREDITS {
                waiting.remove(0);
            }
            waiting.push(KillCredit {
                kill,
                table: table.map(str::to_owned),
                at,
                participants,
                reward,
                expires_ms: now_ms.saturating_add(CREDIT_TTL_MS),
            });
        }
    }

    /// Pay the credits other players earned for this engine's player. A credit
    /// is removed when it is claimed or expires, so a second claim pays nothing.
    pub(super) fn claim_kill_credits(
        &mut self,
        player: u64,
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) {
        for credit in self.ledger.credits.remove(&player).unwrap_or_default() {
            if now_ms < credit.expires_ms {
                if let Some(table) = &credit.table {
                    self.drop_power_ups(
                        table,
                        credit.kill,
                        credit.at,
                        now_ms,
                        credit.participants,
                        out,
                    );
                }
                pay_field_reward(credit.reward, out);
            }
        }
    }
}

/// A zero row is no payout; the reward never carries items in this slice. A
/// non-zero pair becomes one `EncounterReward`, so the payout rides the existing
/// replies: 38/19 (one i32 coin total, client ctor `0x0089e670`) and 38/14 (six
/// i32 profile/rank/progress rows, client reader `0x008d86e0`).
fn pay_field_reward(reward: Option<(u32, u32)>, out: &mut EncounterOutcome) {
    if let Some((coins, job_xp)) = reward {
        if coins > 0 || job_xp > 0 {
            out.rewards.push(EncounterReward {
                coins,
                items: Vec::new(),
                job_xp,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source() -> serde_json::Value {
        serde_json::json!([{"kind":"reconstructed","evidence":"(R) local test balance"}])
    }

    fn table(level_rewards: serde_json::Value, overrides: serde_json::Value) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "schema_version": 1,
            "level_rewards": level_rewards,
            "definition_overrides": overrides,
        }))
        .unwrap()
    }

    fn band(min: u32, max: u32, coins: u32, job_xp: u32) -> serde_json::Value {
        serde_json::json!({
            "min_level": min,
            "max_level": max,
            "coins": coins,
            "job_xp": job_xp,
            "source": source(),
        })
    }

    #[test]
    fn bands_pay_their_levels_and_the_override_wins() {
        let bytes = table(
            serde_json::json!([band(1, 2, 2, 5), band(3, 4, 4, 10)]),
            serde_json::json!([{
                "definition_id": 3156,
                "coins": 1,
                "job_xp": 3,
                "source": source(),
            }]),
        );
        let rewards = FieldKillRewards::parse(&bytes).unwrap();
        assert_eq!(rewards.reward_for(930050, 1), Some((2, 5)));
        assert_eq!(rewards.reward_for(930050, 3), Some((4, 10)));
        assert_eq!(rewards.reward_for(3156, 3), Some((1, 3)));
        assert_eq!(rewards.reward_for(930050, 0), None);
        assert_eq!(rewards.reward_for(930050, 21), None);
        assert_eq!(rewards.reward_for(930050, 5), None);
    }

    #[test]
    fn bad_rows_are_refused_at_load_time() {
        let bytes = table(
            serde_json::json!([band(1, 2, 2, 5), band(2, 3, 4, 10)]),
            serde_json::json!([]),
        );
        assert!(
            FieldKillRewards::parse(&bytes).is_err(),
            "overlapping bands"
        );
        let bytes = table(serde_json::json!([band(0, 2, 2, 5)]), serde_json::json!([]));
        assert!(FieldKillRewards::parse(&bytes).is_err(), "min_level 0");
        let bytes = table(
            serde_json::json!([band(1, 20, 1001, 5)]),
            serde_json::json!([]),
        );
        assert!(FieldKillRewards::parse(&bytes).is_err(), "coins over cap");
        let duplicate = serde_json::json!([{
            "definition_id": 3156,
            "coins": 1,
            "job_xp": 3,
            "source": source(),
        }]);
        let bytes = table(serde_json::json!([]), duplicate.clone());
        assert!(FieldKillRewards::parse(&bytes).is_ok());
        let bytes = table(
            serde_json::json!([]),
            serde_json::json!([duplicate[0], duplicate[0]]),
        );
        assert!(
            FieldKillRewards::parse(&bytes).is_err(),
            "duplicate override"
        );
        let bytes = table(
            serde_json::json!([{
                "min_level": 1,
                "max_level": 2,
                "coins": 0,
                "job_xp": 0,
                "source": [],
            }]),
            serde_json::json!([]),
        );
        assert!(FieldKillRewards::parse(&bytes).is_err(), "no provenance");
        let bytes = table(
            serde_json::json!([{
                "min_level": 1,
                "max_level": 2,
                "coins": 0,
                "job_xp": 0,
                "source": [{"kind":"reconstructed","evidence":"reconstructed"}],
            }]),
            serde_json::json!([]),
        );
        assert!(
            FieldKillRewards::parse(&bytes).is_err(),
            "a reconstructed row starts with (R)"
        );
    }

    #[test]
    fn a_missing_table_is_empty_and_the_shipped_content_matches_its_actors() {
        let missing =
            std::env::temp_dir().join(format!("osfr-field-rewards-missing-{}", std::process::id()));
        assert!(FieldKillRewards::load(&missing).unwrap().bands.is_empty());
        let shipped = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
        let rewards = FieldKillRewards::load(&shipped).unwrap();
        // Every shipped actor must reach a row. A level-14+ actor or an unknown
        // definition would pay nothing, and this fails until the table decides.
        let field: serde_json::Value =
            serde_json::from_slice(&fs::read(shipped.join("field_enemies.json")).unwrap()).unwrap();
        let actors = field["enemies"]
            .as_array()
            .unwrap()
            .iter()
            .map(|actor| {
                (
                    actor["definition_id"].as_i64().unwrap() as i32,
                    actor["level"].as_u64().unwrap_or(1) as u32,
                )
            })
            .collect::<Vec<_>>();
        for (definition_id, level) in &actors {
            assert!(
                rewards.reward_for(*definition_id, *level).is_some(),
                "shipped field actor {definition_id} at level {level} has no reward row"
            );
        }
        // The shipped levels 1/3/5/7/9/10 exercise every band; the level-5
        // Hooligan Wolves stay on the definition override (2/5).
        for (definition_id, level, expected) in [
            (940102, 1, (2, 5)),
            (930050, 3, (4, 10)),
            (3156, 1, (2, 5)),
            (3156, 5, (2, 5)),
            (930060, 7, (7, 15)),
            (930025, 9, (12, 25)),
            (930053, 10, (12, 25)),
        ] {
            assert!(
                actors.contains(&(definition_id, level)),
                "the shipped field content no longer has {definition_id} at level {level}"
            );
            assert_eq!(
                rewards.reward_for(definition_id, level),
                Some(expected),
                "{definition_id} level {level}"
            );
        }
    }
}
