//! L081 (owner 2026-10-31): the admin-only GM wheel on another player's click wheel. This is the
//! pure part: the pages, their entries and what each entry does. The wheel is the client's radial
//! menu (26/9 rows `id, icon, label`); a page is a new 26/9 for the same player, as Sanctuary
//! PR #163's referee wheel does. Labels are locale string ids: stock ids where the client has the
//! word, and a reserved block (`LABEL_BASE`) that the launcher's GM tools write into an admin's own
//! locale (`LABELS`).
use crate::account_store::AccountRole;

/// The GM entry on the player wheel; 1001/1002 are Inspect and Trade.
pub const ENTRY_GM: i32 = 1100;
/// Every GM wheel entry id is in 1100..1200.
pub const ENTRIES: std::ops::Range<i32> = 1100..1200;
/// The stock radial icon Sanctuary's referee wheel uses.
pub const ICON: i32 = 133;

// Stock client strings (exact-text matches in en_us_data.dat, WHEEL-MAP.md section 2).
const MUTE: i32 = 430_587;
const KICK: i32 = 24_540;
const TELEPORT: i32 = 3_249;
const COINS: i32 = 865;
const STATION_CASH: i32 = 385_941;
const ADMIN: i32 = 71_932;
const BACK: i32 = 1_861;

/// Reserved locale ids for labels the client does not have (above the authored-text allocator).
pub const LABEL_BASE: i32 = 7_900_000;
pub const LABELS: [(i32, &str); 16] = [
    (LABEL_BASE, "GM"),
    (LABEL_BASE + 1, "Ban"),
    (LABEL_BASE + 2, "Summon"),
    (LABEL_BASE + 3, "Role"),
    (LABEL_BASE + 4, "10 minutes"),
    (LABEL_BASE + 5, "1 hour"),
    (LABEL_BASE + 6, "1 day"),
    (LABEL_BASE + 7, "Forever"),
    (LABEL_BASE + 8, "100"),
    (LABEL_BASE + 9, "1,000"),
    (LABEL_BASE + 10, "10,000"),
    (LABEL_BASE + 11, "100,000"),
    (LABEL_BASE + 12, "1,000,000"),
    (LABEL_BASE + 13, "Player"),
    (LABEL_BASE + 14, "Tester"),
    (LABEL_BASE + 15, "Moderator"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Root,
    Mute,
    Ban,
    Coins,
    StationCash,
    Role,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GmAction {
    Mute {
        minutes: u32,
    },
    Kick,
    /// `None` = forever.
    Ban {
        minutes: Option<u32>,
    },
    Coins(u32),
    StationCash(u32),
    GoTo,
    Summon,
    Role(AccountRole),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Open(Page),
    Do(GmAction),
}

struct Entry {
    id: i32,
    label: i32,
    step: Step,
}

const fn open(id: i32, label: i32, page: Page) -> Entry {
    Entry {
        id,
        label,
        step: Step::Open(page),
    }
}

const fn act(id: i32, label: i32, action: GmAction) -> Entry {
    Entry {
        id,
        label,
        step: Step::Do(action),
    }
}

const fn label(n: i32) -> i32 {
    LABEL_BASE + n
}

fn entries(page: Page) -> Vec<Entry> {
    use GmAction::*;
    match page {
        Page::Root => vec![
            open(1101, MUTE, Page::Mute),
            act(1102, KICK, Kick),
            open(1103, label(1), Page::Ban),
            open(1104, COINS, Page::Coins),
            open(1105, STATION_CASH, Page::StationCash),
            act(1106, TELEPORT, GoTo),
            act(1107, label(2), Summon),
            open(1108, label(3), Page::Role),
        ],
        Page::Mute => vec![
            act(1111, label(4), Mute { minutes: 10 }),
            act(1112, label(5), Mute { minutes: 60 }),
            act(1113, label(6), Mute { minutes: 1440 }),
            open(1119, BACK, Page::Root),
        ],
        Page::Ban => vec![
            act(
                1121,
                label(6),
                Ban {
                    minutes: Some(1440),
                },
            ),
            act(1122, label(7), Ban { minutes: None }),
            open(1129, BACK, Page::Root),
        ],
        Page::Coins => vec![
            act(1131, label(9), Coins(1_000)),
            act(1132, label(10), Coins(10_000)),
            act(1133, label(11), Coins(100_000)),
            act(1134, label(12), Coins(1_000_000)),
            open(1139, BACK, Page::Root),
        ],
        Page::StationCash => vec![
            act(1141, label(8), StationCash(100)),
            act(1142, label(9), StationCash(1_000)),
            act(1143, label(10), StationCash(10_000)),
            open(1149, BACK, Page::Root),
        ],
        Page::Role => vec![
            act(1151, label(13), Role(AccountRole::Player)),
            act(1152, label(14), Role(AccountRole::Tester)),
            act(1153, label(15), Role(AccountRole::Moderator)),
            act(1154, ADMIN, Role(AccountRole::Admin)),
            open(1159, BACK, Page::Root),
        ],
    }
}

/// A page's wheel rows: `(entry id, icon, label string id)`.
pub fn rows(page: Page) -> Vec<(i32, i32, i32)> {
    entries(page)
        .into_iter()
        .map(|e| (e.id, ICON, e.label))
        .collect()
}

/// The GM entry the player wheel gains for an admin.
pub fn gm_row() -> (i32, i32, i32) {
    (ENTRY_GM, ICON, LABEL_BASE)
}

/// What a chosen GM entry does: open a page, or act on the player.
pub fn resolve(id: i32) -> Option<Step> {
    if id == ENTRY_GM {
        return Some(Step::Open(Page::Root));
    }
    [
        Page::Root,
        Page::Mute,
        Page::Ban,
        Page::Coins,
        Page::StationCash,
        Page::Role,
    ]
    .into_iter()
    .flat_map(entries)
    .find(|e| e.id == id)
    .map(|e| e.step)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_entry_is_unique_fits_a_wheel_resolves_and_has_a_label() {
        let pages = [
            Page::Root,
            Page::Mute,
            Page::Ban,
            Page::Coins,
            Page::StationCash,
            Page::Role,
        ];
        let mut seen = std::collections::BTreeSet::new();
        for page in pages {
            let rows = rows(page);
            assert!(rows.len() <= 9, "{page:?}");
            for (id, icon, label) in rows {
                assert!(
                    ENTRIES.contains(&id) && id != ENTRY_GM && seen.insert(id),
                    "{id}"
                );
                assert_eq!(icon, ICON);
                assert!(resolve(id).is_some());
                if label >= LABEL_BASE {
                    assert!(LABELS.iter().any(|(l, _)| *l == label), "{label}");
                }
            }
        }
        assert_eq!(resolve(ENTRY_GM), Some(Step::Open(Page::Root)));
        assert_eq!(
            resolve(1122),
            Some(Step::Do(GmAction::Ban { minutes: None }))
        );
        assert_eq!(resolve(1001), None, "Inspect is not a GM entry");
    }
}
