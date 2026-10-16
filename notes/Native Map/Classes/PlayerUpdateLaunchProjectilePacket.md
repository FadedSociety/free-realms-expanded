# Class: PlayerUpdateLaunchProjectilePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008d0b10` | `PlayerUpdateLaunchProjectilePacket::PlayerUpdateLaunchProjectilePacket` | ✅ | ctor | Constructor of the LaunchProjectile player-update packet: calls the family constructor FUN_0089fbb0 with sub-opcode 0x3e (62), installs PlayerUpdateLaunchProjectilePacket::vftable, constructs the embedded ProjectilePara… |
| `008d0b80` | `PlayerUpdateLaunchProjectilePacket::~PlayerUpdateLaunchProjectilePacket` | 🟢 | dtor | Destructor body of the LaunchProjectile player-update packet: runs the accepted ProjectileParameters destructor FUN_008d0240 on the embedded payload record, then rewrites BasePlayerUpdatePacket::vftable and BasePacket::… |
| `008dccb0` | `PlayerUpdateLaunchProjectilePacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: runs the destructor body 008d0b80 and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
