# Class: VehiclePartProcessor

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009d2530` | `VehiclePartProcessor::Ctor` | ✅ | ctor | Installs the VehiclePartProcessor vtable, stores the supplied owner, initializes a helper object, clears the cached resource and state fields, and sets its active byte. |
| `009d25a0` | `VehiclePartProcessor::GetOrCreateShipCombatAssetProcessor` | 🟢 | function | If this+0xe10 is null, allocates 0x4f8 bytes, reads the owner value at the object referenced by this+8 offset +0xe8, gets the global manager, calls FUN_00c534f0 with manager and owner values, and caches its returned Shi… |
| `009d2eb0` | `VehiclePartProcessor::Dtor` | ✅ | dtor | Restores the VehiclePartProcessor vtable, releases its cached resource at field 900 when present, then destroys its helper object. |
| `009d89c0` | `VehiclePartProcessor::ProcessVehiclePartMessage` | ✅ | virtual | Reads a tagged message from the caller buffer (param_2,param_3) with the byte-reader helpers FUN_009d4e20 (first tag into local_104c) and FUN_009d88b0 (payload). Tag 2 allocates 0x1b40 bytes with operator new (FUN_00d83… |
