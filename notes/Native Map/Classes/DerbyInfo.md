# Class: DerbyInfo

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009ee630` | `— ⚠reject` | ⚪ | dtor | Destructor of the 0xb0-byte DerbyInfo node: stores the IString vtable at +0x88, and when the length dword at +0x94 is positive atomically decrements the refcount in front of the buffer pointer at +0x8c (freeing through … |
| `009eef70` | `DerbyInfo_DtorWrapper` | 🟢 | function | SEH-framed wrapper preserves incoming ECX as the receiver for destructor body 009ee630, restores the prior exception chain, and returns; the stack-cookie push is frame bookkeeping, not a destructor argument. |
| `009ef2a0` | `DerbyInfo_ctor` | 🟢 | ctor | Constructor of the DerbyInfo record: runs the base constructor FUN_0070bac0 (zeroing the collection triple at +0x7c/+0x80/+0x84) and initialises the embedded IString at +0x88 with the IString vtable, the static empty bu… |
| `009ef840` | `DerbyInfo_scalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor for a DerbyInfo node: calls the destructor body 009ee630 and frees this through _free when bit 0 of param_2 is set; returns this otherwise. |
| `009effd0` | `DerbyInfo_ctorWrapper` | 🟢 | ctor | SEH-framed constructor wrapper: calls 009ef2a0 (the DerbyInfo initialiser, base constructor plus the IString member at +0x88) and returns param_1. |
