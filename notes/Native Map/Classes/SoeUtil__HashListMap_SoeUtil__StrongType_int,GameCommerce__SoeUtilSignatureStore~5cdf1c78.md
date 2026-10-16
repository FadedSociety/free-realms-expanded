# Class: SoeUtil::HashListMap<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleId,1>,AppStoreBundleDefinition,2048,512>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c39080` | `AppStoreBundleMapCtor` | ✅ | ctor | Initializes an empty 2048-bucket bundle map, installs the fixed-capacity derived vtable, and initializes its 512-entry node pool. |
| `00c39f50` | `SoeUtil::HashListMap::~HashListMap` | ✅ | dtor | Destructor of the 2048-bucket AppStoreBundleDefinition map: stores its vtable (marked as the deleting variant), frees every node of the entry chain at this+8 through 00b289d0 until the head is null, then releases the po… |
| `00c3a030` | `PoolAllocSlotThunk` | ✅ | virtual | Adjustor thunk in vtable slot 1: bytes 81 c1 14 20 00 00 ('add ecx,0x2014') then e9 ('jmp 00c399d0'), so the map's virtual allocate-slot call is forwarded to the pool sub-object at +0x2014 (the 2048,512 pool family's al… |
| `00c3a040` | `PoolFreeSlotThunk` | ✅ | virtual | Adjustor thunk in vtable slot 2: adds 0x2014 to ECX and jumps to 00c39a40, the free-slot routine of the 2048,512 pool sub-object, so a virtual release of a slot reaches the pool implementation. |
| `00c3ab80` | `SoeUtil::HashListMap::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of the 2048,512 bundle map: runs the destructor 00c39f50 and, when bit 0 of the flag byte is set, releases the object with _free; returns the object pointer. |
