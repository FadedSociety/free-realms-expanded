//! Fotomat portraits: the client's 156/2 `PacketPortraitDataRequest` and the 156/3
//! `PacketPlayerImageData` reply, plus the quiet 115 portrait CRC.
//!
//! Client evidence:
//! * request sender `00bd3ff0`, header `00bd31d0`: u16 156, u16 2, u64 guid, then the
//!   provider string (i32 length + bytes). "Headshot" and "Portrait" are the 8-byte
//!   providers; the login request is 24 bytes.
//! * reply reader `00bd4180`, record reader `00bd37e0`, image payload `009538e0`, consumer
//!   `00bd4a50`; dispatch case 156 calls `00bd4d30` at `009499e2`. The attachment list reader
//!   `009131c0` is the same one AddPc (`0091aa10`) uses, so `PlayerPresentation::attachments_wire`
//!   is copied here unchanged.
//! * 115 ctor `00b4a9c0` (opcode 115, u32 CRC), sender `00b4fc30`; the CRC is `00704a40`.
//!
//! The consumer stores the appearance record and clears the pending entry even when the image
//! is empty, so an empty image is a valid answer until the lead's upload ingress exists (R).

use crate::players::{PlayerPresentation, PlayerRegistry, SessionId};

pub const FAMILY: u16 = 156;
pub const REQUEST_SUBTYPE: u16 = 2;
pub const REPLY_SUBTYPE: u16 = 3;
/// Top-level portrait upload notice: u16 115, u32 CRC-32 of portrait.png.
pub const CRC_OPCODE: u16 = 115;
/// (R) The brief's provider bound; the two accepted providers are 8 bytes. A longer
/// string is refused before it is read.
const MAX_PROVIDER_BYTES: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Provider {
    Headshot,
    Portrait,
}

impl Provider {
    fn from_wire(text: &str) -> Option<Self> {
        match text {
            "Headshot" => Some(Self::Headshot),
            "Portrait" => Some(Self::Portrait),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Headshot => "Headshot",
            Self::Portrait => "Portrait",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Request {
    guid: u64,
    provider: Provider,
}

/// Whether `body` starts with the 156/2 header, without parsing or allocating.
pub fn is_request(body: &[u8]) -> bool {
    body.starts_with(&[FAMILY as u8, 0, REQUEST_SUBTYPE as u8, 0])
}

/// The top-level 115 portrait upload notice: u16 115 + u32 CRC-32, exactly six bytes. Only the
/// shape is used; the CRC value belongs to the lead's pending-CRC ingress (a later task), so a
/// notice that passes this check is accepted quietly and stores nothing.
pub fn is_upload(body: &[u8]) -> bool {
    body.len() == 6 && body.starts_with(&CRC_OPCODE.to_le_bytes())
}

/// Whether this module owns the body at all. Both the pre-Loaded stage and the in-world route
/// dispatch on this; a malformed 156/2 or 115 is still dropped without a reply.
pub fn handles(body: &[u8]) -> bool {
    is_request(body) || is_upload(body)
}

/// The guid a well-formed 156/2 asks for, without building a presentation. `None` for any other
/// body. The pre-Loaded answer checks this before it clones the session's presentation.
pub fn requested_guid(body: &[u8]) -> Option<u64> {
    parse_request(body).map(|request| request.guid)
}

/// One inbound body this module owns. A 156/2 for a presentation the viewer may see becomes
/// one 156/3; a 115 is shape-checked and dropped; everything else returns `None` and changes
/// nothing.
pub fn handle<'a>(
    body: &[u8],
    presentation: impl FnOnce(u64) -> Option<&'a PlayerPresentation>,
) -> Option<Vec<u8>> {
    if is_upload(body) {
        return None;
    }
    let request = parse_request(body)?;
    let target = presentation(request.guid)?;
    encode_reply(request, target)
}

/// The in-world route's adapter: visibility is `PlayerRegistry`'s directed AddPc rule, plus
/// the player's own character. An invisible target sends no reply and leaves no pending state.
pub fn request(body: &[u8], viewer: SessionId, players: &PlayerRegistry) -> Option<Vec<u8>> {
    handle(body, |guid| players.presentation_of(viewer, guid))
}

fn parse_request(body: &[u8]) -> Option<Request> {
    let mut at = 0;
    if read_u16(body, &mut at)? != FAMILY || read_u16(body, &mut at)? != REQUEST_SUBTYPE {
        return None;
    }
    let guid = read_u64(body, &mut at)?;
    let length = usize::try_from(read_u32(body, &mut at)?).ok()?;
    if length > MAX_PROVIDER_BYTES {
        return None;
    }
    let end = at.checked_add(length)?;
    let text = std::str::from_utf8(body.get(at..end)?).ok()?;
    at = end;
    if at != body.len() {
        return None;
    }
    Some(Request {
        guid,
        provider: Provider::from_wire(text)?,
    })
}

fn encode_reply(request: Request, presentation: &PlayerPresentation) -> Option<Vec<u8>> {
    if presentation.character_guid != request.guid
        || presentation.customization_ids.iter().any(|id| *id < 0)
    {
        return None;
    }
    let mut out = Vec::with_capacity(256);
    put_u16(&mut out, FAMILY);
    put_u16(&mut out, REPLY_SUBTYPE);
    put_u64(&mut out, request.guid);
    put_string(&mut out, request.provider.as_str());
    out.push(0); // compressed: false; 1 makes `010ebee0` inflate the image.
    // (R) Sanctuary sends Unknown 0 and Unknown2 1; the client reader only stores them.
    put_i32(&mut out, 0);
    put_i32(&mut out, 1);
    put_u64(&mut out, request.guid);
    put_i32(&mut out, presentation.model);
    out.extend_from_slice(&presentation.attachments_wire);
    put_string(&mut out, &presentation.head);
    put_string(&mut out, &presentation.hair);
    put_i32(&mut out, presentation.hair_color);
    put_i32(&mut out, presentation.eye_color);
    put_string(&mut out, &presentation.skin_tone);
    put_string(&mut out, &presentation.face_paint);
    put_string(&mut out, &presentation.model_customization);
    for id in presentation.customization_ids {
        put_i32(&mut out, id);
    }
    put_string(&mut out, request.provider.as_str());
    // (R) Empty image until the lead's upload ingress stores one. `00bd4a50` keeps the
    // appearance record and clears the pending request for a zero-length payload.
    put_i32(&mut out, 0);
    Some(out)
}

fn read_u16(bytes: &[u8], at: &mut usize) -> Option<u16> {
    let end = at.checked_add(2)?;
    let value = u16::from_le_bytes(bytes.get(*at..end)?.try_into().ok()?);
    *at = end;
    Some(value)
}

fn read_u32(bytes: &[u8], at: &mut usize) -> Option<u32> {
    let end = at.checked_add(4)?;
    let value = u32::from_le_bytes(bytes.get(*at..end)?.try_into().ok()?);
    *at = end;
    Some(value)
}

fn read_u64(bytes: &[u8], at: &mut usize) -> Option<u64> {
    let end = at.checked_add(8)?;
    let value = u64::from_le_bytes(bytes.get(*at..end)?.try_into().ok()?);
    *at = end;
    Some(value)
}

fn put_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_i32(out: &mut Vec<u8>, value: i32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_string(out: &mut Vec<u8>, value: &str) {
    put_i32(
        out,
        i32::try_from(value.len()).expect("portrait string length exceeds i32"),
    );
    out.extend_from_slice(value.as_bytes());
}
