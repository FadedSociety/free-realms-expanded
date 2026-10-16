# System: platform.win32.clipboard

8 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00440f13` | `RegisterGClipboardWin32` |  | Registers the Win32 clipboard implementation class with the engine's class/type registry. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `0044394f` | `RegisterGClipboardAccessGStringObject` |  | One-shot registration of the GFx clipboard-access class for GStringObject, matching the existing RegisterGClipboardAccessWin32Text/RegisterGClipboardWin32 registrations. | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `004475ba` | `ClipboardReadTextMatching` |  | Text search over the Win32 clipboard for a GString value; the per-format access classes (GClipboardAccess_<format>) are constructed on the fly to decode each format. | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `0044775e` | `ClipboardWriteOrEmpty` |  | Set/clear entry point of the GFC clipboard support: either empties the clipboard or delegates to the format-specific access class. | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |

## 🟢 likely (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00443617` | `CopyGStringToWin32Clipboard` |  | Set-clipboard implementation behind the GFC clipboard-access class for GStringObject values: it exports the GString payload as Win32 CF_TEXT plus CF_UNICODETEXT. | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `00446878` | `TempCtorCallDtor` |  | One-shot wrapper that runs the GString clipboard copy inside a throwaway 01763928 instance; no static caller in this build. | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00440f73` | `— ⚠reject` |  | Format-name resolution for the Win32 clipboard implementation (used when advertising/serialising clipboard content). | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `004435e4` | `— ⚠reject` |  | Registers the Win32 TEXT clipboard-access implementation class (companion of the format-name mapper 00440F73) with the engine's class registry. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |

