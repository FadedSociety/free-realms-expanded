# Class: Factory

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bfecb0` | `Factory::RequestEquipTool ? ⚠rename` | 🟡 | callback | Checks for the required Lua argument, extracts its value, obtains the factory context, and invokes 009e8cd0 to construct/send the tool equip request. |
| `00bfeea0` | `Factory::RequestCreateBlueprint ? ⚠downgrade` | 🟡 | callback | For an even Lua argument count above one, builds a temporary ItemRequestEntry list from argument pairs and passes it to the blueprint request helper. |
