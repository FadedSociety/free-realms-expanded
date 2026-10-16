# Class: ClientCommandProcessor

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a9f410` | `ClientCommandProcessor::ExecuteCommand` | ✅ | virtual | Virtual command dispatcher taking an integer kind: kind 0 calls FUN_00a9e790, kind 1 calls FUN_00a9e8f0, kind 2 calls FUN_00a9ea40; each handled case returns 1, any other kind returns 0. Bytes show the bytecode as repea… |
| `00aa06a0` | `ClientCommandProcessor::ClientCommandProcessor` | ✅ | ctor | Constructor: installs CommandProcessor::CommandEventHandler::vftable then ClientCommandProcessor::vftable, stores the second argument at +4 (the console context), calls FUN_00cb8c30, constructs two history deques throug… |
| `00aa07b0` | `ClientCommandProcessor::~ClientCommandProcessor` | ✅ | dtor | Destructor body: rewrites the ClientCommandProcessor vtable, calls FUN_00986c10, saves the command history with FUN_00a9c420 (writes CommandHistory.txt), rewrites the HashListMap vtable at +0x84, calls FUN_009af010, des… |
| `00aa12c0` | `ClientCommandProcessor::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: calls the destructor body FUN_00aa07b0 (which saves the command history) and, when bit 0 of the flags argument is set, calls _free on the object; returns the object pointer. |
| `00aa2560` | `ClientCommandProcessor::DispatchCommandPacket` | 🟢 | method | Calls FUN_00a9cb50 to read a command header, then dispatches cases 2, 3, 4, 7, 9, 0xc, 0x10, 0x12, 0x16, 0x19, 0x1c, 0x1d, 0x1f, 0x23, 0x26 and 0x27. In case 0x1f, a successful body parse writes the field at nested obje… |
| `00aa2980` | `ClientCommandProcessor::ParseMatchmakingPacket` | 🟢 | method | Reads a sub-opcode into local_14c and dispatches cases 2, 4, 5, 9, 0xc, 0xe, 0xf and 0x12. Case 9 builds an SoeUtil::IString and formats 'String#%d not found' on lookup failure; case 0xc calls FUN_0070ec40, FUN_0070e140… |
| `00aa36c0` | `ClientCommandProcessor::DispatchEncounterPacket` | ✅ | method | Reads the encounter message header and dispatches sub-opcodes 2, 3, 4, 5, 6, 0x66, 0x68, 0x69, 0x6a, 0x6b, 0x72, 0x78, 0x79, 0x82, 0x84 and 0x85. Case 0x72 constructs/parses the details response then copies selected fie… |
