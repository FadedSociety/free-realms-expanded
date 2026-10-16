# Class: PersistentBase

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01279330` | `PersistentBase::DtorBody` | ✅ | dtor | Destructor body: stores PersistentBase::vftable into *this and returns. Called from 98 MSVC EH unwind funclets. |
| `01279370` | `PersistentBase::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores PersistentBase::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
