# System: engine.log

57 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (23)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00897050` | `Logging::LoggingApiCtor` | Logging::LoggingApi | Startup constructor of the client logging API that owns the log message queue and per-file sink table. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `008973e0` | `Logging::LoggingApi::~LoggingApi` | Logging::LoggingApi | Shutdown destructor of the client logging API. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `008977f0` | `SoeGems::IRateTrackerScalarDtor` | SoeGems::IRateTracker | Deleting destructor for rate-tracker objects that are deleted through the IRateTracker interface. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `008978b0` | `SoeGems::IRateTracker::~IRateTracker` | SoeGems::IRateTracker | Base destructor for the rate-tracker interface, emitted for cleanup of derived tracker objects. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `00897c70` | `SoeGems::TRateTracker128Ctor` | SoeGems::TRateTracker<128> | Constructs the rate trackers embedded in the logging subsystem (two per log manager, window re-set to 60000 by the manager constructor FUN_0089a8b0). | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `00897cd0` | `RateTracker128GetWindowMs` | SoeGems::TRateTracker<128> | Accessor for the window length of the rate-tracker family; the class binding is inferred from the identical +0x410/+0x404/+0x408/+0x40c layout shared with FUN_00897c70 and FUN_00897d00. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `00897ce0` | `SoeGems::IRateTrackerScalarDtorB` | SoeGems::IRateTracker | Deleting destructor for rate-tracker objects deleted through the interface from a second calling context. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `00897df0` | `RateTracker128AddSample` | SoeGems::TRateTracker<128> | Sample-entry point of the TRateTracker<128> family used by the logging subsystem to accumulate message/value flow per time bucket. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `00897e50` | `RateTracker128ScaledSum` | SoeGems::TRateTracker<128> | Rate query used by the logging/metrics code to normalise recorded sums to a chosen period. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `00897e80` | `RateTracker128ScaledCount` | SoeGems::TRateTracker<128> | Rate query estimating how many samples fall into the caller's interval; used by the logging subsystem alongside RateTrackerScaledSum. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `00897eb0` | `RateTracker128Average` | SoeGems::TRateTracker<128> | Mean-value query of the TRateTracker<128> family (e.g. average message size/latency). | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `008989c0` | `LogFileWriteQueueWithFailover` |  | The write/flush engine of the logging subsystem: every queue flush, rotation and failover decision in this shard funnels through it. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `00899050` | `LogApiUpdateDateStrings` |  | Daily rotation step of the logging subsystem, called from the periodic tick FUN_00899860 and from the log-manager constructor. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `00899380` | `LogFileWriteLine` |  | Per-line write entry of the logging subsystem: every message formatting call in this shard funnels through it into a Logging::LoggingFile object. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `00899950` | `LogApiWriteToChannel` |  | Top-level 'log to this file name' entry of the logging subsystem; the community/team notes do not cover it yet. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `0089a630` | `LogFileManagerDtor` |  | Shutdown of the manager object constructed by FUN_0089a8b0 (two rate trackers, six 64-char strings, the Logging::LoggingFile map and the file list at +0xa54). | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `0089a8b0` | `LogFileManagerCtor` |  | Builds the object whose layout is consumed by the log flush/rotate/write routines in this shard (FUN_008989c0, FUN_00898f50, FUN_00899860, FUN_00899950) and which the LoggingApi destructor frees at +0xd270. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `008dc940` | `GameUtil::Logging::AppendField18IfPresent` | GameUtil::Logging | Conditional field dump of the shared logging interface: only when the guard word at +0x20 is present does it emit the member at +0x18 through the helper 0071be40 and the constant DAT_01764470. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `008dc970` | `GameUtil::Logging::AppendField28IfPresent` | GameUtil::Logging | Second conditional field dump of the shared logging interface, the sibling of slot 3 (008dc940) one member later in the object. | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `00d07500` | `PrintProfilerReport` |  | Emits profiler report text, not gameplay behavior. | [[Native Map/Shards/S0428 00d07500-00d0bf70|S0428]] |
| `00d0a4e0` | `SoeGems::LoggingHeader::LoggingHeader` | SoeGems::LoggingHeader | Initializes a LoggingHeader with an empty string. | [[Native Map/Shards/S0428 00d07500-00d0bf70|S0428]] |
| `00d0a510` | `SoeGems::LoggingHeader::~LoggingHeader` | SoeGems::LoggingHeader | Releases the LoggingHeader string during destruction. | [[Native Map/Shards/S0428 00d07500-00d0bf70|S0428]] |
| `00d0a550` | `SoeGems::LoggingHeader::ScalarDeletingDtor` | SoeGems::LoggingHeader | Virtual scalar-deleting destructor for LoggingHeader. | [[Native Map/Shards/S0428 00d07500-00d0bf70|S0428]] |

## 🟢 likely (26)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008973b0` | `LoggingApiDtorThunkC0` |  | Deleting-destructor entry of the LoggingApi sub-object vtable at +0xc0. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `008973c0` | `LoggingApiDtorThunkBE8` |  | Deleting-destructor entry of the LogHandler base vtable sub-object at +0xbe8. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `008973d0` | `LoggingApiDtorThunkC4` |  | Deleting-destructor entry of the LoggingApi sub-object vtable at +0xc4. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `008977c0` | `Logging::LoggingApiScalarDtor` | Logging::LoggingApi | Deleting destructor reached from the thunks 008973b0/008973c0/008973d0 and from the primary vtable. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `00897910` | `TRateTracker128Reset` |  | Reset path of the SoeGems::TRateTracker<128> object whose constructor FUN_00897c70 initialises the same fields; no static callers, so it is reached through a vtable or data reference. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `00897d00` | `RateTrackerAdvanceWindow` |  | Window bookkeeping step shared by every rate-tracker query and sample call (FUN_00897df0, FUN_00897e50, FUN_00897e80, FUN_00897eb0); the +0x404..+0x410 layout links it to the TRateTracker<128> constructor FUN_00897c70. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `008980e0` | `HashListFindByName` |  | Log-file lookup by channel/file name used before writing (caller FUN_00899950 creates the file when the lookup misses); the bucket table at +0xa60 is initialised by the log-file manager constructor FUN_0089a8b0 at objec… | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `00898400` | `ExpandLogNamePattern` |  | Builds the time-stamped log file name from a pattern and makes sure the target directory exists; called from the log write path FUN_008989c0. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `00898e20` | `LogFileCtor` |  | Per-file sink construction used by the log-file map insert FUN_008996f0 (the same object layout is consumed by FUN_008989c0/00898f50/00899220/00899380). | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `00898ff0` | `LogApiFlushAllFiles` |  | Bulk flush called by the LoggingApi teardown/update path FUN_00895840 to drain every open log file. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `00899220` | `LogFileCloseAndFree → LogFile::~LogFile (destructor body)` |  | Close-and-release step invoked by the map remove path FUN_008997e0 and by the deleting destructor FUN_00899780. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `00899550` | `LogFileNeedsFlush` |  | Decides in FUN_00899860 whether a log file is flushed now or removed after its rotation timeout; note the function also triggers the flush itself through FUN_008989c0. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `008996f0` | `LogApiAddFile` |  | Add-file entry of the log manager, called by the name-hash wrapper FUN_008997a0 and by the write-by-channel path FUN_00899950. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `00899780` | `LogFileScalarDtor` |  | Deleting destructor used when the map removes a file node through its pool (note FUN_00899220 itself already ends in _free for the buffer). | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `008997a0` | `LogApiAddFileByName` |  | Named file creation wrapper of the log manager; no direct callers were observed, so it is likely reached through a callback table. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `008997e0` | `LogApiRemoveFile` |  | Remove-file path used by the periodic tick (rotation timeout) and by the map teardown FUN_00899ac0; the node's list links are the +0x8d0/+0x8d4 pair also used by 00898710/00898ff0. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `00899860` | `LogApiTick` |  | Housekeeping loop of the log manager, called from the LoggingApi update paths FUN_00895d40/FUN_008964f0. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `00899ac0` | `RemoveAllLogFiles` |  | Map-emptying helper used by the map destructors in this range (FUN_00899ae0/00899b10/00899b50 run the same loop inline). | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `009124c0` | `LoggableBaseResetCleanup` |  | Destructor-style body of the owner class: resets its Loggable base sub-object at +0x148 and tears down sub-objects at +0xe0 and +0x28. Called from ten EH unwind funclets. | [[Native Map/Shards/S0240 009105e0-009161d0|S0240]] |
| `00bab600` | `ScalarDeletingDtor_00bab600` |  | Combines cleanup with conditional scalar deletion. | [[Native Map/Shards/S0364 00ba74d0-00bac070|S0364]] |
| `00d0a5a0` | `LogFormattedToSink_00704db0` |  | Emits a formatted logging message through the selected sink. | [[Native Map/Shards/S0428 00d07500-00d0bf70|S0428]] |
| `00d0a680` | `LogFormattedToSink_00704dd0` |  | Emits a formatted logging message through the selected sink. | [[Native Map/Shards/S0428 00d07500-00d0bf70|S0428]] |
| `00d0a760` | `LogFormattedToSink_00702ab0` |  | Emits a formatted logging message through the selected sink. | [[Native Map/Shards/S0428 00d07500-00d0bf70|S0428]] |
| `00d0a840` | `LogFormattedToSink_006faad0` |  | Emits a formatted logging message through the selected sink. | [[Native Map/Shards/S0428 00d07500-00d0bf70|S0428]] |
| `00d0a920` | `LogFormattedToSink_0089aa90` |  | Emits a formatted logging message through the selected sink. | [[Native Map/Shards/S0428 00d07500-00d0bf70|S0428]] |
| `0136b570` | `FormatTimestampedLog` |  | Timestamped trace-log writer of the card-game engine; six functions call it. | [[Native Map/Shards/S0711 01369c70-0136dbd0|S0711]] |

## 🟡 speculative (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00702ab0` | `LogSeverity3 ?` |  | Forwards the supplied log message and arguments to the shared logger at severity level three. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00704db0` | `LogSeverity5 ?` |  | Forwards the supplied message and arguments to the shared logger at severity level five. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00704dd0` | `LogSeverity4 ?` |  | Forwards the supplied message and arguments to the shared logger at severity level four. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `007990c0` | `CondWriteLogOrDefault ?` |  | Stack args (char *name, arg2). Calls 00769710; when it returns nonzero, writes name through 00704dd0(name, arg2) if name is non-empty, otherwise writes the default string at PTR_DAT_01b2f484 through 00704dd0. Returns no… | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `00bab560` | `LoggableCleanupWrapper ?` |  | Calls FUN_009124c0 inside an EH-framed wrapper and restores the prior exception-list pointer. | [[Native Map/Shards/S0364 00ba74d0-00bac070|S0364]] |
| `0136b720` | `LogWithOptionalBufferFlush ? ⚠rename` |  | When the flag byte at this+0x14d is not 1: if the vector at this+0x2f8 holds fewer than 0x19 (25) 28-byte elements, calls 01233d70 with param_2 and returns. Otherwise it calls 012a9e70 with this+0x2f4 and the first elem… | [[Native Map/Shards/S0711 01369c70-0136dbd0|S0711]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00898f50` | `— ⚠reject` |  | Buffer flush used before writes and on close; the size threshold comparison keyed off the API object ties it to the LoggingApi configuration fields. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `00d0aba0` | `— ⚠reject` |  | Uppercases input bytes, retains letters in a temporary string, updates global lookup tables, and checks selected letter positions against table entries. | [[Native Map/Shards/S0428 00d07500-00d0bf70|S0428]] |

