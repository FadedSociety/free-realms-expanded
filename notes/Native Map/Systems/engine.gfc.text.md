# System: engine.gfc.text

53 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (16)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0048f307` | `TextAccessClassDtor` |  | destruction path of the GInterface-derived text-access class whose storage interface lives at +0x48. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048f479` | `TextAccessCopyCtor` |  | copy-construction path of the text-access class. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048f554` | `TextAccessCtorFrom` |  | construction path of the text-access class from an existing storage plus extra initialisation arguments. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048f5ea` | `GFormattedStringAccess::GFormattedStringAccess` | GFormattedStringAccess | construction of the formatted-string accessor of the framework string API. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048f601` | `GFormattedStringAccess::InvokeSlotEC` | GFormattedStringAccess | Virtual slot 57 shared with GFormattedString; dispatches through receiver vtable entry +0xec. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048f612` | `GFormattedStringAccess::Wrapper6` | GFormattedStringAccess | Virtual slot 6 shared with GFormattedString; delegated helper's exact format contract is not established here. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048f62a` | `GFormattedStringAccess::Wrapper7` | GFormattedStringAccess | Virtual slot 7 shared with GFormattedString; delegated helper's exact format contract is not established here. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048f642` | `GFormattedStringAccess::Wrapper8` | GFormattedStringAccess | Virtual slot 8 shared with GFormattedString; delegated helper's exact format contract is not established here. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048f67f` | `GFormattedStringAccess::AdjustorThunkMinus4To0048f642` | GFormattedStringAccess | Virtual slot 8 shared with GFormattedString; rebases this by -4 for the shared implementation. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048f687` | `GFormattedStringAccess::AdjustorThunkMinus4To0048f612` | GFormattedStringAccess | Virtual slot 6 shared with GFormattedString; rebases this by -4 for the shared implementation. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048f68f` | `GFormattedStringAccess::AdjustorThunkMinus4To0048f717` | GFormattedStringAccess | Virtual slot 0 of GFormattedStringAccess; rebases this by -4 to the scalar deleting destructor body. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048f697` | `GFormattedStringAccess::AdjustorThunkMinus4To00482308` | GFormattedStringAccess | Virtual slot 1 of GFormattedStringAccess; target role is outside this shard. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048f69f` | `GFormattedStringAccess::AdjustorThunkMinus4To0048f62a` | GFormattedStringAccess | Virtual slot 7 shared with GFormattedString; rebases this by -4 for the shared implementation. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048f6a7` | `GFormattedStringAccess::AttachRangeChecked` | GFormattedStringAccess | Virtual slot 60 of GFormattedStringAccess; the holder and range semantics beyond these values are not named here. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048f717` | `GFormattedStringAccess::ScalarDeletingDtor` | GFormattedStringAccess | Deleting-destructor body for GFormattedStringAccess; it resets the base subobject's vtables to GInterface before optional freeing. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0049162f` | `GFormattedString::AttachRangeFromBuffer` | GFormattedString | Virtual slot 60 of GFormattedString; buffer/receiver layout beyond these explicit arguments is not established. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |

## 🟢 likely (31)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0048ef78` | `TextStorageAssignA` |  | range-assignment path of the text-storage object (bind a storage and offset, drop previous accessors). | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048efc7` | `TextStorageAssignB` |  | assignment path of the text-storage object that also carries a 16-bit tag and a helper argument. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048f01f` | `TextStorageSetPrimaryAccessor` |  | Text-storage assignment path installing the primary interface; code order acquires incoming interface before releasing the old +0x1c interface. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048f07f` | `TextStorageSetSecondaryAccessor` |  | Second text-storage assignment variant; order is release +0x1c, acquire incoming pointer, then release and replace +0x20. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048f0df` | `TextStorageReset` |  | clearing path used by the assignment routines to drop the current text state. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048f120` | `TextStorageGetRange` |  | Mechanical two-dword query wrapper; return polarity is stated without assigning success/failure semantics. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048f1a1` | `TextStorageCopyCtor` |  | copy-construction path of the text-storage class. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048f20f` | `TextStorageAssignFrom` |  | assignment path of the text-storage object used when the state is copied from another instance. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048f2df` | `ReleaseTwoInterfacesAndTailJump` |  | Destructor/cleanup step: four EH unwind funclets call it, and wrapper 0048f768 follows it with another cleanup tail jump. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048f3e2` | `TextAccessScalarDeletingDtor` |  | deleting-destructor emission of the GInterface-derived text-access class. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048f65e` | `CheckedIndexReturn` |  | checked element-index accessor (assert-then-return) on the type whose fields are a pointer at +0 and a length at +8; the owner is not identified inside this function. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048f768` | `TextStorageDtor` |  | Destructor wrapper: four EH unwind funclets call it; scalar deleting destructor emissions 0048fd5a and 00491554 also call it. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048f888` | `TextAccessScalarDeletingDtor_2` |  | deleting-destructor emission of the text-access class (second translation-unit copy). | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048f945` | `AddReturnedVectorToStoredOffsets` |  | Generic two-coordinate composition over pointer and offset fields; calling these values an origin or anchor is interpretation, not a name established here. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048f9d1` | `HeldObjectCtor` |  | constructor of a small holder object that acquires the interface it is given and stores it at +0x14 (callers 0049f2c7 and 004a7b19). | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048fa6d` | `HeldObjectDtor` |  | destruction path of the holder object that owns the +0x14 interface pointer. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048fa86` | `HeldObjectCtor_2` |  | constructor of the second two-vtable holder variant (same acquire/store shape as FUN_0048F9D1). | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048fbf6` | `HeldObjectDtor_2` |  | destruction path of the second two-vtable holder variant. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048fc0f` | `TextAccessPairCtor` |  | constructor of the object that carries two embedded accessors; the link to the text-access class is the shared empty-string buffer DAT_01bc7e20, so the text reading stays hedged. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048fc7d` | `CopySelectedRecordFields0x7C` |  | Mechanical selected-field copy; record owner and field meanings are not established inside the function. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048fd5a` | `TextStorageScalarDeletingDtor` |  | deleting-destructor emission of the text-storage object. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048fdfa` | `IndexInRangeWithPageCount` |  | cheap bounds test that adds a page/block count obtained from the +0x1C sub-object; the owner and the meaning of the count are not established inside this function. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048feb9` | `StateObjectCopyCtor` |  | copy-construction path of the state object whose default constructor sets the +0x24/+0x28 sentinels and whose deleting destructor is FUN_0048FF1C (caller 0049fa47). | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048ff1c` | `StateObjectDeletingDtor` |  | deleting-destructor emission of the state object whose copy constructor is FUN_0048FEB9. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048ff38` | `StateObjectCtor` |  | default construction of the state object that FUN_0048FEB9 copies and FUN_0048FF1C deletes. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048ffba` | `ReplaceElement5Interface` |  | Mechanical replacement of an interface pointer on element 5 followed by re-registration; raw bytes identify the stack argument and refcount order. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `00490012` | `InitStateBlock60` |  | state initialiser with 24 recorded callers across the text/UI code; the block owner is not identified inside this function. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `00491393` | `HeldObjectCtorFrom` |  | construction path that copies the held object from another holder instance (callers 0049fe7c and 0049ff0b). | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `004913c7` | `TextAccessPairCopyCtor` |  | copy-construction path of the object built by FUN_0048FC0F (callers 00492f75, 0049ff0b, 004b88b8); the accessor reading rests on the shared empty-string buffer. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `00491554` | `TextStorageScalarDeletingDtor_2` |  | deleting-destructor emission of the text-storage object (second translation-unit copy). | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `004915cb` | `AttachBuiltTextStorage` |  | storage-attachment path used when the text storage must be constructed from extra arguments first. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |

## 🟡 speculative (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00491570` | `AttachTextStorage ? ⚠downgrade` |  | storage-attachment path of the text-access class; twelve UI/game callers use it to bind text storage to an object. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `006373db` | `GuiEditCtrl::LayoutFormattedText ?` | GuiEditCtrl | Vtable slot [177] of GuiEditCtrl (GuiPropEditCtrl has the same slot). Builds a local GFormattedString-vtabled object, using the object at this+0x158 when set and a local default otherwise, and attaches a built text stor… | [[Native Map/Shards/S0807 0062745d-006373db|S0807]] |
| `00d02690` | `WideTextWrapLayout ?` |  | Breaks wide text into width-limited lines; its only caller is the chat-bubble element layout 00a477d0, so the domain is that caller's, not proven here. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |

## ⚪ unknown (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0048ef55` | `— ⚠reject` |  | shared base initialiser of the text-storage class used by the UI/text callers that construct it. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048f274` | `— ⚠reject` |  | initialisation path that binds the text storage to a caller-supplied buffer and optional interface. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `00491678` | `— ⚠reject` |  | thin forwarding helper over the +0x54 virtual of the +0x14 field object; that implementation lives outside this shard. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |

