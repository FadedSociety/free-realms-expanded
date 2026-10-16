# Class: SoeUtil::List<InteractionData,10>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a375b0` | `ListInteractionData10Ctor ⚠downgrade` | 🟢 | ctor | Ctor: stores SoeUtil::List<InteractionData,-1>::vftable, zeroes this+4, this+8 and this+0xc, then stores SoeUtil::List<InteractionData,10>::vftable into *this and calls 00a371d0 (pool init of ten 0x28-byte nodes). Retur… |
| `00a37fe0` | `— ⚠reject` | ⚪ | dtor | Called from EH unwind funclets. Installs SoeUtil::List<InteractionData,10>::vftable, clears the list through 00a37480, then if this+0x10 is non-null frees its binary tree (two child subtrees via 00a37440 with alignment … |
| `00a380b0` | `ListInteractionData10ScalarDtor` | ✅ | dtor | Virtual slot [0] of SoeUtil::List<InteractionData,10>::vftable (binary vtable read). Body shape of a scalar deleting destructor: calls 00a37fe0 (dtor body), then tests bit 0 of param_2 and when set calls _free(this) (00… |
