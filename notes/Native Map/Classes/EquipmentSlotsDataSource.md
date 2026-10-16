# Class: EquipmentSlotsDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a737b0` | `EquipmentSlotsDataSource::ColumnName` | ✅ | virtual | Virtual slot [1]. Switch on the int argument: 0 returns Required, 1 Members Only, 2 Min Profile Rank, 3 Is Equippable, 4 Is Required, 5 Slot Name, any other value returns <error>. Pure lookup; no state is read. |
| `00a73820` | `— ⚠reject` | ⚪ | virtual | Virtual slot [4] (key, column, out). Gets the global object pointer through 00769710 and its +4 sub-object, then the table owner from 00769fe0; returns 0 when that is null, otherwise 1. Looks up the node with the key in… |
| `00a739a0` | `EquipmentSlotsDataSource::DtorBody` | ✅ | dtor | Stores the EquipmentSlotsDataSource vtables at this, +0x30 and +0x34, then calls 0091bf60 (accepted label Dtor_GuiUserDataTableAt34) with ECX = this. Does not free this. |
| `00a73a10` | `EquipmentSlotsDataSource::Ctor` | ✅ | ctor | Calls the base constructor 00922890 with the name string BaseClient.EquipmentSlots, then installs the EquipmentSlotsDataSource vtables at +0, +0x30 and +0x34. Returns this. |
| `00a73a80` | `EquipmentSlotsDataSource::ScalarDeletingDtor` | ✅ | dtor | Virtual slot [0]. Calls the dtor body 00a739a0 on this, then calls _free on this when bit 0 of its stack argument is set; returns this (ret 4). |
