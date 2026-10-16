# System: ui.login

6 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0124fda0` | `Engine::GatewayServerSuccessDisplayAction::ApplyConnectionSucceeded` | Engine::GatewayServerSuccessDisplayActi… | Marks the gateway connection as succeeded and shows the 'logging in' explanation in the login UI. | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |
| `01250040` | `Engine::NetworkQueryUserNameRegistration::OpenRegisterUserNameDialog` | Engine::NetworkQueryUserNameRegistration | Opens the user-name registration dialog in the login UI when a user-name query requires it. | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |

## 🟢 likely (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `01294c60` | `Engine::AccountUsernameRegistration::BuildRegisterUsernameCommand` | Engine::AccountUsernameRegistration | Builds a username-registration command and conditionally hands it to the generic command serializer; the packet identity and final transmission are unresolved. | [[Native Map/Shards/S0671 01291250-01295650|S0671]] |
| `01294de0` | `Engine::AccountUsernameRegistrationResultsDisplayAction::ApplyRegistrationResult` | Engine::AccountUsernameRegistrationResu… | Username-registration result display action: maps its integer argument to a login explanation and updates the login-status action fields. | [[Native Map/Shards/S0671 01291250-01295650|S0671]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0124f780` | `— → Engine::LoginResultsDisplayAction::Invoke ⚠reject` | Engine::LoginResultsDisplayAction | Applies a login result to the login/lobby UI: shows the success or error explanation, navigates to the lobby or the user-name registration dialog, and reports LoginSucceeded/LoginFinished. | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |
| `0124fea0` | `— ⚠reject` | Engine::GatewayServerErrorDisplayAction | Shows a gateway connection error explanation in the login UI and clears the connection-succeeded value. | [[Native Map/Shards/S0658 0124dfe0-01253ee0|S0658]] |

