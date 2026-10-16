# Class: std::basic_ostream<char,std::char_traits<char>_>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012719c0` | `— ⚠reject` | ⚪ | crt | Inserts a NUL-terminated char string into the stream: computes its length by scanning to NUL, reads the stream width from the ios block, builds a sentry with FUN_01270870, pads with the fill character (FUN_0126d230 per … |
| `01271b36` | `std::basic_ostream::SentryCleanup` | 🟢 | crt | Cleanup body reached from 012719c0 without its own prologue (uses the caller's EBP frame): when the saved flag at EBP-0x14 is nonzero, ORs it with the ios state word at the virtual-base offset +8 (adding 4 when the word… |
| `01271b90` | `std::basic_ostream::InsertNumeric` | 🟢 | crt | Builds a sentry with FUN_01270870 and, when it succeeds, takes the locale through FUN_0126b820 and FUN_012709e0, calls the num_put helper FUN_0126c850 with the ios fill byte at +0x30 and the single dword argument, sets … |
