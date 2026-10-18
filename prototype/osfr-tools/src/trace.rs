//! Opt-in JSONL trace output shared by the local gateway and login binaries.

use serde_json::Value;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Write},
    path::Path,
    sync::{Arc, Mutex},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Default)]
pub struct Trace {
    sink: Option<Arc<Mutex<BufWriter<File>>>>,
    started: Option<Instant>,
}

impl Trace {
    pub fn open(path: &Path) -> io::Result<Self> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)?;
        Ok(Self {
            sink: Some(Arc::new(Mutex::new(BufWriter::new(file)))),
            started: Some(Instant::now()),
        })
    }

    #[cfg(test)]
    pub fn is_enabled(&self) -> bool {
        self.sink.is_some()
    }

    pub fn record(&self, mut value: Value) -> io::Result<()> {
        let (Some(sink), Some(started)) = (&self.sink, self.started) else {
            return Ok(());
        };
        let Some(object) = value.as_object_mut() else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "trace event must be a JSON object",
            ));
        };
        object.insert(
            "ts_ms".into(),
            Value::from(started.elapsed().as_millis() as u64),
        );
        if let Ok(wall) = SystemTime::now().duration_since(UNIX_EPOCH) {
            object.insert("wall_ms".into(), Value::from(wall.as_millis() as u64));
        }
        let mut writer = sink
            .lock()
            .map_err(|_| io::Error::other("trace writer lock poisoned"))?;
        serde_json::to_writer(&mut *writer, &value)?;
        writer.write_all(b"\n")?;
        writer.flush()
    }
}

pub struct ProtocolSummary<'a> {
    pub tunnel_outer_opcode: Option<u16>,
    pub family: Option<u16>,
    pub subtype: Option<u16>,
    pub family_name: Option<&'static str>,
    pub body: &'a [u8],
    pub combined: bool,
    pub sensitive: bool,
}

pub fn protocol_summary(message: &[u8]) -> ProtocolSummary<'_> {
    let top = message.get(..2).map(|b| u16::from_le_bytes([b[0], b[1]]));
    let mut outer = None;
    let mut body = message;
    let mut sensitive = top == Some(1);
    if matches!(top, Some(5 | 6)) && message.len() >= 7 {
        let size = u32::from_le_bytes(message[3..7].try_into().unwrap()) as usize;
        if message[2] <= 1 && size <= message.len().saturating_sub(7) {
            outer = top;
            body = &message[7..7 + size];
        }
    }
    let combined = body.starts_with(&[0, 25]);
    let family = body.get(..2).map(|b| u16::from_le_bytes([b[0], b[1]]));
    let subtype = body.get(2..4).map(|b| u16::from_le_bytes([b[0], b[1]]));
    sensitive |= family
        .zip(subtype)
        .is_some_and(|(f, s)| sensitive_payload(f, s));
    ProtocolSummary {
        tunnel_outer_opcode: outer,
        family,
        subtype,
        family_name: family.and_then(|f| subtype.and_then(|s| family_subtype_name(f, s))),
        body,
        combined,
        sensitive,
    }
}

pub fn family_subtype_name(family: u16, subtype: u16) -> Option<&'static str> {
    match (family, subtype) {
        (15, 1) => Some("ChatMessage"),
        (26, 4) => Some("DialogueClose"),
        (26, 6) => Some("DialogueResponse"),
        (26, 8) => Some("NpcInteract"),
        (26, 23) => Some("QuestAbandon"),
        (49, 1) => Some("QuestInfo"),
        (49, 2) => Some("QuestReply"),
        (49, 13) => Some("QuestEnd"),
        (49, 14) => Some("QuestEndReply"),
        (66, 43) => Some("MerchantRequest"),
        (67, 2) => Some("QuickChat"),
        (67, 3) => Some("QuickChatReply"),
        _ => None,
    }
}

/// The one rule for which packet bodies a trace must never show as hex (credentials, chat,
/// friends, mail and other player text). Every tracer uses it; do not copy it.
pub fn sensitive_payload(family: u16, subtype: u16) -> bool {
    family == 1
        || matches!(family, 15 | 22 | 40 | 68 | 74 | 109 | 112 | 149 | 175)
        || (family == 26 && matches!(subtype, 14 | 15 | 16 | 17 | 30))
        || (family == 53 && subtype == 6)
}

pub fn hex_prefix(bytes: &[u8], sensitive: bool) -> Option<String> {
    hex_prefix_len(bytes, sensitive, 32)
}

/// `hex_prefix` with a longer window for the Studio packet inspector. One formatter, one rule.
pub fn hex_prefix_len(bytes: &[u8], sensitive: bool, len: usize) -> Option<String> {
    let head = bytes.iter().take(len).copied().collect::<Vec<_>>();
    let mut letters = 0;
    let looks_like_text = head.iter().any(|byte| {
        if byte.is_ascii_alphabetic() {
            letters += 1;
        } else if !byte.is_ascii_whitespace() {
            letters = 0;
        }
        letters >= 3
    });
    (!sensitive && !looks_like_text)
        .then(|| head.iter().map(|byte| format!("{byte:02x}")).collect())
}

/// Parse separator-free hex. Anything that is not ASCII hex (including non-ASCII text) is
/// refused before slicing: slicing a multi-byte character by byte index panics, and one bad
/// request or script line must never take a process down.
pub fn parse_hex(hex: &str) -> Option<Vec<u8>> {
    if hex.is_empty() || hex.len() % 2 != 0 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}

pub fn transport_name(opcode: u16) -> &'static str {
    match opcode {
        1 => "SessionRequest",
        2 => "SessionResponse",
        5 => "DirectApplication",
        6 => "SessionDisconnect",
        7 => "Ping",
        9 => "ReliableData",
        13 => "ReliableFragment",
        17 => "SelectiveAck",
        21 => "CumulativeAck",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file() -> std::path::PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("osfr-trace-{}-{suffix}.jsonl", std::process::id()))
    }

    #[test]
    fn tracing_is_opt_in_by_default() {
        let trace = Trace::default();
        assert!(!trace.is_enabled());
        trace.record(json!({"event":"ignored"})).unwrap();
    }

    #[test]
    fn writes_timestamped_jsonl_and_known_names() {
        let path = temp_file();
        let trace = Trace::open(&path).unwrap();
        trace
            .record(json!({"event":"packet","direction":"in","family":49,"subtype":1}))
            .unwrap();
        drop(trace);
        let value: Value = serde_json::from_str(fs::read_to_string(&path).unwrap().trim()).unwrap();
        fs::remove_file(path).unwrap();
        assert!(value["ts_ms"].as_u64().is_some());
        assert!(value["wall_ms"].as_u64().is_some());
        assert_eq!(value["direction"], "in");
        assert_eq!(family_subtype_name(49, 1), Some("QuestInfo"));
    }

    #[test]
    fn chat_and_login_prefixes_never_enter_trace() {
        let chat = [5, 0, 1, 6, 0, 0, 0, 15, 0, 1, 0, 0xaa, 0xbb];
        let summary = protocol_summary(&chat);
        assert_eq!(summary.family, Some(15));
        assert_eq!(summary.subtype, Some(1));
        assert!(hex_prefix(summary.body, summary.sensitive).is_none());
        assert!(protocol_summary(&[5, 0, 1, 4, 0, 0, 0, 40, 0, 1, 0]).sensitive);
        let login = protocol_summary(&[1, 0, b't', b'i', b'c', b'k', b'e', b't']);
        assert!(login.sensitive);
        assert!(hex_prefix(login.body, login.sensitive).is_none());
    }

    /// Lead (2026-10-18): gateway.rs redacted these while the binaries' tracer did not, so the
    /// packet trace and the Studio inspector could show chat and friend payloads as hex.
    #[test]
    fn chat_friend_and_client_log_bodies_are_redacted_by_the_one_rule() {
        for (family, subtype) in [
            (109, 0),
            (109, 7),
            (26, 14),
            (26, 15),
            (26, 16),
            (26, 17),
            (26, 30),
            (53, 6),
        ] {
            assert!(
                sensitive_payload(family, subtype),
                "{family}/{subtype} must be redacted"
            );
        }
        assert!(
            !sensitive_payload(26, 8),
            "ordinary NPC interaction stays visible"
        );
    }
}
