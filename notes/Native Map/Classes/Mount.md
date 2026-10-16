# Class: Mount

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bff6f0` | `— ⚠reject` | ⚪ | callback | For one Lua argument, checks mount state; if no mount is active it builds and conditionally sends dismount and spawn packets, records the selected mount id, logs it, and invokes the mount button callback. |
| `00bff8e0` | `Mount::Despawn ⚠downgrade` | 🟢 | callback | For one Lua argument, checks mount state; if active, logs despawn, builds and conditionally sends PacketDismountRequest, otherwise invokes the mount button callback. |
| `00bffa60` | `— ⚠reject` | ⚪ | callback | Checks mount state; when active, logs dismount, builds and conditionally sends PacketDismountRequest, otherwise refreshes the mount button callback. |
| `00bffbc0` | `Mount::Upgrade` | ✅ | callback | Requires two Lua arguments, builds a sub-opcode 14 mount packet, reads mount and item-definition ids, conditionally sends it, and logs both ids. |
