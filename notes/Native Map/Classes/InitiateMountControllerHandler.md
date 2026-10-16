# Class: InitiateMountControllerHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bb3a0` | `— ⚠reject` | ⚪ | handler | Resolves the two stored ids and checks controller state; if eligible, invokes FUN_00b04290 with a temporary id pair and stored field, then resets the stored ids to global defaults. |
| `008bb440` | `InitiateMountControllerHandler::ResetController` | ✅ | virtual | Resolves the stored ids and, when the associated controller state is active, calls two helpers with values 1 and 0; then resets the ids to global defaults. |
