# Class: ScreenDirectionIndicatorDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00995ce0` | `ScreenDirectionIndicatorDataSource::Slot1FieldNameForIndex` | ✅ | virtual | Virtual (slot [1]) taking one stack argument (undefined4 index). Switches on the index and returns a pointer to a static name string: 0 Text, 1 Subtext, 2 IconId, 3 ScreenAngle, 4 Distance, 5 Color, 6 CharacterGuid, 7 I… |
| `00996a80` | `ScreenDirectionIndicatorDataSource::Refresh` | 🟢 | method | Thiscall; ECX is the primary ScreenDirectionIndicatorDataSource object (00996ea0 and 00996ec0 call it with ECX-0x30, per bytes), so the offsets below are primary-object offsets. Calls vtable slot +0xc of the object at t… |
| `00996df0` | `ScreenDirectionIndicatorDataSource::DtorBody` | ✅ | dtor | Fastcall (ECX=this, primary object). Stores the ScreenDirectionIndicatorDataSource vtable (0180ccb0) at [this], the 0180cc8c vtable at +0x30 and the 0180cc74 vtable at +0x34. Calls 009964b0 with ECX=this+0x280 (Array<in… |
| `00996ea0` | `— ⚠reject` | ⚪ | virtual | Virtual slot [2] of the 0180cc8c vtable (secondary interface of ScreenDirectionIndicatorDataSource), thiscall with ECX passed through, no stack args. If the byte at this+0x290 is zero, calls 00996a80 with ECX=this-0x30 … |
| `00996ec0` | `— ⚠reject` | ⚪ | virtual | Virtual slot [4] of the 0180cc8c secondary interface (ECX = this, the +0x30 subobject), two stack args (param_2 switch value, param_3 output). If the byte at this+0x290 is zero, calls 00996a80 with ECX=this-0x30. Calls … |
| `00997030` | `ScreenDirectionIndicatorDataSource::Ctor` | ✅ | ctor | Thiscall (ECX=this, primary object) with one stack arg param_2, returns this in EAX (ret 4). Calls 00922890 with ECX=this and the string pointer to "BaseClient.ScreenDirectionIndicatorData". Stores the ScreenDirectionIn… |
| `009970f0` | `ScreenDirectionIndicatorDataSource::ScalarDeletingDtor` | ✅ | dtor | Thiscall (ECX=this) with one stack arg (byte flags). Calls 00996df0 (dtor body, ECX=this); if bit 0 of the flags arg is set, pushes this and calls the free thunk (00d82463 to _free). Returns this with ret 4. |
