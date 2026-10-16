//! Work item 075 acceptance: recovered Postman delivery routes.
//!
//! ORIGINAL evidence used by these vectors (all outside this test):
//! * `lab/working/OSFR.Client/OSFR Client/locale/en_us_data.dat` - the stock original client
//!   locale (sha256 `9c764d76...`), the hashed-id string store. It carries the route quest lines
//!   (`Postman: Running the Route`, "Deliver mail along the Sanctuary route before time runs out,
//!   then return to Felipe.", "Click on each of the mailboxes specified before time runs out.")
//!   and every mailbox address as its own whole-string row.
//! * `lab/reports/job-catalogue-20261009T022023Z/locale-en_us.json` - the preserved original
//!   locale export (85,932 rows). It resolves the mailbox `NameId`s (e.g. 117040 ->
//!   "1 Seaspray Terrace"), the delivery activity names/descriptions (`Can You Deliver?` 93381,
//!   `Mail Marksman` 93757) and the delivery notification string 388255 (`Mail Delivered!`).
//! * `lab/working/OSFR.Client/OSFR Client/Resources/MiniGameData.txt` - the client activity table
//!   holding the Postman delivery rows (3/5/6/12/13/14/16/24/72/238/239/241/248/263/1002).
//! * `lab/references/Sanctuary/src/Resources/Npcs.json` and `Models.txt` - the community mirror of
//!   the original actor and model tables: 23 mailbox actors, their address `NameId` and the two
//!   mailbox models (910 `sg_mailbox_01.adr`, 340 `sg_mailbox_group_01.adr`).
//! * `lab/references/Sanctuary/src/Scripts/Zone/FabledRealms.lua` - the community zone script; it
//!   spawns all 23 mailbox actors with world positions and headings.
//! * `lab/decompiled/client-ui/ScriptsBase.lua` - the client delivery notification:
//!   `SH_DeliveryGame.DeliverLetter` -> `NotificationHandler:DeliverLetter(0.7)` ->
//!   `CompleteObjective(Ui.GetString(388255), "", "")`.
//! * `lab/references/freerealms-wiki/wiki.sqlite` - the `Postman` page: trainers, the unlock
//!   route, and the quest list including `Postman: Running the Route`.
//!
//! BOUNDARIES: this module is recovery metadata plus a pure validation helper. It grants no job,
//! item, currency or experience and sends no packet. The per-activity route membership, the
//! delivery order and the numeric timer were not recovered; they stay explicit gaps instead of
//! invented values.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the lead
//! registers `pub mod piece_075;` under `src/restoration/mod.rs`/`src/lib.rs`.
#[path = "../src/restoration/piece_075.rs"]
#[allow(dead_code)]
mod piece_075;

use piece_075::{
    DELIVERY_ACTIVITIES, DELIVERY_NOTIFICATION_ICON_ID, DELIVERY_NOTIFICATION_STRING_ID,
    DELIVERY_OBJECTIVE_PACKETS_RECOVERED, DELIVERY_TIMER_SECONDS, DeliveryError, DeliveryRun,
    MAILBOXES, MailboxKind, ROUTE_MEMBERSHIP_RECOVERED, ROUTE_ORDER_RECOVERED, ROUTE_TEXT_CLAIMS,
    mailbox, mailbox_by_address, validate,
};

#[test]
fn mailbox_catalogue_keeps_the_recovered_addresses_and_models() {
    assert_eq!(MAILBOXES.len(), 23);

    let sea = mailbox(1755).expect("1 Seaspray Terrace actor");
    assert_eq!(sea.name_id, 117040);
    assert_eq!(sea.address, "1 Seaspray Terrace");
    assert_eq!(sea.model_id, 910);
    assert_eq!(sea.model_file, "sg_mailbox_01.adr");
    assert_eq!(sea.kind, MailboxKind::StreetMailbox);
    assert_eq!(sea.spawn.guid, 100000001755);
    assert_eq!(sea.spawn.heading, -3.140697);

    let group = mailbox(2297).expect("Merry Vale Postboxes actor");
    assert_eq!(group.address, "Merry Vale Postboxes");
    assert_eq!(group.model_id, 340);
    assert_eq!(group.model_file, "sg_mailbox_group_01.adr");
    assert_eq!(group.kind, MailboxKind::PostboxGroup);

    assert!(mailbox(1756).is_none());

    // Every recovered address is unique and reachable both ways.
    for row in MAILBOXES {
        assert_eq!(
            mailbox_by_address(row.address).map(|found| found.npc_id),
            Some(row.npc_id),
            "{}",
            row.address
        );
    }
    assert!(mailbox_by_address("2 Seaspray Terrace").is_none());
}

#[test]
fn delivery_activities_stay_on_the_recovered_client_rows() {
    let ids: Vec<i32> = DELIVERY_ACTIVITIES.iter().map(|row| row.id).collect();
    assert_eq!(
        ids,
        vec![
            3, 5, 6, 12, 13, 14, 16, 24, 72, 238, 239, 241, 248, 263, 1002
        ]
    );

    let can_you_deliver = DELIVERY_ACTIVITIES
        .iter()
        .find(|row| row.id == 238)
        .expect("Can You Deliver?");
    assert_eq!(can_you_deliver.name, "Can You Deliver?");
    assert_eq!(can_you_deliver.name_id, 93381);
    assert_eq!(can_you_deliver.description_id, 93382);
    assert_eq!(can_you_deliver.difficulty, 1);
    assert!(can_you_deliver.live);

    let special_delivery = DELIVERY_ACTIVITIES
        .iter()
        .find(|row| row.id == 12)
        .expect("Special Delivery row");
    assert_eq!(special_delivery.name, "NOT LIVE - Special Delivery");
    assert!(!special_delivery.live);
    assert!(
        special_delivery
            .description
            .contains("Snowhill Post Office")
    );

    for row in DELIVERY_ACTIVITIES {
        // Every recovered delivery activity is type 2 except `Scattered Mail` (241), which the
        // client table types as 1 (the scavenger-hunt gameplay the same quest family uses).
        let expected = if row.id == 241 { 1 } else { 2 };
        assert_eq!(row.activity_type, expected, "activity {} type", row.id);
    }
}

#[test]
fn route_text_claims_keep_the_recovered_route_lines() {
    let sanctuary_route = ROUTE_TEXT_CLAIMS
        .iter()
        .find(|row| row.hash_id == 29_679_012)
        .expect("Sanctuary route line");
    assert_eq!(
        sanctuary_route.text,
        "Deliver mail along the Sanctuary route before time runs out, then return to Felipe."
    );
    assert_eq!(sanctuary_route.role, "Sanctuary route quest description");

    let mailboxes = ROUTE_TEXT_CLAIMS
        .iter()
        .find(|row| row.hash_id == 557_373_038)
        .expect("mailbox click objective");
    assert_eq!(
        mailboxes.text,
        "Click on each of the mailboxes specified before time runs out."
    );

    let people = ROUTE_TEXT_CLAIMS
        .iter()
        .find(|row| row.hash_id == 667_138_698)
        .expect("npc delivery objective");
    assert_eq!(
        people.text,
        "Interact with each of the specified people to deliver the mail to them."
    );
}

#[test]
fn delivery_notification_matches_the_client_lua() {
    assert_eq!(DELIVERY_NOTIFICATION_STRING_ID, 388255);
    assert_eq!(DELIVERY_NOTIFICATION_ICON_ID, 1237);
}

#[test]
fn delivery_run_validates_targets_and_completion_without_inventing_a_route() {
    let run_targets = [1755, 1816, 2297];
    let mut run = DeliveryRun::start(&run_targets).expect("known targets");
    assert_eq!(run.total(), 3);
    assert_eq!(run.remaining(), 3);
    assert!(!run.is_complete());

    assert!(run.deliver(1755).is_ok());
    assert_eq!(run.remaining(), 2);
    match run.deliver(1755) {
        Err(DeliveryError::AlreadyDelivered(1755)) => {}
        other => panic!("unexpected duplicate result {other:?}"),
    }
    match run.deliver(1756) {
        Err(DeliveryError::UnknownTarget(1756)) => {}
        other => panic!("unexpected unknown result {other:?}"),
    }
    assert!(!run.is_complete());
    assert!(run.deliver(1816).is_ok());
    assert!(run.deliver(2297).is_ok());
    assert!(run.is_complete());
    assert_eq!(run.remaining(), 0);

    match DeliveryRun::start(&[1755, 12345]) {
        Err(DeliveryError::UnknownTarget(12345)) => {}
        other => panic!("unexpected start result {other:?}"),
    }
}

#[test]
fn route_membership_order_and_timer_stay_reported_gaps() {
    assert!(!ROUTE_MEMBERSHIP_RECOVERED);
    assert!(!ROUTE_ORDER_RECOVERED);
    assert!(!DELIVERY_OBJECTIVE_PACKETS_RECOVERED);
    assert_eq!(DELIVERY_TIMER_SECONDS, None);
    assert!(validate().is_ok());
}
