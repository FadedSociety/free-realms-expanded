//! Direct (unreliable) application packets: alone, or batched into a Multi frame.
use super::*;

/// F3: the derby car's pose head (34/5), held per session for the tick like `SHIP_REPORT`.
const DERBY_POSE: [u8; 4] = [34, 0, 5, 0];

impl Session {
    /// A direct application packet, alone or out of a Multi frame.
    pub(super) fn take_direct(
        &mut self,
        message: Vec<u8>,
        options: &Options,
        now_ms: u64,
        world: &mut SharedWorld,
        membership: &mut World,
        sessions: &mut SessionMap,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(trace) = &self.trace {
            let summary = trace::protocol_summary(&message);
            trace.record(json!({
                "event":"packet","session":self.id.0,"peer":self.peer.to_string(),
                "direction":"in","transport":{"soe_opcode":5,"soe_name":"DirectApplication","mode":"direct"},
                "family":summary.family,"subtype":summary.subtype,
                "family_name":summary.family.zip(summary.subtype).and_then(|(f,s)| trace::family_subtype_name(f,s)),
                "body_len":summary.body.len(),"body_head_hex":trace::hex_prefix(summary.body, summary.sensitive),
                "outcome":"received"
            }))?;
        }
        // L965: the ship report comes every client frame; once a character is selected, keep the
        // newest for the tick (before that it is refused at once, as any message is). F3: so does
        // the derby car's pose (34/5), whose stamp is its last 8 bytes (`009f0ce0`).
        let selected = self.selected_character || options.account_store.is_none();
        let inner = message.get(7..).unwrap_or_default();
        if selected && (inner.starts_with(&SHIP_REPORT) || inner.starts_with(&DERBY_POSE)) {
            let stamp = |report: &[u8]| match report.get(7..11) {
                Some(head) if head == DERBY_POSE => {
                    osfr_tools::le::u64(report, report.len().saturating_sub(8))
                }
                _ => osfr_tools::le::u64(report, 20),
            };
            let same_kind = |held: &&[u8]| held.get(7..11) == inner.get(..4);
            let held = self.ship_report.as_deref().filter(same_kind);
            if held.is_none_or(|held| stamp(held) <= stamp(&message)) {
                self.ship_report = Some(message);
            }
            return Ok(());
        }
        self.dispatch_message(
            &message,
            options,
            now_ms,
            world,
            membership,
            sessions,
            Some((5, None, "direct", false)),
        )?;
        Ok(())
    }

    /// L965: the newest held report (ship or derby pose) since the last tick, handed on once.
    pub(super) fn dispatch_held(
        &mut self,
        options: &Options,
        now_ms: u64,
        world: &mut SharedWorld,
        membership: &mut World,
        sessions: &mut SessionMap,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(report) = self.ship_report.take() {
            let transport = Some((5, None, "direct", false));
            self.dispatch_message(
                &report, options, now_ms, world, membership, sessions, transport,
            )?;
        }
        Ok(())
    }

    /// rc-0137 derby: a hit (34/16) or pickup (34/14) report is judged against the car's pose, so
    /// the newest held pose goes first. It arrived before the report, so this is also the order
    /// the client sent them in. Waiting for the tick left the judged pose up to 1.2 s old in run
    /// 20261112T020333Z (the server loop ran late): a ram the client made 3.7 m from the car it hit
    /// was judged 9.7 m away and refused.
    pub(super) fn flush_before_hit(
        &mut self,
        messages: &[Vec<u8>],
        options: &Options,
        now_ms: u64,
        world: &mut SharedWorld,
        membership: &mut World,
        sessions: &mut SessionMap,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let hit = |message: &Vec<u8>| {
            matches!(message.get(7..11), Some([34, 0, 14 | 16, 0]))
                && matches!(message.get(..2), Some([5 | 6, 0]))
        };
        if messages.iter().any(hit) {
            self.dispatch_held(options, now_ms, world, membership, sessions)?;
        }
        Ok(())
    }

    /// The client also batches direct packets into a Multi frame: 05, then the tunnel, whose first
    /// byte reads as the opcode's low byte. About one ship report in nine and every FireAtLocation
    /// came this way (native 194302Z). Each draws on the direct budget, as a lone packet does.
    pub(super) fn multi_direct(
        &mut self,
        packet: &osfr_tools::soe::Packet,
        options: &Options,
        now_ms: u64,
        world: &mut SharedWorld,
        membership: &mut World,
        sessions: &mut SessionMap,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (rate, burst) = DIRECT_BUDGET;
        if !self.direct.take(1.0, rate, burst, Instant::now()) {
            return Ok(());
        }
        let mut tunnel = vec![packet.opcode.to_be_bytes()[1]];
        tunnel.extend(&packet.payload);
        match osfr_tools::soe::admit_direct(tunnel) {
            Ok(message) => self.take_direct(message, options, now_ms, world, membership, sessions),
            Err(error) => {
                if let Some(trace) = &self.trace {
                    trace_transport(
                        trace,
                        Some(self.id.0),
                        self.peer,
                        "in",
                        packet.opcode,
                        &packet.payload,
                        None,
                        "rejected",
                        Some(&error.to_string()),
                    )?;
                }
                Ok(())
            }
        }
    }
}
