# Class: std::num_put<char,std::ostreambuf_iterator<char,std::char_traits<char>_>_>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01271690` | `— ⚠reject` | ⚪ | crt | Takes the sprintf-formatted number text (char buffer and length in the last argument), computes a sign or 0x prefix length from its first character, reads the stream width at ios+0x18 and the adjustfield bits (mask 0x1c… |
| `01271de0` | `std::num_put::DoPutFmt_ld` | ✅ | crt | Virtual do_put override in vtable slot [7]: calls FUN_0126faf0 to build a printf format from the ios flags at +0x10 and the format tag at DAT_018cf79c (bytes 'ld'), formats the value with _sprintf_s into a 64-byte buffe… |
| `01271e80` | `std::num_put::DoPutFmt_lu` | ✅ | crt | Virtual do_put override in vtable slot [6]: same shape as 01271de0 with the format tag at DAT_018cf7a0 (bytes 'lu'): FUN_0126faf0 builds the format, _sprintf_s formats into a 64-byte buffer, and FUN_01271690 emits it. |
| `01271f20` | `std::num_put::DoPutFmt_Ld` | 🟢 | crt | Virtual do_put override in vtable slot [5]: same shape as 01271de0 with the format tag at DAT_018cf7a4 (bytes 'Ld'): FUN_0126faf0 builds the format, _sprintf_s formats into a 64-byte buffer, and FUN_01271690 emits it. |
| `01271fc0` | `std::num_put::DoPutFmt_Lu` | 🟢 | crt | Virtual do_put override in vtable slot [4]: same shape as 01271de0 with the format tag at DAT_018cf7a8 (bytes 'Lu'): FUN_0126faf0 builds the format, _sprintf_s formats into a 64-byte buffer, and FUN_01271690 emits it. |
| `01272060` | `— ⚠reject` | ⚪ | crt | Virtual do_put(double) override in vtable slot [3]: reads the precision at param+0x14 (defaults to 6 when that value is below 1 and flag bit 0x2000 is clear), selects the floatfield mode from flags bits 0x3000 (f or e),… |
| `01272240` | `— ⚠reject` | ⚪ | crt | Virtual do_put(long double) override in vtable slot [2]: same logic as DoPutDouble but the format string gets an 'L' length modifier (pcVar3[2] = 'L') before the f/e conversion character, then _sprintf_s formats the val… |
| `01272420` | `std::num_put::DoPutPointer` | ✅ | crt | Virtual do_put(const void*) override in vtable slot [1]: formats the pointer with _sprintf_s using the literal format "%p" into a 64-byte buffer and passes the text and length to FUN_01271690. |
