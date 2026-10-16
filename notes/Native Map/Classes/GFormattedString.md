# Class: GFormattedString

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0048232e` | `GFormattedString::WriteGlobalAndOptionalArgument` | ✅ | virtual | Writes DAT_01af9c8c into *param_2 and, when param_3 is non-null, writes param_1 into *param_3. |
| `00482520` | `GFormattedString::FetchRecordDword` | ✅ | virtual | Loads the dword stored at *(this+0x1C) and writes it through the out-parameter, returning a value whose low byte is 1 (the success flag kept in AL). |
| `00482530` | `— ⚠reject` | ⚪ | virtual | Forwards its three arguments unchanged to FUN_004e90dd and returns 1 unconditionally. |
| `0049162f` | `GFormattedString::AttachRangeFromBuffer` | ✅ | virtual | Calls FUN_004e90dd(param_1,param_3,param_4), sets range start to param_3 and end to param_3+param_4 for nonnegative length or 0x7ffffffe for negative length, attaches the pair to param_2 through FUN_0052a85c with count … |
