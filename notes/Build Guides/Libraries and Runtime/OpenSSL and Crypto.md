---
type: build-guide
section: Libraries and Runtime
updated: 2026-10-12
---

# OpenSSL and Crypto

Part of [[Build Guides]] › Libraries and Runtime. Related: [[Gateway and Transport]].

<!-- NARRATIVE START -->
## How it works

_Not written yet. An agent fills this from the shard findings, labels and video evidence: what the feature does, its data formats and packets, what the server must do, and open questions._
<!-- NARRATIVE END -->

## Mapped functions (generated)

359 accepted functions: 10 confirmed, 178 likely, 102 speculative, 69 unknown. Network-linked (`wire`): 0.

Subsystems: `lib.openssl.x509` (79), `lib.openssl.ec` (76), `lib.openssl` (45), `lib.openssl.bn` (31), `lib.openssl.crypto` (25), `lib.openssl.stack` (19), `lib.openssl.lhash` (13), `lib.openssl.bio` (12), `lib.openssl.evp` (12), `lib.openssl.buffer` (9), `lib.openssl.pem` (8), `lib.openssl.rand` (6), `lib.crypto.sha1` (5), `lib.crypto.md5` (4), `lib.openssl.engine` (4), `lib.random` (3), `lib.openssl.rsa` (3), `lib.openssl.dh` (3), `lib.openssl.asn1` (2)

_Large or library guide: only confirmed and likely functions are listed (188 of 359)._

### `lib.crypto.md5`

**(no class)** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01519f90` | Md5Transform | likely | Runs the MD5 compression rounds on the four state words at param_1+0x08..+0x14, reading 16 little-endian words from the block pointer in EAX (copied to a local… |
| `0151a6e0` | Md5Init | likely | Zeroes the two 32-bit bit-count words at +0 and +4 and stores the four MD5 initial state words 0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476 at +8 to +0x14. |
| `0151a710` | Md5Update | likely | Adds 8 times len to the 64-bit bit count at +0/+4; if a partial block is pending at +0x18 it fills it and transforms it when full; then transforms each whole 6… |
| `0151a7e0` | Md5Final | likely | Saves the bit count as 8 little-endian bytes, pads through 0151a710 with the pad table at 01912020 (0x80 then zeros until 56 mod 64), appends the saved count, … |

### `lib.crypto.sha1`

**(no class)** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01519900` | Sha1Init | likely | Writes the SHA-1 initial state 0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0 into dwords 0 to 4 of the context and zeroes dwords 5, 6, 0x17, 0x18 … |
| `01519940` | Sha1Transform | likely | Builds the SHA-1 message schedule from the 64-byte buffer at this+0x1c (big-endian words, extended with a rotate-left-by-1 XOR schedule) and runs the compressi… |
| `01519e20` | Sha1PadAndLength | likely | Writes the 0x80 pad byte at the buffer index, zero-fills to 56 bytes (running 01519940 first when the index is 56 or more), copies the bit-count bytes from +0x… |
| `01519f20` | Sha1Update | likely | When the length is non-zero: if the flag at +0x60 or +0x64 is set, sets +0x64 and returns; otherwise for each byte stores it at buffer index +0x5c, adds 8 to t… |

### `lib.openssl`

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f912f0` | X509VerifyErrorString | confirmed | Switch on the X509 verify error code (in EAX) that returns the matching name string: 0 gives OK, codes 2 to 0x16 and beyond give X509_V_ERR_* names, and any ot… |

### `lib.openssl.asn1`

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `011c1b50` | d2i_PrivateKey | likely | Decodes a private key of algorithm type param_1 from DER pointer param_3 with length param_4. Reuses or allocates the EVP_PKEY in *param_2 (011bb3d0), sets its… |

### `lib.openssl.bio`

**(no class)** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `011b2150` | BIO_new_mem_buf | likely | cdecl, 2 stack args (char *buf, int len); plain ret, EAX = new BIO or 0. buf NULL: error push 011afd10(0x20,0x7e,0x73,bss_mem.c,100) and returns 0. len < 0 is … |
| `011b21f0` | BIO_mem_free | likely | cdecl, 1 stack arg (BIO *b) at [esp+4]; plain ret, EAX = 1, or 0 for NULL. When b, its init flag (+0xc), its shutdown flag (+0x10) and ptr (+0x20) are all non-… |
| `011b2240` | BIO_mem_read | likely | cdecl, 3 stack args (BIO *b, void *out, int outl); plain ret. Clears the retry flags with 011b0700(b,0xf). Count = outl when outl < 0 or outl <= bm->length, el… |
| `011b22d0` | BIO_mem_write | likely | cdecl, 3 stack args (BIO *b, void *in, size_t inl); returns EAX = inl, or 0xffffffff for NULL in or a read-only BIO (flag 0x200). Otherwise clears the retry fl… |
| `011b6b10` | sock_free | likely | cdecl, 1 stack arg (BIO *b); EAX = 1, or 0 for NULL. When b->shutdown (+0x10) is set: if b->init (+0xc) is set, calls shutdown(fd, 2) and closesocket(fd) with … |
| `011c1470` | file_read | likely | Returns 0 when the BIO's init flag (+0xc) is clear or param_2 is null. Otherwise calls fread (00d8ae0b) for param_3 items of one byte from the BIO's FILE (+0x2… |
| `011c18c0` | BIO_new_file | likely | Opens file param_1 with mode param_2 on Windows: converts both names to wide strings (MultiByteToWideChar, code page 0xfde9) and opens with __wfopen, falling b… |

### `lib.openssl.bn`

**(no class)** (23)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `011c4080` | BN_num_bits_word | likely | Returns the bit length of a 32-bit word using a 256-byte table at DAT_018ae318: adds 0, 8, 16 or 24 to the table entry for the highest nonzero byte of the word… |
| `011c40d0` | BN_num_bits | likely | Returns 0 when the BIGNUM top (+4) is 0. Otherwise returns the bit length of the top word d[top-1] (011c4080) plus (top-1)*32, computed with a 32-bit shift. Dw… |
| `011c4160` | BN_free | likely | Returns when param_1 is null. Otherwise frees the word array (+0) with 011b73c0 unless the static-data flag (bit 2 of +0x10) is set. If the MALLOCED flag (bit … |
| `011c41c0` | BN_new | likely | Allocates a 0x14-byte BIGNUM (011b71d0, bn_lib.c line 0x12e). On failure reports ERR lib 3 function 0x71 reason 0x41 at line 0x130 and returns 0. Otherwise set… |
| `011c4350` | BN_copy | likely | Copies BIGNUM param_2 into param_1 and returns param_1. Returns immediately when both are the same object. Grows the destination with 011c4310 when its max (+8… |
| `011c4470` | BN_clear | likely | Zeroes the word array (+0, max count at +8 times 4 bytes) with _memset when set, then sets the neg flag (+0xc) and top count (+4) to 0. Returns nothing. |
| `011c44c0` | BN_set_word | likely | Stores the word param_2 as the BIGNUM value: when max (+8) is below 1 it grows the array to one word with 011c4210 (BN_expand2), frees the old array with 011b7… |
| `011c4520` | BN_bin2bn | likely | Converts param_2 big-endian bytes at param_1 into the BIGNUM param_3 (a new one from 011c41c0 when param_3 is null, freed with 011c4160 on growth failure). A z… |
| `011c4610` | BN_bn2bin | likely | Writes the BIGNUM param_1 as big-endian bytes into param_2 and returns the byte count. The bit length comes from the top word (011c4080) plus (top-1)*32, and t… |
| `011c4690` | BN_ucmp | likely | Compares the magnitudes of param_1 and param_2 ignoring sign. Different top counts (+4) decide the result. Equal counts are compared word by word from the top … |
| `011c4780` | BN_set_bit | likely | Sets bit param_2 of the BIGNUM param_1. A negative index returns 0. When the word index (param_2/32) is at or past top, grows the array with 011c4310 (bn_wexpa… |
| `011c4880` | BN_is_bit_set | likely | Returns bit param_2 of the BIGNUM param_1 as 0 or 1. A negative index or a word index at or beyond top (+4) returns 0. |
| `011c48c0` | BN_mask_bits | likely | Keeps the low param_2 bits of the BIGNUM param_1 and clears the rest. A negative count returns 0. When the word index is below top (+4): with a zero bit remain… |
| `011c4940` | BN_set_negative | likely | Sets the neg flag (+0xc) of the BIGNUM param_1 to 1 only when param_2 is nonzero and the value is nonzero (top at +4 nonzero); otherwise stores 0. Returns noth… |
| `011c4970` | bn_cmp_words | likely | Compares param_3 words of two word arrays (param_1 and param_2) from the top word down, starting with the top word at index param_3-1. Returns -1 or 1 on the f… |
| `011c4a60` | BN_dup | likely | Returns null when param_1 is null. Otherwise creates a new BIGNUM (011c41c0), copies param_1 into it with BN_copy (011c4350), and returns it. On copy failure i… |
| `011c4c30` | BnCtxFreePools | likely | Walks the linked list of BN_CTX pools (next pointer at +0x144 of each pool, read from the first pool at the structure's first dword). For each pool it clears e… |
| `011c4cd0` | BnCtxSlotAlloc | likely | Returns the next free BIGNUM slot (the BN_CTX_get internal step) of the BN_CTX held in ESI (Ghidra unaff_ESI), advancing its used count (+0xc). When the curren… |
| `011c4de0` | BN_CTX_new | likely | Allocates a 0x2c-byte BN_CTX (011b71d0, bn_ctx.c line 0xd8). On failure reports ERR lib 3 function 0x6a reason 0x41 at line 0xdb and returns null. Otherwise ze… |
| `011c4e40` | BN_CTX_free | likely | Returns when param_1 is null. Otherwise frees the BN_CTX stack array (+0x14) with 011b73c0 when its capacity (+0x1c) is nonzero, releases the pool chain with 0… |
| `011c4e70` | BN_CTX_start | likely | When the BN_CTX counters at +0x24 and +0x28 are both zero, pushes the current used-count value (+0x20) with 011c4b60 (BnCtxStackPush). A push failure reports E… |
| `011c4ec0` | BN_CTX_end | likely | Ends a BN_CTX scope: when the nested-scope counter at +0x24 is nonzero it is decremented and the function returns. Otherwise it decrements the pool index at +0… |
| `011c4f00` | BN_CTX_get | likely | When the BN_CTX depth counters at +0x24 and +0x28 are both zero, takes a slot with 011c4cd0 (BnCtxSlotAlloc), zeroes it with 011c44c0 (BN_set_word 0), incremen… |

### `lib.openssl.buffer`

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `011bee70` | BUF_MEM_new | likely | Allocates a 0xc-byte BUF_MEM (011b71d0 with file buffer.c line 0x43) and zeroes its three dwords (length, data, max). On allocation failure it reports ERR lib … |
| `011bf090` | BUF_memdup | likely | Returns null when param_1 is null. Otherwise allocates param_2 bytes (011b71d0, buffer.c line 0xc1), copies param_2 bytes from param_1 with _memcpy and returns… |
| `011bf210` | BUF_strndup | likely | Returns null when param_1 is null. Otherwise allocates param_2+1 bytes (011b71d0, buffer.c line 0xb1), copies up to param_2 characters with BUF_strlcpy (011bf0… |

### `lib.openssl.crypto`

**(no class)** (18)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `011b55e0` | OpenSSLShowFatal | confirmed | cdecl, variadic (format and arguments on the stack); no result. If stderr (GetStdHandle(-12)) is a valid file or console handle, formats through 00d8e276/00d8e… |
| `011b5700` | OpenSSLDie | confirmed | cdecl, 3 stack args (file, line, expression); does not return. Calls 011b55e0 (OpenSSLShowFatal) with the format string 'OpenSSL internal error, assertion fail… |
| `011b5740` | CRYPTO_lock | confirmed | cdecl, 4 stack args (mode, type, file, line); no result. If type < 0 (dynamic lock): if the create/lock hook DAT_01cbc42c is NULL returns; otherwise gets the l… |
| `011b5070` | CRYPTO_destroy_dynlockid | likely | cdecl, 1 stack arg (i); no result. If the destroy hook DAT_01cbc430 is set: takes the lock 011b5740(9,0x1d,cryptlib.c,0x131). With index -1-i in the dynamic-lo… |
| `011b5140` | CryptoDynlockGetValueRef | likely | cdecl, 1 stack arg (i); returns EAX = the dynamic-lock value (entry+4) for index -1-i, or 0. Under the lock 011b5740(9,0x1d,cryptlib.c,0x15a) it increments the… |
| `011b51f0` | CRYPTO_set_dynlock_create_callback | likely | cdecl, 1 stack arg (cb) at [esp+4]; plain ret, no result. Stores cb in the global DAT_01cbc428 (the dynamic-lock create hook). 10 bytes, no calls. |
| `011b5200` | CRYPTO_set_dynlock_lock_callback | likely | cdecl, 1 stack arg (cb); plain ret, no result. Stores cb in DAT_01cbc42c, the dynamic-lock lock/unlock hook that 011b5740 (CRYPTO_lock) calls for dynamic lock … |
| `011b5210` | CRYPTO_set_dynlock_destroy_callback | likely | cdecl, 1 stack arg (cb); plain ret, no result. Stores cb in DAT_01cbc430, the dynamic-lock destroy hook: 011b5070 calls it as (lock value, file, line) before f… |
| `011b5240` | CRYPTO_set_locking_callback | likely | cdecl, 1 stack arg (cb); plain ret, no result. Stores cb in DAT_01cbc418, the static lock hook that 011b5740 (CRYPTO_lock) calls for static lock types. 10 byte… |
| `011b52b0` | CRYPTO_THREADID_current | likely | cdecl, 1 stack arg (CRYPTO_THREADID *id); no result. If the thread-id callback DAT_01cbc424 is set it is called with the id (tail indirect call, no fields writ… |
| `011b52f0` | CRYPTO_THREADID_cmp | likely | cdecl, 2 stack args (a, b) of 8-byte CRYPTO_THREADID values; EAX in {-1,0,1}. Compares the two 8-byte structures dword by dword then byte by byte (memcmp of si… |
| `011b57c0` | CRYPTO_add_lock | likely | cdecl, 5 stack args (int *pointer, int amount, int type, file, line); EAX = the new value. If the add hook DAT_01cbc41c is set it is entered with the caller's … |
| `011b7120` | CRYPTO_malloc_locked | likely | cdecl, 3 stack args (num, file, line); EAX = the block or 0. Same shape as CRYPTO_malloc (011b71d0): num < 1 returns 0, global _DAT_01bae308 = 0, debug hook DA… |
| `011b7190` | CRYPTO_free_locked | likely | cdecl, 1 stack arg (ptr); no result. Calls the free debug hook DAT_01cbc450 as (ptr, 0) when set, frees with the locked free hook PTR__free_01bae32c, then call… |
| `011b71d0` | CRYPTO_malloc | likely | cdecl, 3 stack args (num, file, line); EAX = the block or 0. Returns 0 when num < 1. Sets the global _DAT_01bae308 = 0. When the debug hook DAT_01cbc448 is set… |
| `011b7290` | CRYPTO_realloc | likely | cdecl, 4 stack args (ptr, num, file, line); EAX = the new block or 0. A NULL ptr delegates to CRYPTO_malloc (011b71d0) with (num, file, line). num < 1 returns … |
| `011b7310` | CRYPTO_clear_realloc | likely | cdecl, 5 stack args (old ptr, old len, new size, file, line); EAX = new block or 0. A NULL old pointer calls 011b71d0 (CRYPTO_malloc) with (size, file, line). … |
| `011b73c0` | CRYPTO_free | likely | cdecl, 1 stack arg (ptr); no result. Calls the free debug hook DAT_01cbc450 as (ptr, 0) when set, frees with the free hook PTR__free_01bae320, then calls the h… |

### `lib.openssl.dh`

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `011c6690` | DH_generate_key | likely | Generates a DH key pair for the DH object param_1. Creates a BN_CTX (011c4de0) and a private key BIGNUM when missing (011c41c0) and a public key BIGNUM when mi… |
| `011c6820` | DH_compute_key | likely | Computes the shared secret from the peer public key param_2 with DH object param_3, writing it to param_1 as big-endian bytes. Rejects a prime above 10000 bits… |

### `lib.openssl.ec`

**(no class)** (30)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `011c2ea0` | EC_KEY_new | likely | Allocates a 0x20-byte EC key block (011b71d0, ec_key.c line 0x49). On failure it reports ERR lib 0x10 function 0xb6 reason 0x41 at line 0x4c and returns null. … |
| `011c2f00` | EC_KEY_free | likely | Releases an EC key: when non-null, decrements the reference count at +0x18 with 011b57c0 (CRYPTO lock 0x21, ec_key.c line 0x6f); when the count drops below 1 i… |
| `011c2f70` | EC_KEY_copy | likely | Copies EC key param_2 into param_1 and returns param_1. Null arguments report ERR lib 0x10 function 0xb2 reason 0x43 at ec_key.c line 0x90 and return null. Rep… |
| `011c30d0` | EC_KEY_dup | likely | Creates a new EC key with 011c2ea0 (EC_KEY_new), copies param_1 into it with 011c2f70 (EC_KEY_copy) and returns the new key. If the copy fails it releases the … |
| `011c3100` | EC_KEY_up_ref | likely | Increments the reference count at +0x18 with CRYPTO_add (011b57c0, amount 1, lock 0x21, ec_key.c line 0xdb) and returns 1 when the new count is greater than 1,… |
| `011c3130` | EC_KEY_generate_key | likely | Generates a key pair for the EC key param_1, which must have a group (+4); otherwise reports ERR lib 0x10 function 0xb3 reason 0x43 at ec_key.c line 0xf2 and r… |
| `011c3270` | EC_KEY_check_key | likely | Checks EC key param_1 (group at +4, public point at +8, optional private key at +0xc). Missing group or point reports ERR lib 0x10 function 0xb1 reason 0x43 at… |
| `011c3490` | EC_KEY_set_group | likely | Releases the group in the +4 slot with 011c6080 when set, then stores 011c6350 applied to param_2 in that slot and returns whether the stored value is nonzero. |
| `011c34d0` | EC_KEY_set_private_key | likely | Releases the BIGNUM in the +0xc slot with 011c4100 when set, then stores 011c4a60 applied to param_2 in that slot and returns whether the stored value is nonze… |
| `011c3510` | EC_KEY_set_public_key | likely | Releases the point in the +8 slot with 011c55f0 when set, then stores 011c56d0 applied to param_2 and the group at +4 in that slot, and returns whether the sto… |
| `011c5060` | EC_GROUP_get_order | likely | Copies the BIGNUM at param_1+8 (the group order, read by EC_KEY_generate_key and EC_KEY_check_key) into param_2 with BN_copy (011c4350) and returns whether the… |
| `011c5120` | EC_GROUP_set_seed | likely | Replaces the group seed: frees the old buffer at +0x3c with 011b73c0 when set and clears it and its length (+0x40). When param_3 is nonzero and param_2 is non-… |
| `011c51b0` | EC_GROUP_set_curve | likely | Dispatches to the group method's set-curve slot (+0x14 of the method at +0 of param_1) with the same arguments and returns its result. When that slot is null i… |
| `011c51e0` | EC_GROUP_get_curve | likely | Dispatches to the group method's get-curve slot (+0x18 of the method at +0 of param_1) with the same arguments and returns its result. When that slot is null i… |
| `011c5320` | EC_EX_DATA_set_data | likely | Adds an ex-data entry to the list at param_1 (the head pointer). Returns 0 for a null list. Walks the list comparing the three callback pointers (param_3, para… |
| `011c53c0` | EC_EX_DATA_get_data | likely | Walks the ex-data list from param_1 and returns the data value (+4) of the first node whose three callback pointers (+8, +0xc, +0x10) equal param_2, param_3 an… |
| `011c5400` | EC_EX_DATA_free_data | likely | Finds the ex-data node whose three callback pointers (+8, +0xc, +0x10) match param_2, param_3 and param_4, calls the free callback at +0xc with the node's data… |
| `011c54c0` | EC_EX_DATA_free | likely | Frees a whole ex-data list at param_1: for each node calls its free callback (+0xc) with the node's data (+4), frees the node with 011b73c0 and moves on; then … |
| `011c5500` | EC_EX_DATA_clear_free | likely | Releases a whole ex-data list at param_1: for each node calls its clear-free callback (+0x10) with the node's data (+4), frees the node with 011b73c0, and cont… |
| `011c5540` | EC_POINT_new | likely | Returns null when the group param_1 is null (ERR lib 0x10 function 0x79 reason 0x43 at ec_lib.c line 0x2bb... the null-group report uses line 699) or when its … |
| `011c55f0` | EC_POINT_free | likely | Returns when param_1 is null. Otherwise calls the method's point_finish slot (+0x28 of the method at +0 of the point) with the point when that slot is set, the… |
| `011c5620` | EC_POINT_clear_free | likely | Returns when param_1 is null. Otherwise calls the method's point_clear_finish slot (+0x2c) with the point when set, else its point_finish slot (+0x28) when set… |
| `011c5660` | EC_POINT_copy | likely | Copies point param_2 into param_1 through the method's point_copy slot (+0x30). A missing slot reports ERR lib 0x10 function 0x72 reason 0x42 at ec_lib.c line … |
| `011c56d0` | EC_POINT_dup | likely | Returns null when param_1 (the source point) is null. Otherwise creates a new point for the group param_2 (011c5540, EC_POINT_new), copies the source into it w… |
| `011c6080` | EC_GROUP_free | likely | Returns when param_1 is null. Otherwise calls the group's finish slot (+8 of the method) when set, releases the ex-data list at +0x44 with 011c54c0 (EC_EX_DATA… |
| `011c6100` | EC_GROUP_clear_free | likely | Returns when param_1 is null. Otherwise calls the group's clear-finish slot (+0xc of the method), or else its finish slot (+8), then releases the ex-data list … |
| `011c6190` | EC_GROUP_copy | likely | Copies group param_2 into param_1. A method without a group_copy slot (+0x10) reports ERR lib 0x10 function 0x6a reason 0x42 at ec_lib.c line 0xb1; differing m… |
| `011c6350` | EC_GROUP_dup | likely | Returns null when param_1 is null. Otherwise creates a new group with the first dword of the source as method (011c4f60, EC_GROUP_new), copies the source into … |
| `011c6390` | EC_GROUP_set_generator | likely | Sets the group generator. A null param_2 reports ERR lib 0x10 function 0x6f reason 0x43 at ec_lib.c line 0x120 and returns 0. Creates the generator point at +4… |
| `011c6440` | EC_GROUP_cmp | likely | Compares two groups with an optional BN_CTX (created with 011c4de0 when param_3 is null, failure returns -1). Returns 1 when the group methods differ (first dw… |

### `lib.openssl.engine`

**(no class)** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `011b7470` | engine_unlocked_init | likely | cdecl, 1 stack arg (ENGINE *e); EAX = 1 or 0. When the functional reference count (+0x5c) is zero and the init callback (+0x38) is set, calls it; a zero result… |
| `011b74a0` | engine_unlocked_finish | likely | cdecl, 2 stack args (ENGINE *e, int unlock_for_hooks); EAX = 1 or 0. Decrements the functional count (+0x5c). When it reaches zero and the finish hook (+0x3c) … |
| `011b7530` | ENGINE_init | likely | cdecl, 1 stack arg (ENGINE *e); EAX = 1 or 0. e NULL reports error 0x26/0x77/0x43 at eng_init.c line 0x7e and returns 0. Otherwise takes the engine lock (011b5… |
| `011b75b0` | ENGINE_finish | likely | cdecl, 1 stack arg (ENGINE *e); plain ret, EAX = 1 or 0. e==NULL: error push 011afd10(0x26,0x6b,0x43,eng_init.c,0x8e) and returns 0. Otherwise takes the engine… |

### `lib.openssl.evp`

**(no class)** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `011bf4b0` | EVP_DigestFinal_ex | confirmed | Given context param_1, output param_2 and an optional size pointer param_3: asserts the digest md_size is at most 0x40 (file digest.c line 0xfa, failure throug… |
| `011bf2c0` | EVP_MD_CTX_create | likely | Allocates 0x18 bytes (011b71d0, file digest.c line 0x7f). If the allocation succeeds it zeroes the six dwords and returns the pointer; a null result is returne… |
| `011bf2f0` | EVP_DigestInit_ex | likely | Initialises the EVP_MD_CTX param_1 with digest param_2 and optional engine param_3. Clears the CLEANED flag (011c86a0 with 2). If the context has no digest or … |
| `011bf530` | EVP_MD_CTX_cleanup | likely | Releases a digest context param_1: if the digest is set and its cleanup slot (+0x20) is non-null, calls it unless the flag test 011c86b0 (flag 2) is set; if md… |
| `011bf610` | EVP_DigestFinal | likely | Calls EVP_DigestFinal_ex (011bf4b0) with the context, output and size pointer, then EVP_MD_CTX_cleanup (011bf530) on the same context, and returns the first ca… |
| `011bf640` | EVP_MD_CTX_copy_ex | likely | Copies digest context param_2 into param_1. Fails with ERR lib 6 function 0x6e reason 0x6f when the source or its digest is null. Takes the source engine refer… |
| `011bf7a0` | EVP_Digest | likely | One-shot digest: zeroes a local 0x18-byte context, sets flag 1 (ONESHOT, 011c8690), then EVP_DigestInit_ex (011bf2f0) with the type and engine arguments. On su… |
| `011bf850` | EVP_MD_CTX_destroy | likely | Cleans up the digest context param_1 with 011bf530 (EVP_MD_CTX_cleanup) and then frees the context block with 011b73c0. Returns nothing. |
| `011bf870` | EVP_MD_CTX_copy | likely | Zeroes the six dwords of the destination context param_1, then tail-jumps to EVP_MD_CTX_copy_ex (011bf640) with the same destination and the source argument. R… |
| `011c36f0` | EVP_SignFinal | likely | Clears the signature length (*param_3 = 0), copies the context into a temporary EVP_MD_CTX (011bf2a0, 011bf640), finalises the copy into a 64-byte digest with … |

### `lib.openssl.lhash`

**(no class)** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `011b62b0` | lh_new | confirmed | stack args param_1 (hash fn), param_2 (comparator). Allocates the 0x60-byte LHASH with 011b71d0 (lhash.c line 0x77) and 16 bucket slots (0x40 bytes, line 0x79)… |
| `011b5f40` | lh_free | likely | cdecl, 1 stack arg (LHASH *lh); no result. When lh is non-NULL walks every bucket (lh+0 array, count lh+0xc) and frees each chain node with 011b73c0 following … |
| `011b5f90` | doall_util_fn | likely | cdecl with EBX as a register input (use_arg flag: 0 = plain callback, nonzero = callback with argument), stack args (lh, func, func_arg, arg). Walks the bucket… |
| `011b5ff0` | lh_doall | likely | cdecl, 2 stack args (lh, func); no result. For each bucket from the last to the first, walks its chain and calls func(item) for each node in order (the plain f… |
| `011b6030` | lh_doall_arg | likely | cdecl, 3 stack args (lh, func_arg, arg); no result. Calls 011b5f90 (doall_util_fn) with (lh, func = NULL, func_arg, arg), so the walk uses the argument-passing… |
| `011b6140` | lh_contract | likely | register input ESI = LHASH *lh; no stack args; no result. Decrements the split index p (+0x14) or, when p is 0, halves num_alloc_nodes (+0x10) and pmax (+0x18)… |
| `011b6250` | lh_strhash | likely | stack arg param_1 (char *); EAX = 32-bit string hash. Returns 0 for NULL or an empty string. Otherwise for each byte: v = 0x100 \| byte (the counter n steps by… |
| `011b62a0` | lh_num_items | likely | stack arg param_1 (LHASH *lh); EAX = lh->num_items (+0x24), or 0 when lh is NULL. |

### `lib.openssl.pem`

**(no class)** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `011c0e30` | PEM_read_bio_PrivateKey | likely | Reads a PEM block from BIO param_1 through 011d06f0 with the name ANY PRIVATE KEY (password callback param_3 and user data param_4). If the block name is PRIVA… |
| `011c1280` | PemWriteParameters | likely | If the key object at param_2+0xc has a method (+0x3c set), formats the PEM name as the method's name (+0xc) followed by ' PARAMETERS' into an 80-byte stack buf… |
| `011c2c60` | PEM_read_bio_X509 | likely | Forwards its four arguments to 011cefa0 with the decoder 011ae810 and the PEM block name CERTIFICATE as the first two arguments (PEM ASN.1 read shape). Returns… |
| `011c2cc0` | PEM_write_bio_X509 | likely | Writes its two arguments through 011d0070 (PEM ASN.1 write) with the encoder 011ae830 and the PEM block name CERTIFICATE, with no cipher, key string or callbac… |
| `011c2d20` | PEM_read_bio_X509_AUX | likely | Forwards its four arguments to 011cefa0 with the decoder at 011ae8f0 and the PEM block name TRUSTED CERTIFICATE as the first two arguments. Returns the call's … |

### `lib.openssl.rand`

**(no class)** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `011b1df0` | RAND_set_rand_method | likely | cdecl, 1 stack arg (RAND_METHOD *meth) at [esp+4]; plain ret, EAX = 1 always. If the cached engine DAT_01cbc398 is non-NULL: calls 011b75b0 (ENGINE_finish) on … |
| `011b1f60` | RAND_seed | likely | no stack args of its own (arguments forwarded): when the cached method DAT_01cbc39c is NULL it resolves one: ENGINE default lookup 011da750 (if an engine is fo… |
| `011b1fb0` | RAND_add | likely | cdecl, 3 stack args (buf, num, double entropy); the entropy is an 8-byte value stored with fstp into the outgoing frame; ends with add esp,0x10 and plain ret. … |
| `011b2020` | RAND_bytes | likely | no stack args of its own (arguments forwarded): resolves the active RAND method as 011b1f60 does and caches it in DAT_01cbc39c/398. If the method's bytes slot … |
| `011b2080` | RAND_pseudo_bytes | likely | no stack args of its own (arguments forwarded): resolves the active RAND method as 011b1f60 does and caches it in DAT_01cbc39c/398. If the pseudorand slot at +… |
| `011b20e0` | RAND_status | likely | no stack args of its own: resolves the active RAND method as 011b1f60 does and caches it in DAT_01cbc39c/398. If the status slot at +0x14 is NULL returns 0; ot… |

### `lib.openssl.rsa`

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `011c3900` | RSA_sign | likely | Signs a digest param_2 of length param_3 for digest type param_1 with RSA key param_6 into param_4 and writes the length to *param_5. If the key's method sign … |
| `011c3b10` | int_rsa_verify | likely | Verifies a PKCS#1 v1.5 signature. param_7 (the signature length) must equal the RSA size (011bd2e0), else ERR lib 4 function 0x91 reason 0x77 at rsa_sign.c lin… |
| `011c3f00` | RSA_verify | likely | If the key's flag byte (+0x3c) has bit 0x40 and its method's verify slot (+0x30) is set, calls that slot with the same six arguments. Otherwise calls 011c3b10 … |

### `lib.openssl.stack`

**(no class)** (17)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `011b5a00` | sk_new | confirmed | cdecl, 1 stack arg (comp) at [esp+4]; returns EAX = new stack or 0. Allocates the 0x14-byte header with 011b71d0 (CRYPTO_malloc, stack.c line 0x7d) and a 0x10-… |
| `011b5a70` | sk_insert | confirmed | cdecl, 3 stack args (STACK *st, void *data, int loc); returns EAX = new count, or 0 when st is NULL or the grow fails. Grows the array when count+1 >= capacity… |
| `011b5b20` | sk_delete | likely | cdecl, 2 stack args (STACK *st, int loc); returns EAX = the removed item, or 0 when st is NULL or loc is out of range. Saves item loc, shifts items loc+1..coun… |
| `011b5bb0` | sk_shift | likely | cdecl, 1 stack arg (STACK *st); returns EAX = the first item (data[0]) or 0 when st is NULL or empty. Shifts items 1..count-1 down by one slot and decrements c… |
| `011b5c00` | sk_pop | likely | cdecl, 1 stack arg (STACK *st); returns EAX = the last item (via sk_delete) or 0 when st is NULL or empty. |
| `011b5c20` | sk_zero | likely | cdecl, 1 stack arg (STACK *st); no result. When st is non-NULL and count > 0 zeroes the item array (count dwords) with memset and sets count = 0. |
| `011b5c50` | sk_free | likely | cdecl, 1 stack arg (STACK *st); no result. When st is non-NULL frees the item array (st+4) with 011b73c0 when it is set, then frees st itself with 011b73c0. |
| `011b5c80` | sk_num | likely | cdecl, 1 stack arg (STACK *st); EAX = st->count, or -1 when st is NULL. No calls. |
| `011b5c90` | sk_value | likely | cdecl, 2 stack args (STACK *st, int i); EAX = st->data[i] when st is non-NULL and 0 <= i < count, else 0. No calls. |
| `011b5cb0` | sk_set | likely | cdecl, 3 stack args (STACK *st, int i, void *data); EAX = data when st is non-NULL and 0 <= i < count (the slot is overwritten), else 0. No calls. |
| `011b5ce0` | sk_sort | likely | cdecl, 1 stack arg (STACK *st); no result. When st is non-NULL and not yet sorted (+8 == 0), calls _qsort on the item array with 4-byte items and the comparato… |
| `011b5d10` | sk_is_sorted | likely | cdecl, 1 stack arg (STACK *st); EAX = 1 when st is NULL, otherwise the sorted flag at st+8. |
| `011b5d30` | sk_dup | likely | cdecl, 1 stack arg (STACK *st); EAX = a new stack copy or 0. Creates the header with 011b5a00(st->comp at +0x10). Allocates the item array with 011b7290 (CRYPT… |
| `011b5dc0` | sk_new_null | likely | cdecl, no args; plain ret. Tail-calls the body of sk_new with a NULL comparator: calls 011b5a00(0) and returns its EAX (the new stack or 0). |
| `011b5dd0` | sk_delete_ptr | likely | cdecl, 2 stack args (STACK *st, void *ptr); EAX = the removed item, or 0 when not found. Scans the array for the first item equal to ptr; removes it by shiftin… |
| `011b5e30` | internal_find | likely | stack ESI = STACK (register input, not a stack argument); cdecl args: [esp+4] = data, [esp+8] = ret_val_options. With no comparator (st+0x10 NULL) scans linear… |
| `011b5ef0` | sk_pop_free | likely | cdecl, 2 stack args (STACK *st, void (*free_func)(void *)); no result. For each non-NULL item calls free_func(item) in index order, then frees the item array (… |

### `lib.openssl.x509`

**(no class)** (39)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `011b1560` | X509_NAME_ENTRY_create_by_txt | confirmed | cdecl, 5 stack args (X509_NAME_ENTRY **ne, field, type, bytes, len); plain ret, EAX returns the entry or 0. Calls 011bb110(field, 0) (OBJ_txt2obj with no_name=… |
| `011b25c0` | X509_verify_cert_error_string | confirmed | cdecl, 1 stack arg (error code n); returns a char* in EAX. A switch on n (jump table; the case targets are the 1.0.0g X509_V_ERR_* strings): 0 returns the lite… |
| `011b1460` | X509_NAME_ENTRY_create_by_OBJ | likely | cdecl, 5 stack args (X509_NAME_ENTRY **ne, obj, type, bytes, len); plain ret, result in EAX. Uses *ne when non-null, else allocates a new entry via 011b76f0 (A… |
| `011b14d0` | X509_NAME_get_text_by_NID | likely | cdecl, 4 stack args (name, nid, buf, len); plain ret of the target. Converts nid with 011ba4f0 (OBJ_nid2obj). A NULL object returns -1. Otherwise overwrites it… |
| `011b1680` | X509_NAME_add_entry_by_txt | likely | cdecl, 7 stack args (X509_NAME *name, field, type, bytes, len, loc, set); plain ret, EAX is the add result or 0. Calls 011b1560(0, field, type, bytes, len) to … |
| `011b16d0` | X509_issuer_and_serial_hash | likely | cdecl, 1 stack arg (X509 *x); returns the hash in EAX (not void). Initialises an EVP_MD_CTX on the stack (011bf2a0), takes the issuer text from 011d8800(x->cer… |
| `011b1850` | X509_get_subject_name | likely | cdecl, 1 stack arg (X509 *x) at [esp+4]; plain ret, EAX = x->cert_info->subject. Loads the cert_info pointer at x+0 and returns the dword at cert_info+0x14. No… |
| `011b1870` | X509_cmp | likely | cdecl, 2 stack args (X509 *a, X509 *b) at [esp+4],[esp+8]; plain ret, EAX in {-1,0,1}. Calls 011bc9d0(a,-1,0) and 011bc9d0(b,-1,0) (X509_check_purpose with id … |
| `011b1920` | X509_NAME_cmp | likely | cdecl, 2 stack args (X509_NAME *a, X509_NAME *b); plain ret, EAX in {-2,-1,0,1}. For each argument, if canon_enc at +0xc is NULL or modified at +4 is set, call… |
| `011b1a90` | X509_NAME_hash_old | likely | cdecl, 1 stack arg (X509_NAME *x); returns the hash in EAX (not void). Calls 011b7790(x,0) (i2d_X509_NAME refresh), loads BUF_MEM *x->bytes at +8, gets a diges… |
| `011b1b70` | X509_get_pubkey | likely | cdecl, 1 stack arg (X509 *x) at [esp+4]; plain ret, EAX. If x and x->cert_info (at x+0) are non-NULL, replaces the arg with cert_info+0x18 (the key field) and … |
| `011b1bb0` | X509_check_private_key | likely | cdecl, 2 stack args (X509 *x, EVP_PKEY *k); plain ret, AL = 1 only when the comparison result is 1. Gets the public key with 011d2f90(x->cert_info->key) (X509_… |
| `011b1cb0` | X509_subject_name_cmp | likely | cdecl, 2 stack args (X509 *a, X509 *b). Replaces each arg with its cert_info->subject (cert_info at +0, subject at +0x14) and tail-jumps to 011b1920 (X509_NAME… |
| `011b1cd0` | X509_CRL_cmp | likely | cdecl, 2 stack args (X509_CRL *a, X509_CRL *b). Replaces each arg with its crl->issuer (crl_info at +0, issuer at +8) and tail-jumps to 011b1920 (X509_NAME_cmp… |
| `011b2830` | find_issuer | likely | cdecl-style with 2 stack args: [esp+4] = ctx (X509_STORE_CTX), [esp+8] = x (copied into EBP). The candidate stack is in EBX on entry (register input; sk_num(EB… |
| `011b2880` | check_issued | likely | cdecl, 3 stack args (ctx=[esp+4], x=[esp+8], issuer=[esp+0xc]); plain ret, EAX = 1 when issued or when the issuer error is not reported, 0 when reporting is of… |
| `011b2b60` | check_name_constraints | likely | plain ret, EAX = 1 when all constraints pass, 0 on a reported violation. The X509_STORE_CTX is in ESI on entry (register input). Walks the chain from the top d… |
| `011b2f00` | check_crl_chain | likely | fastcall: ECX = stack A on entry (kept in EDI), ESI = stack B (register input). Returns 1 when the last item of A and the last item of B (sk_value at sk_num-1 … |
| `011b31d0` | CertRevocationStatusCheck | likely | cdecl, 3 stack args (ctx=param_1, item=param_2, extra=param_3); plain ret, EAX = 1 (ok), 2 (entry with reason 8, removeFromCRL), or 0 (callback refused). If it… |
| `011b3250` | check_policy | likely | cdecl, 1 stack arg (ctx) at [esp+4]; plain ret, EAX = 1 or 0 (or the callback result). When ctx+0x7c (policy tree) is NULL: calls 011dc880(ctx+0x58, ctx+0x5c, … |
| `011b3500` | X509_STORE_CTX_get_ex_new_index | likely | cdecl, 5 stack args forwarded unchanged to 011b9520 with a leading constant 5 (dis: push 5 then the five args; plain ret). Returns the callee's EAX. |
| `011b37f0` | X509_STORE_CTX_cleanup | likely | cdecl, 1 stack arg (ctx); no result. Calls the cleanup hook at ctx+0x48 when set. If ctx+0x14 (param) is non-NULL: frees it with 011b6570 (X509_VERIFY_PARAM_fr… |
| `011b4000` | check_cert_time | likely | plain ret, EAX = 1 or 0. Register inputs: ESI = X509_STORE_CTX, EDI = certificate X509. Compares the notBefore ASN1_TIME (validity at cert_info+0x10, notBefore… |
| `011b4d50` | check_revocation | likely | cdecl-style, 1 stack arg (ctx) at [esp+4]; plain ret, EAX = 1 or 0. Returns 1 unless the verify flags (ctx->param +0x14, byte +0x14) have bit 0x4 (CRL check). … |
| `011b6500` | X509_VERIFY_PARAM_new | likely | cdecl, no args; EAX = a new 0x28-byte verify parameter block or NULL when allocation fails (the code zeroes the fields before the NULL test). Allocates with 01… |
| `011b6570` | X509_VERIFY_PARAM_free | likely | cdecl, 1 stack arg (param block); no result. When non-NULL zeroes the fields (+0, +0x18, +0x1c, +0x10, +0x14 cleared and depth +0x20 = -1), frees the policy st… |
| `011b66e0` | X509_VERIFY_PARAM_set1_policies | likely | cdecl, 2 stack args (param, policies stack); EAX = 1 on success, 0 on failure, NULL param = 0. Frees the existing policy stack at +0x24 with 011b5ef0 (sk_pop_f… |
| `011b6900` | X509_VERIFY_PARAM_lookup | likely | cdecl, 1 stack arg (name char *); EAX = the matching verify parameter entry or NULL. Puts the name in a local key. If the user table DAT_01cbc444 is set, finds… |
| `011b76f0` | X509_NAME_ENTRY_new | likely | cdecl, no args; EAX = result of 011d6080 (ASN1_item_new) with the X509_NAME_ENTRY template at 0189eb04 (the 2-field SEQUENCE, sname X509_NAME_ENTRY at 0189eac4… |
| `011b7700` | X509_NAME_ENTRY_free | likely | cdecl, 1 stack arg (value); no result. Calls 011d6430 (ASN1_item_free) with the value and the X509_NAME_ENTRY template at 0189eb04. |
| `011b7720` | X509_NAME_ENTRY_dup | likely | cdecl, 1 stack arg (value); EAX = result of 011d8c00 (ASN1_item_dup) with the X509_NAME_ENTRY template at 0189eb04, or NULL. |
| `011b7770` | d2i_X509_NAME | likely | cdecl, 3 stack args (X509_NAME **a, const unsigned char **in, long len); EAX = the decoded name or NULL. Calls 011d4f00 (generic ASN.1 decoder) with the three … |
| `011b7790` | i2d_X509_NAME | likely | cdecl, 2 stack args (X509_NAME *a, unsigned char **pp); returns EAX from 011d5b70(a, pp, &X509_NAME ASN1_ITEM at 0189eb9c). Pushes the template, then pp and a,… |
| `011b77b0` | X509_NAME_new | likely | cdecl, no args; EAX = result of 011d6080 (ASN1_item_new) with the X509_NAME template at 0189eb9c, or NULL. |
| `011b77c0` | X509_NAME_free | likely | cdecl, 1 stack arg (value); no result. Calls 011d6430 (ASN1_item_free) with the value and the X509_NAME template at 0189eb9c. |
| `011b77e0` | X509_NAME_dup | likely | cdecl, 1 stack arg (value); EAX = result of 011d8c00 (ASN1_item_dup) with the X509_NAME template at 0189eb9c, or NULL. |
| `011b7800` | x509_name_ex_new | likely | cdecl, 1 stack arg (X509_NAME **px); EAX = 1 on success, 0 on failure. Allocates a 0x14-byte X509_NAME with 011b71d0 (CRYPTO_malloc, x_name.c line 0x87), sets … |
| `011b7890` | x509_name_ex_free | likely | cdecl, 1 stack arg (X509_NAME **px); no result. When *px and its value are non-NULL: frees the bytes buffer with 011beeb0 (BUF_MEM_free, field +8), pops and fr… |
| `011b7910` | x509_name_encode | likely | register input ESI = X509_NAME *a; cdecl-style plain ret, EAX = 1 on success, 0xffffffff on failure. Builds a stack of SET groups: for each entry, when its set… |

### `lib.random`

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `004c9600` | MT19937_InitGenrand | likely | Thiscall with one stack arg (seed). Stores the seed at state[1] and runs the standard init_genrand recurrence over 624 words (multiplier 0x6c078965 in the pseu… |
| `004c9633` | MT19937_InitByArray | likely | Thiscall with (key pointer, key length) stack args: calls MT19937_InitGenrand(19650218) and then runs the init_by_array mixing loops over the key (multipliers … |
| `004c96e2` | MT19937_SeedFromCounter | likely | No arguments: takes the low dword of QueryPerformanceCounter (or of rdtsc when the query fails) and passes it to MT19937_InitGenrand (004c9600). |

## Source shards

[[S0041|S0041]] (3), [[S0540|S0540]] (1), [[S0634|S0634]] (151), [[S0636|S0636]] (195), [[S0791|S0791]] (9)
