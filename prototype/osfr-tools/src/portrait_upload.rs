//! L071: the Fotomat portrait upload. After the photo booth, the client POSTs two PNGs to
//! `ClientConfig.ini` `[Portrait] UploadUrl` (the launcher points it at the gateway's portrait
//! port) and the gateway shows them in 156/3 (`fotomat.rs`).
//!
//! Adapted from Sanctuary (Open-Source-Free-Realms, AGPL-3.0), commit 3b3aec3:
//! `src/Sanctuary.WebAPI/Endpoints/PortraitEndpoints.cs` (the fixed boundary, field names, file
//! names and sizes) and `src/Sanctuary.Core/Helpers/PortraitStorage.cs` (the CRC-32 the client's
//! 115 notice carries, one folder per character guid).
//!
//! Unlike that endpoint, an upload is only taken from the player's own session (the gateway checks
//! the uploader's address against the live session that plays the character) and every PNG is
//! rebuilt here before any other client sees it: only IHDR, IDAT and IEND survive, the pixel data
//! is inflated under an exact size limit, its row filters are checked, and it is compressed anew.
//! Paths come from the guid alone; nothing in the request names a file on disk.
use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// The client's fixed multipart boundary (`PortraitEndpoints.cs`).
pub const CONTENT_TYPE: &str = "multipart/form-data; boundary=AaBb432101234bBaA";
const BOUNDARY: &[u8] = b"AaBb432101234bBaA";
/// (R) Two PNGs of 180x330 and 70x70 RGBA are at most ~240 KB raw; anything larger is refused.
pub const MAX_REQUEST: usize = 512 * 1024;
const MAX_HEADERS: usize = 8 * 1024;
const PORTRAIT: (u32, u32) = (180, 330);
const HEADSHOT: (u32, u32) = (70, 70);
const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

/// One accepted upload: the character and its rebuilt images.
#[derive(Debug, PartialEq, Eq)]
pub struct Upload {
    pub character_guid: u64,
    /// The rebuilt portrait.png, and the CRC-32 of the bytes the client sent (its 115 carries it).
    pub portrait: Option<(Vec<u8>, u32)>,
    pub headshot: Option<Vec<u8>>,
}

/// How many bytes the whole request needs once its headers have arrived, or `None` while they
/// have not. `Err` refuses a request that can never be valid (too large, no length).
pub fn request_length(raw: &[u8]) -> Result<Option<usize>, &'static str> {
    let Some(end) = find(raw, b"\r\n\r\n") else {
        return if raw.len() > MAX_HEADERS {
            Err("headers too long")
        } else {
            Ok(None)
        };
    };
    let length = header(&raw[..end], "content-length")
        .and_then(|value| value.parse::<usize>().ok())
        .ok_or("no content length")?;
    let total = end + 4 + length;
    if total > MAX_REQUEST {
        return Err("request too large");
    }
    Ok(Some(total))
}

/// Parse one complete HTTP request (`request_length` bytes) into an upload.
pub fn parse_request(raw: &[u8]) -> Result<Upload, &'static str> {
    let end = find(raw, b"\r\n\r\n").ok_or("no headers")?;
    let head = &raw[..end];
    if !head.starts_with(b"POST /") {
        return Err("not a POST");
    }
    if header(head, "content-type").as_deref() != Some(CONTENT_TYPE) {
        return Err("wrong content type");
    }
    let body = &raw[end + 4..];
    if header(head, "content-length").and_then(|v| v.parse().ok()) != Some(body.len()) {
        return Err("length mismatch");
    }
    parse_multipart(body)
}

fn parse_multipart(body: &[u8]) -> Result<Upload, &'static str> {
    let mut delimiter = b"\r\n--".to_vec();
    delimiter.extend_from_slice(BOUNDARY);
    // The first boundary has no leading CRLF; give it one so every part splits alike.
    let mut text = b"\r\n".to_vec();
    text.extend_from_slice(body);
    let mut parts = split(&text, &delimiter);
    if !parts.next().is_some_and(<[u8]>::is_empty) {
        return Err("data before the first boundary");
    }
    let (mut image_type, mut guid, mut portrait, mut headshot) = (None, None, None, None);
    let mut closed = false;
    for (index, part) in parts.enumerate() {
        if index >= 6 {
            return Err("too many parts");
        }
        if part.starts_with(b"--") {
            closed = true;
            break;
        }
        let part = part.strip_prefix(b"\r\n").ok_or("bad part")?;
        let split_at = find(part, b"\r\n\r\n").ok_or("bad part headers")?;
        let (head, data) = (&part[..split_at], &part[split_at + 4..]);
        let disposition = header(head, "content-disposition").ok_or("no disposition")?;
        let name = quoted(&disposition, "name").ok_or("no part name")?;
        let file = quoted(&disposition, "filename");
        match (name.as_str(), file.as_deref()) {
            ("imageType", None) => image_type = Some(data == b"portrait"),
            ("characterId", None) => {
                guid = std::str::from_utf8(data)
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
            }
            ("imageFile", Some("portrait.png")) if portrait.is_none() => {
                png_part(head)?;
                let crc = crc32(data);
                portrait = Some((rebuild_png(data, PORTRAIT)?, crc));
            }
            ("thumbnailFile", Some("headshot.png")) if headshot.is_none() => {
                png_part(head)?;
                headshot = Some(rebuild_png(data, HEADSHOT)?);
            }
            _ => return Err("unexpected part"),
        }
    }
    if !closed || image_type != Some(true) {
        return Err("not a portrait upload");
    }
    let character_guid = guid.filter(|guid| *guid != 0).ok_or("no character")?;
    if portrait.is_none() && headshot.is_none() {
        return Err("no image");
    }
    Ok(Upload {
        character_guid,
        portrait,
        headshot,
    })
}

fn png_part(head: &[u8]) -> Result<(), &'static str> {
    (header(head, "content-type").as_deref() == Some("image/png"))
        .then_some(())
        .ok_or("not image/png")
}

/// Check a client PNG and write it again from its pixels: 8-bit RGB or RGBA, not interlaced,
/// exactly `size`. Unknown critical chunks refuse it; ancillary chunks are dropped.
pub fn rebuild_png(bytes: &[u8], size: (u32, u32)) -> Result<Vec<u8>, &'static str> {
    let mut rest = bytes.strip_prefix(&PNG_SIGNATURE).ok_or("not a PNG")?;
    let (mut colour, mut data, mut ended) = (None, Vec::new(), false);
    while !rest.is_empty() {
        let length = u32::from_be_bytes(rest.get(..4).ok_or("bad chunk")?.try_into().unwrap());
        let length = usize::try_from(length).map_err(|_| "bad chunk")?;
        let kind = rest.get(4..8).ok_or("bad chunk")?;
        let body = rest.get(8..8 + length).ok_or("bad chunk")?;
        let crc = rest.get(8 + length..12 + length).ok_or("bad chunk")?;
        if crc32(&rest[4..8 + length]).to_be_bytes() != crc {
            return Err("bad chunk checksum");
        }
        rest = &rest[12 + length..];
        match kind {
            b"IHDR" if colour.is_none() => {
                let field = |at: usize| u32::from_be_bytes(body[at..at + 4].try_into().unwrap());
                if length != 13 || (field(0), field(4)) != size {
                    return Err("wrong image size");
                }
                // Bit depth 8; colour type 2 (RGB) or 6 (RGBA); deflate, filter 0, no interlace.
                match body[8..] {
                    [8, kind @ (2 | 6), 0, 0, 0] => colour = Some(kind),
                    _ => return Err("unsupported PNG format"),
                }
            }
            b"IDAT" if colour.is_some() => {
                data.extend_from_slice(body);
                if data.len() > MAX_REQUEST {
                    return Err("image data too large");
                }
            }
            b"IEND" if colour.is_some() => {
                ended = true;
                break;
            }
            // A lower-case first letter marks an ancillary chunk: safe to drop.
            _ if colour.is_some() && kind[0].is_ascii_lowercase() => {}
            _ => return Err("unexpected chunk"),
        }
    }
    let colour = colour.filter(|_| ended).ok_or("incomplete PNG")?;
    let pixel = if colour == 6 { 4 } else { 3 };
    let stride = 1 + size.0 as usize * pixel;
    let expected = stride * size.1 as usize;
    let rows = miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(&data, expected)
        .map_err(|_| "bad image data")?;
    if rows.len() != expected || rows.chunks(stride).any(|row| row[0] > 4) {
        return Err("bad image rows");
    }
    let mut out = PNG_SIGNATURE.to_vec();
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&size.0.to_be_bytes());
    ihdr.extend_from_slice(&size.1.to_be_bytes());
    ihdr.extend_from_slice(&[8, colour, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &ihdr);
    chunk(
        &mut out,
        b"IDAT",
        &miniz_oxide::deflate::compress_to_vec_zlib(&rows, 6),
    );
    chunk(&mut out, b"IEND", &[]);
    Ok(out)
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let crc = crc32(&out[start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

/// CRC-32 (IEEE, reflected 0xEDB88320): PNG chunks and the client's 115 portrait notice.
pub fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xEDB8_8320 & (crc & 1).wrapping_neg());
        }
    }
    !crc
}

/// The folder for one character's images under the gateway's portrait folder.
fn character_dir(root: &Path, guid: u64) -> PathBuf {
    root.join(guid.to_string())
}

/// Store an upload's images (each written beside, then renamed into place).
pub fn store(root: &Path, upload: &Upload) -> io::Result<()> {
    let dir = character_dir(root, upload.character_guid);
    fs::create_dir_all(&dir)?;
    let write =
        |name: &str, bytes: &[u8]| crate::character_store::replace_bytes(&dir.join(name), bytes);
    if let Some(headshot) = &upload.headshot {
        write("headshot.png", headshot)?;
    }
    if let Some((portrait, crc)) = &upload.portrait {
        write("portrait.png", portrait)?;
        write("portrait.crc", &crc.to_le_bytes())?;
    }
    Ok(())
}

/// A stored image for 156/3: `Headshot` or `Portrait`, empty when none was uploaded.
pub fn image(root: &Path, guid: u64, provider: &str) -> Vec<u8> {
    let name = match provider {
        "Headshot" => "headshot.png",
        "Portrait" => "portrait.png",
        _ => return Vec::new(),
    };
    fs::read(character_dir(root, guid).join(name))
        .ok()
        .filter(|bytes| bytes.len() <= MAX_REQUEST)
        .unwrap_or_default()
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn split<'a>(text: &'a [u8], delimiter: &'a [u8]) -> impl Iterator<Item = &'a [u8]> {
    let mut rest = Some(text);
    std::iter::from_fn(move || {
        let current = rest?;
        match find(current, delimiter) {
            Some(at) => {
                rest = Some(&current[at + delimiter.len()..]);
                Some(&current[..at])
            }
            None => {
                rest = None;
                Some(current)
            }
        }
    })
}

/// One header's value (name matched without case), trimmed.
fn header(head: &[u8], name: &str) -> Option<String> {
    let head = std::str::from_utf8(head).ok()?;
    head.split("\r\n").find_map(|line| {
        let (key, value) = line.split_once(':')?;
        key.trim()
            .eq_ignore_ascii_case(name)
            .then(|| value.trim().to_owned())
    })
}

/// `key="value"` inside a header value such as Content-Disposition.
fn quoted(value: &str, key: &str) -> Option<String> {
    value.split(';').find_map(|item| {
        let (k, v) = item.trim().split_once('=')?;
        (k == key).then(|| v.trim_matches('"').to_owned())
    })
}
