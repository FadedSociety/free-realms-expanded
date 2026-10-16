# Class: SoeUtil::HashListMap<int,FactoryPacketListToolsResponse::FactoryToolDefinitionNode,256,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009e3520` | `ToolDefinitionMapDtor` | ✅ | dtor | Sets map vtable and destroys head nodes until the 256-bucket map is empty. |
| `009e3640` | `ToolDefinitionMapScalarDeletingDtor` | ✅ | dtor | Destroys map nodes and frees this when deleting flag is set. |
| `009e4140` | `ToolDefinitionMapDtor` | ✅ | dtor | Installs map vtable and destroys head nodes until empty. |
