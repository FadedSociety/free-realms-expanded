//! L072: the referee panel's chat protocol (commands in, `[[SANCTUARY_GM]]` lines out).
use osfr_tools::{
    chat,
    gm_panel::{ListKind, PAGE, Request, TOKEN, alias, line, list, parse},
};

/// The panel's commands we already have are realm commands under the same role gate; anything
/// else is left alone.
#[test]
fn panel_commands_we_have_become_realm_commands() {
    assert_eq!(
        alias("!give 1234 999").as_deref(),
        Some("realm give 1234 999")
    );
    assert_eq!(alias("!godmode on").as_deref(), Some("realm god"));
    assert_eq!(
        alias("!mod mute Bad Actor 30").as_deref(),
        Some("realm mute Bad Actor 30")
    );
    for other in [
        "!give 1",
        "!godmode",
        "!mod mute",
        "!mod promote X",
        "hello !give 1 2",
        "!gm",
    ] {
        assert_eq!(alias(other), None, "{other}");
    }
    // The chat reader hands gameplay the realm form.
    let typed = chat::local_request("!give 1234 2");
    assert_eq!(
        chat::request(&typed, 1).unwrap().as_deref(),
        Some("realm give 1234 2")
    );
}

#[test]
fn gm_lines_parse_into_open_lists_and_the_rest() {
    assert_eq!(parse("!gm"), Some(Request::Open));
    assert_eq!(
        parse(" !gm players 2 rae smith "),
        Some(Request::List {
            kind: ListKind::Players,
            page: 2,
            search: "rae smith".into()
        })
    );
    assert_eq!(
        parse("!gm items x"),
        Some(Request::List {
            kind: ListKind::Items,
            page: 1,
            search: String::new()
        })
    );
    assert_eq!(parse("!gm goto Rae"), Some(Request::Other));
    for other in ["!gmx", "gm players 1", "hello"] {
        assert_eq!(parse(other), None, "{other}");
    }
}

/// Lists page by `PAGE`, filter by search, never let `|` into a field, and end with `|end`.
#[test]
fn a_list_reply_has_the_header_its_page_and_an_end() {
    let rows: Vec<Vec<String>> = (1..=PAGE + 3)
        .map(|n| vec![n.to_string(), format!("Player {n}"), "zone 1".into()])
        .collect();
    let page_two = list(ListKind::Players, 2, "", &rows);
    assert_eq!(page_two[0], format!("{TOKEN}|players||2|2|{}", PAGE + 3));
    assert_eq!(page_two.len(), 1 + 3 + 1);
    assert_eq!(
        page_two[1],
        format!("{TOKEN}|player|{}|Player {}|zone 1", PAGE + 1, PAGE + 1)
    );
    assert_eq!(page_two.last().unwrap(), &format!("{TOKEN}|end"));
    let found = list(ListKind::Players, 9, "player 1", &rows);
    // "Player 1" and "Player 10".."Player 15"; page 9 is out of range, so the last page shows.
    assert_eq!(found[0], format!("{TOKEN}|players|player 1|1|1|7"));
    let empty = list(ListKind::Items, 1, "", &[]);
    assert_eq!(
        empty,
        [format!("{TOKEN}|items||1|1|0"), format!("{TOKEN}|end")]
    );
    assert_eq!(line(&["player", "a|b"]), format!("{TOKEN}|player|a/b"));
}

/// L072: Items rows come from the startup catalogue (id, name id, description id, icon id lead
/// each record), one per id, named from the locale or `#id`.
#[test]
fn item_rows_read_the_catalogue_records() {
    use osfr_tools::{gm_panel::item_rows, startup_snapshot::ItemDefinitionSnapshot};
    let record = |id: i32, name_id: i32, icon: i32| {
        let hex: String = [id, name_id, 0, icon, 261]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .map(|b| format!("{b:02x}"))
            .collect();
        ItemDefinitionSnapshot {
            id,
            serialized_hex: hex,
        }
    };
    let items = [
        record(56757, 393077, 3897),
        record(12, 5, 7),
        record(56757, 1, 1),
    ];
    let named = |id: i32| (id == 393077).then(|| "Heart Bed".to_owned());
    assert_eq!(
        item_rows(&items, named),
        [
            vec!["12", "7", "", "", "#12"],
            vec!["56757", "3897", "", "", "Heart Bed"],
        ]
    );
}

/// L072: a locale row is `hash<TAB>tag<TAB>text`; the text may hold tabs, the BOM and CRLF go.
#[test]
fn locale_rows_keep_each_rows_text() {
    let path = std::env::temp_dir().join(format!("osfr-locale-{}.dat", std::process::id()));
    std::fs::write(
        &path,
        "\u{feff}81901\tugdt\tHeart Bed\r\n88296\tucdt\tNom\tNom\r\nbad row\r\n",
    )
    .unwrap();
    let rows = osfr_tools::world_text::locale_rows(&path).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[&81901], "Heart Bed");
    assert_eq!(rows[&88296], "Nom\tNom");
    let _ = std::fs::remove_file(path);
}
