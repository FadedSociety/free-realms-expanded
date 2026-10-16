//! The original client's chat request and system-message wire shapes.
//! Protocol fields: supplied ChatSystem and pinned Sanctuary PacketChat.
use std::io;

fn bad() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid chat packet")
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> io::Result<&'a [u8]> {
        let end = self.at.checked_add(count).ok_or_else(bad)?;
        let part = self.bytes.get(self.at..end).ok_or_else(bad)?;
        self.at = end;
        Ok(part)
    }
    fn string(&mut self, limit: usize) -> io::Result<&'a str> {
        let count = u32::from_le_bytes(self.take(4)?.try_into().unwrap()) as usize;
        if count > limit {
            return Err(bad());
        }
        std::str::from_utf8(self.take(count)?).map_err(|_| bad())
    }
    fn name(&mut self) -> io::Result<()> {
        self.take(12)?;
        self.string(128)?;
        self.string(128)?;
        Ok(())
    }
}

/// Return text only. The caller's selected session supplies command identity;
/// client-supplied sender/recipient GUIDs and names never select a character.
pub fn request(inner: &[u8], _guid: u64) -> io::Result<Option<String>> {
    if !inner.starts_with(&[15, 0, 1, 0]) {
        return Ok(None);
    }
    let mut r = Reader {
        bytes: inner,
        at: 4,
    };
    let channel = u16::from_le_bytes(r.take(2)?.try_into().unwrap());
    if channel > 10 {
        return Err(bad());
    }
    // Actual local-chat requests contain either zero or unrelated nonzero
    // GUIDs. Consume both advisory fields; never use them as authority.
    r.take(16)?;
    r.name()?;
    r.name()?;
    let message = r.string(2048)?.to_owned();
    r.take(16 + 8 + 4)?;
    if channel == 8 {
        r.take(4)?;
    }
    if r.at != inner.len() {
        return Err(bad());
    }
    Ok(Some(message))
}

/// A client local-chat request (15/1) carrying `text`, as the client sends it;
/// tooling uses it to run developer commands through the normal path.
pub fn local_request(text: &str) -> Vec<u8> {
    let mut packet = vec![15, 0, 1, 0, 0, 0];
    packet.extend(1_u64.to_le_bytes());
    packet.extend(0_u64.to_le_bytes());
    for _ in 0..2 {
        packet.extend([0; 12]);
        packet.extend(0_u32.to_le_bytes());
        packet.extend(0_u32.to_le_bytes());
    }
    packet.extend((text.len() as u32).to_le_bytes());
    packet.extend(text.as_bytes());
    packet.extend([0; 28]);
    packet
}

pub fn system(message: &str) -> Vec<u8> {
    let escaped = message
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    let mut packet = vec![15, 0, 3, 0];
    packet.extend_from_slice(&(escaped.len() as u32).to_le_bytes());
    packet.extend_from_slice(escaped.as_bytes());
    packet.push(1);
    packet
}

#[cfg(test)]
mod tests {
    use super::*;
    fn example() -> Vec<u8> {
        let mut p = vec![15, 0, 1, 0, 0, 0];
        p.extend(1u64.to_le_bytes());
        p.extend(0u64.to_le_bytes());
        for _ in 0..2 {
            p.extend([0; 12]);
            p.extend(0u32.to_le_bytes());
            p.extend(0u32.to_le_bytes());
        }
        p.extend(10u32.to_le_bytes());
        p.extend(b"realm help");
        p.extend([0; 28]);
        p
    }
    #[test]
    fn original_shape_is_consumed_and_session_supplies_identity() {
        let p = example();
        assert_eq!(request(&p, 1).unwrap().as_deref(), Some("realm help"));
        assert_eq!(request(&p, 2).unwrap().as_deref(), Some("realm help"));
        for n in 4..p.len() {
            assert!(request(&p[..n], 1).is_err());
        }
        let mut extra = p;
        extra.push(0);
        assert!(request(&extra, 1).is_err());
    }
    #[test]
    fn feedback_escapes_game_markup() {
        let p = system("<loot>&");
        assert_eq!(&p[8..p.len() - 1], b"&lt;loot&gt;&amp;");
        assert_eq!(p[p.len() - 1], 1);
    }
    #[test]
    fn actual_client_advisory_guids_do_not_select_command_identity() {
        // Synthetic reconstruction of the observed shape: empty name records,
        // GUID0, vectorW1 and language2. No captured player text is embedded.
        let mut p = example();
        p[6..14].copy_from_slice(&0u64.to_le_bytes());
        let tail = p.len() - 28;
        p[tail + 12..tail + 16].copy_from_slice(&1f32.to_le_bytes());
        p[tail + 24..tail + 28].copy_from_slice(&2i32.to_le_bytes());
        assert_eq!(request(&p, 1).unwrap().as_deref(), Some("realm help"));
        assert_eq!(request(&p, 9876).unwrap().as_deref(), Some("realm help"));
        // A later real request had unrelated nonzero sender AND recipient.
        // Synthetic values exercise that shape without retaining raw IDs.
        p[6..14].copy_from_slice(&0xdead_beef_1234_5678u64.to_le_bytes());
        p[14..22].copy_from_slice(&0x1234_5678_dead_beefu64.to_le_bytes());
        assert_eq!(request(&p, 1).unwrap().as_deref(), Some("realm help"));
        p[6..14].fill(0);
        for n in 4..p.len() {
            assert!(request(&p[..n], 1).is_err());
        }
        p.push(0);
        assert!(request(&p, 1).is_err());
    }
}
