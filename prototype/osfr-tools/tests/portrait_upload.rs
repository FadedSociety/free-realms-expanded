//! L071: the portrait upload's request parsing, PNG rebuild and storage. Synthetic images only.
use osfr_tools::portrait_upload::{
    self, CONTENT_TYPE, MAX_REQUEST, crc32, parse_request, rebuild_png, request_length,
};

/// A PNG of `size` with `colour` 2 or 6, every row using `filter`, plus `extra` chunks before
/// IDAT. `rows` overrides the pixel stream (to build broken images).
fn png(size: (u32, u32), colour: u8, filter: u8, extra: &[(&[u8; 4], &[u8])]) -> Vec<u8> {
    let pixel = if colour == 6 { 4 } else { 3 };
    let stride = 1 + size.0 as usize * pixel;
    let mut rows = vec![0x5a; stride * size.1 as usize];
    for row in rows.chunks_mut(stride) {
        row[0] = filter;
    }
    png_with_rows(size, colour, extra, &rows)
}

fn png_with_rows(
    size: (u32, u32),
    colour: u8,
    extra: &[(&[u8; 4], &[u8])],
    rows: &[u8],
) -> Vec<u8> {
    let mut out = vec![137, 80, 78, 71, 13, 10, 26, 10];
    let mut ihdr = size.0.to_be_bytes().to_vec();
    ihdr.extend(size.1.to_be_bytes());
    ihdr.extend([8, colour, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &ihdr);
    for (kind, data) in extra {
        chunk(&mut out, kind, data);
    }
    chunk(
        &mut out,
        b"IDAT",
        &miniz_oxide::deflate::compress_to_vec_zlib(rows, 6),
    );
    chunk(&mut out, b"IEND", &[]);
    out
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend((data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend(kind);
    out.extend(data);
    let crc = crc32(&out[start..]);
    out.extend(crc.to_be_bytes());
}

/// The client's request: the fixed boundary, the two fields, then the two files.
fn request(guid: &str, image_type: &str, files: &[(&str, &str, &[u8])]) -> Vec<u8> {
    let mut body = Vec::new();
    for (name, value) in [("imageType", image_type), ("characterId", guid)] {
        body.extend(
            format!("--AaBb432101234bBaA\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n")
                .bytes(),
        );
    }
    for (name, file, data) in files {
        body.extend(format!("--AaBb432101234bBaA\r\nContent-Disposition: form-data; name=\"{name}\"; filename=\"{file}\"\r\nContent-Type: image/png\r\n\r\n").bytes());
        body.extend(*data);
        body.extend(b"\r\n");
    }
    body.extend(b"--AaBb432101234bBaA--\r\n");
    let mut raw = format!(
        "POST /image HTTP/1.1\r\nHost: realm\r\nContent-Type: {CONTENT_TYPE}\r\nContent-Length: {}\r\n\r\n",
        body.len()
    )
    .into_bytes();
    raw.extend(body);
    raw
}

#[test]
fn crc32_matches_the_standard_vector() {
    assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
}

/// A good image comes back rebuilt (and stable when rebuilt again); ancillary chunks are dropped.
#[test]
fn a_client_png_is_rebuilt_from_its_pixels() {
    for (colour, filter) in [(6, 0), (6, 4), (2, 1), (2, 3)] {
        let text: (&[u8; 4], &[u8]) = (b"tEXt", b"Comment\0hello");
        let original = png((70, 70), colour, filter, &[text]);
        let rebuilt = rebuild_png(&original, (70, 70)).unwrap();
        assert!(
            !rebuilt.windows(4).any(|w| w == b"tEXt"),
            "ancillary chunk dropped"
        );
        assert_eq!(rebuild_png(&rebuilt, (70, 70)).unwrap(), rebuilt);
    }
}

/// Anything other than an exact-size 8-bit RGB/RGBA image with sound chunks and rows is refused.
#[test]
fn malformed_or_unexpected_pngs_are_refused() {
    let good = png((70, 70), 6, 0, &[]);
    let mut bad_crc = good.clone();
    let last = bad_crc.len() - 1;
    bad_crc[last] ^= 1;
    let stride = 1 + 70 * 4;
    let mut bomb = vec![0; stride * 70 * 4];
    for row in bomb.chunks_mut(stride) {
        row[0] = 0;
    }
    let palette: (&[u8; 4], &[u8]) = (b"PLTE", &[0, 0, 0]);
    let unknown: (&[u8; 4], &[u8]) = (b"ZZZZ", b"x");
    for (case, bytes) in [
        ("wrong size", png((70, 71), 6, 0, &[])),
        ("filter 5", png((70, 70), 6, 5, &[])),
        ("palette chunk", png((70, 70), 6, 0, &[palette])),
        ("unknown critical chunk", png((70, 70), 6, 0, &[unknown])),
        ("bad chunk checksum", bad_crc),
        ("too many rows", png_with_rows((70, 70), 6, &[], &bomb)),
        (
            "too few rows",
            png_with_rows((70, 70), 6, &[], &bomb[..stride * 69]),
        ),
        ("not a png", b"GIF89a".to_vec()),
        ("truncated", good[..good.len() - 12].to_vec()),
    ] {
        assert!(rebuild_png(&bytes, (70, 70)).is_err(), "{case}");
    }
    // Grey (0) and other colour types are outside what the client's Fotomat writes (R).
    assert!(rebuild_png(&png((70, 70), 0, 0, &[]), (70, 70)).is_err());
}

/// The client's request parses whole; the portrait's CRC is of the bytes the client sent (its 115
/// carries that), and the stored images are the rebuilt ones.
#[test]
fn the_client_request_parses_and_stores_by_guid() {
    let portrait = png((180, 330), 6, 0, &[]);
    let headshot = png((70, 70), 2, 0, &[]);
    let raw = request(
        "4242",
        "portrait",
        &[
            ("imageFile", "portrait.png", &portrait),
            ("thumbnailFile", "headshot.png", &headshot),
        ],
    );
    let head_end = raw.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
    assert_eq!(
        request_length(&raw[..head_end]),
        Ok(None),
        "headers still arriving"
    );
    assert_eq!(request_length(&raw), Ok(Some(raw.len())));
    let upload = parse_request(&raw).unwrap();
    assert_eq!(upload.character_guid, 4242);
    let (stored, crc) = upload.portrait.clone().unwrap();
    assert_eq!(crc, crc32(&portrait));
    assert_eq!(stored, rebuild_png(&portrait, (180, 330)).unwrap());

    let root = std::env::temp_dir().join(format!("osfr-portraits-{}", std::process::id()));
    portrait_upload::store(&root, &upload).unwrap();
    assert_eq!(portrait_upload::image(&root, 4242, "Portrait"), stored);
    assert_eq!(
        portrait_upload::image(&root, 4242, "Headshot"),
        upload.headshot.unwrap()
    );
    assert!(portrait_upload::image(&root, 4242, "../4242").is_empty());
    assert!(portrait_upload::image(&root, 9, "Portrait").is_empty());
    let _ = std::fs::remove_dir_all(root);
}

/// Every other shape is refused before anything is stored.
#[test]
fn requests_that_are_not_the_clients_upload_are_refused() {
    let portrait = png((180, 330), 6, 0, &[]);
    let file: &[(&str, &str, &[u8])] = &[("imageFile", "portrait.png", &portrait)];
    let good = request("4242", "portrait", file);
    assert!(parse_request(&good).is_ok());
    let replace = |from: &str, to: &str| {
        let text = String::from_utf8_lossy(&good).replacen(from, to, 1);
        // Keep Content-Length honest so only the change itself is judged.
        let (head, body) = text.split_once("\r\n\r\n").unwrap();
        let head = head
            .lines()
            .map(|line| {
                if line.starts_with("Content-Length") {
                    format!("Content-Length: {}", body.len())
                } else {
                    line.to_owned()
                }
            })
            .collect::<Vec<_>>()
            .join("\r\n");
        format!("{head}\r\n\r\n{body}").into_bytes()
    };
    for (case, raw) in [
        ("GET", replace("POST /image", "GET /image")),
        (
            "other boundary",
            replace("boundary=AaBb432101234bBaA", "boundary=x"),
        ),
        (
            "other image type",
            replace("\r\n\r\nportrait\r\n", "\r\n\r\nbanner\r\n"),
        ),
        ("character 0", request("0", "portrait", file)),
        ("no character", request("abc", "portrait", file)),
        (
            "other file name",
            replace("portrait.png", "../../portrait.png"),
        ),
        (
            "other part name",
            replace("name=\"imageFile\"", "name=\"evil\""),
        ),
        (
            "not image/png",
            replace("Content-Type: image/png", "Content-Type: text/html"),
        ),
        ("no image", request("4242", "portrait", &[])),
        ("no closing boundary", replace("--AaBb432101234bBaA--", "")),
    ] {
        assert!(parse_request(&raw).is_err(), "{case}");
    }
    let mut short = good.clone();
    short.pop();
    assert!(parse_request(&short).is_err(), "length mismatch");
    let huge = format!("POST /image HTTP/1.1\r\nContent-Length: {MAX_REQUEST}\r\n\r\n");
    assert!(
        request_length(huge.as_bytes()).is_err(),
        "over the size cap"
    );
    assert!(
        request_length(b"POST /image HTTP/1.1\r\n\r\n").is_err(),
        "no length"
    );
}
