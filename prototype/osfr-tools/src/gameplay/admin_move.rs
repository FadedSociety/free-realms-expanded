//! L043: `realm move <character> <warpstone name | x z | x y z>`, for admins only. The role gate
//! is `command_refusal`, which runs first: `account_store::can_run_realm_command` lists `move`
//! for no other role, so a player or builder is refused before anything here reads the command.
//! Gameplay parses the command and resolves the open-world arrival; the gateway host finds the
//! character and moves it after the admin's own transaction commits
//! (`bin/osfr_gateway/admin_move.rs`). The moved character's side is `admin_relocation`.
use super::*;

const USAGE: &str =
    "Usage: realm move <character name or GUID> <warpstone name | x z | x y z>. Nothing changed.";

/// Who to move, as the admin typed it. The host resolves it against the account store.
#[derive(Clone, Debug, PartialEq)]
pub enum MoveTarget {
    Guid(u64),
    /// A displayed character name, matched ignoring case.
    Name(String),
}

/// A staged admin move: the character and its arrival in the open world.
#[derive(Clone, Debug, PartialEq)]
pub struct AdminMove {
    pub target: MoveTarget,
    pub position: [f32; 3],
    /// `None` keeps the moved character's own facing, as a 90/122 arrival does without a
    /// captured spawn facing.
    pub rotation: Option<[f32; 3]>,
}

#[derive(Debug, PartialEq)]
enum Place {
    Point([f32; 3]),
    Ground(f32, f32),
    Warpstone(String),
}

impl Gameplay {
    /// Stage `realm move` for the host; `None` for every other request.
    pub(super) fn admin_move_request(
        &self,
        inner: &[u8],
        guid: u64,
    ) -> io::Result<Option<Outcome>> {
        let Some(text) = chat::request(inner, guid)? else {
            return Ok(None);
        };
        let text = text.trim();
        let lower = text.to_ascii_lowercase();
        let Some(command) = lower.strip_prefix("realm ") else {
            return Ok(None);
        };
        if command.split_whitespace().next() != Some("move") || !self.may_run(command) {
            return Ok(None);
        }
        // ASCII lowercasing keeps byte offsets, so this slice keeps a name's typed case.
        let raw = text["realm ".len()..].trim_start();
        let words: Vec<&str> = raw["move".len()..].split_whitespace().collect();
        let staged = self.resolve_move(&words);
        Ok(Some(match staged {
            Ok(request) => Outcome {
                admin_move: Some(request),
                ..Outcome::default()
            },
            Err(reply) => Outcome {
                replies: vec![chat::system(&reply)],
                ..Outcome::default()
            },
        }))
    }

    fn resolve_move(&self, words: &[&str]) -> Result<AdminMove, String> {
        let terrain = self.terrain.clone();
        let ground = move |x: f32, z: f32| {
            terrain
                .as_ref()?
                .height_at(piece_077::OPEN_WORLD_ZONE, x, z)
        };
        let world = piece_077::WarpWorld {
            ground: &ground,
            friend: &|_| None,
        };
        let warpstone = |name: &str| self.warp.named_arrival(name, &world);
        let (target, place) = split(words, |name| warpstone(name).is_some()).ok_or(USAGE)?;
        let (position, rotation) = match place {
            Place::Point(position) => (position, None),
            // (R) Half a metre up, as `realm goto x z`: the client grounds from above.
            Place::Ground(x, z) => match ground(x, z) {
                Some(height) => ([x, height + 0.5, z], None),
                None => {
                    return Err(format!(
                        "No terrain heights at x {x:.0}, z {z:.0}; give a height: realm move <character> x y z. Nothing changed."
                    ));
                }
            },
            Place::Warpstone(name) => warpstone(&name).ok_or(USAGE)?,
        };
        let target = match target.parse::<u64>() {
            Ok(guid) => MoveTarget::Guid(guid),
            Err(_) => MoveTarget::Name(target),
        };
        Ok(AdminMove {
            target,
            position,
            rotation,
        })
    }

    /// The moved character's side, on its own staged clone: refused wherever a 122 rescue is
    /// (outside the open world, knocked out, in an encounter or busy by the occupation table),
    /// otherwise the same `apply_relocation` every warp uses.
    pub fn admin_relocation(
        &mut self,
        relocation: dev_teleport::Relocation,
        character: &CharacterState,
    ) -> io::Result<Outcome> {
        let context = self.warp_context(character);
        if context.zone_id != piece_077::OPEN_WORLD_ZONE {
            return Err(io::Error::other("it is not in the open world"));
        }
        if context.blocked.is_some() {
            return Err(io::Error::other(
                "it is knocked out, in an encounter or busy with an activity",
            ));
        }
        let mut out = Outcome::default();
        self.apply_relocation(relocation, character, &mut out)?;
        Ok(out)
    }
}

/// Split `<character> <destination>` from the end: three trailing coordinates are x y z, two
/// are x z, otherwise the longest trailing words (up to three) that name a warpstone. At least
/// one word must remain for the character, so `12 5 7` is GUID 12 at x 5, z 7.
fn split(words: &[&str], is_warpstone: impl Fn(&str) -> bool) -> Option<(String, Place)> {
    let coordinate = |word: &&str| {
        word.parse::<f32>()
            .ok()
            .filter(|v| v.is_finite() && v.abs() <= crate::character_store::MAX_COMPONENT_MAGNITUDE)
    };
    let tail: Vec<f32> = words.iter().rev().map_while(coordinate).take(3).collect();
    let (used, place) = match tail.len().min(words.len().saturating_sub(1)) {
        3 => (3, Place::Point([tail[2], tail[1], tail[0]])),
        2 => (2, Place::Ground(tail[1], tail[0])),
        _ => (1..=3usize)
            .rev()
            .filter(|used| *used < words.len())
            .map(|used| (used, words[words.len() - used..].join(" ")))
            .find(|(_, name)| is_warpstone(name))
            .map(|(used, name)| (used, Place::Warpstone(name)))?,
    };
    Some((words[..words.len() - used].join(" "), place))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Option<(String, Place)> {
        let words: Vec<&str> = text.split_whitespace().collect();
        split(&words, |name| {
            ["Snowhill", "Cobblestone Village", "Lavender Coast Pass"]
                .iter()
                .any(|stone| stone.eq_ignore_ascii_case(name))
        })
    }

    #[test]
    fn character_then_destination_splits_from_the_end() {
        assert_eq!(
            parse("12 5 7"),
            Some(("12".into(), Place::Ground(5.0, 7.0)))
        );
        assert_eq!(
            parse("12 1 -2.5 3"),
            Some(("12".into(), Place::Point([1.0, -2.5, 3.0])))
        );
        assert_eq!(
            parse("Alpha Test 1 2 3"),
            Some(("Alpha Test".into(), Place::Point([1.0, 2.0, 3.0])))
        );
        assert_eq!(
            parse("Alpha Test cobblestone village"),
            Some((
                "Alpha Test".into(),
                Place::Warpstone("cobblestone village".into())
            ))
        );
        assert_eq!(
            parse("Mary Ann Test Lavender Coast Pass"),
            Some((
                "Mary Ann Test".into(),
                Place::Warpstone("Lavender Coast Pass".into())
            ))
        );
    }

    #[test]
    fn missing_character_unknown_place_and_non_finite_numbers_are_refused() {
        for text in [
            "",
            "Snowhill",
            "5 7",
            "Alpha Test",
            "Alpha Test Nowhere",
            "Alpha Test NaN 1",
            "Alpha Test 1 inf",
            "Alpha Test 1 2000000",
        ] {
            assert_eq!(parse(text), None, "{text}");
        }
    }

    /// The moved character's side refuses where a 122 rescue would, changing nothing, and
    /// otherwise stages the relocation the gateway turns into 38/12.
    #[test]
    fn busy_or_elsewhere_targets_refuse_and_free_ones_relocate() {
        let content = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../lab/references/rust-opening-cooking-original-assets");
        let mut game = Gameplay::load(&content, None).unwrap();
        let character = CharacterState {
            guid: 1,
            zone_id: 0,
            position: [-1063.6, 4.4, 305.7],
            rotation: [1.0, 0.0, 0.0],
        };
        let relocation = dev_teleport::Relocation {
            position: [-1904.46, -39.79, 413.316],
            rotation: [1.0, 0.0, 0.0],
        };
        let cave = CharacterState {
            zone_id: 411,
            ..character.clone()
        };
        let before = game.state();
        assert!(game.admin_relocation(relocation.clone(), &cave).is_err());
        assert_eq!(game.state(), before, "a refused move changed gameplay");
        game.cooking.as_mut().unwrap().begin(45).unwrap();
        let busy = game.state();
        assert!(
            game.admin_relocation(relocation.clone(), &character)
                .is_err()
        );
        assert_eq!(game.state(), busy, "a refused move changed gameplay");

        let mut free = Gameplay::load(&content, None).unwrap();
        assert_eq!(free.state(), before);
        let out = free
            .admin_relocation(relocation.clone(), &character)
            .unwrap();
        assert_eq!(out.relocation, Some(relocation));
    }
}
