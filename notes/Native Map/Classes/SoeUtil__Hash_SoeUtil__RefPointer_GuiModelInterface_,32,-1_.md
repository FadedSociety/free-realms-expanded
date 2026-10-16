# Class: SoeUtil::Hash<SoeUtil::RefPointer<GuiModelInterface>,32,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a88780` | `HashVSlot1Alloc12 ?` | 🟡 | virtual | Virtual slot 1 (+4) of Hash<RefPointer<GuiModelInterface>,32,-1> (also slot 1 of HashMap<int,RefPointer<GuiModelInterface>,32,-1>): calls 006fb9c0, then slot +4 of the object from 006fb860 when non-null, else slot +4 of… |
| `00a887c0` | `HashGuiModelVSlot2Free4` | 🟢 | virtual | Virtual slot 2 (+8) of Hash<RefPointer<GuiModelInterface>,32,-1>: pushes alignment 4 and its pointer argument, calls 00401c00 (free through the memory manager), and returns with ret 4. |
| `00a8cb70` | `HashGuiModelCtor` | ✅ | ctor | Constructs an empty 32-bucket hash map of GuiModelInterface references: stores the HashMap<int,RefPointer<GuiModelInterface>,32,-1> vptr, zeroes the count at +8, clears 0x80 bytes of bucket heads at +0xc through _memset… |
| `00a8cc90` | `HashGuiModelScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor shared by the Hash<RefPointer<GuiModelInterface>,32,-1> and HashMap<int,RefPointer<GuiModelInterface>,32,-1> vtables: stores the HashMap vptr at *this, calls 00a8aca0 (HashListClear32Refs), th… |
