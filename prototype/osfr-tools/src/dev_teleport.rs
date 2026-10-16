//! Developer-only `realm goto` relocation to a named actor in the world zone.
//!
//! The command is opt-in (`--dev-tools`), bounded to zone 0 entities, and
//! changes nothing except the character position and facing. It never grants
//! inventory, experience, coin, quest or discovery progress by itself.
use crate::{character_store::CharacterState, opening::OpeningActor};

/// (R) Developer-command arrival standoff in metres. It keeps the arrival
/// inside an actor's interaction radius and facing the actor. It does not parse
/// collision geometry and does not claim a verified floor.
pub const STANDOFF: f32 = 5.0; // 2.0 put the player on top of the NPC (clicks missed, run 165644)
/// Arrival angles in degrees, relative to the character's current facing
/// vector and tried in this order. The first sample that keeps the target at
/// least [`MARGIN`] closer than every other loaded zone actor wins.
const ANGLE_SAMPLES: [f32; 8] = [0.0, 45.0, -45.0, 90.0, -90.0, 135.0, -135.0, 180.0];
/// Required clearance between the target and the next closest zone actor, in
/// metres, measured horizontally.
const MARGIN: f32 = 0.5;
/// The only zone this developer command relocates inside.
pub const WORLD_ZONE: u32 = 0;
const MAX_NAME_CANDIDATES: usize = 10;
const MAX_LIST: usize = 1_800;

#[derive(Clone, Debug, PartialEq)]
pub struct Target {
    pub guid: u64,
    pub name: String,
    pub zone_id: u32,
    pub position: [f32; 3],
    pub kind: String,
}

impl From<&OpeningActor> for Target {
    fn from(actor: &OpeningActor) -> Self {
        Self {
            guid: actor.guid,
            name: actor.name.clone(),
            zone_id: actor.zone_id,
            position: actor.position,
            kind: actor.kind.clone(),
        }
    }
}

impl Target {
    /// (R) Keep nameless content targets addressable from the list by GUID.
    pub fn content_npc(guid: u64, name: Option<&str>, zone_id: u32, position: [f32; 3]) -> Self {
        Self {
            guid,
            name: name
                .filter(|value| !value.trim().is_empty())
                .map(str::to_owned)
                .unwrap_or_else(|| format!("NPC {guid}")),
            zone_id,
            position,
            kind: "npc".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Relocation {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
}

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Reply(String),
    Relocate {
        reply: String,
        relocation: Relocation,
    },
}

/// Parse a lowercased `realm ` command body. Returns `None` for every command
/// that is not `goto`, so unknown commands keep their existing handling.
pub fn handle(text: &str, targets: &[Target], character: &CharacterState) -> Option<Command> {
    let text = text.trim();
    let remainder = text.strip_prefix("goto")?;
    // A keyword boundary, so `gotoxy` stays an ordinary unknown command.
    if remainder.chars().next().is_some_and(|c| !c.is_whitespace()) {
        return None;
    }
    let argument = remainder.trim();
    let zone = character.zone_id;
    // Other zones (an encounter arena) work only when they list their own
    // targets; the position is relative to that zone.
    if zone != WORLD_ZONE && !targets.iter().any(|t| t.zone_id == zone) {
        return Some(Command::Reply(format!(
            "Developer teleport supports only the main zone ({WORLD_ZONE}); the current zone is {zone}. Nothing changed."
        )));
    }
    if !finite(character.position) || !finite(character.rotation) {
        return Some(Command::Reply(
            "Developer teleport needs a finite character position. Nothing changed.".into(),
        ));
    }
    if argument.is_empty() {
        return Some(Command::Reply(
            "Developer teleport usage: realm goto <entity name or GUID>, or realm goto list."
                .into(),
        ));
    }
    let mut parts = argument.splitn(2, char::is_whitespace);
    if parts
        .next()
        .is_some_and(|word| word.eq_ignore_ascii_case("list"))
    {
        return Some(Command::Reply(list(
            targets,
            zone,
            parts.next().unwrap_or("").trim(),
        )));
    }
    match resolve(argument, targets, zone) {
        Ok(actor) => {
            let relocation = standoff(actor, character, targets);
            if !finite(actor.position)
                || !finite(relocation.position)
                || !finite(relocation.rotation)
            {
                return Some(Command::Reply(format!(
                    "Developer teleport target {} has no finite position. Nothing changed.",
                    actor.name
                )));
            }
            Some(Command::Relocate {
                reply: format!(
                    "Developer teleport: {} (GUID {}) at {STANDOFF:.2}m. Position applied; movement rules resume normally.",
                    actor.name, actor.guid
                ),
                relocation,
            })
        }
        Err(reply) => Some(Command::Reply(reply)),
    }
}

/// Byte-exact pinned Sanctuary `ClientUpdatePacketUpdateLocation`: base opcode
/// 38, sub-opcode 12, position XYZ with W=1, facing XYZ with W=0, the
/// `Teleport` flag and the writer's default `Unknown` byte.
pub fn location_packet(position: [f32; 3], rotation: [f32; 3]) -> Vec<u8> {
    let mut packet = vec![38, 0, 12, 0];
    for value in position.into_iter().chain([1.0]) {
        packet.extend(value.to_le_bytes());
    }
    for value in rotation.into_iter().chain([0.0]) {
        packet.extend(value.to_le_bytes());
    }
    packet.push(1);
    packet.push(0);
    packet
}

/// (R) The developer-only list uses a case-insensitive substring filter.
fn list(targets: &[Target], zone: u32, filter: &str) -> String {
    let mut text = format!("Developer teleport targets in zone {zone}:");
    let filter = filter.to_lowercase();
    let mut matched = 0;
    let mut shown = 0;
    let mut omitted = 0;
    for target in targets
        .iter()
        .filter(|target| target.zone_id == zone && target.name.to_lowercase().contains(&filter))
    {
        matched += 1;
        let entry = format!(" {} ({}, {})", target.name, target.guid, target.kind);
        if text.len() + entry.len() > MAX_LIST {
            omitted += 1;
            continue;
        }
        text.push_str(&entry);
        text.push(';');
        shown += 1;
    }
    if omitted != 0 {
        text.push_str(&format!(
            " {omitted} more omitted for length; use a GUID from the content."
        ));
    }
    if matched == 0 {
        if filter.is_empty() {
            text.push_str(" none loaded.");
        } else {
            text.push_str(&format!(" no targets match '{filter}'."));
        }
    } else if shown == 0 {
        text.push_str(" matches omitted for length.");
    }
    text
}

/// (R) Developer-only lookup tries GUID, exact name, then unique prefix.
/// Ambiguous or unknown queries leave the character untouched.
fn resolve<'a>(query: &str, targets: &'a [Target], zone: u32) -> Result<&'a Target, String> {
    let candidates: Vec<&Target> = targets
        .iter()
        .filter(|target| target.zone_id == zone)
        .collect();
    if let Ok(guid) = query.parse::<u64>() {
        return match candidates
            .iter()
            .copied()
            .filter(|target| target.guid == guid)
            .collect::<Vec<_>>()
        {
            found if found.len() == 1 => Ok(found[0]),
            _ => Err(format!(
                "No zone {zone} entity has GUID {guid}. Use realm goto list."
            )),
        };
    }
    let lowered = query.to_lowercase();
    let exact: Vec<&Target> = candidates
        .iter()
        .copied()
        .filter(|target| target.name.to_lowercase() == lowered)
        .collect();
    if let Some(target) = select_match(query, &exact)? {
        return Ok(target);
    }
    let prefixes: Vec<&Target> = candidates
        .iter()
        .copied()
        .filter(|target| target.name.to_lowercase().starts_with(&lowered))
        .collect();
    if let Some(target) = select_match(query, &prefixes)? {
        return Ok(target);
    }
    Err(format!(
        "Unknown zone {zone} entity: {query}. Use realm goto list."
    ))
}

fn select_match<'a>(query: &str, found: &[&'a Target]) -> Result<Option<&'a Target>, String> {
    match found {
        [] => Ok(None),
        [target] => Ok(Some(target)),
        _ => Err(format!(
            "Ambiguous entity {query}: {}. Use the GUID.",
            describe(found)
        )),
    }
}

fn describe(targets: &[&Target]) -> String {
    let mut text = String::new();
    for target in targets.iter().take(MAX_NAME_CANDIDATES) {
        if !text.is_empty() {
            text.push_str(", ");
        }
        text.push_str(&format!("{} {}", target.name, target.guid));
    }
    if targets.len() > MAX_NAME_CANDIDATES {
        text.push_str(&format!(", +{} more", targets.len() - MAX_NAME_CANDIDATES));
    }
    text
}

/// Choose the arrival around the actor that keeps the target closest to the
/// client's proximity prompt. Angle0 is the approach side, so the arrival is
/// where the character came from and a repeated `goto` stays put; positive
/// angles rotate counter-clockwise in the XZ plane. Only finite actors in the
/// same zone are considered, and the target itself is skipped.
fn standoff(actor: &Target, character: &CharacterState, actors: &[Target]) -> Relocation {
    let base = approach_direction(actor, character);
    let others: Vec<[f32; 3]> = actors
        .iter()
        .filter(|other| {
            other.zone_id == actor.zone_id && other.guid != actor.guid && finite(other.position)
        })
        .map(|other| other.position)
        .collect();
    let mut best: Option<((f32, f32), f32)> = None;
    for degrees in ANGLE_SAMPLES {
        let direction = rotate(base, degrees);
        let position = arrive(actor.position, direction);
        let margin = others
            .iter()
            .map(|other| horizontal(position, *other) - STANDOFF)
            .fold(f32::INFINITY, f32::min);
        if margin >= MARGIN {
            return Relocation {
                position,
                rotation: face(direction),
            };
        }
        if best.is_none_or(|(_, best_margin)| margin > best_margin) {
            best = Some((direction, margin));
        }
    }
    // Never reached empty: every sample is evaluated above.
    let (direction, _) = best.expect("angle samples are always evaluated");
    Relocation {
        position: arrive(actor.position, direction),
        rotation: face(direction),
    }
}

fn arrive(actor: [f32; 3], direction: (f32, f32)) -> [f32; 3] {
    [
        actor[0] + direction.0 * STANDOFF,
        actor[1],
        actor[2] + direction.1 * STANDOFF,
    ]
}

fn face(direction: (f32, f32)) -> [f32; 3] {
    [-direction.0, 0.0, -direction.1]
}

/// The direction from the actor toward the side the character is on: the
/// reversed facing vector, or the actor-to-character direction when the saved
/// facing vector is missing, or `+X` when both are degenerate.
fn approach_direction(actor: &Target, character: &CharacterState) -> (f32, f32) {
    let (x, z) = (character.rotation[0], character.rotation[2]);
    let length = (x * x + z * z).sqrt();
    if length > 0.001 {
        return (-x / length, -z / length);
    }
    let (dx, dz) = (
        character.position[0] - actor.position[0],
        character.position[2] - actor.position[2],
    );
    let length = (dx * dx + dz * dz).sqrt();
    if length > 0.001 {
        return (dx / length, dz / length);
    }
    (1.0, 0.0)
}

fn rotate(direction: (f32, f32), degrees: f32) -> (f32, f32) {
    let (sin, cos) = degrees.to_radians().sin_cos();
    (
        direction.0 * cos - direction.1 * sin,
        direction.0 * sin + direction.1 * cos,
    )
}

fn horizontal(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn finite(values: [f32; 3]) -> bool {
    values
        .iter()
        .all(|v| v.is_finite() && v.abs() <= 1_000_000.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::Digest;

    fn actor(guid: u64, name: &str, zone_id: u32, position: [f32; 3]) -> Target {
        Target {
            guid,
            name: name.into(),
            zone_id,
            position,
            kind: "guide".into(),
        }
    }

    fn character(zone_id: u32, position: [f32; 3]) -> CharacterState {
        CharacterState {
            guid: 1,
            zone_id,
            position,
            rotation: [1.0, 0.0, 0.0],
        }
    }

    /// Hex captured from the pinned Sanctuary writer compiled unchanged into
    /// `lab/reports/developer-teleport-20261009T003739Z/oracle`:
    /// `LocationWriterOracle -2112.839 -56.52175 208.5168 0 0 -1` and
    /// `LocationWriterOracle -1054.135 4.629629 305.7209 -0.70710677 0 -0.70710677`.
    #[test]
    fn location_packet_matches_the_pinned_c_sharp_writer() {
        let vectors: [([f32; 3], [f32; 3], &str, &str); 2] = [
            (
                [-2112.839, -56.52175, 208.5168],
                [0.0, 0.0, -1.0],
                "26000c006d0d04c5461662c24d8450430000803f0000000000000000000080bf000000000100",
                "0a26a9d0202c2a47fe161befa9fea66d8bccb1bc5e1a486408019a4b601b686e",
            ),
            (
                [-1054.135, 4.629629, 305.7209],
                [-0.70710677, 0.0, -0.70710677],
                "26000c0052c483c4ec25944046dc98430000803ff30435bf00000000f30435bf000000000100",
                "ddd1827f84b7dcea7c2c3291d9e108aa2f4ee9e87e208316420600f2fb30e2c7",
            ),
        ];
        for (position, rotation, hex, sha) in vectors {
            let packet = location_packet(position, rotation);
            assert_eq!(packet.len(), 38);
            let encoded = packet
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>();
            assert_eq!(encoded, hex);
            assert_eq!(
                format!("{:x}", sha2::Sha256::digest(&packet)),
                sha,
                "oracle digest"
            );
        }
    }

    #[test]
    fn exact_prefix_and_guid_targets_resolve() {
        let actors = [
            actor(10, "Mae Farnum", 0, [100.0, 5.0, 100.0]),
            actor(11, "Bartle", 0, [200.0, 5.0, 100.0]),
            actor(12, "Fire", 0, [300.0, 5.0, 100.0]),
            actor(13, "Fire", 0, [400.0, 5.0, 100.0]),
            actor(14, "Gloamling", 411, [0.0, 0.0, 0.0]),
        ];
        let character = character(0, [0.0, 5.0, 0.0]);
        for query in ["mae farnum", "mae", "MAE", "10"] {
            match handle(&format!("goto {query}"), &actors, &character) {
                Some(Command::Relocate { relocation, .. }) => {
                    assert_eq!(relocation.position[1], 5.0, "{query}");
                }
                other => panic!("{query} did not relocate: {other:?}"),
            }
        }
        // Ambiguous, unknown, cave-only and empty targets never relocate.
        for query in ["fire", "gloamling", "nobody", "411", ""] {
            match handle(&format!("goto {query}"), &actors, &character) {
                Some(Command::Reply(text)) => {
                    assert!(!text.is_empty(), "{query}");
                }
                other => panic!("{query} must be refused: {other:?}"),
            }
        }
    }

    #[test]
    fn list_filter_is_case_insensitive_and_name_resolution_prefers_exact() {
        let actors = [
            actor(10, "Sam", 0, [100.0, 5.0, 100.0]),
            actor(11, "Samantha", 0, [200.0, 5.0, 100.0]),
            actor(12, "Samwise", 0, [300.0, 5.0, 100.0]),
        ];
        let character = character(0, [0.0, 5.0, 0.0]);
        match handle("goto list sAM", &actors, &character) {
            Some(Command::Reply(text)) => {
                assert!(text.contains("Sam (10"), "{text}");
                assert!(text.contains("Samantha (11"), "{text}");
                assert!(text.contains("Samwise (12"), "{text}");
            }
            other => panic!("filtered listing must reply: {other:?}"),
        }
        match handle("goto sam", &actors, &character) {
            Some(Command::Relocate { reply, .. }) => assert!(reply.contains("Sam (GUID 10)")),
            other => panic!("exact name must beat longer prefixes: {other:?}"),
        }
        match handle("goto sa", &actors, &character) {
            Some(Command::Reply(text)) => {
                assert!(text.contains("Ambiguous entity sa"), "{text}");
                assert!(text.contains("Sam 10"), "{text}");
                assert!(text.contains("Samantha 11"), "{text}");
            }
            other => panic!("ambiguous prefixes must be refused: {other:?}"),
        }
        assert!(matches!(
            handle("goto mantha", &actors, &character),
            Some(Command::Reply(text)) if text.contains("Unknown zone")
        ));
    }

    #[test]
    fn ambiguous_names_show_at_most_ten_guid_candidates() {
        let actors: Vec<_> = (0..12)
            .map(|index| {
                actor(
                    100 + index,
                    &format!("Sage {index}"),
                    0,
                    [index as f32, 0.0, 0.0],
                )
            })
            .collect();
        let Some(Command::Reply(text)) = handle("goto sage", &actors, &character(0, [0.0; 3]))
        else {
            panic!("ambiguous name must produce a reply");
        };
        assert!(text.contains("Sage 0 100"), "{text}");
        assert!(text.contains("Sage 9 109"), "{text}");
        assert!(!text.contains("Sage 10 110"), "{text}");
        assert!(text.contains("+2 more"), "{text}");
    }

    #[test]
    fn other_commands_and_other_zones_are_left_alone() {
        let actors = [actor(10, "Mae Farnum", 0, [100.0, 5.0, 100.0])];
        assert!(handle("status", &actors, &character(0, [0.0; 3])).is_none());
        assert!(handle("goto", &actors, &character(411, [0.0; 3])).is_some());
        // Only `goto` itself or a whitespace-separated argument is a command.
        for text in ["gotomae", "gotolist", "go to mae", "got"] {
            assert!(
                handle(text, &actors, &character(0, [0.0; 3])).is_none(),
                "{text} must fall through to the ordinary handler"
            );
        }
        for text in ["goto", "goto ", "goto\tmae", "goto   mae"] {
            assert!(
                handle(text, &actors, &character(0, [0.0; 3])).is_some(),
                "{text} must be treated as goto"
            );
        }
        match handle("goto list", &actors, &character(0, [0.0; 3])) {
            Some(Command::Reply(text)) => {
                assert!(text.contains("Mae Farnum"));
                assert!(text.contains("10"));
            }
            other => panic!("list must not relocate: {other:?}"),
        }
    }

    #[test]
    fn non_finite_targets_and_characters_are_refused() {
        let broken = [actor(10, "Mae Farnum", 0, [f32::NAN, 5.0, 100.0])];
        match handle("goto mae", &broken, &character(0, [0.0, 5.0, 0.0])) {
            Some(Command::Reply(text)) => assert!(text.contains("finite"), "{text}"),
            other => panic!("a non-finite target must refuse: {other:?}"),
        }
        let actors = [actor(10, "Mae Farnum", 0, [100.0, 5.0, 100.0])];
        for position in [
            [f32::INFINITY, 5.0, 0.0],
            [2_000_000.0, 5.0, 0.0],
            [f32::NAN, 5.0, 0.0],
        ] {
            match handle("goto mae", &actors, &character(0, position)) {
                Some(Command::Reply(text)) => assert!(text.contains("finite"), "{text}"),
                other => panic!("a non-finite character must refuse: {other:?}"),
            }
        }
    }

    #[test]
    fn standoff_is_bounded_finite_and_faces_the_actor() {
        let target = actor(10, "Mae Farnum", 0, [-2112.839, -56.52175, 208.5168]);
        let character = character(0, [-1000.0, 4.4188066, 305.68225]);
        let relocation = standoff(&target, &character, std::slice::from_ref(&target));
        assert!(relocation.position.iter().all(|v| v.is_finite()));
        assert_eq!(relocation.position[1], target.position[1]);
        // Angle0 is the approach side (the facing vector reversed), so with no
        // competing actor the arrival is exactly one standoff behind the target.
        assert!((relocation.position[0] - (target.position[0] - STANDOFF)).abs() < 1e-3);
        assert_eq!(relocation.position[2], target.position[2]);
        let dx = target.position[0] - relocation.position[0];
        let dz = target.position[2] - relocation.position[2];
        let distance = (dx * dx + dz * dz).sqrt();
        assert!((distance - STANDOFF).abs() < 1e-3, "{distance}");
        let norm = (relocation.rotation[0].powi(2) + relocation.rotation[2].powi(2)).sqrt();
        assert!((norm - 1.0).abs() < 1e-6, "{norm}");
        assert!(relocation.rotation[0] * dx + relocation.rotation[2] * dz > 0.0);

        // A missing facing vector falls back to the approach direction, and a
        // fully degenerate character still yields a finite, unit, facing arrival.
        let approaching = CharacterState {
            position: [
                target.position[0],
                target.position[1],
                target.position[2] - 100.0,
            ],
            rotation: [0.0, 0.0, 0.0],
            ..character.clone()
        };
        let approach = standoff(&target, &approaching, std::slice::from_ref(&target));
        assert!(approach.position[2] < target.position[2]);
        let same = CharacterState {
            position: target.position,
            ..approaching
        };
        let degenerate = standoff(&target, &same, std::slice::from_ref(&target));
        assert!(degenerate.position.iter().all(|v| v.is_finite()));
        assert!(
            (degenerate.rotation[0].powi(2) + degenerate.rotation[2].powi(2) - 1.0).abs() < 1e-6
        );
    }

    #[test]
    fn arrival_skips_an_occupied_approach_sample() {
        let target = actor(10, "Mae Farnum", 0, [0.0, -56.0, 0.0]);
        let occupied = actor(11, "Occupied Spot", 0, [-STANDOFF, -56.0, 0.0]);
        let actors = [target.clone(), occupied.clone()];
        let character = CharacterState {
            guid: 1,
            zone_id: 0,
            position: [100.0, -56.0, 0.0],
            rotation: [1.0, 0.0, 0.0],
        };
        let Some(Command::Relocate { relocation, .. }) = handle("goto mae", &actors, &character)
        else {
            panic!("Mae must relocate");
        };
        assert!(
            horizontal(relocation.position, occupied.position) >= STANDOFF + MARGIN - 1e-3,
            "the occupied approach spot must be rejected: {relocation:?}"
        );
        // The first passing sample after0/+45/-45 is +90, i.e. -Z here.
        assert!((relocation.position[0] - target.position[0]).abs() < 1e-3);
        assert!((relocation.position[2] - (target.position[2] - STANDOFF)).abs() < 1e-3);
    }

    /// Regression for the native Mae arrival: the Cooking Table sits about4m
    /// from Mae at XZ `(-2109.244,210.265)` versus Mae `(-2112.839,208.5168)`
    /// with nearby Gummyfruit/Water Well resources, so an arrival chosen only
    /// from the old approach direction put the table closer than the target.
    #[test]
    fn arrival_keeps_the_target_closest_with_a_margin() {
        let target = actor(10, "Mae Farnum", 0, [-2112.839, -56.52175, 208.5168]);
        let actors = [
            target.clone(),
            actor(11, "Cooking Table", 0, [-2109.244, -56.61908, 210.265]),
            actor(12, "Water Well", 0, [-2113.0, -56.5, 222.0]),
            actor(13, "Gummyfruit", 0, [-2116.325, -56.67786, 199.2877]),
            actor(14, "Cleetus Farnum", 0, [-2078.389, -55.64841, 200.7649]),
            actor(15, "Cave Only", 411, [-2110.0, -56.5, 208.0]),
        ];
        let character = CharacterState {
            guid: 1,
            zone_id: 0,
            position: [-1054.135, 4.629629, 305.7209],
            rotation: [1.0, 0.0, 0.0],
        };
        let Some(Command::Relocate { relocation, .. }) = handle("goto mae", &actors, &character)
        else {
            panic!("Mae must relocate");
        };
        let horizontal =
            |a: [f32; 3], b: [f32; 3]| ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
        let to_target = horizontal(relocation.position, target.position);
        assert!((to_target - STANDOFF).abs() < 1e-3, "{to_target}");
        for other in actors.iter().filter(|a| a.guid != target.guid) {
            let distance = horizontal(relocation.position, other.position);
            assert!(
                distance >= to_target + 0.5,
                "arrival {:?} puts {} at {distance} vs target {to_target}",
                relocation.position,
                other.name
            );
        }
    }
}
