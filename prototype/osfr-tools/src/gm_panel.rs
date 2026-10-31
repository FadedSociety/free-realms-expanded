//! L072: the referee (GM) panel's chat protocol, for the client mod from Sanctuary PR #163.
//!
//! Adapted from Sanctuary (Open-Source-Free-Realms, AGPL-3.0), PR #163 (`feature/gm-authoring`,
//! e9242e6), `ClientMods/chatWindowCtrl/gm_panel.as`: the panel types `!`-commands into chat
//! (`gmSend`) and reads back system lines that carry `[[SANCTUARY_GM]]|kind|field|...`
//! (`gmHandleData`), hiding them from the chat window. The server side here is ours.
//!
//! Commands we already have become their `realm` form ([`alias`]), so the existing role gate
//! decides them. `!gm` itself (open the panel, list players) is answered by the gateway, which
//! sees every player; it is admin only and never reaches chat.

/// The marker the panel looks for in a system line (`gmToken`).
pub const TOKEN: &str = "[[SANCTUARY_GM]]";
/// Results per page: Sanctuary's `GmChatCommand.PageSize`, the panel's 3x6 grid.
pub const PAGE: usize = 18;

/// The panel's commands that map onto an existing realm command, as that command.
/// (R) `!godmode on|off` toggles: the realm command keeps no state the panel can name.
pub fn alias(text: &str) -> Option<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    Some(match words.as_slice() {
        ["!give", item, count] => format!("realm give {item} {count}"),
        // L081: the World tab's Promote / Demote prompts (admin only: `realm role`).
        ["!admin", "promote", name @ ..] if !name.is_empty() => {
            format!("realm role {} moderator", name.join(" "))
        }
        ["!admin", "demote", name @ ..] if !name.is_empty() => {
            format!("realm role {} player", name.join(" "))
        }
        // L081: the World tab's "Teleport x y z" prompt.
        ["!teleport", x, y, z] => format!("realm goto {x} {y} {z}"),
        // L081: redeem codes (the gold hub tree, the redeem box, the Codes tab).
        ["!redeem", rest @ ..] => format!("realm redeem {}", rest.join(""))
            .trim_end()
            .to_owned(),
        ["!code", rest @ ..] => format!("realm code {}", rest.join(" "))
            .trim_end()
            .to_owned(),
        ["!godmode", "on" | "off"] => "realm god".to_owned(),
        [
            "!mod",
            verb @ ("mute" | "unmute" | "kick" | "ban" | "unban"),
            rest @ ..,
        ] if !rest.is_empty() => {
            format!("realm {verb} {}", rest.join(" "))
        }
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListKind {
    Items,
    Npcs,
    Players,
    Houses,
}

impl ListKind {
    fn name(self) -> &'static str {
        match self {
            Self::Items => "items",
            Self::Npcs => "npcs",
            Self::Players => "players",
            Self::Houses => "houses",
        }
    }

    /// The row kind the panel collects under this list.
    fn row(self) -> &'static str {
        match self {
            Self::Items => "item",
            Self::Npcs => "npc",
            Self::Players => "player",
            Self::Houses => "house",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    /// `!gm` (the hub's Referee Tools button): toggle the panel.
    Open,
    /// `!gm <tab> <page> [search]`.
    List {
        kind: ListKind,
        page: usize,
        search: String,
    },
    /// L081 `!gm pos`: where the referee stands.
    Pos,
    /// L081 `!gm goto <First Last>` (Players tab "Go to"): the referee moves to that player.
    GoTo(String),
    /// L081 `!gm summon <First Last>` (Players tab "Summon"): that player moves to the referee.
    Summon(String),
    /// Any other `!gm` line: not built yet.
    Other,
}

/// A `!gm` line, or `None` for anything else.
pub fn parse(text: &str) -> Option<Request> {
    let rest = text.trim().strip_prefix("!gm")?;
    if !(rest.is_empty() || rest.starts_with(' ')) {
        return None;
    }
    let mut words = rest.split_whitespace();
    let Some(first) = words.next() else {
        return Some(Request::Open);
    };
    let kind = match first {
        "items" => ListKind::Items,
        "npcs" => ListKind::Npcs,
        "players" => ListKind::Players,
        "houses" => ListKind::Houses,
        "pos" => return Some(Request::Pos),
        "goto" => return Some(Request::GoTo(words.collect::<Vec<_>>().join(" "))),
        "summon" => return Some(Request::Summon(words.collect::<Vec<_>>().join(" "))),
        _ => return Some(Request::Other),
    };
    let page = words
        .next()
        .and_then(|page| page.parse().ok())
        .filter(|page| *page > 0)
        .unwrap_or(1);
    Some(Request::List {
        kind,
        page,
        search: words.collect::<Vec<_>>().join(" "),
    })
}

/// The panel's free-text escape (`PanelTextHelper.Escape`): `\\`, `\n`, `\p` for `|`, `\t`.
pub fn panel_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '|' => out.push_str("\\p"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

/// The panel's send escape undone (`\\`, `\n`, `\p`, `\t`, `\s` for a kept space).
pub fn panel_unescape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('p') => out.push('|'),
            Some('t') => out.push('\t'),
            Some('s') => out.push(' '),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    out
}

/// One panel data line. `|` separates fields, so it never appears inside one; `chat::system`
/// escapes `&`, `<` and `>`, which the panel's `gmUnescape` turns back.
pub fn line(fields: &[&str]) -> String {
    let mut out = TOKEN.to_owned();
    for field in fields {
        out.push('|');
        out.push_str(&field.replace('|', "/"));
    }
    out
}

/// A list reply: the header (`|kind|search|page|pages|total`), one row line per result on the
/// page, then `|end`. Rows match the search (any field, ignoring case); an out-of-range page
/// shows the last one.
pub fn list(kind: ListKind, page: usize, search: &str, rows: &[Vec<String>]) -> Vec<String> {
    let needle = search.to_lowercase();
    let matching: Vec<&Vec<String>> = rows
        .iter()
        .filter(|row| needle.is_empty() || row.iter().any(|f| f.to_lowercase().contains(&needle)))
        .collect();
    let pages = matching.len().div_ceil(PAGE).max(1);
    let page = page.clamp(1, pages);
    let mut out = vec![line(&[
        kind.name(),
        search,
        &page.to_string(),
        &pages.to_string(),
        &matching.len().to_string(),
    ])];
    for row in matching.iter().skip((page - 1) * PAGE).take(PAGE) {
        let mut fields = vec![kind.row()];
        fields.extend(row.iter().map(String::as_str));
        out.push(line(&fields));
    }
    out.push(line(&["end"]));
    out
}

/// L072: the Items tab's rows, one per item id, from `(id, serialized ClientItemDefinition hex)`.
/// A record starts with the fields id, name id, description id and icon id; the row is
/// `[id, icon, "", "", name]` (`gmItemCell` reads 0, 1 and 4). L081: the whole item table. An item
/// without a name shows `#id` (Sanctuary hides them; a referee can still give one by id).
pub fn item_rows<'a>(
    items: impl IntoIterator<Item = (i32, &'a str)>,
    name: impl Fn(i32) -> Option<String>,
) -> Vec<Vec<String>> {
    let field = |hex: &str, index: usize| {
        let bytes = hex.get(index * 8..index * 8 + 8)?;
        let mut le = [0u8; 4];
        for (at, byte) in le.iter_mut().enumerate() {
            *byte = u8::from_str_radix(bytes.get(at * 2..at * 2 + 2)?, 16).ok()?;
        }
        Some(i32::from_le_bytes(le))
    };
    let mut rows = std::collections::BTreeMap::new();
    for (id, hex) in items {
        let (Some(name_id), Some(icon)) = (field(hex, 1), field(hex, 3)) else {
            continue;
        };
        rows.entry(id).or_insert_with(|| {
            vec![
                id.to_string(),
                icon.to_string(),
                String::new(),
                String::new(),
                name(name_id).unwrap_or_else(|| format!("#{id}")),
            ]
        });
    }
    rows.into_values().collect()
}
