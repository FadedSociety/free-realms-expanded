//! L081: redeem codes (owner 2026-10-31: kept for handing out items, including the realm's own
//! assets later). The GM panel's Codes tab and Redeem Code hub tree speak Sanctuary PR #163's
//! `!code` / `!redeem` protocol (`CodeChatCommand.cs`, `RedeemChatCommand.cs`); the store and its
//! rules here are ours. One JSON file beside the account store, written whole under a lock.
use crate::gm_panel::{line, panel_escape, panel_unescape};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io,
    path::{Path, PathBuf},
    sync::OnceLock,
};

/// Items one code may hold (Sanctuary's `MaxItems`).
pub const MAX_ITEMS: usize = 8;
/// (R) Bounds the file read into memory.
const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Code {
    pub coins: u32,
    pub items: Vec<(i32, u32)>,
    /// Redemptions allowed in all; 0 = no limit.
    pub max_uses: u32,
    pub uses: u32,
    /// Redemptions allowed per character; 0 = no limit.
    pub per_character: u32,
    /// Days since 1970-01-01 (UTC); `None` = always.
    pub starts: Option<i64>,
    pub expires: Option<i64>,
    pub members: bool,
    pub enabled: bool,
    pub note: String,
    /// Character guid -> times redeemed.
    pub redeemed: BTreeMap<u64, u32>,
    /// gifts-codes: the one account that may redeem it (a code made for an event's winner).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bound: Option<u64>,
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Codes {
    pub schema_version: u32,
    pub codes: BTreeMap<String, Code>,
}

static STORE: OnceLock<PathBuf> = OnceLock::new();

/// The realm's code file, set once by the gateway (`redeem-codes.json` beside the account store).
pub fn set_store(path: PathBuf) {
    let _ = STORE.set(path);
}

pub fn store() -> Option<&'static Path> {
    STORE.get().map(PathBuf::as_path)
}

impl Codes {
    pub fn load(path: &Path) -> io::Result<Self> {
        match std::fs::metadata(path) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Ok(Self {
                    schema_version: 1,
                    ..Self::default()
                });
            }
            Err(e) => return Err(e),
            Ok(meta) if meta.len() > MAX_FILE_BYTES => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "redeem codes file too large",
                ));
            }
            Ok(_) => {}
        }
        serde_json::from_slice(&std::fs::read(path)?).map_err(io::Error::other)
    }

    /// Whole-file replace (`character_store::replace_bytes`), so a crash leaves the old file or the new one.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let bytes = serde_json::to_vec_pretty(self).map_err(io::Error::other)?;
        crate::character_store::replace_bytes(path, &bytes)
    }

    /// [`Self::redeem`] for `account`'s character: a code bound to another account is refused
    /// before any limit is read or any use recorded.
    pub fn redeem_for(
        &mut self,
        code: &str,
        character: u64,
        account: u64,
        member: bool,
        today: i64,
    ) -> Result<(u32, Vec<(i32, u32)>), &'static str> {
        let bound = self.codes.get(&normalize(code)).and_then(|c| c.bound);
        if bound.is_some_and(|owner| owner != account) {
            return Err("That code belongs to someone else.");
        }
        self.redeem(code, character, member, today)
    }

    /// Check a player's code and record the use; the grant is the caller's.
    pub fn redeem(
        &mut self,
        code: &str,
        character: u64,
        member: bool,
        today: i64,
    ) -> Result<(u32, Vec<(i32, u32)>), &'static str> {
        let code = self
            .codes
            .get_mut(&normalize(code))
            .filter(|c| c.enabled)
            .ok_or("That code is not valid.")?;
        if code.starts.is_some_and(|day| today < day) {
            return Err("That code is not active yet.");
        }
        if code.expires.is_some_and(|day| today > day) {
            return Err("That code has expired.");
        }
        if code.members && !member {
            return Err("That code is for members.");
        }
        if code.max_uses > 0 && code.uses >= code.max_uses {
            return Err("That code has been used up.");
        }
        let mine = code.redeemed.entry(character).or_default();
        if code.per_character > 0 && *mine >= code.per_character {
            return Err("You already redeemed that code.");
        }
        *mine += 1;
        code.uses += 1;
        Ok((code.coins, code.items.clone()))
    }

    /// One admin `!code` command (words after `code`, original case); the reply lines.
    pub fn admin(
        &mut self,
        words: &[&str],
        today: i64,
        random: impl FnOnce() -> String,
    ) -> Result<(Vec<String>, bool), String> {
        let lower: Vec<String> = words.iter().map(|w| w.to_ascii_lowercase()).collect();
        let lower: Vec<&str> = lower.iter().map(String::as_str).collect();
        let usage = || {
            "Usage: !code list | show <code> | new [code|random] | set <code> <field> <value> | item add|del <code> <item> [qty] | delete <code>".to_owned()
        };
        match lower.as_slice() {
            ["list"] | [] => Ok((self.list_lines(), false)),
            ["show", _] => self.show_lines(words[1]).map(|l| (l, false)),
            ["new", rest @ ..] => {
                let code = match rest {
                    [] | ["random"] => random(),
                    _ => normalize(words[1]),
                };
                if !valid_code(&code) {
                    return Err("A code is 4 to 24 letters, digits or dashes.".into());
                }
                if self.codes.contains_key(&code) {
                    return Err(format!("Code failed: {code} already exists."));
                }
                self.codes.insert(
                    code.clone(),
                    Code {
                        per_character: 1,
                        enabled: true,
                        ..Code::default()
                    },
                );
                self.show_lines(&code).map(|l| (l, true))
            }
            ["set", _, field, ..] if words.len() >= 4 => {
                let code = self.code_mut(words[1])?;
                let value = words[3..].join(" ");
                match *field {
                    "coins" => code.coins = number(&value)?.min(1_000_000),
                    "uses" => code.max_uses = number(&value)?,
                    "percharacter" => code.per_character = number(&value)?,
                    "starts" => code.starts = day(&value, today)?,
                    "expires" => code.expires = day(&value, today)?,
                    "members" => code.members = switch(&value)?,
                    "enabled" => code.enabled = switch(&value)?,
                    "note" => code.note = panel_unescape(&value).chars().take(200).collect(),
                    _ => return Err(usage()),
                }
                self.show_lines(words[1]).map(|l| (l, true))
            }
            ["item", "add", _, item, qty] => {
                let (item, qty) = (
                    item.parse::<i32>().map_err(|_| usage())?,
                    number(qty)?.clamp(1, 999),
                );
                let code = self.code_mut(words[2])?;
                if let Some(entry) = code.items.iter_mut().find(|(id, _)| *id == item) {
                    entry.1 = qty;
                } else if code.items.len() >= MAX_ITEMS {
                    return Err(format!(
                        "Code failed: a code holds at most {MAX_ITEMS} items."
                    ));
                } else {
                    code.items.push((item, qty));
                }
                self.show_lines(words[2]).map(|l| (l, true))
            }
            ["item", "del", _, item] => {
                let item = item.parse::<i32>().map_err(|_| usage())?;
                self.code_mut(words[2])?.items.retain(|(id, _)| *id != item);
                self.show_lines(words[2]).map(|l| (l, true))
            }
            ["delete", _] => {
                self.codes
                    .remove(&normalize(words[1]))
                    .ok_or_else(|| format!("Code failed: no code {}.", normalize(words[1])))?;
                Ok((self.list_lines(), true))
            }
            _ => Err(usage()),
        }
    }

    fn code_mut(&mut self, code: &str) -> Result<&mut Code, String> {
        let code = normalize(code);
        self.codes
            .get_mut(&code)
            .ok_or_else(|| format!("Code failed: no code {code}."))
    }

    /// `codes|begin`, one `codes|entry|code|enabled|uses|maxUses|expires|note` per code, `codes|end`.
    fn list_lines(&self) -> Vec<String> {
        let mut out = vec![line(&["codes", "begin"])];
        for (name, c) in &self.codes {
            out.push(line(&[
                "codes",
                "entry",
                name,
                flag(c.enabled),
                &c.uses.to_string(),
                &c.max_uses.to_string(),
                &c.expires.map(date).unwrap_or_default(),
                &panel_escape(&c.note),
            ]));
        }
        out.push(line(&["codes", "end"]));
        out
    }

    /// `codeedit|begin|...`, one `codeitem|item|qty` per item, `codeedit|end`.
    fn show_lines(&self, code: &str) -> Result<Vec<String>, String> {
        let name = normalize(code);
        let c = self
            .codes
            .get(&name)
            .ok_or_else(|| format!("Code failed: no code {name}."))?;
        let mut out = vec![line(&[
            "codeedit",
            "begin",
            &name,
            &c.coins.to_string(),
            &c.max_uses.to_string(),
            &c.uses.to_string(),
            &c.per_character.to_string(),
            &c.starts.map(date).unwrap_or_default(),
            &c.expires.map(date).unwrap_or_default(),
            flag(c.members),
            flag(c.enabled),
            &panel_escape(&c.note),
        ])];
        for (item, qty) in &c.items {
            out.push(line(&["codeitem", &item.to_string(), &qty.to_string()]));
        }
        out.push(line(&["codeedit", "end"]));
        Ok(out)
    }
}

pub fn normalize(code: &str) -> String {
    code.trim().to_ascii_uppercase()
}

pub fn valid_code(code: &str) -> bool {
    (4..=24).contains(&code.len()) && code.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
}

/// Days since 1970-01-01 (UTC), the unit `starts` and `expires` count in.
pub fn today() -> i64 {
    i64::try_from(crate::gifts::unix_now() / 86_400).unwrap_or(i64::MAX)
}

/// `FR-XXXX-XXXX` from the OS random source (letters and digits without 0/O/1/I).
pub fn random_code() -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut bytes = [0u8; 8];
    let _ = getrandom::getrandom(&mut bytes);
    let pick: String = bytes
        .iter()
        .map(|b| ALPHABET[usize::from(*b) % ALPHABET.len()] as char)
        .collect();
    format!("FR-{}-{}", &pick[..4], &pick[4..])
}

fn flag(on: bool) -> &'static str {
    if on { "1" } else { "0" }
}

fn number(text: &str) -> Result<u32, String> {
    text.trim()
        .parse()
        .map_err(|_| format!("Code failed: {text} is not a number."))
}

fn switch(text: &str) -> Result<bool, String> {
    match text.trim().to_ascii_lowercase().as_str() {
        "on" | "1" | "true" | "yes" => Ok(true),
        "off" | "0" | "false" | "no" => Ok(false),
        _ => Err(format!("Code failed: {text} is not on or off.")),
    }
}

/// `yyyy-MM-dd`, `+days` or `never` (the panel also sends `0` or nothing for "no date").
fn day(text: &str, today: i64) -> Result<Option<i64>, String> {
    let text = text.trim();
    if text.is_empty() || text == "0" || text.eq_ignore_ascii_case("never") {
        return Ok(None);
    }
    if let Some(days) = text.strip_prefix('+') {
        return days
            .parse::<i64>()
            .map(|d| Some(today + d))
            .map_err(|_| format!("Code failed: bad date {text}."));
    }
    let parts: Vec<&str> = text.split('-').collect();
    match parts.as_slice() {
        [y, m, d] => match (y.parse::<i64>(), m.parse::<i64>(), d.parse::<i64>()) {
            (Ok(y), Ok(m @ 1..=12), Ok(d @ 1..=31)) => Ok(Some(days_from_civil(y, m, d))),
            _ => Err(format!("Code failed: bad date {text}.")),
        },
        _ => Err(format!("Code failed: bad date {text}.")),
    }
}

/// Days since 1970-01-01 for a proleptic Gregorian date (H. Hinnant's algorithm).
pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// `yyyy-MM-dd` for days since 1970-01-01.
pub fn date(days: i64) -> String {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_check_limits_dates_and_membership_and_record_each_use() {
        let mut codes = Codes::default();
        let today = days_from_civil(2026, 10, 31);
        assert_eq!(date(today), "2026-10-31");
        let new = codes
            .admin(&["new", "fr-test1"], today, String::new)
            .unwrap();
        assert!(new.1 && new.0[0].contains("|codeedit|begin|FR-TEST1|0|0|0|1|"));
        codes
            .admin(&["set", "FR-TEST1", "coins", "500"], today, String::new)
            .unwrap();
        codes
            .admin(
                &["item", "add", "FR-TEST1", "2644", "1"],
                today,
                String::new,
            )
            .unwrap();
        codes
            .admin(&["set", "FR-TEST1", "expires", "+1"], today, String::new)
            .unwrap();
        assert_eq!(
            codes.redeem("fr-test1", 7, false, today),
            Ok((500, vec![(2644, 1)]))
        );
        assert_eq!(
            codes.redeem("FR-TEST1", 7, false, today),
            Err("You already redeemed that code.")
        );
        assert_eq!(
            codes.redeem("FR-TEST1", 8, false, today + 2),
            Err("That code has expired.")
        );
        codes
            .admin(&["set", "FR-TEST1", "members", "on"], today, String::new)
            .unwrap();
        assert_eq!(
            codes.redeem("FR-TEST1", 8, false, today),
            Err("That code is for members.")
        );
        codes
            .admin(&["set", "FR-TEST1", "uses", "1"], today, String::new)
            .unwrap();
        assert_eq!(
            codes.redeem("FR-TEST1", 8, true, today),
            Err("That code has been used up.")
        );
        assert_eq!(
            codes.redeem("NOPE", 8, true, today),
            Err("That code is not valid.")
        );
        for n in 0..MAX_ITEMS as i32 {
            let _ = codes.admin(
                &["item", "add", "FR-TEST1", &(n + 1).to_string(), "2"],
                today,
                String::new,
            );
        }
        assert!(
            codes
                .admin(
                    &["item", "add", "FR-TEST1", "99999", "1"],
                    today,
                    String::new
                )
                .is_err()
        );
        let list = codes.admin(&["list"], today, String::new).unwrap().0;
        assert!(list[1].contains("|codes|entry|FR-TEST1|1|1|1|2026-11-01|"));
        codes
            .admin(&["delete", "fr-test1"], today, String::new)
            .unwrap();
        assert!(codes.codes.is_empty());
    }
}
