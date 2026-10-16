# Class: ReportPacketReportPlayer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008dd550` | `ReportPacketReportPlayer::ReportPacketReportPlayer` | ✅ | ctor | Packet constructor: calls the report family constructor 008a5650 with sub-opcode 1, installs ReportPacketReportPlayer::vftable, initialises an embedded SoeUtil::IString at +0xc (vtable plus shared empty-buffer word and … |
| `008dd5e0` | `ReportPacketReportPlayer::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: runs the destructor body 008d1e10 (destroys the IString and the 10-element report vector) and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
