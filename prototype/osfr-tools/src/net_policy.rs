//! Which addresses the servers may listen on and accept packets from
//! (Security Model L1). Loopback is always fine; `--allow-lan` additionally
//! allows private networks and the VPN range, never wildcard or public.

use std::collections::BTreeMap;
use std::net::IpAddr;
use std::time::Instant;

/// A private-network address: RFC 1918, link-local, IPv4 shared space
/// 100.64.0.0/10 (Tailscale and other VPNs), or IPv6 unique-local.
pub fn is_private_network(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let [a, b, ..] = v4.octets();
            v4.is_private() || v4.is_link_local() || (a == 100 && (64..128).contains(&b))
        }
        IpAddr::V6(v6) => (v6.segments()[0] & 0xfe00) == 0xfc00,
    }
}

/// May a server bind or accept traffic from `ip`?
pub fn allowed(ip: IpAddr, allow_lan: bool) -> bool {
    ip.is_loopback() || (allow_lan && is_private_network(ip))
}

/// Token bucket on a monotonic clock (Security Model L6).
#[derive(Clone, Copy, Debug)]
pub struct Bucket {
    tokens: f64,
    last: Instant,
}

impl Bucket {
    pub fn full(burst: f64, now: Instant) -> Self {
        Self {
            tokens: burst,
            last: now,
        }
    }

    /// Spend `cost` tokens refilled at `rate` per second up to `burst`;
    /// false means over budget (nothing is spent).
    pub fn take(&mut self, cost: f64, rate: f64, burst: f64, now: Instant) -> bool {
        let elapsed = now.saturating_duration_since(self.last).as_secs_f64();
        self.tokens = (self.tokens + elapsed * rate).min(burst);
        self.last = now;
        if self.tokens < cost {
            return false;
        }
        self.tokens -= cost;
        true
    }
}

/// One bucket per key, with a bounded key set so a spray of keys cannot grow
/// memory: when full, idle (refilled) keys are evicted first and a new key is
/// refused if none are idle.
#[derive(Debug)]
pub struct KeyedBuckets<K: Ord> {
    buckets: BTreeMap<K, Bucket>,
    rate: f64,
    burst: f64,
    max_keys: usize,
}

impl<K: Ord> KeyedBuckets<K> {
    pub fn new(rate: f64, burst: f64, max_keys: usize) -> Self {
        Self {
            buckets: BTreeMap::new(),
            rate,
            burst,
            max_keys,
        }
    }

    pub fn allow(&mut self, key: K, now: Instant) -> bool {
        let (rate, burst) = (self.rate, self.burst);
        if !self.buckets.contains_key(&key) && self.buckets.len() >= self.max_keys {
            // A bucket that could pay a whole burst is idle.
            // ponytail: at most `max_keys` probes on a new key at capacity; no LRU/cache dependency.
            self.buckets.retain(|_, b| {
                let mut probe = *b;
                !probe.take(burst, rate, burst, now)
            });
            if self.buckets.len() >= self.max_keys {
                return false;
            }
        }
        self.buckets
            .entry(key)
            .or_insert_with(|| Bucket::full(burst, now))
            .take(1.0, rate, burst, now)
    }

    pub fn len(&self) -> usize {
        self.buckets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buckets.is_empty()
    }
}

/// One bucket per source IP.
pub type IpLimiter = KeyedBuckets<IpAddr>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_always_lan_only_when_allowed_never_public_or_wildcard() {
        let ip = |s: &str| s.parse::<IpAddr>().unwrap();
        for lan in [
            "192.168.1.20",
            "10.0.0.5",
            "172.16.3.4",
            "169.254.1.1",
            "100.101.2.3",
            "fd12::1",
        ] {
            assert!(allowed(ip(lan), true), "{lan}");
            assert!(!allowed(ip(lan), false), "{lan} without --allow-lan");
        }
        for bad in ["0.0.0.0", "8.8.8.8", "100.128.0.1", "::", "2001:db8::1"] {
            assert!(!allowed(ip(bad), true), "{bad}");
        }
        assert!(allowed(ip("127.0.0.1"), false));
        assert!(allowed(ip("::1"), false));
    }

    #[test]
    fn buckets_allow_a_burst_then_refill_at_their_rate() {
        let start = Instant::now();
        let mut bucket = Bucket::full(10.0, start);
        assert!((0..10).all(|_| bucket.take(1.0, 5.0, 10.0, start)));
        assert!(!bucket.take(1.0, 5.0, 10.0, start));
        let later = start + std::time::Duration::from_millis(400); // 2 tokens
        assert!(bucket.take(1.0, 5.0, 10.0, later) && bucket.take(1.0, 5.0, 10.0, later));
        assert!(!bucket.take(1.0, 5.0, 10.0, later));
    }

    #[test]
    fn ip_limiter_keeps_its_key_set_bounded() {
        let now = Instant::now();
        let ip = |n: u8| IpAddr::from([10, 0, 0, n]);
        let mut limiter = IpLimiter::new(5.0, 10.0, 2);
        assert!(limiter.allow(ip(1), now) && limiter.allow(ip(2), now));
        assert!(!limiter.allow(ip(3), now)); // both keys busy: refused, not grown
        let idle = now + std::time::Duration::from_secs(3);
        assert!(limiter.allow(ip(3), idle)); // idle keys evicted
        assert!(limiter.len() <= 2);
    }
}
