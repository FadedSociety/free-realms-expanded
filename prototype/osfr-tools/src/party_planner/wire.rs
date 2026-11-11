//! The packets a party uses, all the client's own.
//!
//! * Booking: the planner click (26/8) is answered with the stock dialog window, 26/4 then 26/3
//!   (`npc_interaction`), the planner's own line as its body and stock labels on its rows; the
//!   row the player presses comes back as 26/6 with its response id.
//! * Invite: the client's house invite. 127/47 (housing dispatcher `HousingProcessor__v0`
//!   `00aca5d0` case 0x2f, reader `HousingPacketInviteNotification_ReadBody` `00abd510` ->
//!   `00ab8bd0`): u16 127, u16 47, four u64 (`00ab3540`), two NameData records (`008e7290`: three
//!   i32, then two counted strings), one u64. `GaqItemHouseInvite__ctor_2` (`00a0b830`) keys the
//!   prompt by the third u64 and words it with the first NameData through `GAQHouseInvite`
//!   (CodeStringMappings 420620, "[*Player*] has invited you to enter their house."). Accept sends
//!   127/19 with that u64 (`GaqItemHouseInvite__v1` `00a09580` -> `SendEnterRequestByGuid`
//!   `00ac3fe0`, writer `00abca80`: u64 guid, u32 0, u8 flag); decline sends 127/16 with it (`__v0` `00a09550` ->
//!   `SendDeclineInvite` `00ac4e00`). The third u64 is the party guid, so an answer names the
//!   party and nothing else.
//!
//! (R) The other three u64 carry the host, the guest and 0, the second NameData the guest and the
//! last u64 0: the client reads them but the prompt uses none. The prompt says "house", so the
//! invite also comes with a chat line that names the party.
use super::{Venue, is_party};
use crate::{
    housing_service::{Request, decode_request},
    npc_interaction::{
        self as dialog, DialoguePacket26_3, GOODBYE_CHOICE_RESPONSE, dialogue_choice_row,
        goodbye_dialogue_choice,
    },
    social_chat::{NameData, put_name_data},
};

/// 127/47 HousingPacketInviteNotification.
const INVITE: u8 = 47;
/// 127/19 ClientHousingPacketEnterRequest.
const ENTER: u8 = 19;
/// 127/16 ClientHousingPacketDeclineInvite.
const DECLINE: u8 = 16;
/// "Friends" (Global.Text): the friends-and-group row.
pub const FRIENDS_LABEL_ID: u32 = 3_482;
/// "Join the Party".
pub const JOIN_LABEL_ID: u32 = 385_979;
/// "Invite Friends".
pub const INVITE_LABEL_ID: u32 = 427_559;
/// "End".
pub const END_LABEL_ID: u32 = 118_044;
/// (R) A dialog row id with 'P' in its top byte, clear of the NPC rows (1..=5, 300, 301, 310 and
/// up) and mail's 'M' rows; the low byte is the choice.
const RESPONSE_TAG: i32 = 0x5000_0000;

/// One row of the booking dialog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    Private,
    FriendsAndGroup,
    Join,
    Invite,
    End,
    Goodbye,
}

const CHOICES: [Choice; 5] = [
    Choice::Private,
    Choice::FriendsAndGroup,
    Choice::Join,
    Choice::Invite,
    Choice::End,
];

impl Choice {
    fn response(self) -> i32 {
        match self {
            Choice::Goodbye => GOODBYE_CHOICE_RESPONSE,
            other => {
                RESPONSE_TAG | (CHOICES.iter().position(|c| *c == other).unwrap_or(0) as i32 + 1)
            }
        }
    }

    fn row(self, venue: &Venue) -> [u32; 5] {
        let label = match self {
            Choice::Private => venue.private_label_id,
            Choice::FriendsAndGroup => FRIENDS_LABEL_ID,
            Choice::Join => JOIN_LABEL_ID,
            Choice::Invite => INVITE_LABEL_ID,
            Choice::End => END_LABEL_ID,
            Choice::Goodbye => return goodbye_dialogue_choice(),
        };
        dialogue_choice_row(self.response(), label)
    }

    /// The row a 26/6 response id names; `None` for any other dialog's row.
    pub fn from_response(response: i32) -> Option<Self> {
        if response == GOODBYE_CHOICE_RESPONSE {
            return Some(Choice::Goodbye);
        }
        let index = response.checked_sub(RESPONSE_TAG)?.checked_sub(1)?;
        CHOICES.get(usize::try_from(index).ok()?).copied()
    }
}

/// The booking dialog for `venue` with `choices` (at most five rows), spoken by its planner. The
/// client rebuilds the window only while it is hidden (T844), so a 26/4 goes first.
pub fn dialog(venue: &Venue, choices: &[Choice]) -> Vec<Vec<u8>> {
    let actor = venue.planner;
    let packet = dialog::encode_dialogue_26_3(&DialoguePacket26_3 {
        initial_dwords: [venue.greeting_id, 0, actor as u32, (actor >> 32) as u32],
        initial_flag: false,
        initial_float_bits: 0,
        choices: choices.iter().map(|choice| choice.row(venue)).collect(),
        choice_vectors: [[0; 4]; 2],
        vector_flag: false,
        final_vector: [0; 4],
        float_after_vector_bits: 0,
        final_flags: [false; 3],
        final_float_bits: 0,
        final_dword: 0,
    });
    let mut packets = vec![dialog::encode_dialogue_close_26_4()];
    packets.extend(packet.ok());
    packets
}

/// The 26/4 that closes the dialog.
pub fn close() -> Vec<u8> {
    dialog::encode_dialogue_close_26_4()
}

/// 127/47 for `guest`: `host` invites them to party `party`. Layout in the module doc.
pub fn invite(
    party: u64,
    host: u64,
    guest: u64,
    host_name: &NameData,
    guest_name: &NameData,
) -> Vec<u8> {
    let mut out = vec![127, 0, INVITE, 0];
    for word in [host, guest, party, 0] {
        out.extend(word.to_le_bytes());
    }
    put_name_data(&mut out, host_name);
    put_name_data(&mut out, guest_name);
    out.extend(0_u64.to_le_bytes());
    out
}

/// The party a 127/19 accept names; `None` for a house's enter (housing answers those).
pub fn accepted(inner: &[u8]) -> Option<u64> {
    if !inner.starts_with(&[127, 0, ENTER, 0]) {
        return None;
    }
    match decode_request(inner) {
        Ok(Request::Enter { house_guid }) => is_party(house_guid).then_some(house_guid),
        _ => None,
    }
}

/// The party a 127/16 decline names (u64 guid after the header).
pub fn declined(inner: &[u8]) -> Option<u64> {
    let ([127, 0, DECLINE, 0], rest) = inner.split_first_chunk::<4>()? else {
        return None;
    };
    let guid = u64::from_le_bytes(*rest.first_chunk::<8>()?);
    (rest.len() == 8 && is_party(guid)).then_some(guid)
}

/// A party chat word, typed as `realm party ...` (the stock client keeps `/` lines in its own
/// console, mail's finding) or `/party ...` for a client that sends slash lines.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Word {
    Help,
    /// Invite one player by name, or (no name) every online mutual friend and group member.
    Invite(Option<String>),
    End,
    Leave,
}

pub fn word(text: &str) -> Option<Word> {
    let text = text.trim();
    let lower = text.to_ascii_lowercase();
    let rest = ["realm party", "/party"]
        .iter()
        .find_map(|prefix| lower.strip_prefix(prefix).map(|_| &text[prefix.len()..]))?;
    if !(rest.is_empty() || rest.starts_with(' ')) {
        return None;
    }
    let mut words = rest.split_whitespace();
    let verb = words.next().map(str::to_ascii_lowercase);
    let name = words.collect::<Vec<_>>().join(" ");
    Some(match verb.as_deref() {
        Some("invite") if name.is_empty() => Word::Invite(None),
        Some("invite") => Word::Invite(Some(name)),
        Some("end") if name.is_empty() => Word::End,
        Some("leave") if name.is_empty() => Word::Leave,
        _ => Word::Help,
    })
}

/// The help line every unknown party word gets.
pub const HELP: &str = "Party: talk to a party planner to book one. While you host, \
    realm party invite <name> invites a player, realm party invite invites your online friends \
    and group, and realm party end ends it. A guest leaves with realm party leave.";
