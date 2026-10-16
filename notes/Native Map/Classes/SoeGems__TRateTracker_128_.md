# Class: SoeGems::TRateTracker<128>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00897c70` | `SoeGems::TRateTracker128Ctor` | ✅ | ctor | Constructor of SoeGems::TRateTracker<128>: installs the class vtable, stores the default window length 1000 at +0x410, zeroes the two 128-entry int arrays at +4 and +0x204 with _memset(...,0x200), and clears the running… |
| `00897cd0` | `RateTracker128GetWindowMs` | ✅ | method | Returns the dword at object+0x410, the rate-window field initialized by FUN_00897c70 and set by FUN_00897910. |
| `00897df0` | `RateTracker128AddSample` | ✅ | method | Records a sample: advances the window (FUN_00897d00), adds param_2 to the current bucket's sum at +4 + (index & 0x7f)*4 and increments the matching count slot at +0x204, updates running sum +0x408 and count +0x40c, and … |
| `00897e50` | `RateTracker128ScaledSum` | ✅ | method | Advances the window (FUN_00897d00) and then returns runningSum(+0x408) * param_2 / window(+0x410), a 64-bit signed division, i.e. the accumulated total rescaled to the caller's interval. |
| `00897e80` | `RateTracker128ScaledCount` | ✅ | method | Advances the window (FUN_00897d00) and returns sampleCount(+0x40c) * param_2 / window(+0x410) via __alldiv, i.e. the per-interval event count. |
| `00897eb0` | `RateTracker128Average` | ✅ | method | Advances the window (FUN_00897d00) and returns runningSum(+0x408) divided by sampleCount(+0x40c) when the count is positive, otherwise 0. |
