//! Property acquisition acceptance for `src/restoration/piece_021.rs` (work item 021).
//!
//! ORIGINAL evidence used by these vectors (all read outside this test):
//! * `lab/decompiled/client/FreeRealms/pseudocode.c` — the housing family is base opcode 127. The
//!   client constructs `ClientHousingPacketRequestGrant` with inner opcode 23 at `FUN_00aa4fd0`
//!   (`FUN_00aa4990(0x17)`) and sends it once per UI state from `FUN_00ac7af0` behind the `+0x2880`
//!   guard; it constructs `HousingPacketNotifyHouseAdded` with inner opcode 49 at `FUN_00aa86d0`
//!   (`FUN_00aa4990(0x31)`), whose handler prints the `HousingNotifyNewHouse` key with the fallback
//!   text "You have received a new %s." (`pseudocode.c:1400165`, `:1400185`). The visit picker prints
//!   the `HousingServerNoUnlockedHouses` key as "That player does not have any houses you can visit
//!   at this time." (`:1407515`, `:1407517`), so unlocked houses are tracked per character.
//! * `lab/decompiled/server/LibSOE/SOE/BaseHousingPackets.cs` and the community packet model agree
//!   that member 23 is `ClientHousingPacketRequestGrant` and member 49 is
//!   `HousingPacketNotifyHouseAdded`.
//! * `lab/references/Sanctuary/src/Resources/Houses.json` — 31 property rows (id, zone id, name id,
//!   icon, spawn, build areas). `lab/references/Sanctuary/src/Resources/ClientItemDefinitions.json`
//!   carries 36 `Type == 16` property items whose `NameId` joins to those rows; the community packet
//!   model special-cases `Type == 16` (`Sanctuary.Packet.Common/BaseItemDefinition.cs:145`).
//! * `lab/references/freerealms-wiki/wiki.sqlite` — the Lot pages record the Station Cash price pair
//!   (regular, member) and the grant channels: every new player gets one free home (Apartment, rev
//!   49504; Wilds Condo replaced it on 2012-01-26 and existing Apartment owners received it too, rev
//!   49531), members claim the Small Wilds House (rev 49611), the Sunset Party Boat came from the
//!   February 2014 membership pack (rev 49527) and the Rumbledome Lot from a TCG loot card (rev
//!   49518). Membership gives "10% discount on SC and Coin items" (rev 48146), matching the item
//!   `MemberDiscount` field and the recorded price pairs (549/494, 300/270, 1499/1349).
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the lead
//! registers `pub mod restoration;` in `src/lib.rs`. Integration step for the lead: register the
//! module and drop the include.

#[path = "../src/restoration/piece_021.rs"]
#[allow(dead_code)]
mod piece_021;

use piece_021::{
    AcquisitionChannel, AcquisitionError, AcquisitionOutcome, AcquisitionRequest,
    PROPERTY_CATALOGUE_SCHEMA_VERSION, PropertyAcquisition, StationCashPrice,
};

const CHARACTER: u64 = 0x1122_3344_5566_7788;
const OTHER: u64 = 0x8877_6655_4433_2211;
const APARTMENT: i32 = 24;
const WILDS_CONDO: i32 = 25;
const BAT_CAVERN_LOT: i32 = 15;
const SEASIDE_LOT: i32 = 14;
const SMALL_WILDS_HOUSE: i32 = 27;
const SUNSET_PARTY_BOAT: i32 = 2;

/// Recovered rows. Ids, zone ids and name ids come from `Houses.json`; the deed item ids, the
/// members-only flag and the Station Cash pairs come from `ClientItemDefinitions.json` and the wiki
/// lot pages cited in the header.
const CATALOGUE: &str = r#"{
  "schema_version": 1,
  "properties": [
    {
      "property_id": 24, "zone_id": 24, "name_id": 420151, "name": "Apartment",
      "deed_item_ids": [10337], "members_only": false, "channels": ["starter_grant"],
      "station_cash": null, "evidence": {"wiki_revision": "49504", "evidence": "ItemDefinitions:10337 Type 16"}
    },
    {
      "property_id": 25, "zone_id": 25, "name_id": 296, "name": "Wilds Condo",
      "deed_item_ids": [17184], "members_only": false, "channels": ["starter_grant", "bonus_grant"],
      "station_cash": null, "evidence": {"wiki_revision": "49531"}
    },
    {
      "property_id": 15, "zone_id": 15, "name_id": 69977, "name": "Bat Cavern Lot",
      "deed_item_ids": [17195], "members_only": false, "channels": [],
      "station_cash": null, "evidence": {"wiki_revision": "49505", "channels": "no recorded price or grant"}
    },
    {
      "property_id": 14, "zone_id": 14, "name_id": 8320, "name": "Seaside Lot",
      "deed_item_ids": [10384], "members_only": false, "channels": ["marketplace"],
      "station_cash": {"regular": 300, "member": 270}, "evidence": {"wiki_revision": "49520"}
    },
    {
      "property_id": 27, "zone_id": 27, "name_id": 6289, "name": "Small Wilds House",
      "deed_item_ids": [2213], "members_only": true, "channels": ["marketplace", "membership_claim"],
      "station_cash": {"regular": 549, "member": 494}, "evidence": {"wiki_revision": "49611"}
    },
    {
      "property_id": 2, "zone_id": 2, "name_id": 5103265, "name": "Sunset Party Boat",
      "deed_item_ids": [79149], "members_only": false, "channels": ["bonus_grant"],
      "station_cash": null, "evidence": {"wiki_revision": "49527"}
    }
  ]
}"#;

fn acquisition() -> PropertyAcquisition {
    PropertyAcquisition::from_json(CATALOGUE.as_bytes()).expect("recovered catalogue must validate")
}

#[test]
fn catalogue_accepts_recovered_rows_and_pins_schema() {
    let catalogue = acquisition();
    assert_eq!(
        catalogue.schema_version(),
        PROPERTY_CATALOGUE_SCHEMA_VERSION
    );
    assert_eq!(catalogue.properties().count(), 6);
    let seaside = catalogue.property(SEASIDE_LOT).expect("seaside lot");
    assert_eq!(seaside.name_id, 8320);
    assert_eq!(seaside.deed_item_ids, vec![10384]);
    assert_eq!(seaside.station_cash.map(|price| price.member), Some(270));
}

/// A property with no recovered acquisition channel (Bat Cavern Lot has a wiki page, a deed item and
/// a Houses.json row, but no recorded price or grant) stays known and refuses every channel instead
/// of inventing one.
#[test]
fn a_property_without_a_recovered_channel_refuses_every_channel() {
    let mut engine = acquisition();
    engine.set_station_cash(CHARACTER, 10_000);
    for channel in [
        AcquisitionChannel::StarterGrant,
        AcquisitionChannel::Marketplace,
        AcquisitionChannel::MembershipClaim,
        AcquisitionChannel::BonusGrant,
    ] {
        assert!(matches!(
            engine.acquire(AcquisitionRequest {
                character_guid: CHARACTER,
                property_id: BAT_CAVERN_LOT,
                channel,
                member: true,
            }),
            Err(AcquisitionError::ChannelNotAllowed {
                property_id: BAT_CAVERN_LOT,
                ..
            })
        ));
    }
    assert!(!engine.owns(CHARACTER, BAT_CAVERN_LOT));
    assert_eq!(engine.station_cash(CHARACTER), 10_000);
}

/// RECONSTRUCTED policy check: the wiki price pairs are all the floor of a 10% member discount
/// (`300 -> 270`, `549 -> 494`, `1499 -> 1349`, `499 -> 449`). A catalogue that disagrees with that
/// recovered rule is refused rather than silently trusted.
#[test]
fn catalogue_rejects_a_member_price_that_is_not_the_recovered_discount() {
    let mut document: serde_json::Value = serde_json::from_str(CATALOGUE).unwrap();
    document["properties"][3]["station_cash"]["member"] = serde_json::json!(275);
    let error = PropertyAcquisition::from_json(document.to_string().as_bytes())
        .expect_err("a member price that is not floor(regular * 9 / 10) must be refused");
    assert!(
        matches!(error, AcquisitionError::CorruptCatalogue { ref reason } if reason.contains("member price")),
        "unexpected error: {error:?}"
    );
}

#[test]
fn catalogue_rejects_unknown_schema_and_duplicate_deed_items() {
    let mut document: serde_json::Value = serde_json::from_str(CATALOGUE).unwrap();
    document["schema_version"] = serde_json::json!(2);
    assert!(matches!(
        PropertyAcquisition::from_json(document.to_string().as_bytes()),
        Err(AcquisitionError::UnsupportedSchema { version: 2 })
    ));

    let mut document: serde_json::Value = serde_json::from_str(CATALOGUE).unwrap();
    document["properties"][3]["deed_item_ids"] = serde_json::json!([2213]);
    let error = PropertyAcquisition::from_json(document.to_string().as_bytes())
        .expect_err("one deed item cannot unlock two properties");
    assert!(matches!(error, AcquisitionError::CorruptCatalogue { .. }));
}

/// Once-only ownership plus currency accounting: the member pays the recorded 270 and keeps a debit
/// proposal for the lead-owned economy, and a second acquisition of the same property is refused
/// without charging again.
#[test]
fn marketplace_purchase_charges_once_and_ownership_is_once_only() {
    let mut engine = acquisition();
    engine.set_station_cash(CHARACTER, 1_000);
    let first = engine
        .acquire(AcquisitionRequest {
            character_guid: CHARACTER,
            property_id: SEASIDE_LOT,
            channel: AcquisitionChannel::Marketplace,
            member: true,
        })
        .expect("member purchase");
    assert_eq!(
        first,
        AcquisitionOutcome {
            character_guid: CHARACTER,
            property_id: SEASIDE_LOT,
            channel: AcquisitionChannel::Marketplace,
            charged: 270,
            debit: Some(piece_021::StationCashDebit {
                character_guid: CHARACTER,
                amount: 270,
            }),
        }
    );
    assert_eq!(engine.station_cash(CHARACTER), 730);
    assert!(engine.owns(CHARACTER, SEASIDE_LOT));

    let second = engine
        .acquire(AcquisitionRequest {
            character_guid: CHARACTER,
            property_id: SEASIDE_LOT,
            channel: AcquisitionChannel::Marketplace,
            member: true,
        })
        .expect_err("the same property is never granted twice");
    assert!(matches!(
        second,
        AcquisitionError::AlreadyOwned {
            property_id: SEASIDE_LOT
        }
    ));
    assert_eq!(
        engine.station_cash(CHARACTER),
        730,
        "a refused acquisition never charges"
    );
}

#[test]
fn non_member_pays_the_regular_price() {
    let mut engine = acquisition();
    engine.set_station_cash(CHARACTER, 300);
    let outcome = engine
        .acquire(AcquisitionRequest {
            character_guid: CHARACTER,
            property_id: SEASIDE_LOT,
            channel: AcquisitionChannel::Marketplace,
            member: false,
        })
        .expect("regular purchase");
    assert_eq!(outcome.charged, 300);
    assert_eq!(engine.station_cash(CHARACTER), 0);
    assert!(engine.owns(CHARACTER, SEASIDE_LOT));
}

#[test]
fn insufficient_station_cash_leaves_ownership_and_balance_untouched() {
    let mut engine = acquisition();
    engine.set_station_cash(CHARACTER, 100);
    let error = engine
        .acquire(AcquisitionRequest {
            character_guid: CHARACTER,
            property_id: SEASIDE_LOT,
            channel: AcquisitionChannel::Marketplace,
            member: false,
        })
        .expect_err("100 station cash cannot buy a 300 station cash lot");
    assert!(matches!(
        error,
        AcquisitionError::InsufficientStationCash {
            required: 300,
            available: 100
        }
    ));
    assert_eq!(engine.station_cash(CHARACTER), 100);
    assert!(!engine.owns(CHARACTER, SEASIDE_LOT));
}

/// The members-only package is a claim, not a purchase: it is free, it still cannot be claimed twice,
/// and a non-member is refused.
#[test]
fn membership_claim_is_free_once_only_and_member_gated() {
    let mut engine = acquisition();
    let refused = engine
        .acquire(AcquisitionRequest {
            character_guid: CHARACTER,
            property_id: SMALL_WILDS_HOUSE,
            channel: AcquisitionChannel::MembershipClaim,
            member: false,
        })
        .expect_err("a non-member cannot claim the members-only house");
    assert!(matches!(
        refused,
        AcquisitionError::MembershipRequired {
            property_id: SMALL_WILDS_HOUSE
        }
    ));
    assert!(!engine.owns(CHARACTER, SMALL_WILDS_HOUSE));

    let claimed = engine
        .acquire(AcquisitionRequest {
            character_guid: CHARACTER,
            property_id: SMALL_WILDS_HOUSE,
            channel: AcquisitionChannel::MembershipClaim,
            member: true,
        })
        .expect("member claim");
    assert_eq!(claimed.charged, 0);
    assert_eq!(
        claimed.debit, None,
        "a claim never mints a station-cash debit"
    );
    assert!(engine.owns(CHARACTER, SMALL_WILDS_HOUSE));

    assert!(matches!(
        engine.acquire(AcquisitionRequest {
            character_guid: CHARACTER,
            property_id: SMALL_WILDS_HOUSE,
            channel: AcquisitionChannel::MembershipClaim,
            member: true,
        }),
        Err(AcquisitionError::AlreadyOwned { .. })
    ));
}

/// Every new character receives exactly one free home; the 2012-01-26 Wilds Condo migration is a
/// separate recorded grant, so an existing Apartment owner can also hold the Condo.
#[test]
fn starter_grant_is_once_per_character_and_migration_is_explicit() {
    let mut engine = acquisition();
    engine
        .acquire(AcquisitionRequest {
            character_guid: CHARACTER,
            property_id: APARTMENT,
            channel: AcquisitionChannel::StarterGrant,
            member: false,
        })
        .expect("the starter home is free");
    assert!(engine.starter_grant_used(CHARACTER));

    let refused = engine
        .acquire(AcquisitionRequest {
            character_guid: CHARACTER,
            property_id: WILDS_CONDO,
            channel: AcquisitionChannel::StarterGrant,
            member: false,
        })
        .expect_err("a character receives one starter home, not two");
    assert!(matches!(
        refused,
        AcquisitionError::StarterGrantAlreadyUsed {
            character_guid: CHARACTER
        }
    ));

    engine
        .acquire(AcquisitionRequest {
            character_guid: CHARACTER,
            property_id: WILDS_CONDO,
            channel: AcquisitionChannel::BonusGrant,
            member: false,
        })
        .expect("the recorded 2012-01-26 migration granted the condo to existing owners");
    assert!(engine.owns(CHARACTER, APARTMENT));
    assert!(engine.owns(CHARACTER, WILDS_CONDO));

    engine
        .acquire(AcquisitionRequest {
            character_guid: OTHER,
            property_id: WILDS_CONDO,
            channel: AcquisitionChannel::StarterGrant,
            member: false,
        })
        .expect("another character still has its own starter grant");
}

#[test]
fn unknown_property_and_unsupported_channel_are_refused() {
    let mut engine = acquisition();
    assert!(matches!(
        engine.acquire(AcquisitionRequest {
            character_guid: CHARACTER,
            property_id: 9_999,
            channel: AcquisitionChannel::Marketplace,
            member: false,
        }),
        Err(AcquisitionError::UnknownProperty { property_id: 9_999 })
    ));

    engine.set_station_cash(CHARACTER, 10_000);
    assert!(matches!(
        engine.acquire(AcquisitionRequest {
            character_guid: CHARACTER,
            property_id: APARTMENT,
            channel: AcquisitionChannel::Marketplace,
            member: false,
        }),
        Err(AcquisitionError::ChannelNotAllowed {
            property_id: APARTMENT,
            channel: AcquisitionChannel::Marketplace
        })
    ));
    assert_eq!(engine.station_cash(CHARACTER), 10_000);
}

#[test]
fn bonus_grant_is_free_and_once_only() {
    let mut engine = acquisition();
    let granted = engine
        .acquire(AcquisitionRequest {
            character_guid: CHARACTER,
            property_id: SUNSET_PARTY_BOAT,
            channel: AcquisitionChannel::BonusGrant,
            member: true,
        })
        .expect("promotional grant");
    assert_eq!(granted.charged, 0);
    assert!(matches!(
        engine.acquire(AcquisitionRequest {
            character_guid: CHARACTER,
            property_id: SUNSET_PARTY_BOAT,
            channel: AcquisitionChannel::BonusGrant,
            member: true,
        }),
        Err(AcquisitionError::AlreadyOwned { .. })
    ));
}

#[test]
fn an_ownerless_request_is_rejected_without_mutation() {
    let mut engine = acquisition();
    engine.set_station_cash(CHARACTER, 500);
    assert!(matches!(
        engine.acquire(AcquisitionRequest {
            character_guid: 0,
            property_id: SEASIDE_LOT,
            channel: AcquisitionChannel::Marketplace,
            member: false,
        }),
        Err(AcquisitionError::InvalidOwner)
    ));
    assert_eq!(engine.station_cash(CHARACTER), 500);
    assert!(!engine.owns(CHARACTER, SEASIDE_LOT));
}

#[test]
fn corrupted_catalogue_bytes_are_refused() {
    assert!(matches!(
        PropertyAcquisition::from_json(b"not json"),
        Err(AcquisitionError::CorruptCatalogue { .. })
    ));
}

/// The real extractor output: `scripts/pieces/piece_021.py` writes
/// `lab/reports/piece-021-<UTC>/property-acquisition.json` from the pinned lab corpus. This is
/// opt-in because the lab is private, so the vector is ignored unless the environment names the
/// file:
///
/// ```text
/// $env:OSFR_PIECE_021_CATALOGUE = '..\lab\reports\piece-021-...\property-acquisition.json'
/// scripts/cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml',
///   '--test','piece_021','real_recovered_catalogue_loads','--','--ignored','--exact')
/// ```
#[test]
#[ignore = "needs OSFR_PIECE_021_CATALOGUE from the private lab corpus"]
fn real_recovered_catalogue_loads_and_charges_the_recorded_prices() {
    let path = std::env::var("OSFR_PIECE_021_CATALOGUE")
        .expect("set OSFR_PIECE_021_CATALOGUE to the extracted catalogue");
    let bytes = std::fs::read(&path).expect("the catalogue file must be readable");
    let mut engine =
        PropertyAcquisition::from_json(&bytes).expect("the real recovered catalogue must validate");
    assert_eq!(
        engine.properties().count(),
        31,
        "Houses.json carries 31 rows"
    );
    let seaside = engine.property(SEASIDE_LOT).expect("Seaside Lot");
    assert_eq!(seaside.name, "Seaside Lot");
    assert_eq!(seaside.deed_item_ids, vec![10384]);
    assert_eq!(
        seaside.station_cash,
        Some(StationCashPrice {
            regular: 300,
            member: 270
        })
    );
    let small_wilds = engine
        .property(SMALL_WILDS_HOUSE)
        .expect("Small Wilds House");
    assert!(small_wilds.members_only);
    assert!(
        small_wilds
            .channels
            .contains(&AcquisitionChannel::MembershipClaim)
    );

    engine.set_station_cash(CHARACTER, 270);
    let outcome = engine
        .acquire(AcquisitionRequest {
            character_guid: CHARACTER,
            property_id: SEASIDE_LOT,
            channel: AcquisitionChannel::Marketplace,
            member: true,
        })
        .expect("a member buys the recorded 270 Station Cash lot");
    assert_eq!(outcome.charged, 270);
    assert_eq!(engine.station_cash(CHARACTER), 0);
}
