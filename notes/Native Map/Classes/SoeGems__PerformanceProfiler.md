# Class: SoeGems::PerformanceProfiler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d057e0` | `PerformanceProfilerPopSample` | ✅ | virtual | Virtual slot [2] of SoeGems::PerformanceProfiler: gets the thread context via 006fb860, decrements its counter, and when the enabled flag at context+0x10 is set and the current record's word at +0x40 is zero, reads a 64… |
| `00d08d30` | `SoeGems::PerformanceProfiler::RegisterCall` | ✅ | virtual | Finds or creates thread context and callstack sample entry, initializes counters, and links entry into profiler sample maps. |
| `00d08f10` | `SoeGems::PerformanceProfiler::RecordTiming` | ✅ | virtual | Finds or creates keyed sample, increments 64-bit call count and accumulated timings, and updates maximum and minimum timing pairs. |
| `00d0a350` | `SoeGems::PerformanceProfiler::PerformanceProfiler` | ✅ | ctor | Installs profiler vtable, initializes context list and embedded StringFixed hash list, resets flags, allocates two 4000-byte buffers, and initializes helper. |
| `00d0a410` | `SoeGems::PerformanceProfiler::~PerformanceProfiler` | ✅ | dtor | Clears global profiler pointer when it refers to this object, destroys profiler lists and buffers, then restores base vtables. |
| `00d0a4c0` | `SoeGems::PerformanceProfiler::ScalarDeletingDtor` | ✅ | virtual | Calls profiler destructor, frees object when deleting flag is set, and otherwise returns original pointer. |
