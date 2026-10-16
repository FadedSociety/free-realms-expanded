# Class: AssetManagement::AssetRequestInternal::AssetReadyHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00927e10` | `AssetReadyHandler::Ctor` | ✅ | ctor | Installs its vtable, clears a field, initializes a deliverable-asset list, and calls helper initialization. |
| `01098260` | `— ⚠reject` | ⚪ | virtual | Thiscall with one stack arg (record pointer), ret 4, returns 1 in EAX. Builds a copy of the receiver's name string (receiver+8) through FUN_0071c050 and calls FUN_01097590 (AllocCtorAppendTail_Assign) with the argument … |
| `01098310` | `— ⚠reject` | ⚪ | virtual | Thiscall with one stack arg (record pointer), ret 4, no return value. Same body as 01098260: builds a copy of the receiver's name string (receiver+8) through FUN_0071c050 and calls FUN_01097590 (AllocCtorAppendTail_Assi… |
| `010988d0` | `AssetReadyHandler::DtorBody` | ✅ | dtor | Fastcall (ECX=this), no stack args, ret 0. Stores the AssetReadyHandler vtable at this, runs the DeliverableAsset list destructor FUN_0091fd60 and then the ready-handler hash destructor FUN_0091e8a0 (both labelled confi… |
