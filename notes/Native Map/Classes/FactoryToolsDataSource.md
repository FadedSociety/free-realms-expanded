# Class: FactoryToolsDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a72210` | `FactoryToolsDataSource::ColumnName` | ✅ | virtual | Virtual slot [1]. Switch on the int argument: 0 returns ToolId, 1 Name, 2 IconId, 3 IsCurrentlyUseable, 4 ChargesRemaining, 5 UsesCharges, 6 TooltipStringId, any other value returns <error>. Pure lookup; no state is rea… |
| `00a722b0` | `FactoryToolsDataSource::LazyInitThenGetField23c ?` | 🟡 | virtual | Virtual slot [2]. If the byte at this+0x228 is zero it calls vtable slot +0xc of the sub-object at this+4 (ECX = this+4) and sets the byte at +0x228 to 1. Returns the dword at this+0x23c. |
| `00a722e0` | `FactoryToolsDataSource::GetCell` | ✅ | virtual | Virtual slot [4] (row, column, out). Sets the byte at this+0x228 to 1 when it is zero. If row < count at +0x23c it walks the tool list from +0x234 (next pointer at node+0x4c) to that row and, if the node exists, switche… |
| `00a72620` | `FactoryToolsDataSource::DtorBody` | ✅ | dtor | Stores the FactoryToolsDataSource vtables at this+0, +0x30 and +0x34, calls 009e4140 (accepted label ToolDefinitionMapDtor) on the tool-definition map, then calls 0091bf60 (accepted label Dtor_GuiUserDataTableAt34) with… |
| `00a726b0` | `FactoryToolsDataSource::Ctor` | ✅ | ctor | Calls the base constructor 00922890 with the name string BaseClient.Factory.Tools, stores its stack argument at this+0x254, installs the FactoryToolsDataSource vtables at +0, +0x30 and +0x34, clears the byte at +0x258, … |
| `00a72740` | `FactoryToolsDataSource::ScalarDeletingDtor` | ✅ | dtor | Virtual slot [0]. Calls the dtor body 00a72620 on this, then calls _free on this when bit 0 of its stack argument is set; returns this (ret 4). |
