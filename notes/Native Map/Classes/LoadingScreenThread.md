# Class: LoadingScreenThread

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ca8530` | `LoadingScreenThread::Ctor` | ✅ | ctor | Calls the thread base constructor 00fc4cc0 with the name string LoadingScreenThread and argument 2, installs LoadingScreenThread::vftable, stores the two arguments at +0xc4 and +0xc8, copies a dword from the third argum… |
| `00ca8660` | `LoadingScreenThread::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0 of LoadingScreenThread): calls base destructor 00fc4c30 on this, then frees this through _free when bit 0 of the flag argument is set; returns this. |
| `00cabcb0` | `LoadingScreenThread::Run` | ✅ | virtual | Pumps queued Windows messages, calls the loading-screen frame routine, sleeps 25 ms, and repeats until the stop flag is set. |
