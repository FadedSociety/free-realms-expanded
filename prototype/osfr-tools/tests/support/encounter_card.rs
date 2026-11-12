//! The 41/114 EncounterDetailsResponse read the way the client reads it, so a card a test gets
//! from the gateway is proven to parse whole. The client drops a card it cannot read to the last
//! byte: `ReadEncounterDetailsBody` (`00aa3430`) fails when bytes are left or missing, and
//! `ClientCommandProcessor__DispatchEncounterPacket` (`00aa36c0`) then never calls
//! `HandleMiniGameStart`, so Go sends nothing (rc-0137 recheck: the two-goal Bixie Hive card).
//!
//! Reader chain: `00aa32d0` (header, `00a29120` record, launch flag, word, int tree), `00a29120`
//! (two words, two counted lists, two words, three flags, two words, `009bdd70`, two flags),
//! `009bdd70` (six words and a flag, three `008e7930` bundles, the `009bc380` goal list, five
//! flags, a string, then words and flags), the goal row `008fd770` and the bundle `008e7930`.
#![allow(dead_code)] // each includer uses a subset

/// One goal row as the client stores it (`00c42870`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardGoal {
    pub key: i32,
    pub name: i32,
    pub description: i32,
    /// The four counters: state, count, total, kind.
    pub counters: [i32; 4],
    pub category: i32,
}

/// A reward bundle's coins (`+0x50`), XP (`+0x48`), coin multiplier (`+0x78`) and Item entries
/// as (item, name id, quantity).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CardBundle {
    pub coins: i32,
    pub xp: i32,
    pub multiplier: f32,
    pub items: Vec<(i32, i32, i32)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Card {
    pub id: i32,
    pub instance: i32,
    pub title: i32,
    pub difficulty: i32,
    pub preview: CardBundle,
    pub goals: Vec<CardGoal>,
    /// The details' own flag: set, the client refreshes the running game (`HandlerMiniGameStatus_Populate`)
    /// instead of opening the start screen (`HandleMiniGameStart`).
    pub launch: bool,
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn take(&mut self, n: usize) -> &[u8] {
        assert!(self.at + n <= self.bytes.len(), "the client reads past the card's end at byte {}", self.at);
        self.at += n;
        &self.bytes[self.at - n..self.at]
    }
    fn u8(&mut self) -> u8 {
        self.take(1)[0]
    }
    fn i32(&mut self) -> i32 {
        i32::from_le_bytes(self.take(4).try_into().unwrap())
    }
    /// A counted list the client reads with nothing in it here (teams, nodes, the int tree).
    fn empty_list(&mut self, what: &str) {
        assert_eq!(self.i32(), 0, "{what}: the cards we send carry none");
    }
    /// `008e7930`, in its field order: the flag (`+0x74`); coins (`+0x50`), XP (`+0x48`), `+0x4c`,
    /// `+0x54`, `+0x6c`, `+0x70`; the coin multiplier (`+0x78`, a float); `+0x5c`, `+0x60`; two
    /// 8-byte pairs; icon (`+0x64`) and name (`+0x68`); the counted entries; the trailing `+0x58`.
    /// The client's own bundle starts the multiplier at 1.0 (`0091c110`), and a bundle it reads
    /// with 0.0 shows and scales its coins to nothing, so every bundle must carry 1.0.
    fn bundle(&mut self) -> CardBundle {
        self.u8();
        let coins = self.i32();
        let xp = self.i32();
        for _ in 0..4 {
            self.i32();
        }
        let multiplier = f32::from_le_bytes(self.take(4).try_into().unwrap());
        assert_eq!(multiplier, 1.0, "+0x78, the coin multiplier, at the client's own 1.0");
        self.i32();
        self.i32();
        self.take(16);
        self.take(8);
        let count = self.i32();
        let mut items = Vec::new();
        for _ in 0..count {
            assert_eq!(self.i32(), 1, "only Item entries (type 1)");
            // Item (`0103c460`): hidden, icon, tint, name, quantity, item, param, string, colour,
            // members-only.
            self.u8();
            let [_icon, _tint, name, quantity, item, _, _, _] = [(); 8].map(|()| self.i32());
            self.u8();
            items.push((item, name, quantity));
        }
        self.i32();
        CardBundle {
            coins,
            xp,
            multiplier,
            items,
        }
    }
}

/// A 39/45 MiniGameLootWheelSetItemToLandOn read like the client (`009b9700`): the minigame
/// header (family, sub-op, three words), then one whole `008e7930` bundle, to the last byte.
pub fn read_landing(packet: &[u8]) -> CardBundle {
    let mut r = Reader { bytes: packet, at: 0 };
    assert_eq!(r.take(3), [39, 0, 45], "a 39/45");
    for _ in 0..3 {
        r.i32();
    }
    let landing = r.bundle();
    assert_eq!(r.at, packet.len(), "the client reads the landing to its last byte");
    landing
}

/// Where `StopMiniGameLootWheelAt` (`009b6da0`) stops the wheel the card previewed: on the
/// preview entry the landing's entry matches, else, with coins, on the coin slot after them.
pub fn landing_slot(preview: &CardBundle, landing: &CardBundle) -> Option<usize> {
    match landing.items.first() {
        Some(entry) => preview.items.iter().position(|item| item.0 == entry.0),
        None => (landing.coins > 0 && preview.coins > 0).then_some(preview.items.len()),
    }
}

/// Read a 41/114 like the client, panicking where the client would fail the read.
pub fn read_card(card: &[u8]) -> Card {
    let mut r = Reader { bytes: card, at: 0 };
    assert_eq!(r.take(4), [41, 0, 114, 0], "a 41/114");
    let (id, instance) = (r.i32(), r.i32());
    // 00a29120
    r.i32();
    r.i32();
    r.empty_list("teams");
    r.empty_list("nodes");
    r.i32();
    r.i32();
    r.take(3);
    r.i32();
    r.i32();
    // 009bdd70
    let [title, _icon, _description, difficulty, _profile, _kind] = [(); 6].map(|()| r.i32());
    r.u8();
    r.bundle();
    r.bundle();
    let preview = r.bundle();
    let count = r.i32();
    let goals = (0..count)
        .map(|_| {
            let (key, name, description) = (r.i32(), r.i32(), r.i32());
            r.u8();
            r.bundle();
            let counters = [(); 4].map(|()| r.i32());
            r.u8();
            let category = r.i32();
            CardGoal {
                key,
                name,
                description,
                counters,
                category,
            }
        })
        .collect();
    r.take(5);
    let string = r.i32();
    r.take(string as usize);
    r.i32();
    r.u8();
    r.i32();
    r.take(4);
    r.i32();
    r.take(2);
    // 00aa32d0
    let launch = r.u8() != 0;
    r.i32();
    r.empty_list("int tree");
    assert_eq!(r.at, card.len(), "the client reads the card to its last byte or drops it");
    Card {
        id,
        instance,
        title,
        difficulty,
        preview,
        goals,
        launch,
    }
}
