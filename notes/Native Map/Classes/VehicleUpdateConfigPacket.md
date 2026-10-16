# Class: VehicleUpdateConfigPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009142f0` | `— ⚠reject` | ⚪ | ctor | __fastcall (this=ECX). Under an SEH frame: stores zero at [this]; calls FUN_008d3900 (ArrayItemGuid16 constructor, label accepted:confirmed) with an argument not recovered by Ghidra; calls FUN_0090e940() (VehicleEquipme… |
| `00915490` | `— ⚠reject` | ⚪ | ctor | __fastcall (this=ECX, returns this). Calls FUN_008a4c40(1) (base packet constructor BaseVehicleLoadoutPacket::BaseVehicleLoadoutPacket, label accepted:confirmed), then FUN_009142f0; installs VehicleUpdateConfigPacket::v… |
| `009154f0` | `ScalarDeletingDtor → VehicleUpdateConfigPacketScalarDeletingDtor` | ✅ | dtor | Virtual at vftable[0] of 0180504c (thiscall, one byte argument). Calls FUN_0090bca0 (destructor body; label accepted:confirmed BasePacketSubobjectCleanup) with ECX=this; if (arg & 1) != 0 calls _free(this); returns this. |
