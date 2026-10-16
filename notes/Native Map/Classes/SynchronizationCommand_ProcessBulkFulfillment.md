# Class: SynchronizationCommand_ProcessBulkFulfillment

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0143af90` | `— ⚠reject` | ⚪ | virtual | Thiscall with one stack argument, ret 4. Calls 01241650 with ECX=argument and the literal ProcessBulkFulfillment Synchronization Command plus newline (length 0x2f), which assigns into the std::string argument. Then call… |
| `0143afc0` | `— ⚠reject` | ⚪ | virtual | One stack argument, ret 4, ECX not used on entry. Loads ECX from the argument, pushes 0x2e and the literal ProcessBulkFulfillment Synchronization Command (without newline), and calls 01241650 (std::string assign). |
| `0143b0c0` | `— ⚠reject` | ⚪ | virtual | Thiscall with one stack argument, ret 4. Calls 00d8d382 as an RTTI dynamic_cast of the argument from NetworkCommand to SynchronizationCommand_ProcessBulkFulfillment. On failure it calls 012f5a60 with synchCommand, a sou… |
