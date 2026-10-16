# System: ui.headshot

13 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00cc5a00` | `VisibleProxy::ForwardField10ToDelegate → VisibleProxy::TailCallMember10Process` | VisibleProxy | Shared VisibleProxy and HeadshotProxy implementation for vtable slot 14; it is an adjustor thunk, not the target function body. | [[Native Map/Shards/S0415 00cc1a00-00cc6bc0|S0415]] |
| `00cc5a70` | `VisibleProxy::VisibleProxy` | VisibleProxy | Initializes the visible-proxy object and clears its embedded state fields. | [[Native Map/Shards/S0415 00cc1a00-00cc6bc0|S0415]] |
| `00cc5cd0` | `VisibleProxy::ScalarDeletingDestructor` | VisibleProxy | Implements the VisibleProxy scalar deleting destructor entry. | [[Native Map/Shards/S0415 00cc1a00-00cc6bc0|S0415]] |
| `00cc5d00` | `HeadshotProxy::~HeadshotProxy` | HeadshotProxy | Destroys HeadshotProxy-owned state before running its VisibleProxy base cleanup. | [[Native Map/Shards/S0415 00cc1a00-00cc6bc0|S0415]] |
| `00cc5e10` | `HeadshotProxy::ScalarDeletingDestructor` | HeadshotProxy | Implements the HeadshotProxy scalar deleting destructor entry. | [[Native Map/Shards/S0415 00cc1a00-00cc6bc0|S0415]] |
| `00cc5e30` | `HeadshotProxy::ReleaseAndClearField20` | HeadshotProxy | Releases and clears the stored reference through the shared cleanup helper. | [[Native Map/Shards/S0415 00cc1a00-00cc6bc0|S0415]] |
| `00cc5e50` | `HeadshotProxy::ClearField20` | HeadshotProxy | Clears the stored object field without calling the release helper. | [[Native Map/Shards/S0415 00cc1a00-00cc6bc0|S0415]] |
| `00cc5e60` | `HeadshotProxy::HeadshotProxy` | HeadshotProxy | Builds a headshot proxy from a character appearance record and its associated visual entries. | [[Native Map/Shards/S0415 00cc1a00-00cc6bc0|S0415]] |

## 🟢 likely (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00cc5a30` | `ForwardField20ThroughDelegate` |  | The recorded screenshot-manager callers use this forwarding wrapper on the headshot path; the function itself only delegates. | [[Native Map/Shards/S0415 00cc1a00-00cc6bc0|S0415]] |
| `00cc5d80` | `HeadshotProxy::DispatchWhenDelegateReady ⚠downgrade` | HeadshotProxy | Runs the headshot delegate's follow-up sequence only after both readiness checks pass. | [[Native Map/Shards/S0415 00cc1a00-00cc6bc0|S0415]] |
| `00cc5df0` | `ForwardFieldB8ThroughDelegate` |  | The recorded screenshot capture caller uses this wrapper; its own body only forwards a field to a delegate. | [[Native Map/Shards/S0415 00cc1a00-00cc6bc0|S0415]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00cc5cf0` | `HeadshotProxy::SetNestedReadyFlag ? ⚠rename` | HeadshotProxy | Sets a readiness flag in a nested HeadshotProxy object. | [[Native Map/Shards/S0415 00cc1a00-00cc6bc0|S0415]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00cc5c50` | `— ⚠reject` | VisibleProxy | Destroys the visible-proxy members and releases its stored object. | [[Native Map/Shards/S0415 00cc1a00-00cc6bc0|S0415]] |

