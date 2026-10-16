//! Work item 075: recovered Postman delivery routes.
//!
//! Recovered original facts, copied from the pinned sources listed in
//! `notes/Research/Pieces/075 Postman delivery routes.md`:
//!
//! * The stock original client locale (`lab/working/OSFR.Client/OSFR Client/locale/en_us_data.dat`,
//!   sha256 `9c764d76...`) carries the route quest lines and every mailbox address as its own
//!   whole-string row.
//! * The preserved original locale export (`lab/reports/job-catalogue-20261009T022023Z/
//!   locale-en_us.json`, 85,932 rows) resolves the mailbox `NameId`s, the delivery activity
//!   names/descriptions and the delivery notification string 388255 (`Mail Delivered!`).
//! * `MiniGameData.txt` holds the 15 Postman delivery rows (3/5/6/12/13/14/16/24/72/238/239/241/
//!   248/263/1002).
//! * The community `Npcs.json` / `Models.txt` mirror the 23 mailbox actors and the two mailbox
//!   models (910 `sg_mailbox_01.adr`, 340 `sg_mailbox_group_01.adr`); the community
//!   `Scripts/Zone/FabledRealms.lua` spawns each actor with a world position and heading.
//! * The decompiled client Lua wires the delivery notification:
//!   `SH_DeliveryGame.DeliverLetter -> NotificationHandler:DeliverLetter(0.7) ->
//!   CompleteObjective(Ui.GetString(388255))` with icon 1237.
//!
//! BOUNDARIES: recovery metadata plus a pure validation helper. It grants no job, item, currency
//! or experience and sends no packet. The per-activity route membership, the delivery order and the
//! numeric timer were not recovered; `ROUTE_MEMBERSHIP_RECOVERED`, `ROUTE_ORDER_RECOVERED` and
//! `DELIVERY_TIMER_SECONDS` report them as gaps rather than inventing values.

/// Where a recovered delivery row comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeliveryEvidence {
    /// Original client table or locale preserved in the lab.
    Original,
    /// Community mirror of an original table (`Sanctuary`).
    CommunityTable,
}

/// The mailbox actor family; `model_id` disambiguates the two recovered models.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MailboxKind {
    /// `sg_mailbox_01.adr` (model 910): a single street mailbox.
    StreetMailbox,
    /// `sg_mailbox_group_01.adr` (model 340): a postbox or mailbox group.
    PostboxGroup,
    /// A group without a recovered proper address.
    GenericMailbox,
}

/// A world spawn as the community zone script declares it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spawn {
    pub guid: i64,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub heading: f32,
}

/// One recovered mailbox actor: the delivery target the client can resolve.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mailbox {
    pub npc_id: i32,
    pub name_id: u32,
    pub address: &'static str,
    pub model_id: i32,
    pub model_file: &'static str,
    pub kind: MailboxKind,
    pub spawn: Spawn,
    pub evidence: DeliveryEvidence,
}

/// One recovered delivery activity row from `MiniGameData.txt`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeliveryActivity {
    pub id: i32,
    pub activity_type: u8,
    pub name_id: u32,
    pub description_id: u32,
    pub name: &'static str,
    pub description: &'static str,
    pub difficulty: u8,
    /// True when the shipped name does not carry the `NOT LIVE - ` prefix.
    pub live: bool,
}

/// A string the module quotes verbatim from the small-id locale export.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextClaim {
    pub string_id: u32,
    pub role: &'static str,
    pub text: &'static str,
}

/// A string the module quotes verbatim from the stock hashed-id client locale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RouteTextClaim {
    pub hash_id: u64,
    pub role: &'static str,
    pub text: &'static str,
}

/// The client delivery notification recovered from `ScriptsBase.lua`.
pub const DELIVERY_NOTIFICATION_STRING_ID: u32 = 388_255;
pub const DELIVERY_NOTIFICATION_ICON_ID: i32 = 1237;
pub const DELIVERY_NOTIFICATION_CLIENT_PATH: &str =
    "SH_DeliveryGame.DeliverLetter -> NotificationHandler:DeliverLetter";

const fn spawn(guid: i64, x: f32, y: f32, z: f32, heading: f32) -> Spawn {
    Spawn {
        guid,
        x,
        y,
        z,
        heading,
    }
}

const fn mailbox_row(
    npc_id: i32,
    name_id: u32,
    address: &'static str,
    kind: MailboxKind,
    spawn: Spawn,
) -> Mailbox {
    let (model_id, model_file) = match kind {
        MailboxKind::StreetMailbox => (910, "sg_mailbox_01.adr"),
        MailboxKind::PostboxGroup | MailboxKind::GenericMailbox => (340, "sg_mailbox_group_01.adr"),
    };
    Mailbox {
        npc_id,
        name_id,
        address,
        model_id,
        model_file,
        kind,
        spawn,
        evidence: DeliveryEvidence::CommunityTable,
    }
}

const fn activity(
    id: i32,
    activity_type: u8,
    name_id: u32,
    description_id: u32,
    name: &'static str,
    description: &'static str,
    difficulty: u8,
) -> DeliveryActivity {
    DeliveryActivity {
        id,
        activity_type,
        name_id,
        description_id,
        name,
        description,
        difficulty,
        live: !matches!(name.as_bytes(), [b'N', b'O', b'T', b' ', ..]),
    }
}

/// Every recovered mailbox actor with its address, model and community spawn.
pub const MAILBOXES: &[Mailbox] = &[
    mailbox_row(
        1426,
        104027,
        "Shrouded Glade Postboxes",
        MailboxKind::PostboxGroup,
        spawn(100000001426, 76.22441, 61.77683, 1199.034, -2.597929),
    ),
    mailbox_row(
        1750,
        102072,
        "3 Information Center",
        MailboxKind::StreetMailbox,
        spawn(100000001750, -562.0126, 5.506827, -1119.104, 1.92798),
    ),
    mailbox_row(
        1755,
        117040,
        "1 Seaspray Terrace",
        MailboxKind::StreetMailbox,
        spawn(100000001755, -449.7726, -2.652187, -1161.712, -3.140697),
    ),
    mailbox_row(
        1772,
        102074,
        "1 Sand Dog Terrace",
        MailboxKind::StreetMailbox,
        spawn(100000001772, -347.2917, -16.41668, -1157.403, -2.491922),
    ),
    mailbox_row(
        1784,
        130196,
        "1 Boardwalk Blvd.",
        MailboxKind::StreetMailbox,
        spawn(100000001784, -374.4905, -13.40512, -1192.744, -2.968257),
    ),
    mailbox_row(
        1806,
        117224,
        "8 Boardwalk Blvd.",
        MailboxKind::StreetMailbox,
        spawn(100000001806, -472.835, -24.27097, -1235.587, 1.558637),
    ),
    mailbox_row(
        1814,
        117222,
        "6 Boardwalk Blvd.",
        MailboxKind::StreetMailbox,
        spawn(100000001814, -447.4717, -21.87126, -1233.457, 1.615146),
    ),
    mailbox_row(
        1815,
        117221,
        "4 Boardwalk Blvd.",
        MailboxKind::StreetMailbox,
        spawn(100000001815, -426.2382, -18.72435, -1223.686, 1.257251),
    ),
    mailbox_row(
        1816,
        117219,
        "2 Boardwalk Blvd.",
        MailboxKind::StreetMailbox,
        spawn(100000001816, -404.7081, -18.81829, -1212.867, 1.167364),
    ),
    mailbox_row(
        1876,
        102070,
        "1 Information Center",
        MailboxKind::StreetMailbox,
        spawn(100000001876, -572.7861, 5.591041, -1115.479, 1.924527),
    ),
    mailbox_row(
        1881,
        101587,
        "Seaside Postboxes",
        MailboxKind::PostboxGroup,
        spawn(100000001881, -376.7556, 10.82801, -1123.785, 2.87694),
    ),
    mailbox_row(
        1927,
        102082,
        "3 Dunemere Trail",
        MailboxKind::StreetMailbox,
        spawn(100000001927, -242.2048, -26.90136, -1161.251, 1.513628),
    ),
    mailbox_row(
        1931,
        102076,
        "1 Dunemere Trail",
        MailboxKind::StreetMailbox,
        spawn(100000001931, -274.3073, -21.88009, -1161.279, 0.703442),
    ),
    mailbox_row(
        1934,
        102080,
        "4 Dunemere Trail",
        MailboxKind::StreetMailbox,
        spawn(100000001934, -229.569, -30.40546, -1187.396, 2.656706),
    ),
    mailbox_row(
        1937,
        102084,
        "5 Dunemere Trail",
        MailboxKind::StreetMailbox,
        spawn(100000001937, -223.9258, -24.65282, -1153.462, 1.064814),
    ),
    mailbox_row(
        1938,
        102086,
        "6 Dunemere Trail",
        MailboxKind::StreetMailbox,
        spawn(100000001938, -186.958, -30.33606, -1176.257, 0.412117),
    ),
    mailbox_row(
        1956,
        102054,
        "3 Sandy Lane",
        MailboxKind::StreetMailbox,
        spawn(100000001956, -522.7161, -16.0775, -1189.549, 1.135123),
    ),
    mailbox_row(
        1957,
        102052,
        "1 Sandy Lane",
        MailboxKind::StreetMailbox,
        spawn(100000001957, -482.1762, -12.16066, -1195.569, 1.927082),
    ),
    mailbox_row(
        1963,
        117227,
        "10 Boardwalk Blvd.",
        MailboxKind::StreetMailbox,
        spawn(100000001963, -490.1996, -30.39422, -1236.059, 1.304428),
    ),
    mailbox_row(
        2297,
        41452,
        "Merry Vale Postboxes",
        MailboxKind::PostboxGroup,
        spawn(100000002297, -1109.795, -40.10367, -397.9945, -1.943663),
    ),
    mailbox_row(
        2375,
        23500,
        "mailbox",
        MailboxKind::GenericMailbox,
        spawn(100000002375, -1100.981, 5.187627, 223.4131, -1.391073),
    ),
    mailbox_row(
        3569,
        5016,
        "1 Rock 'n Roll Rd.",
        MailboxKind::StreetMailbox,
        spawn(100000003569, 316.0503, 27.17472, 504.23, -2.127895),
    ),
    mailbox_row(
        3598,
        5014,
        "1 Overlook Drive",
        MailboxKind::StreetMailbox,
        spawn(100000003598, 345.7799, 48.04789, 320.4939, -1.3134),
    ),
];

/// The 15 recovered Postman delivery activity rows from `MiniGameData.txt`.
pub const DELIVERY_ACTIVITIES: &[DeliveryActivity] = &[
    activity(
        3,
        2,
        2200,
        2201,
        "NOT LIVE - Neither Rain, Nor Sleet...",
        "Bert's legs are too short and it's making him fall behind in his deliveries!  Help him out by giving the mail to the right postal customers.<BR><BR>Use the Hot and Cold meter to help find your way.",
        2,
    ),
    activity(
        5,
        2,
        2204,
        2205,
        "NOT LIVE - That darn dog...",
        "Rover is running around town with a bunch of mail in his mouth!  We need to get it back so we can deliver it to the proper customers.",
        2,
    ),
    activity(
        6,
        2,
        2206,
        2207,
        "NOT LIVE - Who is the joker?",
        "Someone is playing a joke on Bert.  Instead of putting the names of the person who is supposed to get the mail, someone has decided to put riddles instead.  Help Bert figure out who gets the mail by unraveling the clues.",
        2,
    ),
    activity(
        12,
        2,
        6361,
        5018,
        "NOT LIVE - Special Delivery",
        "You have been recruited at the Snowhill Post Office to help deliver mail.<BR>Deliver all of the letters in the specified time to prove you are the best postal worker in the history of Snowhill Village.",
        2,
    ),
    activity(
        13,
        2,
        16443,
        5047,
        "NOT LIVE - Play Ernie's Lost Letters minigame",
        "We have so much mail to deliver that it's falling out of our mail bags. Find all of the lost mail and then deliver it to the correct addresses.",
        3,
    ),
    activity(
        14,
        2,
        115976,
        5073,
        "What a Mess!",
        "A bunch of excess mail got scattered around the tree in the center of town. Find it all and then make deliveries to the correct addresses.",
        2,
    ),
    activity(
        16,
        2,
        6206,
        6209,
        "NOT LIVE - Skate Shop Ads",
        "Deliver as many advertisements for the Skate Shop Showdown as you can to mailboxes in Snowhill Village.",
        2,
    ),
    activity(
        24,
        2,
        15674,
        15675,
        "NOT LIVE - Postal Route: Sanctuary",
        "Deliver Mail to the Addresses",
        1,
    ),
    activity(
        72,
        2,
        21966,
        21967,
        "NOT LIVE - Postal Carrier Foot Race",
        "Try to beat the fastest time set in the postal carrier foot race. If you beat the clock you win!",
        2,
    ),
    activity(
        238,
        2,
        93381,
        93382,
        "Can You Deliver?",
        "If you want to be a postman, you've got to learn your territory. See if you can find these addresses in Sanctuary before time runs out.",
        1,
    ),
    activity(
        239,
        2,
        93757,
        93758,
        "Mail Marksman",
        "Use the Throw Mail ability to deliver mail to the addresses specified. First, click on a mail box to target it, then click the ability icon or use the hot-key (1) to toss mail into it.",
        1,
    ),
    activity(
        241,
        1,
        21424,
        100309,
        "Scattered Mail",
        "Flitter has dropped her mail all over the Inner Circle of Sanctuary. Help her find the mail before time runs out!",
        2,
    ),
    activity(
        248,
        2,
        100908,
        100966,
        "Deliver the Mail in Merry Vale",
        "Postmaster Vixie is behind on getting the mail out. Help her deliver to the residents of Merry Vale before time runs out!",
        1,
    ),
    activity(
        263,
        2,
        101614,
        101615,
        "Seaside Mail Delivery",
        "Deliver the mail to the addresses specified.",
        1,
    ),
    activity(
        1002,
        2,
        15674,
        15675,
        "NOT LIVE - Postal Route: Sanctuary",
        "Deliver Mail to the Addresses",
        1,
    ),
];

/// Strings this module quotes from the preserved locale export. The Python checker re-reads them.
pub const LOCALE_TEXT_CLAIMS: &[TextClaim] = &[
    TextClaim {
        string_id: 388_255,
        role: "delivery notification",
        text: "Mail Delivered!",
    },
    TextClaim {
        string_id: 933_82,
        role: "Can You Deliver? description",
        text: "If you want to be a postman, you've got to learn your territory. See if you can find these addresses in Sanctuary before time runs out.",
    },
    TextClaim {
        string_id: 6362,
        role: "Special Delivery start objective",
        text: "Deliver letters to their recipients in Snowhill. Go to the Post Boxes near Loryn to start the challenge.",
    },
    TextClaim {
        string_id: 103_840,
        role: "Can You Deliver? objective",
        text: "Click the mailboxes to deliver the mail.",
    },
];

/// Strings this module quotes from the stock hashed-id client locale.
pub const ROUTE_TEXT_CLAIMS: &[RouteTextClaim] = &[
    RouteTextClaim {
        hash_id: 29_679_012,
        role: "Sanctuary route quest description",
        text: "Deliver mail along the Sanctuary route before time runs out, then return to Felipe.",
    },
    RouteTextClaim {
        hash_id: 708_303_488,
        role: "Sanctuary route objective",
        text: "Deliver mail to mailboxes in Sanctuary",
    },
    RouteTextClaim {
        hash_id: 557_373_038,
        role: "mailbox click objective",
        text: "Click on each of the mailboxes specified before time runs out.",
    },
    RouteTextClaim {
        hash_id: 667_138_698,
        role: "NPC delivery objective",
        text: "Interact with each of the specified people to deliver the mail to them.",
    },
    RouteTextClaim {
        hash_id: 1_179_122_926,
        role: "route progress objective",
        text: "Inform him of your progress on the mail route.",
    },
    RouteTextClaim {
        hash_id: 1_313_439_205,
        role: "Running The Route objective",
        text: "Complete the Running The Route minigame",
    },
    RouteTextClaim {
        hash_id: 707_853_610,
        role: "Running The Route quest title",
        text: "Postman: Running the Route",
    },
    RouteTextClaim {
        hash_id: 1_049_400_037,
        role: "Mail Marksman control text",
        text: "Use the Throw Mail ability to deliver mail to the addresses specified. First, interact with a mail box to target it, then click the Square Button again to toss mail into it.",
    },
    RouteTextClaim {
        hash_id: 1_150_441_937,
        role: "mail toss tutorial line",
        text: "With so much mail to deliver, I could use some help from a postman like you!  <BR><BR>If the mail isn't fragile, you can toss it into the mailbox from a distance to save time!<BR><BR>Try using your <font color='#D9781A'>mail toss</font> ability on your next route.",
    },
    RouteTextClaim {
        hash_id: 330_643_173,
        role: "Downtown Sanctuary delivery objective",
        text: "Deliver mail to various addresses in Downtown Sanctuary.",
    },
    RouteTextClaim {
        hash_id: 657_254_818,
        role: "Ernie's Lost Letters description",
        text: "We have so much mail to deliver that it's falling out of our mail bags. Find all of the lost mail and then deliver it to the correct addresses.",
    },
    RouteTextClaim {
        hash_id: 361_651_179,
        role: "route training line",
        text: "You need to learn the route before you can become a postman!",
    },
    RouteTextClaim {
        hash_id: 585_153_882,
        role: "Postal Carrier Foot Race line",
        text: "It's a race around the postal route to get Sanctuary's mail delivered on time, thankfully [*player*] works with blazing speed!",
    },
    RouteTextClaim {
        hash_id: 433_123_487,
        role: "Merry Vale mail sorting start",
        text: "Use the mailboxes next to Postmaster Vixie to begin your mail sorting.",
    },
    RouteTextClaim {
        hash_id: 276_975_481,
        role: "Shrouded Glade postmaster thank-you",
        text: "Postmasters are forever in your debt, [*player*]. You have restored balance and order to the postal service. Be assured that when it comes time to expand Shrouded Glade and add new postal routes, I'll be suggesting we name a street after you.",
    },
    RouteTextClaim {
        hash_id: 1_533_049_349,
        role: "Can You Deliver? stock-locale description",
        text: "If you want to be a postman, you've got to learn your territory. See if you can find these addresses in Sanctuary before time runs out.",
    },
];

/// The per-activity route membership was not recovered: no target list for any activity.
pub const ROUTE_MEMBERSHIP_RECOVERED: bool = false;
/// The delivery order was not recovered: no source carries the objective sequence.
pub const ROUTE_ORDER_RECOVERED: bool = false;
/// The numeric delivery timer was not recovered; only "before time runs out" text.
pub const DELIVERY_TIMER_SECONDS: Option<u32> = None;
/// No captured delivery objective/route packet was recovered.
pub const DELIVERY_OBJECTIVE_PACKETS_RECOVERED: bool = false;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeliveryError {
    /// The actor id is not one of the recovered mailbox actors.
    UnknownTarget(i32),
    /// The target already received its delivery in this run.
    AlreadyDelivered(i32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeliveryProgress {
    pub delivered: usize,
    pub total: usize,
}

/// A validation helper for a caller-supplied delivery route.
///
/// RECONSTRUCTED POLICY: the original per-activity target list was not recovered, so the caller
/// supplies the ordered targets. This helper then only enforces what the recovered evidence
/// supports: every target must resolve to a recovered mailbox actor, and a delivery objective
/// completes once per target (the client's `CompleteObjective` fires once). It grants nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeliveryRun {
    targets: Vec<i32>,
    delivered: Vec<i32>,
}

impl DeliveryRun {
    pub fn start(targets: &[i32]) -> Result<Self, DeliveryError> {
        for target in targets {
            if mailbox(*target).is_none() {
                return Err(DeliveryError::UnknownTarget(*target));
            }
        }
        Ok(Self {
            targets: targets.to_vec(),
            delivered: Vec::new(),
        })
    }

    pub fn total(&self) -> usize {
        self.targets.len()
    }

    pub fn remaining(&self) -> usize {
        self.targets
            .iter()
            .filter(|target| !self.delivered.contains(target))
            .count()
    }

    pub fn is_complete(&self) -> bool {
        !self.targets.is_empty() && self.remaining() == 0
    }

    pub fn deliver(&mut self, npc_id: i32) -> Result<DeliveryProgress, DeliveryError> {
        if mailbox(npc_id).is_none() || !self.targets.contains(&npc_id) {
            return Err(DeliveryError::UnknownTarget(npc_id));
        }
        if self.delivered.contains(&npc_id) {
            return Err(DeliveryError::AlreadyDelivered(npc_id));
        }
        self.delivered.push(npc_id);
        Ok(self.progress())
    }

    pub fn progress(&self) -> DeliveryProgress {
        DeliveryProgress {
            delivered: self.delivered.len(),
            total: self.targets.len(),
        }
    }
}

pub fn mailbox(npc_id: i32) -> Option<&'static Mailbox> {
    MAILBOXES.iter().find(|row| row.npc_id == npc_id)
}

pub fn mailbox_by_address(address: &str) -> Option<&'static Mailbox> {
    MAILBOXES.iter().find(|row| row.address == address)
}

fn same_float(left: f32, right: f32) -> bool {
    (left - right).abs() <= f32::EPSILON
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouteError {
    DuplicateActor(i32),
    DuplicateAddress(String),
    DuplicateActivity(i32),
    EmptyText { id: u32 },
    ModelMismatch { npc_id: i32, expected: i32 },
    InvalidSpawn { npc_id: i32 },
}

/// Structural invariants over the recovered tables. Source equality is checked by the Python
/// checker, which re-reads the pinned lab evidence.
pub fn validate() -> Result<(), RouteError> {
    let mut actors: Vec<i32> = Vec::new();
    let mut addresses: Vec<&str> = Vec::new();
    for row in MAILBOXES {
        if actors.contains(&row.npc_id) {
            return Err(RouteError::DuplicateActor(row.npc_id));
        }
        if addresses.contains(&row.address) {
            return Err(RouteError::DuplicateAddress(row.address.to_string()));
        }
        let (model_id, model_file) = match row.kind {
            MailboxKind::StreetMailbox => (910, "sg_mailbox_01.adr"),
            MailboxKind::PostboxGroup | MailboxKind::GenericMailbox => {
                (340, "sg_mailbox_group_01.adr")
            }
        };
        if row.model_id != model_id || row.model_file != model_file {
            return Err(RouteError::ModelMismatch {
                npc_id: row.npc_id,
                expected: model_id,
            });
        }
        if row.spawn.guid <= 0 || !same_float(row.spawn.heading, row.spawn.heading) {
            return Err(RouteError::InvalidSpawn { npc_id: row.npc_id });
        }
        actors.push(row.npc_id);
        addresses.push(row.address);
    }
    let mut activities: Vec<i32> = Vec::new();
    for row in DELIVERY_ACTIVITIES {
        if activities.contains(&row.id) {
            return Err(RouteError::DuplicateActivity(row.id));
        }
        if row.name.trim().is_empty() || row.description.trim().is_empty() {
            return Err(RouteError::EmptyText { id: row.name_id });
        }
        activities.push(row.id);
    }
    for row in LOCALE_TEXT_CLAIMS {
        if row.text.trim().is_empty() {
            return Err(RouteError::EmptyText { id: row.string_id });
        }
    }
    for row in ROUTE_TEXT_CLAIMS {
        if row.text.trim().is_empty() {
            return Err(RouteError::EmptyText {
                id: row.hash_id as u32,
            });
        }
    }
    Ok(())
}
