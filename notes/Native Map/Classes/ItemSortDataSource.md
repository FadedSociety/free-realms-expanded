# Class: ItemSortDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bb1dd0` | `ItemSortDataSource::GetStaticTableByIndex ⚠downgrade` | 🟢 | method | Bare selector over the index argument (decoded bytes: mov eax,[esp+4]; sub eax,0; je; sub eax,1; je; two mov eax,imm32; ret 4): returns the static table at 0x0180c470 for index 0, the table at 0x017ea850 for index 1 and… |
| `00bb1e50` | `ItemSortDataSource::GetItemCount` | 🟢 | method | Bare field getter (decoded bytes: mov eax,[ecx+0x224]; mov eax,[eax+0x30]; ret): returns the dword at model+0x30 where model is the object stored at this+0x224. |
| `00bb1e80` | `ItemSortDataSource::RefreshList` | 🟢 | method | Calls vtable slot 3 of the model object at this+0x34 (reset), then walks the singly-linked list whose head is at [this+0x254]+0x28 and whose next pointer is at node+0xc, calling vtable slot 1 of the model once per node … |
| `00bb1ec0` | `— ⚠reject` | ⚪ | virtual | Vtable slot 4 override: returns 0 unless the index argument is below the count *(*(this+0x224)+0x30); it then resolves the entry through vtable slot 4 of [this+4] and dispatches on the index: for index 0 it calls FUN_00… |
| `00bb1f50` | `— ⚠reject` | ⚪ | dtor | Destructor body: rewrites ItemSortDataSource::vftable into this+0, this+0x30 and this+0x34 (the three sub-object vtable slots) and then calls the base destructor 0091bf60 inside an SEH frame; nothing else is released. |
| `00bb1fd0` | `— ⚠reject` | ⚪ | ctor | Constructor: registers the data-source type through 00922890 with the name looked up at PTR_s_BaseClient.ItemSortTypes, stores the argument at this+0x254, installs ItemSortDataSource::vftable into this+0, this+0x30 and … |
| `00bb2050` | `ItemSortDataSource::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor at vtable slot 0 (decoded: push esi; mov esi,ecx; call 00bb1f50; test byte [esp+8],1; je; push esi; call 00d82463; ret): runs the destructor body 00bb1f50 and, when bit 0 of the flag argument … |
