//! Which addresses the servers may listen on and accept packets from
//! (Security Model L1). Loopback is always fine; `--allow-lan` additionally
//! allows private networks and the VPN range, never wildcard or public.

use std::net::IpAddr;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_always_lan_only_when_allowed_never_public_or_wildcard() {
        let ip = |s: &str| s.parse::<IpAddr>().unwrap();
        for lan in ["192.168.1.20", "10.0.0.5", "172.16.3.4", "169.254.1.1", "100.101.2.3", "fd12::1"] {
            assert!(allowed(ip(lan), true), "{lan}");
            assert!(!allowed(ip(lan), false), "{lan} without --allow-lan");
        }
        for bad in ["0.0.0.0", "8.8.8.8", "100.128.0.1", "::", "2001:db8::1"] {
            assert!(!allowed(ip(bad), true), "{bad}");
        }
        assert!(allowed(ip("127.0.0.1"), false));
        assert!(allowed(ip("::1"), false));
    }
}
