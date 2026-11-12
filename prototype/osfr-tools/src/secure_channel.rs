//! The client's secure channel ("s-channel", CSC): the Marketplace preview and order requests
//! (66/1, 66/3) and the Station Cash window's buy (66/43) arrive sealed, as a top-level gateway
//! message the tunnel parser cannot read (rc-0137 recheck: a 112-byte "family 55094" message after
//! the zip-code prompt ended the session).
//!
//! Recovered client facts (`FreeRealms.exe`):
//! * `ConnectGateway` (`008fecb0`) runs `EstablishCSC` (`008a8030`), which installs a
//!   `Crypto::CipherCCM` provider (`ClientGatewayApi__SetApiKind2` `00acc950` ->
//!   `ReplaceApiProvider` `00d333c0` -> `CreateCipherByKind` `010fe940` kind 2) keyed by the
//!   holder at client `+0x394` (`maybe_SetKeyChecked` `01100720` passes the holder's bytes to the
//!   AES key setup).
//! * A secure send (`CryptoBaseApi__InternalSendSecure` `00d336f0`) seals the whole message
//!   (`SealBlockAndAppendNonce` `01100b10`): AES-CCM with a 16-byte tag and a 13-byte nonce, no
//!   associated data, laid out ciphertext, tag, nonce. The nonce is a 64-bit little-endian
//!   message counter (1 for the first message) and five random bytes. `WrapPacket` (`00d33b10`)
//!   then appends the marker: a byte (1 = sealed, 0 = plain) and the dword `0xF3D4A5C9`.
//! * The receiving side (`maybe_InspectTrailingCryptoMarker` `00d33a60`, `CheckHeaderAndVerify`
//!   `01100790`) strips a plain marker, opens a sealed one, and refuses a zero nonce, a zero tag
//!   and a counter not above the last one seen. A message with no marker passes unchanged, which
//!   is why the server's own replies need none.
//!
//! The key is the login reply's second string (`login::character_login_reply` writes the gateway
//! ticket there), which the client also sends back as the ticket of its gateway login; native run
//! 20261112T020333Z's sealed 66/1 opens with that ticket's 32 ASCII bytes as an AES-256 key.
//!
//! ponytail: a forward-only AES (CCM never runs the inverse cipher) and CCM decryption in one
//! file; nothing here encrypts, because the server never seals a reply.

use std::borrow::Cow;
use subtle::ConstantTimeEq;

/// `WrapPacket`'s trailer dword (`-204167735`), little-endian on the wire.
const MARKER: [u8; 4] = 0xF3D4_A5C9_u32.to_le_bytes();
const TAG: usize = 16;
const NONCE: usize = 13;

/// What [`SecureChannel::open`] made of one top-level message.
#[derive(Debug, PartialEq, Eq)]
pub enum Opened {
    /// No marker: the message as it came.
    Plain,
    /// The message without its trailer (a plain marker) or its opened plaintext.
    Unwrapped(Vec<u8>),
    /// A sealed message this channel cannot open (no key, a bad tag, a replayed counter).
    Refused(&'static str),
}

/// One session's channel: the key from its gateway login and the last counter opened.
/// No `Debug`: the round keys are the client's session key.
#[derive(Clone, Default)]
pub struct SecureChannel {
    round_keys: Option<Vec<[u8; 16]>>,
    last_counter: u64,
}

impl SecureChannel {
    /// Learn the key from the client's gateway login (u16 1, then the ticket as a u32-counted
    /// string); every other message leaves the channel as it is. A ticket that is not an AES key
    /// length leaves no key, so a sealed message is refused rather than misread.
    pub fn learn(&mut self, message: &[u8]) {
        let Some(&[1, 0, a, b, c, d]) = message.get(..6) else {
            return;
        };
        // One login per session: a later message that happens to lead with 01 00 keeps the key.
        if self.round_keys.is_some() {
            return;
        }
        let length = u32::from_le_bytes([a, b, c, d]) as usize;
        if let Some(ticket) = message.get(6..6usize.saturating_add(length)) {
            self.round_keys = expand_key(ticket);
            self.last_counter = 0;
        }
    }

    /// The session's entry point for one top-level message: learn the key from a login, then
    /// strip or open the trailer. A message it cannot open is returned as it came, so the
    /// gateway's top-level parser traces it as unknown and drops it.
    ///
    /// A Group (`00 19`) is opened child by child, in order (one replay counter): the client seals
    /// each message before the SOE driver groups what is queued (`InternalSendSecure` ->
    /// `WrapPacket` -> `TryDriverHandoffA` `010d58b0`; run 20261112T020333Z grouped 21 Marketplace
    /// 66/27s with other messages). A group that does not parse is left to the gateway as it came.
    pub fn take<'a>(&mut self, message: &'a [u8]) -> Cow<'a, [u8]> {
        self.learn(message);
        if message.starts_with(&[0, 25]) {
            return self.take_group(message);
        }
        match self.open_logged(message) {
            Some(plain) => Cow::Owned(plain),
            None => Cow::Borrowed(message),
        }
    }

    /// The opened bytes, or `None` to keep the message as it came (logging a refusal).
    fn open_logged(&mut self, message: &[u8]) -> Option<Vec<u8>> {
        match self.open(message) {
            Opened::Plain => None,
            Opened::Unwrapped(plain) => Some(plain),
            Opened::Refused(reason) => {
                eprintln!("secure gateway message dropped: {reason}");
                None
            }
        }
    }

    fn take_group<'a>(&mut self, group: &'a [u8]) -> Cow<'a, [u8]> {
        let mut children = Vec::new();
        let mut at = 2;
        while at < group.len() {
            let Some((size, prefix)) = crate::soe::variable_length(&group[at..]) else {
                return Cow::Borrowed(group);
            };
            let Some(child) = group.get(at + prefix..(at + prefix).saturating_add(size)) else {
                return Cow::Borrowed(group);
            };
            children.push(child);
            at += prefix + size;
        }
        let opened: Vec<Option<Vec<u8>>> = children
            .iter()
            .map(|child| self.open_logged(child))
            .collect();
        if opened.iter().all(Option::is_none) {
            return Cow::Borrowed(group);
        }
        let mut out = vec![0, 25];
        for (child, plain) in children.iter().zip(&opened) {
            let child = plain.as_deref().unwrap_or(child);
            // PutVariableValue (`010dae40`): one byte below FF, FF + u16, or FF FF FF + u32.
            match u16::try_from(child.len()) {
                Ok(small) if small < 0xff => out.push(small as u8),
                Ok(medium) if medium < 0xffff => {
                    out.push(0xff);
                    out.extend(medium.to_be_bytes());
                }
                _ => {
                    out.extend([0xff; 3]);
                    out.extend(u32::try_from(child.len()).unwrap_or(u32::MAX).to_be_bytes());
                }
            }
            out.extend(child);
        }
        Cow::Owned(out)
    }

    /// Strip or open a message's secure trailer.
    pub fn open(&mut self, message: &[u8]) -> Opened {
        let Some(body_end) = message.len().checked_sub(MARKER.len() + 1) else {
            return Opened::Plain;
        };
        if message[body_end + 1..] != MARKER {
            return Opened::Plain;
        }
        let body = &message[..body_end];
        match message[body_end] {
            0 => Opened::Unwrapped(body.to_vec()),
            1 => match self.unseal(body) {
                Ok(plain) => Opened::Unwrapped(plain),
                Err(reason) => Opened::Refused(reason),
            },
            _ => Opened::Refused("unknown secure marker"),
        }
    }

    fn unseal(&mut self, sealed: &[u8]) -> Result<Vec<u8>, &'static str> {
        let keys = self.round_keys.as_ref().ok_or("no secure channel key")?;
        let length = sealed
            .len()
            .checked_sub(TAG + NONCE)
            .ok_or("short sealed message")?;
        let (cipher, rest) = sealed.split_at(length);
        let (tag, nonce) = rest.split_at(TAG);
        let mut counter = [0; 8];
        counter.copy_from_slice(&nonce[..8]);
        let counter = u64::from_le_bytes(counter);
        if nonce.iter().all(|byte| *byte == 0) || tag.iter().all(|byte| *byte == 0) {
            return Err("empty nonce or tag");
        }
        if counter <= self.last_counter {
            return Err("replayed secure counter");
        }
        let plain = ccm_open(keys, nonce, cipher, tag).ok_or("secure tag mismatch")?;
        self.last_counter = counter;
        Ok(plain)
    }
}

/// AES-CCM decryption and check (SP 800-38C; 13-byte nonce so a 2-byte length, 16-byte tag, no
/// associated data). `None` when the tag does not match or the message is too long for CCM.
fn ccm_open(keys: &[[u8; 16]], nonce: &[u8], cipher: &[u8], tag: &[u8]) -> Option<Vec<u8>> {
    let length = u16::try_from(cipher.len()).ok()?;
    let block = |flags: u8, tail: [u8; 2]| {
        let mut out = [0; 16];
        out[0] = flags;
        out[1..14].copy_from_slice(nonce);
        out[14..].copy_from_slice(&tail);
        out
    };
    let mut plain = Vec::with_capacity(cipher.len());
    for (index, chunk) in cipher.chunks(16).enumerate() {
        let counter = u16::try_from(index + 1).ok()?;
        let stream = encrypt_block(keys, block(1, counter.to_be_bytes()));
        plain.extend(chunk.iter().zip(stream).map(|(byte, key)| byte ^ key));
    }
    // CBC-MAC over B0 (flags: tag length (16 - 2) / 2 << 3, length size 2 - 1) and the plaintext.
    let mut mac = encrypt_block(keys, block(0x39, length.to_be_bytes()));
    for chunk in plain.chunks(16) {
        chunk
            .iter()
            .zip(mac.iter_mut())
            .for_each(|(byte, x)| *x ^= byte);
        mac = encrypt_block(keys, mac);
    }
    let first = encrypt_block(keys, block(1, [0, 0]));
    let expected: Vec<u8> = mac.iter().zip(first).map(|(x, s)| x ^ s).collect();
    bool::from(expected.ct_eq(tag)).then_some(plain)
}

const SBOX: [u8; 256] = [
    0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5, 0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab, 0x76,
    0xca, 0x82, 0xc9, 0x7d, 0xfa, 0x59, 0x47, 0xf0, 0xad, 0xd4, 0xa2, 0xaf, 0x9c, 0xa4, 0x72, 0xc0,
    0xb7, 0xfd, 0x93, 0x26, 0x36, 0x3f, 0xf7, 0xcc, 0x34, 0xa5, 0xe5, 0xf1, 0x71, 0xd8, 0x31, 0x15,
    0x04, 0xc7, 0x23, 0xc3, 0x18, 0x96, 0x05, 0x9a, 0x07, 0x12, 0x80, 0xe2, 0xeb, 0x27, 0xb2, 0x75,
    0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0, 0x52, 0x3b, 0xd6, 0xb3, 0x29, 0xe3, 0x2f, 0x84,
    0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b, 0x6a, 0xcb, 0xbe, 0x39, 0x4a, 0x4c, 0x58, 0xcf,
    0xd0, 0xef, 0xaa, 0xfb, 0x43, 0x4d, 0x33, 0x85, 0x45, 0xf9, 0x02, 0x7f, 0x50, 0x3c, 0x9f, 0xa8,
    0x51, 0xa3, 0x40, 0x8f, 0x92, 0x9d, 0x38, 0xf5, 0xbc, 0xb6, 0xda, 0x21, 0x10, 0xff, 0xf3, 0xd2,
    0xcd, 0x0c, 0x13, 0xec, 0x5f, 0x97, 0x44, 0x17, 0xc4, 0xa7, 0x7e, 0x3d, 0x64, 0x5d, 0x19, 0x73,
    0x60, 0x81, 0x4f, 0xdc, 0x22, 0x2a, 0x90, 0x88, 0x46, 0xee, 0xb8, 0x14, 0xde, 0x5e, 0x0b, 0xdb,
    0xe0, 0x32, 0x3a, 0x0a, 0x49, 0x06, 0x24, 0x5c, 0xc2, 0xd3, 0xac, 0x62, 0x91, 0x95, 0xe4, 0x79,
    0xe7, 0xc8, 0x37, 0x6d, 0x8d, 0xd5, 0x4e, 0xa9, 0x6c, 0x56, 0xf4, 0xea, 0x65, 0x7a, 0xae, 0x08,
    0xba, 0x78, 0x25, 0x2e, 0x1c, 0xa6, 0xb4, 0xc6, 0xe8, 0xdd, 0x74, 0x1f, 0x4b, 0xbd, 0x8b, 0x8a,
    0x70, 0x3e, 0xb5, 0x66, 0x48, 0x03, 0xf6, 0x0e, 0x61, 0x35, 0x57, 0xb9, 0x86, 0xc1, 0x1d, 0x9e,
    0xe1, 0xf8, 0x98, 0x11, 0x69, 0xd9, 0x8e, 0x94, 0x9b, 0x1e, 0x87, 0xe9, 0xce, 0x55, 0x28, 0xdf,
    0x8c, 0xa1, 0x89, 0x0d, 0xbf, 0xe6, 0x42, 0x68, 0x41, 0x99, 0x2d, 0x0f, 0xb0, 0x54, 0xbb, 0x16,
];

/// FIPS-197 key expansion for a 16-, 24- or 32-byte key: the round keys, or `None` for any other
/// length.
fn expand_key(key: &[u8]) -> Option<Vec<[u8; 16]>> {
    let nk = match key.len() {
        16 | 24 | 32 => key.len() / 4,
        _ => return None,
    };
    let rounds = nk + 6;
    let mut words: Vec<[u8; 4]> = key.chunks(4).map(|w| [w[0], w[1], w[2], w[3]]).collect();
    let mut rcon = 1u8;
    for index in nk..4 * (rounds + 1) {
        let mut word = words[index - 1];
        if index % nk == 0 {
            word = [
                SBOX[word[1] as usize],
                SBOX[word[2] as usize],
                SBOX[word[3] as usize],
                SBOX[word[0] as usize],
            ];
            word[0] ^= rcon;
            rcon = xtime(rcon);
        } else if nk > 6 && index % nk == 4 {
            word = word.map(|byte| SBOX[byte as usize]);
        }
        let back = words[index - nk];
        words.push([
            word[0] ^ back[0],
            word[1] ^ back[1],
            word[2] ^ back[2],
            word[3] ^ back[3],
        ]);
    }
    Some(
        words
            .chunks(4)
            .map(|round| {
                let mut out = [0; 16];
                round
                    .iter()
                    .enumerate()
                    .for_each(|(at, w)| out[at * 4..at * 4 + 4].copy_from_slice(w));
                out
            })
            .collect(),
    )
}

fn xtime(byte: u8) -> u8 {
    (byte << 1) ^ if byte & 0x80 != 0 { 0x1b } else { 0 }
}

/// One AES block encryption with the expanded round keys (column-major state, as FIPS-197).
fn encrypt_block(keys: &[[u8; 16]], input: [u8; 16]) -> [u8; 16] {
    let add = |state: &mut [u8; 16], key: &[u8; 16]| {
        state.iter_mut().zip(key).for_each(|(byte, k)| *byte ^= k);
    };
    let mut state = input;
    let last = keys.len() - 1;
    add(&mut state, &keys[0]);
    for (round, key) in keys.iter().enumerate().skip(1) {
        let mut shifted = [0; 16];
        for column in 0..4 {
            for row in 0..4 {
                shifted[column * 4 + row] = SBOX[state[((column + row) % 4) * 4 + row] as usize];
            }
        }
        state = shifted;
        if round != last {
            for column in state.chunks_mut(4) {
                let [a, b, c, d] = [column[0], column[1], column[2], column[3]];
                let all = a ^ b ^ c ^ d;
                column[0] ^= all ^ xtime(a ^ b);
                column[1] ^= all ^ xtime(b ^ c);
                column[2] ^= all ^ xtime(c ^ d);
                column[3] ^= all ^ xtime(d ^ a);
            }
        }
        add(&mut state, key);
    }
    state
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(text: &str) -> Vec<u8> {
        crate::trace::parse_hex(text).expect("test hex")
    }

    #[test]
    fn aes_256_matches_fips_197_appendix_c3() {
        let keys = expand_key(&hex(
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
        ))
        .unwrap();
        let mut block = [0; 16];
        block.copy_from_slice(&hex("00112233445566778899aabbccddeeff"));
        assert_eq!(
            encrypt_block(&keys, block).to_vec(),
            hex("8ea2b7ca516745bfeafc49904b496089")
        );
    }

    #[test]
    fn aes_192_matches_fips_197_appendix_c2() {
        let keys = expand_key(&hex("000102030405060708090a0b0c0d0e0f1011121314151617")).unwrap();
        let mut block = [0; 16];
        block.copy_from_slice(&hex("00112233445566778899aabbccddeeff"));
        assert_eq!(
            encrypt_block(&keys, block).to_vec(),
            hex("dda97ca4864cdfe06eaf70a0ec0d7191")
        );
    }

    #[test]
    fn aes_128_matches_fips_197_appendix_c1() {
        let keys = expand_key(&hex("000102030405060708090a0b0c0d0e0f")).unwrap();
        let mut block = [0; 16];
        block.copy_from_slice(&hex("00112233445566778899aabbccddeeff"));
        assert_eq!(
            encrypt_block(&keys, block).to_vec(),
            hex("69c4e0d86a7b0430d8cdb78070b4c55a")
        );
    }

    #[test]
    fn a_group_is_opened_child_by_child() {
        let mut channel = SecureChannel::default();
        let plain = vec![5, 0, 1, 2];
        let mut marked = plain.clone();
        marked.push(0);
        marked.extend(MARKER);
        let group = |children: &[&[u8]]| {
            let mut out = vec![0, 25];
            for child in children {
                out.push(child.len() as u8);
                out.extend(*child);
            }
            out
        };
        let sent = group(&[&plain, &marked]);
        assert_eq!(&*channel.take(&sent), &group(&[&plain, &plain])[..]);
        // Nothing to open: the group as it came.
        let unmarked = group(&[&plain]);
        assert!(matches!(channel.take(&unmarked), Cow::Borrowed(_)));
    }

    #[test]
    fn a_plain_marker_is_stripped_and_an_unmarked_message_passes() {
        let mut channel = SecureChannel::default();
        assert_eq!(channel.open(&[5, 0, 1, 2]), Opened::Plain);
        let mut marked = vec![5, 0, 1, 2, 0];
        marked.extend(MARKER);
        assert_eq!(channel.open(&marked), Opened::Unwrapped(vec![5, 0, 1, 2]));
        let mut sealed = vec![9; 40];
        sealed.push(1);
        sealed.extend(MARKER);
        assert_eq!(
            channel.open(&sealed),
            Opened::Refused("no secure channel key")
        );
    }
}
