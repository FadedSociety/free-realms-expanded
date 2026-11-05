//! Direct (unreliable) application packets: alone, or batched into a Multi frame.
use super::*;

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
        // newest for the tick (before that it is refused at once, as any message is).
        let selected = self.selected_character || options.account_store.is_none();
        if selected
            && message
                .get(7..)
                .is_some_and(|inner| inner.starts_with(&SHIP_REPORT))
        {
            let stamp = |report: &[u8]| osfr_tools::le::u64(report, 20);
            let held = self.ship_report.as_deref();
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
