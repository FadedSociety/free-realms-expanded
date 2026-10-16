# Class: GameCommerce::StoreBundleOfTheDay<AppStoreBundleDefinition>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00998450` | `StoreBundleOfTheDayAppStoreBundleDtorBody` | 🟢 | dtor | Fastcall (ECX=this). Stores the GameCommerce::StoreBundleOfTheDay<AppStoreBundleDefinition> vtable (0180d864) at [this], then overwrites it with the GameCommerce::BaseDefinition vtable (0180d2a8). Releases nothing and d… |
| `0099b070` | `StoreBundleOfTheDayAppStoreBundleIsValid` | 🟢 | virtual | Virtual slot [0] (body is an ordinary predicate, not a deleting destructor), fastcall ECX=this, no stack args. Same body as 0099afd0: returns 1 when [this+4], [this+0xc], [this+8] and [this+0x10] are positive, [this+0x2… |
| `0099d9d0` | `StoreBundleOfTheDay::StoreBundleOfTheDay` | ✅ | ctor | Installs the StoreBundleOfTheDay vtable, initializes fields including maxAvailable to 0x7FFFFFFF, and calls FUN_0099C2B0(1). |
| `0099e5b0` | `StoreBundleOfTheDay::ScalarDeletingDtor` | ✅ | virtual | Restores the StoreBundleOfTheDay vtable, then BaseDefinition, conditionally frees the object, and returns the object pointer. |
| `009a0180` | `StoreBundleOfTheDay::Format` | ✅ | virtual | Formats 11 integer fields into a wide diagnostic string containing ids, date, availability, price, message id, and store id. |
