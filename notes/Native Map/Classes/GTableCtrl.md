# Class: GTableCtrl

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00663b50` | `GTableCtrl::WriteTableProperties` | ✅ | virtual | thiscall(this, param_2 property sink). Returns 0 when 00418ce1(sink) is false. Otherwise writes TableFlags (value string GFC_CF_TABLE_NOBORDER, this+0xe8 and +0xec) through sink vtbl+0x48, then a further sink write thro… |
| `00663ed9` | `GTableCtrl::AccumulateTotalSlot65` | 🟢 | virtual | fastcall, this in ECX. total = 0040eb1d(); when this+0x1c4 is non-null adds its vtbl+0x3c(0,0). Loop 1: for each index below 0055244a() calls 00552460(index) and adds 004ca721(). Then adds 0x2c and the 0055244a() count.… |
