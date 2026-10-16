# Class: LobbyCommand_AddGroups

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0143e520` | `— ⚠reject` | ⚪ | virtual | Thiscall with one stack argument, ret 4. Calls vtable slot 2 of this with the argument, then 013c26e0 with the framing text Starting LobbyCommand_AddGroups, then 013b2960 with the argument, then 0143e440 with ECX unchan… |
| `0143e6a0` | `LobbyCommand_AddGroups::TypeName` | ✅ | virtual | No arguments. Returns a pointer to the literal LobbyCommand_AddGroups (mov eax,018ea044; ret). |
| `0143e6b0` | `LobbyCommand_AddGroups::ReadGlobal01cbe38c` | 🟢 | virtual | No arguments. Returns the 32-bit global at 01cbe38c (mov eax,[01cbe38c]; ret). The global is uninitialised .data in the file. |
| `0143e6c0` | `LobbyCommand_AddGroups::WriteGlobal01cbe38c` | 🟢 | virtual | One stack argument, ret 4. Stores the argument to the global at 01cbe38c. |
| `0143ee40` | `LobbyCommand_AddGroups::DescribeGroupIds` | 🟢 | virtual | One stack argument, ECX=this. Writes Add Groups Lobby Command plus newline with 01241650, calls 013b2af0 with the argument, then loops from [this+0xc] to [this+0x10] in steps of 4: value from 01301f30(), sprintf GroupID… |
| `0143ef30` | `— ⚠reject` | ⚪ | virtual | One stack argument, ret 4. Loads ECX from the argument, pushes 0x18 and the literal Add Groups Lobby Command, and calls 01241650 (std::string assign). |
| `0143f260` | `— ⚠reject` | ⚪ | virtual | One stack argument, ret 4. dynamic_casts the argument from NetworkCommand to LobbyCommand_AddGroups (00d8d382). On failure it calls 012f5a60 with clone and the LobbyCommandAddGroups.cpp path and line 0xb6. Then calls 01… |
