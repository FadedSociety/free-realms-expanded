# Class: ScheduledActivityPacketListOfActivities

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00be5690` | `ScheduledActivityPacketListOfActivities::Ctor` | ✅ | ctor | Constructor body: calls FUN_00bdee60 with the constant 1 (the scheduled-activity base constructor with payload), stores the ScheduledActivityPacketListOfActivities vtable at *this, stores the SoeUtil::List<ClientSchedul… |
| `00be61b0` | `ScheduledActivityPacketListOfActivities_ScalarDtor` | 🟢 | dtor | Calls 00be5750 (list destructor body) on this, then calls _free(this) when bit 0 of the flag argument is set (ret 4); returns this. |
