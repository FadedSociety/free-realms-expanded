# Class: SocialShare

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bfe600` | `SocialShare::GetProviderStatus ⚠downgrade` | 🟢 | callback | Requires a Lua stack of two arguments and provider ID 1 or 2, formats a diagnostic string, invokes the provider status callback, and returns zero. |
| `00bfe710` | `SocialShare::RequestLoginURL` | ✅ | callback | Requires a Lua stack of two arguments and provider ID 1 or 2, formats a diagnostic string, calls the provider login-URL helper, and returns zero. |
| `00bfe7e0` | `SocialShare::Callback ? ⚠downgrade` | 🟡 | callback | Requires three Lua arguments with types accepted by the checks, extracts values from positions two and three, and calls 00adffe0 for provider ID 1 or 2. |
