//! Typed, capture-derived startup content. The snapshot excludes message 07;
//! the gateway keeps that character payload on its existing patch path.

use crate::{
    startup_catalogues as catalogues,
    startup_messages::{self, ZoneDetails},
    startup_world as world,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, io, path::Path};

const MAX_SNAPSHOT_BYTES: usize = 16 * 1024 * 1024;
const MAX_ROWS: usize = 100_000;

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StartupSnapshot {
    pub schema_version: u32,
    pub login: LoginSnapshot,
    pub character: CharacterSnapshot,
    pub world: WorldSnapshot,
    pub catalogues: CatalogueSnapshot,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoginSnapshot {
    pub result: u8,
    pub environment: String,
    pub zone: ZoneSnapshot,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ZoneSnapshot {
    pub name: String,
    pub zone_type: i32,
    pub tutorial: bool,
    pub sky: String,
    pub zone_id: u32,
    pub geometry_id: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CharacterSnapshot {
    // Capture values retained for the reference character; they are not
    // reconstructed health, mana, combat, quest, or stat policy.
    pub in_world_combat: bool,
    pub hit_points: [i32; 2],
    pub mana: [i32; 2],
    pub mana_flag: u8,
    pub journal_entries: Vec<[u32; 2]>,
    pub stats: Vec<StatSnapshot>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StatSnapshot {
    pub id: i32,
    /// Native reader value kind: 0 is i32; 1 is f32.
    pub kind: u32,
    /// Raw value bits preserve the captured f32 representation.
    pub value_bits: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorldSnapshot {
    pub quick_chat: Vec<world::QuickChatRow>,
    pub points_of_interest: Vec<world::PointOfInterest>,
    pub npc_spawns: Vec<world::NpcSpawn>,
    pub journal_definitions: world::JournalDefinitions,
    pub welcome_screen: world::WelcomeScreen,
    pub motd: world::Motd,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogueSnapshot {
    pub item_definitions: Vec<ItemDefinitionSnapshot>,
    pub coin_store: CoinStoreSnapshot,
    pub ability_set: AbilitySetSnapshot,
    pub effect_tags: Vec<EffectTagSnapshot>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ItemDefinitionSnapshot {
    pub id: i32,
    /// The source reader retains these serialized catalogue records as opaque bytes.
    pub serialized_hex: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CoinStoreSnapshot {
    pub static_items: Vec<CoinStoreEntrySnapshot>,
    pub dynamic_items: Vec<CoinStoreEntrySnapshot>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CoinStoreEntrySnapshot {
    pub id: i32,
    pub unknown_words: [u32; 3],
    pub unknown_flags: [u8; 3],
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AbilitySetSnapshot {
    pub id: i32,
    pub entries: Vec<AbilitySetEntrySnapshot>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AbilitySetEntrySnapshot {
    pub kind: i32,
    /// Kind-specific tails remain opaque until their field meanings are mapped.
    pub serialized_fields_hex: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EffectTagSnapshot {
    pub id: i32,
    pub definition_blob_hex: String,
}

impl StartupSnapshot {
    /// Empty but structurally valid content for synthetic gateway tests.
    pub fn empty() -> Self {
        Self {
            schema_version: 1,
            login: LoginSnapshot {
                result: 0,
                environment: String::new(),
                zone: ZoneSnapshot {
                    name: String::new(),
                    zone_type: 0,
                    tutorial: false,
                    sky: String::new(),
                    zone_id: 0,
                    geometry_id: 0,
                },
            },
            character: CharacterSnapshot {
                in_world_combat: false,
                hit_points: [0, 0],
                mana: [0, 0],
                mana_flag: 0,
                journal_entries: Vec::new(),
                stats: Vec::new(),
            },
            world: WorldSnapshot {
                quick_chat: Vec::new(),
                points_of_interest: Vec::new(),
                npc_spawns: (0..23).map(|_| world::NpcSpawn::default()).collect(),
                journal_definitions: world::JournalDefinitions {
                    regions: Vec::new(),
                    hubs: Vec::new(),
                    hub_quests: Vec::new(),
                    stickers: Vec::new(),
                },
                welcome_screen: world::WelcomeScreen {
                    show_welcome_screen: 0,
                    contents: Vec::new(),
                    claim_codes: Vec::new(),
                    seconds_since_last_login: 0,
                    starting_wallet_balance: 0,
                },
                motd: world::Motd {
                    first_string: String::new(),
                    second_string: String::new(),
                },
            },
            catalogues: CatalogueSnapshot {
                item_definitions: Vec::new(),
                coin_store: CoinStoreSnapshot {
                    static_items: Vec::new(),
                    dynamic_items: Vec::new(),
                },
                ability_set: AbilitySetSnapshot {
                    id: 0,
                    entries: Vec::new(),
                },
                effect_tags: vec![
                    EffectTagSnapshot {
                        id: 0,
                        definition_blob_hex: String::new(),
                    },
                    EffectTagSnapshot {
                        id: 0,
                        definition_blob_hex: String::new(),
                    },
                ],
            },
        }
    }

    pub fn load(path: &Path) -> io::Result<Self> {
        let metadata = fs::symlink_metadata(path)?;
        if !metadata.file_type().is_file() || metadata.len() > MAX_SNAPSHOT_BYTES as u64 {
            return Err(invalid("startup snapshot is not a bounded regular file"));
        }
        let bytes = fs::read(path)?;
        if bytes.len() > MAX_SNAPSHOT_BYTES {
            return Err(invalid("startup snapshot exceeds its size limit"));
        }
        let snapshot: Self =
            serde_json::from_slice(&bytes).map_err(|_| invalid("invalid startup snapshot JSON"))?;
        snapshot.validate()?;
        Ok(snapshot)
    }

    pub fn export_from_reference(reference: &Path, output: &Path) -> io::Result<Self> {
        let manifest: Value =
            serde_json::from_slice(&read_file(&reference.join("manifest.json"), 65_536)?)
                .map_err(|_| invalid("invalid gateway reference manifest JSON"))?;
        let character_guid = manifest["initial_state"]["guid"]
            .as_u64()
            .ok_or_else(|| invalid("reference manifest has no initial character GUID"))?;
        let snapshot = Self::parse_reference(reference, character_guid)?;
        snapshot.verify_capture(reference, character_guid)?;

        let lab_root = reference
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| invalid("reference directory must be under lab/references"))?;
        let expected_output = lab_root.join("content/startup-snapshot/startup.json");
        if output != expected_output {
            return Err(invalid(
                "startup snapshot output must be lab/content/startup-snapshot/startup.json",
            ));
        }
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(
            output,
            serde_json::to_vec_pretty(&snapshot)
                .map_err(|_| invalid("cannot serialize startup snapshot"))?,
        )?;
        Ok(snapshot)
    }

    pub fn validate(&self) -> io::Result<()> {
        if self.schema_version != 1
            || self.login.environment.as_bytes().contains(&0)
            || self.world.npc_spawns.len() != 23
            || self.catalogues.effect_tags.len() != 2
            || self.character.journal_entries.len() > MAX_ROWS
            || self.character.stats.len() > MAX_ROWS
            || self.catalogues.item_definitions.len() > MAX_ROWS
            || self.catalogues.coin_store.static_items.len() > MAX_ROWS
            || self.catalogues.coin_store.dynamic_items.len() > MAX_ROWS
            || self.catalogues.ability_set.entries.len() > MAX_ROWS
            || self.world.quick_chat.len() > MAX_ROWS
            || self.world.points_of_interest.len() > MAX_ROWS
            || self.world.journal_definitions.regions.len() > MAX_ROWS
            || self.world.journal_definitions.hubs.len() > MAX_ROWS
            || self.world.journal_definitions.hub_quests.len() > MAX_ROWS
            || self.world.journal_definitions.stickers.len() > MAX_ROWS
            || self.world.welcome_screen.contents.len() > MAX_ROWS
            || self.world.welcome_screen.claim_codes.len() > MAX_ROWS
        {
            return Err(invalid(
                "startup snapshot schema or row counts are unsupported",
            ));
        }
        for definition in &self.catalogues.item_definitions {
            let bytes = decode_hex(&definition.serialized_hex)?;
            if bytes.len() < 4
                || i32::from_le_bytes(bytes[..4].try_into().unwrap()) != definition.id
            {
                return Err(invalid("startup item record ID differs from its typed key"));
            }
        }
        for entry in &self.catalogues.ability_set.entries {
            let bytes = decode_hex(&entry.serialized_fields_hex)?;
            if !(0..=3).contains(&entry.kind) {
                return Err(invalid("unsupported startup ability record kind"));
            }
            if entry.kind == 0 && !bytes.is_empty() {
                return Err(invalid("empty ability slot has serialized fields"));
            }
        }
        for tag in &self.catalogues.effect_tags {
            decode_hex(&tag.definition_blob_hex)?;
        }
        if self.character.stats.iter().any(|stat| {
            stat.kind > 1 || (stat.kind == 1 && f32::from_bits(stat.value_bits).is_nan())
        }) {
            return Err(invalid("unsupported startup stat value kind"));
        }
        Ok(())
    }

    /// Build one captured startup index. Index 07 is intentionally excluded.
    pub fn build_packet(&self, index: usize, character_guid: u64) -> io::Result<Vec<u8>> {
        let packet = match index {
            0 => catalogues::build_overworld_combat_flag(self.character.in_world_combat),
            1 => startup_messages::build_login_reply(self.login.result),
            2 => startup_messages::build_initialization_parameters(&self.login.environment),
            3 => self.build_zone(self.login.zone.zone_id, None),
            4 => catalogues::build_client_game_settings(),
            5 => self.build_item_definitions()?,
            6 => catalogues::build_announcement_data(),
            7 => {
                return Err(invalid(
                    "startup message 07 remains on the self-data patch path",
                ));
            }
            8 => world::build_quick_chat(&self.world.quick_chat),
            9 => world::build_points_of_interest(&self.world.points_of_interest),
            10..=32 => {
                let spawn = self
                    .world
                    .npc_spawns
                    .get(index - 10)
                    .ok_or_else(|| invalid("startup snapshot is missing an NPC spawn"))?;
                world::build_add_npc(spawn)
            }
            33 => startup_messages::build_hit_points(
                self.character.hit_points[0],
                self.character.hit_points[1],
            ),
            34 => startup_messages::build_mana(
                self.character.mana[0],
                self.character.mana[1],
                self.character.mana_flag,
            ),
            35 => self.build_ability_set()?,
            36 | 37 => {
                let tag = &self.catalogues.effect_tags[index - 36];
                let bytes = decode_hex(&tag.definition_blob_hex)?;
                catalogues::build_effect_tag(catalogues::EffectTag {
                    id: tag.id,
                    definition_blob: &bytes,
                })
            }
            38 => world::build_journal_definitions(&self.world.journal_definitions),
            39 => {
                let entries = self
                    .character
                    .journal_entries
                    .iter()
                    .map(|entry| (entry[0], entry[1]))
                    .collect::<Vec<_>>();
                startup_messages::build_journal_set(&entries)
            }
            40 => self.build_coin_store(),
            41 => world::build_welcome_screen(&self.world.welcome_screen),
            42 => catalogues::build_opcode_14_marker(),
            43 => startup_messages::build_done_sending_preload_characters(),
            44 => self.build_stats(character_guid),
            45 => world::build_motd(&self.world.motd),
            _ => return Err(invalid("startup message index is outside 0..46")),
        };
        Ok(packet)
    }

    pub fn build_zone(
        &self,
        zone_id: u32,
        override_zone: Option<&crate::encounter::ZoneTransition>,
    ) -> Vec<u8> {
        match override_zone {
            Some(zone) => startup_messages::build_zone_details(&ZoneDetails {
                name: &zone.name,
                // (R) Existing gateway transition serializer supplied type 2; no new client mapping exists.
                zone_type: 2,
                tutorial: zone.tutorial,
                sky: &zone.sky,
                zone_id: zone.zone_id,
                geometry_id: zone.geometry_id as u32,
            }),
            None => startup_messages::build_zone_details(&ZoneDetails {
                name: &self.login.zone.name,
                zone_type: self.login.zone.zone_type,
                tutorial: self.login.zone.tutorial,
                sky: &self.login.zone.sky,
                zone_id,
                geometry_id: self.login.zone.geometry_id,
            }),
        }
    }

    fn build_item_definitions(&self) -> io::Result<Vec<u8>> {
        let records = self
            .catalogues
            .item_definitions
            .iter()
            .map(|definition| decode_hex(&definition.serialized_hex))
            .collect::<io::Result<Vec<_>>>()?;
        let definitions = records
            .iter()
            .zip(&self.catalogues.item_definitions)
            .map(
                |(serialized_record, definition)| catalogues::ItemDefinition {
                    id: definition.id,
                    serialized_record,
                },
            )
            .collect::<Vec<_>>();
        Ok(catalogues::build_item_definitions(&definitions))
    }

    fn build_ability_set(&self) -> io::Result<Vec<u8>> {
        let fields = self
            .catalogues
            .ability_set
            .entries
            .iter()
            .map(|entry| decode_hex(&entry.serialized_fields_hex))
            .collect::<io::Result<Vec<_>>>()?;
        let entries = fields
            .iter()
            .zip(&self.catalogues.ability_set.entries)
            .map(|(serialized_fields, entry)| catalogues::AbilitySetEntry {
                kind: entry.kind,
                serialized_fields,
            })
            .collect::<Vec<_>>();
        Ok(catalogues::build_ability_set_definition(
            catalogues::AbilitySetDefinition {
                id: self.catalogues.ability_set.id,
                entries: &entries,
            },
        ))
    }

    fn build_coin_store(&self) -> Vec<u8> {
        let static_items = self
            .catalogues
            .coin_store
            .static_items
            .iter()
            .map(|entry| catalogues::CoinStoreEntry {
                id: entry.id,
                unknown_words: entry.unknown_words,
                unknown_flags: entry.unknown_flags,
            })
            .collect::<Vec<_>>();
        let dynamic_items = self
            .catalogues
            .coin_store
            .dynamic_items
            .iter()
            .map(|entry| catalogues::CoinStoreEntry {
                id: entry.id,
                unknown_words: entry.unknown_words,
                unknown_flags: entry.unknown_flags,
            })
            .collect::<Vec<_>>();
        catalogues::build_coin_store_item_lists(catalogues::CoinStoreItemLists {
            static_items: &static_items,
            dynamic_items: &dynamic_items,
        })
    }

    fn build_stats(&self, character_guid: u64) -> Vec<u8> {
        let stats = self
            .character
            .stats
            .iter()
            .map(|stat| catalogues::CharacterStat {
                id: stat.id,
                value: match stat.kind {
                    0 => catalogues::StatValue::Integer(stat.value_bits as i32),
                    _ => catalogues::StatValue::Float(f32::from_bits(stat.value_bits)),
                },
            })
            .collect::<Vec<_>>();
        catalogues::build_stat_update(catalogues::StatUpdate {
            target_guid: character_guid,
            stats: &stats,
        })
    }

    fn parse_reference(reference: &Path, character_guid: u64) -> io::Result<Self> {
        let inner = |index, phase| read_capture_inner(reference, index, phase);
        let (mana, mana_flag) = parse_mana(&inner(34, "ready")?)?;
        let login = LoginSnapshot {
            result: parse_login_result(&inner(1, "login")?)?,
            environment: parse_environment(&inner(2, "login")?)?,
            zone: parse_zone(&inner(3, "login")?)?,
        };
        let character = CharacterSnapshot {
            in_world_combat: parse_combat_flag(&inner(0, "login")?)?,
            hit_points: parse_hit_points(&inner(33, "ready")?)?,
            mana,
            mana_flag,
            journal_entries: parse_journal_set(&inner(39, "ready")?)?,
            stats: parse_stats(&inner(44, "ready")?, character_guid)?,
        };
        let mut npc_spawns = Vec::with_capacity(23);
        for index in 10..=32 {
            npc_spawns.push(
                world::parse_add_npc(&inner(index, "ready")?).map_err(|error| {
                    invalid(format!("cannot parse startup NPC {index}: {error}"))
                })?,
            );
        }
        let quick_chat = world::parse_quick_chat(&inner(8, "ready")?)
            .map_err(|error| invalid(format!("cannot parse startup quick chat: {error}")))?;
        let points_of_interest = world::parse_points_of_interest(&inner(9, "ready")?)
            .map_err(|error| invalid(format!("cannot parse startup POIs: {error}")))?;
        let journal_definitions =
            world::parse_journal_definitions(&inner(38, "ready")?).map_err(|error| {
                invalid(format!("cannot parse startup journal definitions: {error}"))
            })?;
        let welcome_screen = world::parse_welcome_screen(&inner(41, "ready")?)
            .map_err(|error| invalid(format!("cannot parse startup welcome screen: {error}")))?;
        let motd = world::parse_motd(&inner(45, "ready")?)
            .map_err(|error| invalid(format!("cannot parse startup MOTD: {error}")))?;
        let catalogue = parse_catalogues(reference)?;
        let snapshot = Self {
            schema_version: 1,
            login,
            character,
            world: WorldSnapshot {
                quick_chat,
                points_of_interest,
                npc_spawns,
                journal_definitions,
                welcome_screen,
                motd,
            },
            catalogues: catalogue,
        };
        snapshot.validate()?;
        Ok(snapshot)
    }

    fn verify_capture(&self, reference: &Path, character_guid: u64) -> io::Result<()> {
        for index in (0..46).filter(|index| *index != 7) {
            let built = self.build_packet(index, character_guid)?;
            let phase = if index < 8 { "login" } else { "ready" };
            let captured = read_capture(reference, index, phase)?;
            if built != captured {
                return Err(invalid(format!(
                    "startup snapshot builder differs from capture index {index:02}"
                )));
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take(&mut self, count: usize) -> io::Result<&'a [u8]> {
        let end = self
            .offset
            .checked_add(count)
            .ok_or_else(|| invalid("startup field offset overflow"))?;
        let bytes = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| invalid("truncated startup capture field"))?;
        self.offset = end;
        Ok(bytes)
    }

    fn u8(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> io::Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn i32(&mut self) -> io::Result<i32> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> io::Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn count(&mut self) -> io::Result<usize> {
        let count = self.i32()?;
        if !(0..=MAX_ROWS as i32).contains(&count) {
            return Err(invalid("startup capture row count outside limit"));
        }
        Ok(count as usize)
    }

    fn string(&mut self) -> io::Result<String> {
        let length = self.count()?;
        String::from_utf8(self.take(length)?.to_vec())
            .map_err(|_| invalid("invalid startup capture UTF-8"))
    }

    fn finish(self) -> io::Result<()> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(invalid("trailing startup capture bytes"))
        }
    }
}

fn read_file(path: &Path, maximum: usize) -> io::Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.file_type().is_file() || metadata.len() > maximum as u64 {
        return Err(invalid("startup input is not a bounded regular file"));
    }
    let bytes = fs::read(path)?;
    if bytes.len() > maximum {
        return Err(invalid("startup input grew beyond its limit"));
    }
    Ok(bytes)
}

fn read_capture(reference: &Path, index: usize, phase: &str) -> io::Result<Vec<u8>> {
    read_file(
        &reference.join(format!("{index:04}-{phase}.bin")),
        1_048_583,
    )
}

fn read_capture_inner(reference: &Path, index: usize, phase: &str) -> io::Result<Vec<u8>> {
    let packet = read_capture(reference, index, phase)?;
    if packet.len() < 7 || packet[..3] != [5, 0, 1] {
        return Err(invalid("startup capture has invalid tunnel prefix"));
    }
    let length = u32::from_le_bytes(packet[3..7].try_into().unwrap()) as usize;
    if packet.len() != 7 + length {
        return Err(invalid("startup capture tunnel length mismatch"));
    }
    Ok(packet[7..].to_vec())
}

fn header(reader: &mut Reader<'_>, family: u16, subtype: Option<u16>) -> io::Result<()> {
    if reader.u16()? != family {
        return Err(invalid("startup capture family mismatch"));
    }
    if let Some(expected) = subtype {
        if reader.u16()? != expected {
            return Err(invalid("startup capture subtype mismatch"));
        }
    }
    Ok(())
}

fn parse_login_result(bytes: &[u8]) -> io::Result<u8> {
    let mut reader = Reader::new(bytes);
    header(&mut reader, 2, None)?;
    let value = reader.u8()?;
    reader.finish()?;
    Ok(value)
}

fn parse_environment(bytes: &[u8]) -> io::Result<String> {
    let mut reader = Reader::new(bytes);
    header(&mut reader, 166, None)?;
    let rest = reader.take(bytes.len().saturating_sub(reader.offset))?;
    if rest.last() != Some(&0) || rest[..rest.len() - 1].contains(&0) {
        return Err(invalid("invalid startup environment terminator"));
    }
    String::from_utf8(rest[..rest.len() - 1].to_vec())
        .map_err(|_| invalid("invalid startup environment UTF-8"))
}

fn parse_zone(bytes: &[u8]) -> io::Result<ZoneSnapshot> {
    let mut reader = Reader::new(bytes);
    header(&mut reader, 43, None)?;
    let name = reader.string()?;
    let zone_type = reader.i32()?;
    let tutorial = match reader.u8()? {
        0 => false,
        1 => true,
        _ => return Err(invalid("invalid startup zone tutorial flag")),
    };
    if reader.u8()? != 0 {
        return Err(invalid("unsupported captured zone flag"));
    }
    let sky = reader.string()?;
    if reader.u8()? != 0 {
        return Err(invalid("unsupported captured sky flag"));
    }
    let zone_id = reader.u32()?;
    let geometry_id = reader.u32()?;
    if reader.u8()? != 0 || reader.u8()? != 0 {
        return Err(invalid("unsupported captured trailing zone flags"));
    }
    reader.finish()?;
    Ok(ZoneSnapshot {
        name,
        zone_type,
        tutorial,
        sky,
        zone_id,
        geometry_id,
    })
}

fn parse_combat_flag(bytes: &[u8]) -> io::Result<bool> {
    let mut reader = Reader::new(bytes);
    header(&mut reader, 41, Some(132))?;
    if reader.u32()? != 0 || reader.u32()? != 0 {
        return Err(invalid("unsupported captured combat fields"));
    }
    let value = match reader.u8()? {
        0 => false,
        1 => true,
        _ => return Err(invalid("invalid captured combat flag")),
    };
    reader.finish()?;
    Ok(value)
}

fn parse_hit_points(bytes: &[u8]) -> io::Result<[i32; 2]> {
    let mut reader = Reader::new(bytes);
    header(&mut reader, 38, Some(1))?;
    let values = [reader.i32()?, reader.i32()?];
    reader.finish()?;
    Ok(values)
}

fn parse_mana(bytes: &[u8]) -> io::Result<([i32; 2], u8)> {
    let mut reader = Reader::new(bytes);
    header(&mut reader, 38, Some(13))?;
    let values = [reader.i32()?, reader.i32()?];
    let flag = reader.u8()?;
    reader.finish()?;
    Ok((values, flag))
}

fn parse_journal_set(bytes: &[u8]) -> io::Result<Vec<[u32; 2]>> {
    let mut reader = Reader::new(bytes);
    header(&mut reader, 209, None)?;
    if reader.u32()? != 2 {
        return Err(invalid("startup journal-set subtype mismatch"));
    }
    let count = reader.count()?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        entries.push([reader.u32()?, reader.u32()?]);
    }
    reader.finish()?;
    Ok(entries)
}

fn parse_stats(bytes: &[u8], expected_guid: u64) -> io::Result<Vec<StatSnapshot>> {
    let mut reader = Reader::new(bytes);
    header(&mut reader, 38, Some(7))?;
    if reader.u64()? != expected_guid {
        return Err(invalid(
            "startup stat target differs from reference character",
        ));
    }
    let count = reader.count()?;
    let mut stats = Vec::with_capacity(count);
    for _ in 0..count {
        let id = reader.i32()?;
        let kind = reader.u32()?;
        let value_bits = reader.u32()?;
        if kind > 1 || (kind == 1 && f32::from_bits(value_bits).is_nan()) {
            return Err(invalid("unsupported captured startup stat value"));
        }
        stats.push(StatSnapshot {
            id,
            kind,
            value_bits,
        });
    }
    reader.finish()?;
    Ok(stats)
}

fn parse_catalogues(reference: &Path) -> io::Result<CatalogueSnapshot> {
    let inventory_path = reference
        .parent()
        .ok_or_else(|| invalid("reference directory has no lab/references parent"))?
        .join("rust-gameplay/inventory.json");
    let inventory: Value = serde_json::from_slice(&read_file(&inventory_path, MAX_SNAPSHOT_BYTES)?)
        .map_err(|_| invalid("invalid private inventory content JSON"))?;
    let definitions = inventory["definitions"]
        .as_array()
        .ok_or_else(|| invalid("inventory definitions are missing"))?
        .iter()
        .map(|definition| {
            let id = definition["id"]
                .as_i64()
                .ok_or_else(|| invalid("inventory definition has no numeric id"))?;
            let serialized_hex = definition["serialized_hex"]
                .as_str()
                .ok_or_else(|| invalid("inventory definition has no serialized record"))?
                .to_owned();
            Ok(ItemDefinitionSnapshot {
                id: i32::try_from(id)
                    .map_err(|_| invalid("inventory definition ID outside i32"))?,
                serialized_hex,
            })
        })
        .collect::<io::Result<Vec<_>>>()?;

    let coin = read_capture_inner(reference, 40, "ready")?;
    let mut reader = Reader::new(&coin);
    header(&mut reader, 165, Some(1))?;
    let static_items = read_coin_entries(&mut reader)?;
    let dynamic_items = read_coin_entries(&mut reader)?;
    reader.finish()?;

    let ability = read_capture_inner(reference, 35, "ready")?;
    let mut reader = Reader::new(&ability);
    header(&mut reader, 36, Some(5))?;
    let ability_id = reader.i32()?;
    let ability_count = reader.count()?;
    let mut ability_entries = Vec::with_capacity(ability_count);
    for _ in 0..ability_count {
        let kind = reader.i32()?;
        if kind != 0 {
            return Err(invalid(
                "captured ability record kind has no mapped field parser",
            ));
        }
        ability_entries.push(AbilitySetEntrySnapshot {
            kind,
            serialized_fields_hex: String::new(),
        });
    }
    reader.finish()?;

    let mut effect_tags = Vec::with_capacity(2);
    for index in [36, 37] {
        let effect = read_capture_inner(reference, index, "ready")?;
        let mut reader = Reader::new(&effect);
        header(&mut reader, 38, Some(16))?;
        let id = reader.i32()?;
        let length = reader.count()?;
        let blob = reader.take(length)?;
        reader.finish()?;
        effect_tags.push(EffectTagSnapshot {
            id,
            definition_blob_hex: encode_hex(blob),
        });
    }

    Ok(CatalogueSnapshot {
        item_definitions: definitions,
        coin_store: CoinStoreSnapshot {
            static_items,
            dynamic_items,
        },
        ability_set: AbilitySetSnapshot {
            id: ability_id,
            entries: ability_entries,
        },
        effect_tags,
    })
}

fn read_coin_entries(reader: &mut Reader<'_>) -> io::Result<Vec<CoinStoreEntrySnapshot>> {
    let count = reader.count()?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let id = reader.i32()?;
        let unknown_words = [reader.u32()?, reader.u32()?, reader.u32()?];
        let unknown_flags = [reader.u8()?, reader.u8()?, reader.u8()?];
        entries.push(CoinStoreEntrySnapshot {
            id,
            unknown_words,
            unknown_flags,
        });
    }
    Ok(entries)
}

fn decode_hex(value: &str) -> io::Result<Vec<u8>> {
    if value.len() % 2 != 0 {
        return Err(invalid("odd-length startup hex field"));
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let digit = |byte: u8| match byte {
                b'0'..=b'9' => Some(byte - b'0'),
                b'a'..=b'f' => Some(byte - b'a' + 10),
                b'A'..=b'F' => Some(byte - b'A' + 10),
                _ => None,
            };
            let high = digit(pair[0]).ok_or_else(|| invalid("invalid startup hex field"))?;
            let low = digit(pair[1]).ok_or_else(|| invalid("invalid startup hex field"))?;
            Ok((high << 4) | low)
        })
        .collect()
}

fn encode_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(DIGITS[(byte >> 4) as usize] as char);
        out.push(DIGITS[(byte & 15) as usize] as char);
    }
    out
}
