# Class: BaseAdventurersJournalPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a426c0` | `BaseAdventurersJournalPacket::BaseAdventurersJournalPacket` | ✅ | ctor | Base constructor of the Adventurer's Journal packet family: installs BasePacket::vftable, stores the family opcode 0xd1 at +4, installs BaseAdventurersJournalPacket::vftable and stores the caller's sub-opcode at +8. |
| `00a42730` | `BaseAdventurersJournalPacket::~BaseAdventurersJournalPacket` | 🟢 | dtor | Base-part teardown of journal packets: reinstalls BasePacket::vftable at +0 and returns. |
| `00a43110` | `BaseAdventurersJournalPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0) of the journal family base packet: installs BasePacket::vftable and frees the object with _free when bit 0 of the delete flag is set. |
