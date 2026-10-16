# Class: SoeGems::LoggingHeader

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d0a4e0` | `SoeGems::LoggingHeader::LoggingHeader` | ✅ | ctor | Installs LoggingHeader and IString<char> vtables, sets shared empty-string pointer, and zeros string metadata and flags. |
| `00d0a510` | `SoeGems::LoggingHeader::~LoggingHeader` | ✅ | dtor | Restores LoggingHeader and IString vtables, decrements shared string reference count, and frees buffer when count reaches zero. |
| `00d0a550` | `SoeGems::LoggingHeader::ScalarDeletingDtor` | ✅ | virtual | Runs LoggingHeader string cleanup, frees object when deleting flag is set, and otherwise returns it. |
