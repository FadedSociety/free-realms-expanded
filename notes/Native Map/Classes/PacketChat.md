# Class: PacketChat

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b0f8f0` | `PacketChat::Ctor ⚠downgrade` | 🟢 | ctor | Initializes BaseChatPacket mode 1, constructs chat strings and fixed text storage, installs PacketChat's vtable, and zeros trailing fields. |
| `00b0f9b0` | `PacketChat::ScalarDeletingDtor` | ✅ | dtor | Calls BaseChatPacket cleanup and frees the object when the low flag bit is set. |
| `00b11600` | `DeserializePacketChatBody` | ✅ | method | Reads the two 16-bit header fields, bounded scalar and pair fields, a length-counted byte span, a float4, and trailing fields into the packet object; truncation writes zero defaults and marks the reader failed. |
| `00b14fa0` | `— ⚠reject` | ⚪ | method | Writes the PacketChat fields through the writer: delegates the first two 16-bit values, writes the field at +0xc, serializes members at +0x10, +0x18, +0x130, +0x1d0, +0x1e8 and +0x1f0, and writes +0x1e0 only when +0xc e… |
