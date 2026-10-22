//! Gateway tick timing for the stress gate (T431, plan 3.6). With `--tick-stats-seconds N`
//! (0, the default, is off; otherwise 5..=600) the gateway times every `Server::step` and every
//! step on which the 20 Hz tick is due, and prints one `tick_stats` event per window: steps,
//! ticks, sessions at the window's end, and p50/p99/max in microseconds.
//!
//! - step: the whole `Server::step`, including its socket wait (at most 10 ms when idle).
//! - tick: from the `tick_due` decision to the end of that step: the shared-field simulation,
//!   every session's tick and the step's flush.
//!
//! Memory is fixed per window: each series keeps at most `capacity` samples, sized from the
//! window and the expected rate and never above `HARD_CAP`. Past that it keeps a uniform random
//! subset (reservoir sampling); the counts and the max stay exact. Cost per step when on: two
//! `Instant::now()` reads and one store into a preallocated buffer; one sort per window.
use serde_json::json;
use std::time::{Duration, Instant};

/// (R) Expected rates, used only to size the buffers: an idle step waits at most 10 ms on the
/// socket (100 a second), doubled for traffic; the simulation ticks every 50 ms.
const STEPS_PER_SECOND: u64 = 200;
const TICKS_PER_SECOND: u64 = 20;
/// Samples kept per series per window (256 KiB of `u32`), whatever the window or the rate.
const HARD_CAP: u64 = 65_536;
const FLAG: &str = "--tick-stats-seconds";

#[cfg(test)]
thread_local! {
    /// Test-only: every event this thread emitted, so a test can read what `event` printed.
    static EMITTED: std::cell::RefCell<Vec<serde_json::Value>> = const { std::cell::RefCell::new(Vec::new()) };
}

struct Series {
    samples: Vec<u32>,
    capacity: usize,
    seen: u64,
    max: u32,
}

impl Series {
    fn new(window: Duration, per_second: u64) -> Self {
        let wanted = (window.as_millis() as u64).saturating_mul(per_second) / 1000;
        let capacity = if window.is_zero() {
            0
        } else {
            wanted.clamp(1, HARD_CAP) as usize
        };
        Self {
            samples: Vec::with_capacity(capacity),
            capacity,
            seen: 0,
            max: 0,
        }
    }

    fn record(&mut self, elapsed: Duration, rng: &mut u64) {
        let micros = u32::try_from(elapsed.as_micros()).unwrap_or(u32::MAX);
        self.seen += 1;
        self.max = self.max.max(micros);
        if self.samples.len() < self.capacity {
            self.samples.push(micros);
            return;
        }
        // Algorithm R: the n-th sample replaces a kept one with probability capacity/n.
        *rng ^= *rng << 13;
        *rng ^= *rng >> 7;
        *rng ^= *rng << 17;
        if let Some(slot) = self.samples.get_mut((*rng % self.seen) as usize) {
            *slot = micros;
        }
    }

    /// Count, p50, p99 and max of the window, then empty the series for the next one.
    fn close(&mut self) -> (u64, u32, u32, u32) {
        self.samples.sort_unstable();
        let rank = |percent: usize| {
            // Nearest rank, as `percentile` in scripts/stress/run.py.
            let index = (self.samples.len() * percent).div_ceil(100).max(1) - 1;
            self.samples.get(index).copied().unwrap_or(0)
        };
        let summary = (self.seen, rank(50), rank(99), self.max);
        self.samples.clear();
        (self.seen, self.max) = (0, 0);
        summary
    }
}

pub(super) struct TickStats {
    window: Duration,
    started: Instant,
    steps: Series,
    ticks: Series,
    rng: u64,
}

impl TickStats {
    /// A zero window is off: nothing is recorded, allocated or printed.
    pub(super) fn new(window: Duration) -> Self {
        Self {
            window,
            started: Instant::now(),
            steps: Series::new(window, STEPS_PER_SECOND),
            ticks: Series::new(window, TICKS_PER_SECOND),
            rng: 0x9E37_79B9_7F4A_7C15,
        }
    }

    /// The window from `--tick-stats-seconds` (0 or absent: off), refused outside 5..=600. The
    /// hub reads it before the socket binds and builds `TickStats` once the server is up.
    pub(super) fn window_from_args(
        mut args: impl Iterator<Item = String>,
    ) -> Result<Duration, Box<dyn std::error::Error>> {
        let mut seconds = 0;
        while let Some(arg) = args.next() {
            if arg == FLAG {
                seconds = args.next().ok_or("missing option value")?.parse()?;
            }
        }
        if seconds != 0 && !(5..=600).contains(&seconds) {
            return Err(format!("{FLAG} must be 0 (off) or 5..=600").into());
        }
        Ok(Duration::from_secs(seconds))
    }

    /// Record one step (and its tick, if one was due); print the window once it has elapsed.
    pub(super) fn record(&mut self, step: Instant, tick: Option<Instant>, sessions: usize) {
        if self.window.is_zero() {
            return;
        }
        let now = Instant::now();
        self.steps.record(now.duration_since(step), &mut self.rng);
        if let Some(tick) = tick {
            self.ticks.record(now.duration_since(tick), &mut self.rng);
        }
        if now.duration_since(self.started) < self.window {
            return;
        }
        self.started = now;
        let (steps, step_p50, step_p99, step_max) = self.steps.close();
        let (ticks, tick_p50, tick_p99, tick_max) = self.ticks.close();
        let stats = json!({
            "event":"tick_stats","window_s":self.window.as_secs_f64(),"sessions":sessions,
            "steps":steps,"step_p50_us":step_p50,"step_p99_us":step_p99,"step_max_us":step_max,
            "ticks":ticks,"tick_p50_us":tick_p50,"tick_p99_us":tick_p99,"tick_max_us":tick_max,
        });
        #[cfg(test)]
        EMITTED.with(|emitted| emitted.borrow_mut().push(stats.clone()));
        super::event(stats);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn series(window_s: u64) -> Series {
        Series::new(Duration::from_secs(window_s), STEPS_PER_SECOND)
    }

    #[test]
    fn percentiles_are_nearest_rank_and_p99_is_not_the_max() {
        let mut rng = 1;
        let mut steps = series(5);
        // 1..=100 µs in a scrambled order: p50 = 50, p99 = 99, max = 100.
        for micros in (1..=100u64).map(|n| n * 37 % 101) {
            steps.record(Duration::from_micros(micros), &mut rng);
        }
        assert_eq!(steps.close(), (100, 50, 99, 100));
        assert_eq!(steps.close(), (0, 0, 0, 0), "a closed window starts empty");
    }

    #[test]
    fn a_window_keeps_a_bounded_sample_with_exact_count_and_max() {
        assert_eq!(series(5).capacity, 1_000, "5 s at 200 steps a second");
        assert_eq!(series(600).capacity, HARD_CAP as usize, "capped");
        let mut rng = 1;
        let mut steps = series(5);
        let allocated = steps.samples.capacity();
        for n in 0..50_000u64 {
            steps.record(Duration::from_micros(n % 1_000 + 1), &mut rng);
        }
        steps.record(Duration::from_millis(30), &mut rng);
        assert_eq!(steps.samples.len(), 1_000);
        assert_eq!(
            steps.samples.capacity(),
            allocated,
            "no growth past the window's buffer"
        );
        let (count, p50, p99, max) = steps.close();
        assert_eq!((count, max), (50_001, 30_000));
        // The kept subset is uniform over 1..=1000 µs, so its median sits near 500.
        assert!((400..=600).contains(&p50), "p50 {p50}");
        assert!((950..=1_000).contains(&p99), "p99 {p99}");
    }

    #[test]
    fn a_zero_window_is_off_and_the_flag_is_bounded() {
        let args = |list: &[&str]| list.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let window = |list: &[&str]| TickStats::window_from_args(args(list).into_iter());
        let mut off = TickStats::new(window(&["127.0.0.1:0"]).unwrap());
        assert_eq!((off.steps.capacity, off.ticks.capacity), (0, 0));
        let past = Instant::now() - Duration::from_secs(1);
        EMITTED.with(|e| e.borrow_mut().clear());
        off.record(past, Some(past), 3);
        assert_eq!(off.steps.seen, 0);
        assert!(
            EMITTED.with(|e| e.borrow().is_empty()),
            "off prints nothing"
        );
        for refused in ["4", "601", "x"] {
            assert!(window(&[FLAG, refused]).is_err());
        }
        let on = TickStats::new(window(&[FLAG, "5"]).unwrap());
        assert_eq!(
            (on.window, on.ticks.capacity),
            (Duration::from_secs(5), 100)
        );
    }

    /// Through the real `Server::step`: a window prints exactly one event when it runs out,
    /// carrying the session count. The window's start is moved back instead of waiting, so a
    /// loaded machine cannot stretch it.
    #[test]
    fn a_window_emits_one_event_with_the_session_count() {
        let (mut server, _keys, scratch) =
            super::super::t020_group_routing::server_rig("tick-stats");
        let window = Duration::from_secs(5);
        server.tick_stats = TickStats::new(window);
        EMITTED.with(|e| e.borrow_mut().clear());
        let step = |server: &mut super::super::Server| {
            server.next_tick_ms = 0;
            server.step().unwrap();
        };
        for _ in 0..3 {
            step(&mut server);
        }
        assert!(
            EMITTED.with(|e| e.borrow().is_empty()),
            "the window is still open"
        );
        server.tick_stats.started -= window;
        step(&mut server);
        for _ in 0..3 {
            step(&mut server); // the next window: nothing more is printed
        }
        let emitted = EMITTED.with(|e| e.take());
        assert_eq!(emitted.len(), 1, "{emitted:?}");
        let stats = &emitted[0];
        assert_eq!(stats["event"], "tick_stats");
        assert_eq!(stats["sessions"], 3);
        assert_eq!(stats["steps"], 4, "the window's four steps");
        assert_eq!(
            stats["ticks"], stats["steps"],
            "a tick was due on every step"
        );
        let us = |key: &str| stats[key].as_u64().unwrap();
        assert!(us("step_p50_us") <= us("step_p99_us") && us("step_p99_us") <= us("step_max_us"));
        assert!(
            us("tick_max_us") <= us("step_max_us"),
            "a tick is part of its step"
        );
        let _ = std::fs::remove_dir_all(scratch);
    }
}
