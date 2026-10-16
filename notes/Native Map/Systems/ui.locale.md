# System: ui.locale

40 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (13)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0097b520` | `IStringFormatGlobalTextKey` |  | Builds the localisation key Global.Text.<id> for a numeric text id. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `0097b940` | `IStringFormatGlobalTextKeyAtPlus10` |  | Stores the Global.Text.<id> localisation key for a numeric text id in the object's IString member at +0x10. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `0097bca0` | `ExpandGlobalTextIdOrLog` |  | Expands a Global.Text id: finds its localised key, falls back to a pattern expansion, and logs failed expansions to StringLookup.log when logging is enabled. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00a85140` | `LocaleLanguageDataSource::GetColumnName` | LocaleDataSource | Supplies the column titles of the BaseClient.Locale / BaseClient.Language tables. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a851d0` | `LocaleLanguageDataSource::GetRowCount` | LocaleDataSource | Number of rows in the locale/language string tables. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a85370` | `LocaleLanguageDataSource::GetField` | LocaleDataSource | Supplies one column of one locale/language row to the Lua/Scaleform UI. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a855b0` | `LocaleDataSourceDtor` | LocaleDataSource | Tears down the BaseClient.Locale table storage. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a85660` | `LanguageDataSourceDtor` | LanguageDataSource | Tears down the BaseClient.Language table. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a85780` | `LocaleDataSourceScalarDeletingDtor` | LocaleDataSource | Vtable slot 0 of the locale data source. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a85860` | `LanguageDataSourceCtor` | LanguageDataSource | Creates the language table the Lua UI reads language rows from. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a858e0` | `LanguageDataSourceScalarDeletingDtor` | LanguageDataSource | Vtable slot 0 of the language data source. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00c34880` | `FormatTimeAgo` |  | Formats elapsed time as a localized relative-time label. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00d04b60` | `LanguageIdToCode` |  | Maps a numeric language index to its two-letter locale code. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |

## 🟢 likely (15)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00979fc0` | `LocaleNameForLanguageId` |  | Returns the locale name string for a language id, in the underscore or dash spelling. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `0097afe0` | `IStringFormatWithLocaleNameFour` |  | Formats a localised string with four arguments for the object's language id. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `0097b0e0` | `IStringFormatWithLocaleNameFourB` |  | Formats a localised string with four arguments for the object's language id; the four-argument caller 00c1cd20 (accepted label) uses it. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `0097b1e0` | `IStringFormatWithLocaleNameOne` |  | Formats a localised string with one argument for the object's language id. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `0097b2d0` | `IStringFormatWithLocaleNameTwo` |  | Formats a localised string with two arguments for the object's language id. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `0097b3d0` | `IStringFormatWithLocaleNameOneB` |  | Formats a localised string with one argument for the object's language id. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `0097be40` | `StringLookupByIdWithFallback` |  | Resolves a numeric string id into the output IString through the text provider, with a visible not-found fallback. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `0097bf40` | `ExpandTextWithRoleNodes` |  | Expands a Global.Text id into a new IString, building the role description nodes first on the provider path that uses expansion. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `0097c090` | `ExpandTextWithRoleArray` |  | Expands a text id after building role descriptions for an array of role nodes, on the provider path that uses expansion. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `0097c170` | `StringLookupByIdToOut` |  | Looks up a numeric string id into a fresh IString through the text provider in ECX. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `0097c1e0` | `ExpandTextWithRoleArrayToIString` |  | Creates an empty IString result and fills it by expanding a text id over a role array passed on the stack. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `0097c280` | `PlayerNameTemplateExpand` |  | Expands the PlayerName template for a player with the 'player' role text. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00d04bf0` | `IndexedEntryOrEmpty` |  | Looks up a 234-entry locale/country table entry by index; caller 00b46200 (HandleCountryCodesResponse, accepted label) is the country-list consumer. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00d04e80` | `ParseLocaleTag` |  | Resolves a locale tag string to language and country ids. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00ebf440` | `T4Macro_PT_PT::DoPrepositionArticleMacro::Dispatch → T4Macro_PT_PT::DoPrepositi…` | T4Macro_PT_PT::DoPrepositionArticleMacro | Preposition-article macro: forwards three arguments to 00ebe9f0 with one of two constant four-pointer records selected by a flag. | [[Native Map/Shards/S0809 0066ca4d-0147b010|S0809]] |

## 🟡 speculative (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009a0770` | `FormatStringLocalized ? ⚠rename` |  | Localization or display-string formatting wrapper inferred from callers; language selection is delegated to FUN_00703620. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `009a0830` | `FormatStringWithFallback ? ⚠rename` |  | Fallback-aware localization formatter. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `00d04d70` | `TableFind11ByString ?` |  | Searches the 11-entry table at 0x183d700 with 00d04c10 and the predicate at 00d04d10 against param_1; when a record is found it compares the string at its +4 with the string at param_1+4 via 006fc5b0 and returns the rec… | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00d04db0` | `TableFind234ByString ?` |  | Searches the 234-entry table at 0x183d758 with 00d04c90 and the predicate at 00d04d40 against param_1; when a record is found it compares the string at its +4 with param_1+4 via 006fc5b0 and returns the first dword when… | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |

## ⚪ unknown (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0097a180` | `— ⚠reject` |  | Calls LocaleNameForLanguageId (00979fc0) with the dword at this+0xc and the param_2 flag, discarding the returned string. Returns nothing. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `0097ae80` | `— ⚠reject` |  | Formats text that includes the locale name for the object's language id. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `0097c260` | `— ⚠reject` |  | Calls 0097c090 with (param_2, param_3, param_4, the address of its stack argument block) and returns nothing; param_1 is unused. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00a85240` | `— ⚠reject` | LocaleDataSource | Rebuilds the rows of the BaseClient.Locale table from the loaded locale data. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a852a0` | `— ⚠reject` | LanguageDataSource | Rebuilds the rows of the BaseClient.Language table, skipping languages that are already present. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a856c0` | `— ⚠reject` | LocaleDataSource | Creates the locale table that lists the client's available locales/languages. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a857a0` | `— ⚠reject` | LocaleDataSource | Common initialisation of the BaseClient.Locale and BaseClient.Language tables (the Language constructor calls it with the Language name). | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00d04e00` | `— ⚠reject` |  | Builds a locale tag such as language-country from numeric ids. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |

