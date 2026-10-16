# Class: ClientProgressiveQuestData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bddcb0` | `ClientProgressiveQuestData_Init → ClientProgressiveQuestData_ArrayMembersInit` | 🟢 | method | Initialises the two embedded arrays of the progressive-quest data record: installs SoeUtil::Array<ClientProgressiveQuestSlot,0,1>::vftable and zeroes the fields at +0x34, +0x38 and +0x3c, then installs SoeUtil::Array<Cl… |
| `00bddd20` | `ClientProgressiveQuestData::~ClientProgressiveQuestData` | 🟢 | dtor | Destructor body of the progressive-quest data record: clears the prize array size at +0x48, installs its vtable and frees its buffer at +0x44 with alignment 4, then does the same for the slot array (size +0x38, buffer +… |
| `00bdddb0` | `ClientProgressiveQuestData::ScalarDeletingDtor` | 🟢 | virtual | Scalar deleting destructor of the progressive-quest data record: calls the destructor body 00bddd20 and then frees the object through the 00d82463 _free thunk when the low bit of the flag argument is set. |
| `00bde1f0` | `ClientProgressiveQuestData_Read` | 🟢 | method | Reads the client-data record: eight u32 values into +0..+0x1c, then calls 00bddfa0 for the ClientProgressiveQuestSlot array member at +0x30 and 00bde020 for the ClientProgressiveQuestPrizeSlot array member at +0x40, the… |
