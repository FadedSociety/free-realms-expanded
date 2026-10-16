# Class: GameCommerce::StoreItemOfTheDay<AppStoreBundleDefinition>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009983c0` | `StoreItemOfTheDayAppStoreBundleDtorBody` | 🟢 | dtor | Fastcall (ECX=this). Stores the GameCommerce::StoreItemOfTheDay<AppStoreBundleDefinition> vtable (0180d850) at [this], then overwrites it with the GameCommerce::BaseDefinition vtable (0180d2a8). Releases nothing and doe… |
| `0099afd0` | `StoreItemOfTheDayAppStoreBundleIsValid` | 🟢 | virtual | Virtual slot [0] (body is an ordinary predicate, not a deleting destructor), fastcall ECX=this, no stack args. Returns 1 when [this+4], [this+0xc], [this+8] and [this+0x10] are positive, [this+0x20] and [this+0x24] are … |
| `0099d8d0` | `StoreItemOfTheDay::StoreItemOfTheDay` | ✅ | ctor | Installs the StoreItemOfTheDay vtable, initializes scalar fields and defaults, and calls FUN_0099C1B0 with argument 1. |
| `0099e540` | `— ⚠reject` | ⚪ | virtual | Restores the StoreItemOfTheDay vtable, then the BaseDefinition vtable, conditionally frees the object when the deleting flag is set, and returns it. |
| `009a00a0` | `StoreItemOfTheDay::Format` | ✅ | virtual | Formats 11 integer fields into a wide debug string, including id, date, time remaining, availability, price, message, store, and item ids. |
