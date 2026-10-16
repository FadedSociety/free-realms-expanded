# Class: SoeUtil::List<PacketInGamePurchaseStoreBundleContentRequest::RequestData,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b247a0` | `SoeUtil::List_StoreBundleContentRequestDataAllocSlot` | ✅ | virtual | Calls FUN_006fb9c0, then FUN_006fb860. If the thread-local manager is non-null, invokes its vtable slot +4 and returns; otherwise, if DAT_01bdf280 is non-null, invokes that manager slot +4 and returns; otherwise calls F… |
| `00b247e0` | `SoeUtil::List_StoreBundleContentRequestDataFreeSlot` | ✅ | virtual | Virtual free slot (vtable index 3): calls FreeMemory(param_1, 4) on the node pointer. |
| `00b40340` | `— ⚠reject` | ⚪ | dtor | Destructor body of the store-bundle-content-request list: writes its vtable at +0 and drains the element list at +8 through FUN_00b3a8a0 on each head until the head is null. |
| `00b40af0` | `— ⚠reject` | ⚪ | dtor | Scalar deleting destructor (vtable slot 0): rewrites the vtable, drains the element list at +8 through FUN_00b3a8a0, and when bit 0 of the flag byte argument is set releases this with _free; otherwise returns this. |
