# Class: ClientAreaData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b0ac90` | `ClientAreaData::Dtor` | ✅ | dtor | Restores ClientAreaData and IString base vtables, destroys child nodes, and decrements the shared string buffer reference count. |
| `00b0ad90` | `ClientAreaData::GetField_1b4` | ✅ | virtual | Returns the 32-bit value stored at object offset 0x1b4. |
| `00b0adc0` | `ClientAreaData::RemoveAndResetBackgroundAudio` | ✅ | virtual | Unlinks a node, looks up Sound(Background) and Music(Background), and resets each associated field when its lookup count is below one. |
| `00b0ae20` | `ClientAreaData::ScalarDeletingDtor` | ✅ | dtor | Calls ClientAreaData cleanup and frees the object when the deleting flag's low bit is set; otherwise returns the pointer. |
| `00be8e10` | `ClientAreaData::ForwardSlot24` | ✅ | virtual | Loads the object's vtable at [ecx] and tail-jumps to its slot +0x60 (vtable slot 24) with the same this; no arguments, no local state (bytes: mov eax,[ecx]; mov edx,[eax+0x60]; jmp edx). |
| `00be8e20` | `ClientAreaData::FlagSlot17Fallback` | 🟢 | virtual | If the byte at this+0x1a9 equals 1, calls vtable slot 17 (+0x44); when that returns zero it tail-jumps to 01076090 with this; otherwise returns without action (bytes: cmp byte [esi+0x1a9],1; call [eax+0x44]; test eax,ea… |
| `00bed0c0` | `ClientAreaData::Enable` | ✅ | virtual | Increments the active reference count, sets the enabled byte on the first reference, then notifies linked listeners or resolves and invokes the selected area target. |
| `00bed130` | `ClientAreaData::Disable` | ✅ | virtual | Decrements the active reference count, clears the enabled byte when it drops below one, and notifies linked listeners or the selected area target. |
| `00bed1a0` | `ClientAreaData::GetCenter` | ✅ | virtual | Initializes an output vector, copies a center returned by virtual slot +0x38, or computes the midpoint of the bounds returned by slot +0x3c. |
| `00bed250` | `ClientAreaData::SetCenter` | ✅ | virtual | Writes the supplied center into the direct shape center when available, otherwise translates both corners of the eight-float bounds by the center delta. |
| `00bed590` | `ClientAreaData::LoadXmlProperties` | ✅ | virtual | Reads parentAreaId and AreaEnabled attributes, then iterates Property elements and dispatches each to the selected property object's XML loader. |
| `00bed690` | `ClientAreaData::GetMinimumPropertyValue ⚠downgrade` | 🟢 | virtual | Returns the minimum virtual value at slot +0x2c among linked properties, or calls the selected area's virtual slot +0x6c. |
| `00bedeb0` | `ClientAreaData::Ctor` | ✅ | ctor | Initializes the base area object, clears property links and state fields, stores parameters, sets defaults and obtains an additional default value. |
| `00bee200` | `ClientAreaData::CreateProperty` | ✅ | virtual | Parses the property type name, allocates and constructs a matching property object, then links it into the area list and a hash bucket. |
