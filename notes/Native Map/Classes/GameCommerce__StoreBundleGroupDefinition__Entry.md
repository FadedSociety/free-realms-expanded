# Class: GameCommerce::StoreBundleGroupDefinition::Entry

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d58400` | `GameCommerce::StoreBundleGroupDefinition::Entry::Ctor` | ✅ | ctor | Bytes 8b c1 c7 00 04 0c 84 01 8b 0d c8 2c bf 01 89 48 04 c7 40 08 00 00 00 00 c3: stores GameCommerce::StoreBundleGroupDefinition::Entry::vftable into *this, copies the global DAT_01bf2cc8 into +4 and zeroes +8. |
| `00d58740` | `GameCommerce::StoreBundleGroupDefinition::Entry::FormatIdAndOrder` | ✅ | virtual | Formats the entry's fields at +4 and +8 with the format 'storeBundleId=%d, displayOrder=%d' through FUN_00402fd0 into the output buffer, and returns the output's pointer field at +4. |
