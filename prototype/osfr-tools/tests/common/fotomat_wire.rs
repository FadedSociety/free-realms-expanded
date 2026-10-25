//! Test decoder for the client's 156/3 `PacketPlayerImageData` reader (`00bd4180`), record
//! reader (`00bd37e0`), attachment list (`009131c0`) and image payload (`009538e0`).
//!
//! `00bd4a50` stores the packet only when the reader's error flag is clear and the cursor
//! reached the end, so [`decode`] requires every byte to be consumed. Test-only.

#[allow(dead_code)]
pub struct ImageData<'a> {
    pub guid: u64,
    pub provider: String,
    pub compressed: bool,
    pub unknown: i32,
    pub unknown2: i32,
    pub record_guid: u64,
    pub model: i32,
    pub attachments: &'a [u8],
    pub head: String,
    pub hair: String,
    pub hair_color: i32,
    pub eye_color: i32,
    pub skin_tone: String,
    pub face_paint: String,
    pub model_customization: String,
    pub customization_ids: [i32; 4],
    pub record_provider: String,
    pub image: &'a [u8],
}

/// The inner 156/2 request (`00bd3ff0`): u16 156, u16 2, u64 guid, i32 length + provider.
pub fn request(guid: u64, provider: &str) -> Vec<u8> {
    let mut out = vec![156, 0, 2, 0];
    out.extend_from_slice(&guid.to_le_bytes());
    out.extend_from_slice(&(provider.len() as u32).to_le_bytes());
    out.extend_from_slice(provider.as_bytes());
    out
}

/// One attachment row: three strings then three i32 values (reader `009131c0`).
pub fn one_attachment() -> Vec<u8> {
    let mut out = 1_i32.to_le_bytes().to_vec();
    for text in ["model.adr", "", "tint"] {
        out.extend_from_slice(&(text.len() as i32).to_le_bytes());
        out.extend_from_slice(text.as_bytes());
    }
    for value in [7_i32, 8, 9] {
        out.extend_from_slice(&value.to_le_bytes());
    }
    out
}

/// Strip the 5/1 tunnel the server wraps an application reply in.
pub fn tunnel_inner(packet: &[u8]) -> &[u8] {
    assert_eq!(packet.get(..3), Some(&[5, 0, 1][..]), "5/1 tunnel");
    let length = u32::from_le_bytes(packet[3..7].try_into().expect("length")) as usize;
    assert_eq!(packet.len(), 7 + length, "tunnel length");
    &packet[7..]
}

pub fn decode(packet: &[u8]) -> ImageData<'_> {
    let mut at = 0;
    assert_eq!(u16_at(packet, &mut at), 156, "family");
    assert_eq!(u16_at(packet, &mut at), 3, "subtype");
    let guid = u64_at(packet, &mut at);
    let provider = string_at(packet, &mut at);
    let compressed = byte_at(packet, &mut at) != 0;
    let unknown = i32_at(packet, &mut at);
    let unknown2 = i32_at(packet, &mut at);
    let record_guid = u64_at(packet, &mut at);
    let model = i32_at(packet, &mut at);

    let attachments_start = at;
    let rows = i32_at(packet, &mut at);
    assert!((0..=64).contains(&rows), "attachment count {rows}");
    for _ in 0..rows {
        for _ in 0..3 {
            let _ = string_at(packet, &mut at);
        }
        for _ in 0..3 {
            let _ = i32_at(packet, &mut at);
        }
    }
    let attachments = &packet[attachments_start..at];

    let head = string_at(packet, &mut at);
    let hair = string_at(packet, &mut at);
    let hair_color = i32_at(packet, &mut at);
    let eye_color = i32_at(packet, &mut at);
    let skin_tone = string_at(packet, &mut at);
    let face_paint = string_at(packet, &mut at);
    let model_customization = string_at(packet, &mut at);
    let customization_ids = std::array::from_fn(|_| i32_at(packet, &mut at));
    let record_provider = string_at(packet, &mut at);

    // `009538e0`: i32 length, then that many image bytes; negative or overrun sets the error.
    let image_len = i32_at(packet, &mut at);
    assert!(image_len >= 0, "negative image length");
    let end = at.checked_add(image_len as usize).expect("image length");
    let image = packet.get(at..end).expect("image bytes");
    at = end;
    assert_eq!(at, packet.len(), "156/3 fully consumed");

    ImageData {
        guid,
        provider,
        compressed,
        unknown,
        unknown2,
        record_guid,
        model,
        attachments,
        head,
        hair,
        hair_color,
        eye_color,
        skin_tone,
        face_paint,
        model_customization,
        customization_ids,
        record_provider,
        image,
    }
}

fn byte_at(bytes: &[u8], at: &mut usize) -> u8 {
    let value = *bytes.get(*at).expect("byte");
    *at += 1;
    value
}

fn u16_at(bytes: &[u8], at: &mut usize) -> u16 {
    let end = *at + 2;
    let value = u16::from_le_bytes(
        bytes
            .get(*at..end)
            .expect("u16")
            .try_into()
            .expect("u16 size"),
    );
    *at = end;
    value
}

fn i32_at(bytes: &[u8], at: &mut usize) -> i32 {
    let end = *at + 4;
    let value = i32::from_le_bytes(
        bytes
            .get(*at..end)
            .expect("i32")
            .try_into()
            .expect("i32 size"),
    );
    *at = end;
    value
}

fn u64_at(bytes: &[u8], at: &mut usize) -> u64 {
    let end = *at + 8;
    let value = u64::from_le_bytes(
        bytes
            .get(*at..end)
            .expect("u64")
            .try_into()
            .expect("u64 size"),
    );
    *at = end;
    value
}

fn string_at(bytes: &[u8], at: &mut usize) -> String {
    let length = i32_at(bytes, at);
    assert!(length >= 0, "negative string length");
    let end = *at + length as usize;
    let text = std::str::from_utf8(bytes.get(*at..end).expect("string bytes"))
        .expect("string utf-8")
        .to_owned();
    *at = end;
    text
}
