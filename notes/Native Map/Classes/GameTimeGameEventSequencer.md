# Class: GameTimeGameEventSequencer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008eabe0` | `GameTimeGameEventSequencerCtor` | ✅ | ctor | Constructs sequencer base, installs vtable, stores manager pointer, allocates a GameTimeTimeline and stores it at +4. |
| `008eac70` | `GameTimeGameEventSequencer::UpdateTimelineFromCurrentTime` | 🟢 | method | If the optional time source at +0xa0 exists, calls its first virtual method, converts the result to float, wraps it using the period at timeline +0x53c and the fixed boundary, then calls the timeline virtual at byte off… |
| `008eacf0` | `GameTimeGameEventSequencerDtor` | ✅ | dtor | Installs sequencer vtable and invokes owned cleanup helper under exception frame. |
| `008ead40` | `GameTimeGameEventSequencer::ScalarDeletingDtor` | ✅ | dtor | Runs sequencer cleanup, frees object if flag bit zero is set, otherwise returns pointer. |
