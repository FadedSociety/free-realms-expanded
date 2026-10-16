# System: net.login

41 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (18)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00715ec0` | `ClientAccountManager::Run` | ClientAccountManager | The client-side account/login state machine of Retail Free Realms: it consumes the login-server replies handled by the neighbouring ClientAccountManager callbacks, drives the character-select and creator UI, and launche… | [[Native Map/Shards/S0148 00715ec0-00719d20|S0148]] |
| `00719330` | `ClientAccountManager::OnAccountInfo` | ClientAccountManager | Handles the login server account-info result: caches the membership / slot-count / admin flags that the character-select UI reads, resets the cached character list, and shows the unable-to-log-in message on failure. | [[Native Map/Shards/S0148 00715ec0-00719d20|S0148]] |
| `00719550` | `ClientAccountManager::OnRequestCharacterList` | ClientAccountManager | Handles the login server character-list reply: rebuilds the manager's cached character list from the response records (guid plus appearance fields), records whether the account has any character, and moves the account s… | [[Native Map/Shards/S0148 00715ec0-00719d20|S0148]] |
| `0090e6a0` | `PlayClient::HandleInitializationParameters` | PlayClient | Checks client/server environment agreement during initialization. | [[Native Map/Shards/S0239 0090a890-009105b0|S0239]] |
| `00938a50` | `PlayClient::CompleteConnectionSetup` | PlayClient | Completes the client-side connection and player-data setup path. | [[Native Map/Shards/S0247 00934740-00939200|S0247]] |
| `009566c0` | `ClientLoginManager::OnPacketD3` | ClientLoginManager | ClientLoginManager virtual slot 11 dispatches the check-name reply payload after its D3 opcode check. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `0124e870` | `Engine::NetworkRequestPasswordLogin::DebugName` | Engine::NetworkRequestPasswordLogin | Debug or log label of the password-login request action. | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |
| `0124e880` | `Engine::NetworkRequestPasswordLogin::ScalarDeletingDtor` | Engine::NetworkRequestPasswordLogin | Scalar deleting destructor of the password-login request action. | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |
| `0124edb0` | `Engine::NetworkQueryUserNameRegistration::DebugName` | Engine::NetworkQueryUserNameRegistration | Debug or log label of the user-name-registration query action. | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |
| `0124edc0` | `Engine::NetworkQueryUserNameRegistration::ScalarDeletingDtor` | Engine::NetworkQueryUserNameRegistration | Scalar deleting destructor of the QueryUserNameRegistration request action. | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |
| `0124f270` | `Engine::NetworkRequestSessionIDLogin::DebugName` | Engine::NetworkRequestSessionIDLogin | Debug or log label of the session-ID login request action. | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |
| `0124f280` | `Engine::NetworkRequestSessionIDLogin::ScalarDeletingDtor` | Engine::NetworkRequestSessionIDLogin | Scalar deleting destructor of the session-ID login request action. | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |
| `013c15d0` | `LoginCommand_RegisterUsername::LoginCommand_RegisterUsername` | LoginCommand_RegisterUsername | Constructs one register-username login command with empty username and password and both flags cleared. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c1670` | `LoginCommand_RegisterUsername::GetName` | LoginCommand_RegisterUsername | Name getter that identifies the register-username login command by its class-name string. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c1840` | `LoginCommand_RegisterUsername::ScalarDeletingDtor` | LoginCommand_RegisterUsername | Scalar deleting destructor of the register-username login command. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c18a0` | `LoginCommand_RegisterUsername::CopyFieldsTo` | LoginCommand_RegisterUsername | Copies this register-username command's fields into another login command object (the target is cast to the register-username class). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c1920` | `LoginCommand_RegisterUsername::TraceFields` | LoginCommand_RegisterUsername | Trace-logs the fields of the register-username login command: username, password, Migrating and Creating flags. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c1a30` | `LoginCommand_RegisterUsername::TraceName` | LoginCommand_RegisterUsername | Emits the trace line naming this login command. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |

## 🟢 likely (9)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00956100` | `ClientLoginManager::ParseNameSuggestionReply` | ClientLoginManager | Its only in-shard caller, 009566c0, dispatches a D3 reply here; this routine parses the payload and delivers decoded suggestions to the manager. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `0124e7e0` | `Engine::NetworkRequestPasswordLogin::ctor` | Engine::NetworkRequestPasswordLogin | Constructs the RequestPasswordLogin action (Scaleform handler base, then the password-login derived class). | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |
| `0124e850` | `Engine::NetworkRequestPasswordLogin::TypeName ⚠downgrade` | Engine::NetworkRequestPasswordLogin | Action type name 'RequestPasswordLogin' used as the request key. | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |
| `0124ed20` | `Engine::NetworkQueryUserNameRegistration::ctor` | Engine::NetworkQueryUserNameRegistration | Constructs the QueryUserNameRegistration request action (Scaleform handler base, then derived class). | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |
| `0124ed90` | `Engine::NetworkQueryUserNameRegistration::TypeName ⚠downgrade` | Engine::NetworkQueryUserNameRegistration | Action type name 'QueryUserNameRegistration' used as the request key. | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |
| `0124f1e0` | `Engine::NetworkRequestSessionIDLogin::ctor` | Engine::NetworkRequestSessionIDLogin | Constructs the RequestSessionIDLogin request action (Scaleform handler base, then the session-ID login derived class). | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |
| `0124f250` | `Engine::NetworkRequestSessionIDLogin::TypeName ⚠downgrade` | Engine::NetworkRequestSessionIDLogin | Action type name 'RequestSessionIDLogin' used as the request key. | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |
| `0124f610` | `Engine::NetworkRequestPasswordLogin::ExecuteCredentialFields` | Engine::NetworkRequestPasswordLogin | Passes the UserName and Password field names to the current password-login request object. | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |
| `013c14a0` | `LoginCommand_RegisterUsername::Deserialize` | LoginCommand_RegisterUsername | Inbound decode of the register-username login command: username at +0x04, password at +0x20, and the flags at +0x3c (Migrating) and +0x3d (Creating), the names taken from the trace function 013c1920. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |

## 🟡 speculative (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `013c1680` | `LoginCommand_RegisterUsername::GetSharedDword ?` | LoginCommand_RegisterUsername | Virtual slot [12] of the class: returns the 32-bit global DAT_01cbdd78 (bytes a1 78 dd cb 01 c3: mov eax,[0x01cbdd78]; ret). Slot [13] writes the same global. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c1690` | `LoginCommand_RegisterUsername::SetSharedDword ?` | LoginCommand_RegisterUsername | Virtual slot [13] of the class: stores its 32-bit argument into the global DAT_01cbdd78 (bytes 8b 44 24 04 a3 78 dd cb 01 c2 04 00: mov eax,[esp+4]; mov [0x01cbdd78],eax; ret 4). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c1750` | `LoginCommand_RegisterUsername::CreateDefaultAndSink ?` | LoginCommand_RegisterUsername | Virtual slot [17] of the class: allocates 0x40 bytes with FUN_00d833a6; when that succeeds it constructs a LoginCommand_RegisterUsername with FUN_013c15d0 (otherwise the result is 0). It then calls virtual slot [0x48/4 … | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c17c0` | `CreateObj0x40Default ? → LoginCommand_RegisterUsername::CreateDefault` |  | Allocates 0x40 bytes with FUN_00d833a6; when that succeeds it constructs a LoginCommand_RegisterUsername with FUN_013c15d0 and returns it; returns 0 on allocation failure. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c1820` | `RegisterFactory0x198 ?` |  | Pushes the address of FUN_013c17c0 and the key 0x198, calls FUN_013ce130(0x198, FUN_013c17c0), moves the returned object into ecx and calls FUN_013ce230, then returns. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c1860` | `SetStringAt4 ?` |  | Assigns the argument std::string to the string at param_1+0x04 (assign(src, 0, npos); bytes 8b 44 24 04 / 6a ff 6a 00 50 / 83 c1 04 / e8 / c2 04 00 pattern with this in ecx). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c1880` | `SetStringAt20 ?` |  | Assigns the argument std::string to the string at param_1+0x20 (assign(src, 0, npos)). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |

## ⚪ unknown (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `007198b0` | `— ⚠reject` | ClientAccountManager | Handles the login server character-delete reply: on success it drops the deleted character from the client cache and rewinds the selection, on refusal it routes the guid back into the client's web-service error path. | [[Native Map/Shards/S0148 00715ec0-00719d20|S0148]] |
| `00956230` | `— ⚠reject` |  | The only in-shard caller, 009564a0, supplies a PacketCheckNameRequest; this helper itself forwards a generic byte array. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `009564a0` | `— ⚠reject` |  | Builds the outbound check-name request on the first/last-name input path; row-filter details remain in 0070d300. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00956590` | `— ⚠reject` |  | Submits manager data through a route helper; helper 00fa9db0 labels its mode-1 branch CharacterCreateRequest, but this function does not expose the selected route mode. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `0124f520` | `— ⚠reject` | Engine::NetworkRequestSessionIDLogin | Hands the 'SessionID' field name to the current session-ID login request object. | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |
| `013c1550` | `— ⚠reject` | LoginCommand_RegisterUsername | Outbound field encoding for the register-username login command (mirror of Deserialize, slot [9]). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c16b0` | `— ⚠reject` | LoginCommand_RegisterUsername | Destructor body of the register-username login command, run from its scalar deleting destructor (013c1840) and from an EH unwind funclet. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `0090e6a0` | `PlayClient::HandleInitializationParameters` | recv | PacketInitializationParameters |  | Hypothesis: initialization sends an environment identifier for comparison with the client's configured environment; this depends on the native packet constructor/deserializer and this handler, not original server source. |
| `00938a50` | `PlayClient::CompleteConnectionSetup` | send | PacketClientInitializationDetails |  |  |
| `00938a50` | `PlayClient::CompleteConnectionSetup` | send | PacketTunneledClientWorldPacket |  |  |
| `00938a50` | `PlayClient::CompleteConnectionSetup` | send | PacketTunneledClientPacket |  |  |
| `009566c0` | `ClientLoginManager::OnPacketD3` | recv | PacketCheckNameReply | 0xd3 | Hypothesis: the original name-check service answered PacketCheckNameRequest (D2) with PacketCheckNameReply (D3). This depends on request construction/send at 009564a0/00956230 and reply dispatch/parsing at 009566c0/00956100; the third-party catalogue supplies the class/opcode mapping. No original server logic or captured native 210/211 exchange is recovered, so validation rules remain unknown. |
| `00956100` | `ClientLoginManager::ParseNameSuggestionReply` | recv | PacketCheckNameReply |  |  |
| `00956230` | `— ⚠reject` | send | PacketCheckNameRequest |  |  |
| `009564a0` | `— ⚠reject` | send | PacketCheckNameRequest |  |  |
| `00956590` | `— ⚠reject` | send | CharacterCreateRequest |  |  |
