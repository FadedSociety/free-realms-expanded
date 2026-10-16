# Class: TileEventHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a8a80` | `TileEventHandler::MatchesValues_8_10_40_8` | 🟢 | virtual | Predicate on two integers: returns 0 only when param_2 is neither 8 nor 0x10 and param_1 is neither 0x40 nor 8; i.e. it returns 1 when param_2 is 8 or 0x10, or param_1 is 0x40 or 8. No calls. Installed at slot 2 of Tile… |
| `008b88e0` | `TileEventHandler::TileEventHandler` | ✅ | ctor | Calls TargetProcessorTileEventHandler construction, installs TileEventHandler::vftable, and stores param_2 at +0x0c. |
| `008c58c0` | `TileEventHandler::ScalarDeletingDtor` | ✅ | dtor | Stores TargetProcessorEventHandler::vftable at this+0. If flag bit 0 is set, calls _free(this); otherwise returns this. Returns this with ret 4. |
| `0093bcc0` | `TileEventHandler::HandleTargetEvent ⚠downgrade` | 🟢 | virtual | For event kind 8, initializes a temporary Target, copies two supplied values into its state, then calls the shared input-event handler. |
