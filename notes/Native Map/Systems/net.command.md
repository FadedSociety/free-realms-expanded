# System: net.command

16 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009aad80` | `CommandPacketInteractionStartWheel::CommandPacketInteractionStartWheel` | CommandPacketInteractionStartWheel | Builds the command packet that starts the interaction wheel (base CommandPacket 26 sub 11 - the catalogue names sub 9 List and sub 10 Select, so 11 is the StartWheel sibling); constructed by 009bdbd0. | [[Native Map/Shards/S0269 009a6cf0-009aceb0|S0269]] |
| `0143e6a0` | `LobbyCommand_AddGroups::TypeName` | LobbyCommand_AddGroups | Returns the command type-name string. | [[Native Map/Shards/S0809 0066ca4d-0147b010|S0809]] |

## 🟢 likely (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009aade0` | `PacketDtor_009aade0` |  | Destructor of the interaction-start-wheel command packet class; the concrete class is not carried by this function's own evidence (only the two base vtables), so the name stays mechanical. | [[Native Map/Shards/S0269 009a6cf0-009aceb0|S0269]] |
| `009aceb0` | `DeletingDtor_009aceb0 → CommandPacketInteractionStartWheel::ScalarDeletingDestr…` | CommandPacketInteractionStartWheel | Slot 0 of the CommandPacketInteractionStartWheel vftable at 0x0180e8c4 (bound by reading the vtable data in .rdata; the constructor 009aad80 stores that same address). The class name is not carried by this function's ow… | [[Native Map/Shards/S0269 009a6cf0-009aceb0|S0269]] |
| `0143e6b0` | `LobbyCommand_AddGroups::ReadGlobal01cbe38c` | LobbyCommand_AddGroups | Getter for the class-wide value that slot 13 sets. | [[Native Map/Shards/S0809 0066ca4d-0147b010|S0809]] |
| `0143e6c0` | `LobbyCommand_AddGroups::WriteGlobal01cbe38c` | LobbyCommand_AddGroups | Setter paired with slot 12 for the class-wide value. | [[Native Map/Shards/S0809 0066ca4d-0147b010|S0809]] |
| `0143ee40` | `LobbyCommand_AddGroups::DescribeGroupIds` | LobbyCommand_AddGroups | Writes a text description of the AddGroups command: a header line and one GroupID line per entry. | [[Native Map/Shards/S0809 0066ca4d-0147b010|S0809]] |

## ⚪ unknown (9)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0093bff0` | `— ⚠reject` |  | Send the command that selects a player or interaction target. | [[Native Map/Shards/S0248 009392c0-0093d740|S0248]] |
| `00c24b00` | `— ⚠reject` |  | Submits the ignore-request packet with constructor flag one; the flag's action semantics are unresolved. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c24c00` | `— ⚠reject` |  | Submits the ignore-request packet with constructor flag zero; the flag's action semantics are unresolved. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `0143af90` | `— ⚠reject` | SynchronizationCommand_ProcessBulkFulfi… | Virtual slot 19: assigns the command name (with a trailing newline) into its string argument, then calls the NetworkCommand slot-19 stub. | [[Native Map/Shards/S0809 0066ca4d-0147b010|S0809]] |
| `0143afc0` | `— ⚠reject` | SynchronizationCommand_ProcessBulkFulfi… | Virtual slot 20: assigns the command name into its std::string argument. | [[Native Map/Shards/S0809 0066ca4d-0147b010|S0809]] |
| `0143b0c0` | `— ⚠reject` | SynchronizationCommand_ProcessBulkFulfi… | Virtual slot 18: copies the command from its argument (dynamic_cast checked), calling the NetworkCommand base stub and a +4 member copy. | [[Native Map/Shards/S0809 0066ca4d-0147b010|S0809]] |
| `0143e520` | `— ⚠reject` | LobbyCommand_AddGroups | Brackets a LobbyCommand_AddGroups body with base-class virtual calls; the framing text is passed to a no-op (013c26e0) and is not emitted, per the lead. | [[Native Map/Shards/S0809 0066ca4d-0147b010|S0809]] |
| `0143ef30` | `— ⚠reject` | LobbyCommand_AddGroups | Virtual slot 20: assigns the command name into its std::string argument. | [[Native Map/Shards/S0809 0066ca4d-0147b010|S0809]] |
| `0143f260` | `— ⚠reject` | LobbyCommand_AddGroups | Virtual slot 18: the clone path for AddGroups, copying base state and member +8 from its argument. | [[Native Map/Shards/S0809 0066ca4d-0147b010|S0809]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `009aad80` | `CommandPacketInteractionStartWheel::CommandPacketInteractionStartWheel` | send |  | 0xb | Hypothesis: the client asked the server to start the interaction wheel (26/11) so the server would send the interaction list (26/9) and answer selections (26/10). |
| `0093bff0` | `— ⚠reject` | send | CommandPacketSelectPlayer |  | Hypothesis: server records or acknowledges the selected target; depends on this command and client send path, with original server code unavailable. |
| `00c24b00` | `— ⚠reject` | send | CommandPacketIgnoreRequest |  |  |
| `00c24c00` | `— ⚠reject` | send | CommandPacketIgnoreRequest |  |  |
