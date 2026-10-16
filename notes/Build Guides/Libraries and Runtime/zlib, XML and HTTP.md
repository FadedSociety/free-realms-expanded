---
type: build-guide
section: Libraries and Runtime
updated: 2026-10-12
---

# zlib, XML and HTTP

Part of [[Build Guides]] › Libraries and Runtime.

<!-- NARRATIVE START -->
## How it works

_Not written yet. An agent fills this from the shard findings, labels and video evidence: what the feature does, its data formats and packets, what the server must do, and open questions._
<!-- NARRATIVE END -->

## Mapped functions (generated)

325 accepted functions: 84 confirmed, 126 likely, 65 speculative, 50 unknown. Network-linked (`wire`): 0.

Subsystems: `lib.pugixml` (82), `lib.zlib` (76), `lib.tinyhttp` (58), `lib.uploader` (50), `lib.gparse` (33), `lib.tinyxml` (26)

_Large or library guide: only confirmed and likely functions are listed (210 of 325)._

### `lib.gparse`

**(no class)** (16)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `005ff64c` | NodeValueCopyCtor | likely | Copy-constructor for the small value type stored in the grammar map's tree nodes: copies the first dword (*param_2 into *param_1) and then copy-constructs the … |
| `005ff73b` | Ctor_017a0210 | likely | Constructor of an engine object: stores the constant 0x017a0210 (address of a function-pointer datum whose slot 0 is FUN_005feb60) into this+0, calls FUN_005db… |
| `005ff7a5` | DeletingDtorVectorAt4 | likely | MSVC scalar deleting destructor: runs the destructor body FUN_005fdb30 on this (005fdb30 destroys and frees the vector whose begin/end/capacity are at this+4/+… |
| `005ffb05` | TreeNodeCtorB | likely | Constructor of one tree node of the second map instantiation: writes the three pointer fields at +0/+4/+8 from param_2/param_3/param_4, copy-constructs the nod… |
| `005ffb58` | DeletingDtorVectorAt10 | likely | MSVC scalar deleting destructor: loads this into ESI, runs FUN_005fdb30 on this+0x10 (destroying and freeing the vector whose begin/end/capacity live at +0x10/… |
| `005ffd0d` | Record4cDtor | likely | Destructor body of the 0x4c-byte grammar record: if the reference-counted array pointer at this+0x14 is non-null it decrements the count stored in the block's … |
| `005ffd5c` | DeletingDtor_005ffd5c | likely | MSVC scalar deleting destructor for the grammar record: runs the destructor body FUN_005ffd0d on this, then frees this with _free when bit 0 of the flag argume… |
| `0060016d` | Record4cCtorDefault | likely | Default constructor of the 0x4c-byte grammar record: stores the shared empty-string constant 0x01bc7e28 in the first field, calls FUN_005e76dc and FUN_005e7706… |
| `006001a3` | Record4cCtorArgs | likely | Parameterised constructor of the 0x4c-byte grammar record: initialises the empty-string field and the two member sub-objects (FUN_005e76dc/FUN_005e7706), store… |
| `00600235` | Record4cAssign | likely | Copy-assignment of the 0x4c-byte grammar record: releases the old reference-counted id block at +0x14 (decrement, _free at zero) and adopts the source's block … |
| `00600408` | Record4cCopyCtor | likely | Copy constructor of the 0x4c-byte grammar record: initialises the empty-string field and the two member sub-objects (FUN_005e76dc, FUN_005e7706), zeroes the ve… |
| `006008b9` | ConstructRecord4cIfNotNull | likely | Conditional construction of one 0x4c-byte grammar record: loads param_1 into ECX and, if it is non-null, calls the record copy constructor FUN_00600408 with th… |
| `006008cd` | Record4cUninitializedCopy | likely | Constructs a run of 0x4c-byte grammar records into consecutive destination slots: while the source pointer (EBP+8) differs from the source end (EBP+0xc) it cal… |
| `00600b2c` | ResetParseData | likely | Resets a grammar/parse object: calls FUN_005ea600, FUN_004fd053 twice, deletes the records held in the this+0x28 map with FUN_006006db, zeroes the symbol count… |
| `00602412` | Record4cFillConstructN | likely | Constructs a counted run of 0x4c-byte grammar records into a destination: while the counter (EBP+0xc) is non-zero it calls FUN_006008b9 to construct one record… |
| `006024a9` | GrammarEngineTeardown | likely | Releases the whole grammar engine object: deletes the records held in the this+0x28 map (FUN_006006db), calls FUN_005fd1ae and FUN_004a674f, releases the cache… |

### `lib.pugixml`

**pugi::xml_tree_walker** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f5c280` | pugi::xml_tree_walker::ScalarDeletingDtor | confirmed | Stores pugi::xml_tree_walker::vftable into the object and, when bit 0 of the flag argument is set, calls _free on the object; returns the object pointer. |

**pugi::xml_writer** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f58ac0` | pugi::xml_writer::ScalarDeletingDtor | confirmed | Stores the pugi::xml_writer vtable at *this; when bit 0 of the byte argument is set it frees the object with _free; returns this (scalar deleting destructor sh… |
| `00f59760` | pugi::xml_writer::~xml_writer | confirmed | Stores the pugi::xml_writer vtable at *this and returns; it frees nothing (base destructor body). |
| `00f5c260` | pugi::xml_writer::ScalarDeletingDtor | confirmed | Stores pugi::xml_writer::vftable into the object and, when bit 0 of the flag argument is set, calls _free on the object; returns the object pointer. |

**pugi::xml_writer_file** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f59770` | pugi::xml_writer_file::write | confirmed | Writes param_3 bytes from param_2 with fwrite(data, size, 1, file) to the FILE* at this+4 (thiscall, two stack arguments, ret 8). |

**(no class)** (33)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f5ec50` | SaveDocumentToFile | confirmed | Opens the target file with _fopen(name, "wb"); on failure returns 0; otherwise builds a pugi::xml_writer_file on the stack, calls 00f5eb30 to write the documen… |
| `00f58db0` | EncodeUtf8CodePoint | likely | Writes the UTF-8 bytes of the code point param_2 (1 to 4 bytes) into the buffer at param_1, with the lead byte taken from table UNK_01873150, and returns the e… |
| `00f59010` | DecodeEntityRef | likely | Decodes one character reference whose '&' is at param_1: numeric '&#...;' and '&#x...;' references are encoded with FUN_00f58db0; the named forms &amp; &apos; … |
| `00f59200` | ScanCommentEnd | likely | Scans a buffer for the end of an XML comment '-->', converting CR LF pairs to LF with memmove compaction. At the terminator it NUL-terminates at the first '-' … |
| `00f592b0` | ScanCdataEnd | likely | Scans a buffer for the end of a CDATA section ']]>', converting CR LF pairs to LF with memmove compaction. At the terminator it NUL-terminates at the first ']'… |
| `00f59470` | GlobStarMatch | likely | Handles a '*' in the wildcard pattern: skips a run of '*' and '?' while advancing the subject, then retries the remaining pattern (FUN_00f596a0) at successive … |
| `00f59570` | BufferedWriterFlush | likely | If the buffered count at this+0x2004 is nonzero, calls the writer's vtable slot [1] (through *this, with ECX = writer) with (this+4, count), then sets the coun… |
| `00f595a0` | BufferedWriterWrite | likely | Appends param_3 bytes from param_2 to the 8 KB buffer at this+4 (count at this+0x2004). If they do not fit it flushes the buffered bytes through the writer's v… |
| `00f59620` | BufferedWriterWriteString | likely | Writes the NUL-terminated string in the stack argument (object in ECX) without its terminator, by calling FUN_00f595a0 with the string and its strlen. |
| `00f59650` | BufferedWriterPutByte | likely | Appends one byte (stack argument) to the 8 KB buffer at this+4 with count at this+0x2004; flushes first through the writer's vtable slot [1] when the buffer is… |
| `00f596a0` | GlobMatch | likely | Matches a wildcard pattern (param_1, with '*', '?' and '[...]') against the subject string (param_2) using FUN_00f59470 for '*' and FUN_00f593d0 for '[...]'. R… |
| `00f5a8a0` | ParseTextEntitiesToLt | likely | Reads character data up to '<': for each '&' it calls DecodeEntityRef (00f59010), compacts the decoded text in place with memmove, NUL-terminates at '<' and re… |
| `00f5a950` | ParseTextCrlfToLt | likely | Reads character data up to '<', turning each CR LF pair into LF with memmove compaction and no entity decoding; NUL-terminates at '<' and returns the address a… |
| `00f5aa00` | ParseTextCrlfEntitiesToLt | likely | Like ParseTextEntitiesToLt, and also turns each CR LF pair into LF with memmove compaction; entity references go through DecodeEntityRef (00f59010); stops at '… |
| `00f5ab20` | ParseEntitiesToDelimiter | likely | Scans to the delimiter param_2 among class-bit-2 bytes, decoding '&' references with DecodeEntityRef (00f59010) and compacting the result with memmove; NUL-ter… |
| `00f5abc0` | ParseCrlfToDelimiter | likely | Scans to the delimiter param_2 among class-bit-2 bytes, turning CR LF pairs into LF with memmove compaction and no entity decoding; NUL-terminates at the delim… |
| `00f5ac60` | ParseCrlfEntitiesToDelimiter | likely | As ParseEntitiesToDelimiter, and also turns CR LF pairs into LF with memmove compaction; references go through DecodeEntityRef (00f59010). |
| `00f5ad30` | NormalizeSpacesToDelimiter | likely | Normalizes an attribute value up to the delimiter param_2: strips leading whitespace (class bit 8 of table 0x01873158, via FUN_00f58f90), collapses internal wh… |
| `00f5bb80` | XmlEscapeAttrValue | likely | Writes a NUL-terminated string through 00f595a0 with XML attribute escaping: '&', '<', '>' become entities, double quote becomes &quot;, CR and LF become &#13;… |
| `00f5bd30` | XmlEscapePcdata | likely | Writes a NUL-terminated string through 00f595a0 with text (PCDATA) escaping: '&', '<', '>' become entities, control bytes other than tab, LF and CR become &#<d… |
| `00f5bee0` | PageAllocate | likely | Bump-allocates n bytes from the current 0x8000-byte memory page (used-size counter in the page header); when the request does not fit, mallocs a new 0x8008-byt… |
| `00f5c090` | StrconvDispatchByFlags | likely | Switches on bits 4-7 of param_3 (switch(param_3 >> 4 & 0xf)) and returns the result of one of sixteen in-place scanners (00f5aae0..00f5b9e0); default returns 0. |
| `00f5c210` | BufferedWriterFlushDtor | likely | If the pending byte count at offset 0x2004 is non-zero, calls the writer's vtable slot +4 with (buffer at +4, count) and then clears the count; called from EH … |
| `00f5c710` | FirstElementByPath | likely | Resolves a delimiter-separated path (delimiter param_4) from a node: a leading delimiter climbs to the top ancestor, '.' keeps the node, '..' moves to the pare… |
| `00f5cad0` | AllocDocumentStruct | likely | Allocates a 0x38-byte block through 00f5bee0, sets its node-type bits to 1 (document), clears the fields, stores the block in *ECX and writes a back-pointer at… |
| `00f5cc70` | ParseXmlBuffer | likely | Parses an in-memory XML buffer (skips a UTF-8 BOM EF BB BF) into the node tree: dispatches on '<' markup (processing instruction, comment, CDATA, DOCTYPE, end … |
| `00f5d9c0` | NodeOutputRecursive | likely | Serialises one node into the buffered writer: document nodes recurse over children, element nodes write '<name', attributes (through 00f5bb80), children and '<… |
| `00f5e980` | NodePrintBuffered | likely | Prints a node through an 8192-byte stack buffer: builds a buffered writer over the given writer object, runs 00f5d9c0 on the node tree, then flushes any pendin… |
| `00f5ea90` | XmlDocumentLoadInplace | likely | Resets the document through 00f5cb40, stores the caller's buffer at document-struct+0x34, parses it with 00f5cc70 using the given flags and returns the result … |
| `00f5ead0` | XmlDocumentLoadInplaceOwn | likely | Resets the document through 00f5cb40, stores the buffer at document-struct+0x34, parses it with 00f5cc70; on success records the buffer at the object's +4 for … |
| `00f5eb30` | XmlDocumentSaveToWriter | likely | Writes an optional UTF-8 BOM (flag 2), the XML declaration <?xml version="1.0"?> unless flag 8, a newline unless flag 4, then the document node through 00f5d9c… |
| `00f5ecf0` | CopyNodeContents | likely | Copies node contents from source to destination after asserting equal node types (pugixml.cpp line 0x660): element nodes copy name and attributes (through 00f5… |
| `00f5f040` | XmlDocumentLoadStringCopy | likely | Resets the document through 00f5cb40, measures the NUL-terminated input, copies it into a malloc block of length+1, then loads the copy as an owned buffer thro… |

### `lib.tinyhttp`

**TinyHttp::HttpManager::GetRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d30e40` | GetRequest::ScalarDeletingDtor | confirmed | Installs the GetRequest vtable, calls the Request destructor, conditionally frees the object, and returns its pointer. |

**TinyHttp::HttpManager::HttpConnection** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d2f910` | HttpConnection::OnTerminated | confirmed | Logs termination, requeues sent requests, checks retry count, then releases transport state. |
| `00d2fb90` | HttpConnection::~HttpConnection | confirmed | Releases the attached transport object, two fixed strings, and the 8 KB byte array, then restores the TCP base vtable. |
| `00d302f0` | HttpConnection::HttpConnection | confirmed | Installs the connection vtable, initializes the 8 KB buffer and fixed strings, stores constructor inputs, and zeros connection state. |
| `00d30420` | HttpConnection::ScalarDeletingDtor | confirmed | Calls the HttpConnection destructor, frees this when flag bit 0 is set, and returns this. |
| `00d30440` | HttpConnection::GiveTime | confirmed | Creates a connection when requests are pending, pumps connection work, and disconnects an idle connection after its configured timeout. |

**TinyHttp::HttpManager::PostRequest** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d2f420` | ProcessResult | confirmed | Virtual slot 2 of PostRequest: logs 'HttpManager::PostRequest::ProcessResult(%s:mod%d) HttpResultCode=%d' with the request name, the mod id and the result code… |
| `00d2fed0` | PostRequest::~PostRequest | confirmed | Releases the reference-counted member and 512-byte string, then calls the base Request destructor. |
| `00d30e90` | PostRequest::ScalarDeletingDtor | confirmed | Calls the PostRequest destructor, conditionally frees the object when flag bit zero is set, and returns the object pointer. |
| `00d31c60` | PostRequest::PostRequest | likely | Initializes the Request base, stores the optional body reference, and copies the supplied request metadata into the derived object. |

**TinyHttp::HttpManager::PutRequest** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d2f3b0` | ProcessResult | confirmed | Virtual slot 2 of PutRequest: logs 'HttpManager::PutRequest::ProcessResult(%s:mod%d) HttpResultCode=%d' with the request name, the mod id and the result code a… |
| `00d2fdf0` | PutRequest::~PutRequest | confirmed | Releases the reference-counted member and 512-byte string, then calls the base Request destructor. |
| `00d30e70` | PutRequest::ScalarDeletingDtor | confirmed | Calls the PutRequest destructor, conditionally frees the object when flag bit zero is set, and returns the object pointer. |
| `00d31ba0` | PutRequest::PutRequest | likely | Initializes the Request base, stores the optional body reference, and copies the supplied request metadata into the derived object. |

**TinyHttp::HttpManager::Request** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d2fd10` | Request::~Request | confirmed | Decrements the reference-counted member at +0x228, then releases the fixed 512-byte URL string. |
| `00d31ac0` | Request::Request | likely | Stores request inputs, escapes the supplied URL into the request string, and initializes request status, size, and retry state. |

**TinyHttp::HttpManagerHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ade900` | TinyHttp::HttpManagerHandler::~HttpManagerHandler | confirmed | Resets the vtable pointer to TinyHttp::HttpManagerHandler::vftable and returns; the function is reached from 10 EH unwind funclets. |
| `00ade940` | TinyHttp::HttpManagerHandler::ScalarDeletingDtor | confirmed | Virtual deleting destructor: stores TinyHttp::HttpManagerHandler::vftable back into the object and calls _free(this) when the flag argument has bit 0 set; it i… |

**(no class)** (16)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f96090` | HttpRequestHeadBuild | confirmed | Thiscall(request, out string*): reserves 0x200 bytes, appends the method name from the table at 01b97ee0 indexed by the dword at request+0, a space, the reques… |
| `00d2f8c0` | FailHeadRequestAndDelete | likely | Unlinks the queue head, calls its vtable slot +8, subtracts its size, then deletes it. |
| `00d30100` | SplitUrlAuthorityAndPath | likely | Finds the scheme delimiter, scans authority to the next slash, and assigns optional authority and path outputs. |
| `00d30240` | ReadLocalFileAndRespond | likely | Reads the supplied path into a ref-counted byte array and invokes the callback with status 200 or 404. |
| `00d31770` | RemoveConnection | likely | Unlinks a connection from its hash bucket and doubly linked list, decrements the manager count, then destroys and deletes the connection. |
| `00d317e0` | CreateConnection | likely | Allocates a pooled connection object, constructs it, computes its hash key, and links it into the manager list and bucket. |
| `00d31e50` | EnqueueGetRequest | likely | Allocates and constructs a GetRequest, appends it to the pending queue, and updates pending size accounting. |
| `00d31f10` | EnqueuePutRequest | likely | Allocates and constructs a PutRequest, appends it to the pending queue, and updates request count and total size. |
| `00d31fd0` | EnqueuePostRequest | likely | Allocates and constructs a PostRequest, appends it to the pending queue, and updates request count and total size. |
| `00d32100` | RouteGetUrl | likely | Routes file:// URLs to the local-file callback path; otherwise splits the URL, selects a connection, and queues a GetRequest. |
| `00d32210` | RoutePutUrl | likely | Splits the supplied URL, selects or creates a connection, then constructs and queues a PutRequest with the supplied body and callback state. |
| `00d32330` | RoutePostUrl | likely | Splits the supplied URL, selects or creates a connection, then constructs and queues a PostRequest with the supplied body and callback state. |
| `00d32f60` | HttpManager::Cleanup | likely | Removes connections until the list is empty, releases a reference-counted member, then cleans the map and two fixed strings. |
| `00d33050` | HttpManager::HttpManager | likely | Copies manager settings, clears manager counters, initializes embedded connection-map storage, and creates an auxiliary helper object. |
| `00f95440` | ParseUrlIntoFlagAndInt | likely | Thiscall(url): calls 00f99d70 with two local string buffers, a numeric local and a flag byte. On success it calls 00f63d10(url) and 00f60dd0 on the first buffe… |
| `00f98960` | RequestBaseCtor | likely | Fastcall(this): zeroes the object's words and byte fields across about 0x190 bytes (the byte at +4 and the words at +0 through +0x19c, with the gaps listed in … |

### `lib.tinyxml`

**TiXmlAttribute** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d54fd0` | TiXmlAttribute::Parse | confirmed | Reads one attribute: the name into this+0x14 via FUN_00d53ea0, requires '=' via FUN_00d53de0 lookahead, then reads a single- or double-quoted value into this+0… |

**TiXmlComment** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007a10f0` | TiXmlComment::TiXmlComment | confirmed | Constructor (ECX=this, one stack argument char* text): calls 00d514c0 with 2 (base node initialisation), stores TiXmlComment::vftable at this, scans the text t… |
| `007a1190` | TiXmlComment::DtorBody | confirmed | Destructor body (ECX=this, no stack arguments): stores TiXmlComment::vftable at this and calls 00d51500 on this (base destructor); no free. |
| `007a11e0` | TiXmlComment::SelfSlot8 | confirmed | Bytes 8b c1 c3: mov eax,ecx; ret. Returns this unchanged (3 bytes, no stack arguments). |
| `007a11f0` | TiXmlComment::SelfSlot7 | confirmed | Bytes 8b c1 c3: mov eax,ecx; ret. Returns this unchanged (3 bytes, no stack arguments). |
| `007a1200` | TiXmlComment::~TiXmlComment | confirmed | Scalar deleting destructor (ECX=this, byte stack argument): stores TiXmlComment::vftable at this, calls 00d51500 (base destructor) on this, then calls _free(th… |

**TiXmlDeclaration** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00819390` | TiXmlDeclaration::DtorBody | confirmed | Destructor body. Stores TiXmlDeclaration::vftable at *this, then frees the buffers at this+0x34, this+0x30 and this+0x2c through _free unless each one points a… |
| `00819430` | TiXmlDeclaration::ForwardSlot17 | confirmed | Thiscall with two stack arguments. Calls the object virtual at vtable offset +0x44 (slot 17) with the two arguments and a zero third argument, then returns 8 (… |
| `00819450` | TiXmlDeclaration::ReturnThis_Slot14 | confirmed | Fastcall stub: mov eax, ecx; ret. Returns its this pointer unchanged. |
| `00819460` | TiXmlDeclaration::ReturnThis_Slot13 | confirmed | Fastcall stub: mov eax, ecx; ret. Returns its this pointer unchanged. |
| `00819470` | TiXmlDeclaration::ScalarDeletingDtor | confirmed | Thiscall scalar deleting destructor, vtable slot 0 of TiXmlDeclaration. Calls the dtor body 00819390 on this, then if the low bit of the flag argument is set c… |

**TiXmlDocument** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00796020` | TiXmlDocument::DtorBody | confirmed | Fastcall (ECX = this). Stores the TiXmlDocument vtable at [ecx], frees the buffer at this+0x34 through _free (00d82463 thunk) unless it equals the static senti… |
| `00796150` | TiXmlDocument::ScalarDeletingDtor | confirmed | Thiscall with one flag byte. Calls 00796020 (TiXmlDocument dtor body) on this, then calls _free (00d82463 thunk) on this when flag bit 0 is set. Returns this w… |

**TiXmlPrinter** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009f9b70` | TiXmlPrinter::Ctor | confirmed | Installs TiXmlVisitor then TiXmlPrinter vtables, clears printer state and initializes two owned strings from global defaults. |
| `009f9c00` | TiXmlPrinter::ScalarDeletingDtor | confirmed | Calls the TiXmlVisitor cleanup, conditionally frees the object when flag bit 0 is set, and otherwise returns it. |

**TiXmlVisitor** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009f86b0` | TiXmlVisitor::Dtor | confirmed | Replaces the object's first dword with TiXmlVisitor::vftable and returns. |
| `009f8740` | TiXmlVisitor::ScalarDeletingDtor | confirmed | Restores TiXmlVisitor::vftable, frees the object when deleting flag bit 0 is set, and otherwise returns the object pointer. |

### `lib.uploader`

**FunctionObjectT** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f6f730` | FunctionObjectT::Init | confirmed | Init, layout with state at +0x1a0. Stores 2 at this+0x1a0 and parses the value at this+0x10c (URL block at this+0x108) with 00f95440. Parse failure: calls logg… |
| `00f6fb70` | FunctionObjectT::Init | confirmed | Init, layout with state at +0xa8. Stores 2 at this+0xa8 and parses the value at this+0x2c (URL block at this+0x28) with 00f95440. Parse failure: calls logger 0… |
| `00f6ff90` | FunctionObjectT::Init | confirmed | Init, layout with state at +0x1d0. Stores 2 at this+0x1d0 and parses the value at this+0xac (URL block at this+0xa8) with 00f95440. Parse failure: calls logger… |
| `00f703d0` | FunctionObjectT::Init | confirmed | Init, layout with state at +0x170. Stores 2 at this+0x170 and parses the value at this+0x8c (URL block at this+0x88) with 00f95440. Parse failure: calls logger… |
| `00f70810` | FunctionObjectT::Init | confirmed | Init, layout with state at +0x178. Stores 2 at this+0x178 and parses the value at this+0x5c (URL block at this+0x58) with 00f95440. Parse failure: calls logger… |
| `00f70c40` | FunctionObjectT::Init | confirmed | Init, layout with state at +0x98. Stores 2 at this+0x98 and parses the value at this+0x2c (URL block at this+0x28) with 00f95440. Parse failure: calls logger 0… |
| `00f71060` | FunctionObjectT::Init | confirmed | Init, layout with state at +0x190 (decimal 400 in the pseudocode). Stores 2 at this+0x190 and parses the value at this+0x11c (URL block at this+0x118) with 00f… |
| `00f714a0` | FunctionObjectT::Init | confirmed | Init, layout with state at +0xb8. Stores 2 at this+0xb8 and parses the value at this+0x5c (URL block at this+0x58) with 00f95440. Parse failure: calls logger 0… |
| `00f718c0` | FunctionObjectT::Init | confirmed | Init, layout with state at +0x58. Stores 2 at this+0x58 and parses the value at this+0x1c (URL block at this+0x18) with 00f95440. Parse failure: calls logger 0… |
| `00f71cc0` | FunctionObjectT::Init | confirmed | Init, layout with state at +0x78. Stores 2 at this+0x78 and parses the value at this+0x1c (URL block at this+0x18) with 00f95440. Parse failure: calls logger 0… |
| `00f720d0` | FunctionObjectT::Init | confirmed | Init, layout with state at +0xd0. Stores 2 at this+0xd0 and parses the value at this+0x4c (URL block at this+0x48) with 00f95440. Parse failure: calls logger 0… |
| `00f724f0` | FunctionObjectT::Init | confirmed | Init, layout with state at +0xa0. Stores 2 at this+0xa0 and parses the value at this+0x30 (URL block at this+0x2c) with 00f95440. Parse failure: calls logger 0… |
| `00f72910` | FunctionObjectT::Init | confirmed | Init, layout with state at +0x68. Stores 2 at this+0x68 and parses the value at this+0x1c (URL block at this+0x18) with 00f95440. Parse failure: calls logger 0… |

**Smart::ReportHandler** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f5f230` | ScalarDeletingDtorBody | confirmed | Stores Smart::ReportHandler::vftable into the object and, when bit 0 of the flag argument is set, calls _free on the object; returns the object pointer. Lives … |

**UploaderImpl** (16)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f60a40` | Config | confirmed | Configures the uploader: rejects a second configuration (log text 'uploader is already configured'), a null handler and a null SSL certificate file/path; other… |
| `00f733d0` | UploaderImpl::AddRequest | confirmed | Thiscall, ECX = UploaderImpl this. If byte this+0x70 is 0, calls logger 00f99fd0 with 'UploaderImpl::AddRequest, failed with uploader not configured' (level 2,… |
| `00f734b0` | UploaderImpl::AddRequest | confirmed | Thiscall, ECX = UploaderImpl this. If byte this+0x70 is 0, calls logger 00f99fd0 with 'UploaderImpl::AddRequest, failed with uploader not configured' (level 2,… |
| `00f73590` | UploaderImpl::AddRequest | confirmed | Thiscall, ECX = UploaderImpl this. If byte this+0x70 is 0, calls logger 00f99fd0 with 'UploaderImpl::AddRequest, failed with uploader not configured' (level 2,… |
| `00f73670` | UploaderImpl::AddRequest | confirmed | Thiscall, ECX = UploaderImpl this. If byte this+0x70 is 0, calls logger 00f99fd0 with 'UploaderImpl::AddRequest, failed with uploader not configured' (level 2,… |
| `00f73750` | UploaderImpl::AddRequest | confirmed | Thiscall, ECX = UploaderImpl this. If byte this+0x70 is 0, calls logger 00f99fd0 with 'UploaderImpl::AddRequest, failed with uploader not configured' (level 2,… |
| `00f73830` | UploaderImpl::AddRequest | confirmed | Thiscall, ECX = UploaderImpl this. If byte this+0x70 is 0, calls logger 00f99fd0 with 'UploaderImpl::AddRequest, failed with uploader not configured' (level 2,… |
| `00f73910` | UploaderImpl::AddRequest | confirmed | Thiscall, ECX = UploaderImpl this. If byte this+0x70 is 0, calls logger 00f99fd0 with 'UploaderImpl::AddRequest, failed with uploader not configured' (level 2,… |
| `00f739f0` | UploaderImpl::AddRequest | confirmed | Thiscall, ECX = UploaderImpl this. If byte this+0x70 is 0, calls logger 00f99fd0 with 'UploaderImpl::AddRequest, failed with uploader not configured' (level 2,… |
| `00f73ad0` | UploaderImpl::AddRequest | confirmed | Thiscall, ECX = UploaderImpl this. If byte this+0x70 is 0, calls logger 00f99fd0 with 'UploaderImpl::AddRequest, failed with uploader not configured' (level 2,… |
| `00f73bb0` | UploaderImpl::AddRequest | confirmed | Thiscall, ECX = UploaderImpl this. If byte this+0x70 is 0, calls logger 00f99fd0 with 'UploaderImpl::AddRequest, failed with uploader not configured' (level 2,… |
| `00f73c90` | UploaderImpl::AddRequest | confirmed | Thiscall, ECX = UploaderImpl this. If byte this+0x70 is 0, calls logger 00f99fd0 with 'UploaderImpl::AddRequest, failed with uploader not configured' (level 2,… |
| `00f73d70` | UploaderImpl::AddRequest | confirmed | Thiscall, ECX = UploaderImpl this. If byte this+0x70 is 0, calls logger 00f99fd0 with 'UploaderImpl::AddRequest, failed with uploader not configured' (level 2,… |
| `00f73e50` | UploaderImpl::AddRequest | confirmed | Thiscall, ECX = UploaderImpl this. If byte this+0x70 is 0, calls logger 00f99fd0 with 'UploaderImpl::AddRequest, failed with uploader not configured' (level 2,… |
| `00f73f30` | UploaderImpl::AddRequest | confirmed | Thiscall, ECX = UploaderImpl this. If byte this+0x70 is 0, calls logger 00f99fd0 with 'UploaderImpl::AddRequest, failed with uploader not configured' (level 2,… |
| `00f74010` | UploaderImpl::AddRequest | confirmed | Thiscall, ECX = UploaderImpl this. If byte this+0x70 is 0, calls logger 00f99fd0 with 'UploaderImpl::AddRequest, failed with uploader not configured' (level 2,… |

**Uram::FunctionObject** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f5f430` | Uram::FunctionObject::Dtor | confirmed | Stores Uram::FunctionObject::vftable into the object and returns; no free, no other work. |
| `00f5f450` | Uram::FunctionObject::ScalarDeletingDtor | confirmed | Stores Uram::FunctionObject::vftable into the object and, when bit 0 of the flag argument is set, calls _free on the object; returns the object pointer. |

**Uram::HttpHandler** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f60090` | HttpHandler::OnTerminated | confirmed | Reads the disconnect code at connection+0x78; when it is not 8, calls 010e78d0 for the reason name, fetches WSAGetLastError and logs 'HttpHandler::OnTerminated… |
| `00f603d0` | Uram::HttpHandler::DtorBody | confirmed | Installs the Uram::HttpHandler vtables at +0 and +4, sets the byte-buffer vtable at +0x150 and clears its count and capacity, frees the byte buffer through 004… |
| `00f60460` | HttpHandlerAppendBytes | confirmed | Virtual slot 1 of the +4 secondary vtable: appends param_4 bytes from the source at data+count of the byte buffer at +0x150 (count +0x154, capacity +0x158), gr… |
| `00f5ffd0` | Uram::HttpHandler::Ctor | likely | Installs Uram::HttpHandler vtables at +0 and +4 and a TcpLibrary::TcpConnectionHandler vtable at +4 (before the HttpHandler one), zeroes the field at +8, runs … |
| `00f604a0` | Uram::HttpHandler::ScalarDeletingDtor | likely | Calls the non-deleting destructor body 00f603d0 and, when bit 0 of the flag argument is set, calls _free on the object; returns the object pointer. |

**Uram::UploaderLogHandlerAdapter** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f5f2f0` | ForwardChild8Slot40 | likely | If the object at +8 is non-null, tail-jumps to its vtable slot +0x40 with the same stack arguments; otherwise returns 4 bytes popped (ret 4). |
| `00f5f310` | ForwardChild8Slot44 | likely | If the object at +8 is non-null, tail-jumps to its vtable slot +0x44 with the same stack arguments; otherwise ret 4. |
| `00f5f330` | ForwardChild8Slot48 | likely | If the object at +8 is non-null, tail-jumps to its vtable slot +0x48 with the same stack arguments; otherwise ret 4. |
| `00f5f350` | ForwardChild4Slot40 | likely | Through the secondary base at +4: if the object at +4 is non-null, tail-jumps to its vtable slot +0x40 with the same stack arguments; otherwise ret 4. |
| `00f5f370` | ForwardChild4Slot44 | likely | Through the secondary base at +4: if the object at +4 is non-null, tail-jumps to its vtable slot +0x44 with the same stack arguments; otherwise ret 4. |
| `00f5f390` | ForwardChild4Slot48 | likely | Through the secondary base at +4: if the object at +4 is non-null, tail-jumps to its vtable slot +0x48 with the same stack arguments; otherwise ret 4. |
| `00f5f3d0` | ScalarDeletingDtorBody | likely | Stores three vtables (Uram::UploaderLogHandlerAdapter at +4, Smart::ReportHandler at +4, SslLibrary::LogHandler at +0), and when bit 0 of the flag argument is … |

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f8de10` | UploaderErrorTextByCode | likely | Returns the entry at index negative-code of the pointer table starting at 0x01b97ac0 (the first entry is the UploaderErrNone text). No other work. |

### `lib.zlib`

**(no class)** (60)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `006f6ca0` | InflateTreesBits | confirmed | Allocates a 0x13-entry (4-byte each) work array through z->zalloc (+0x20), calls HuftBuild on the 19 code-length code lengths, sets z->msg for the -3 result or… |
| `006f6d40` | InflateTreesDynamic | confirmed | Allocates a 0x120-entry work array and builds the literal/length tree with HuftBuild (nl symbols, base 257, tables DAT_017e2a68 and DAT_017e2ae8). A -3 result … |
| `006f7020` | InflateCodes | confirmed | Runs the codes state machine (modes 0 to 9) over the bit buffer: decodes literal/length symbols through the inflate_mask table DAT_017e3d58, copies literals an… |
| `006f7700` | InflateBlocks | confirmed | Runs the block-level state machine (modes 0 to 9): reads the 3-bit block header, handles stored blocks (LEN/NLEN check, then byte copy), builds fixed or dynami… |
| `006f8280` | Inflate | confirmed | Returns -2 (0xfffffffe) for a null stream or state. Runs the zlib header parser (method, window size, header check mod 31, FDICT) and then the block states: In… |
| `010edcb0` | ZlibDeflateInit2 | confirmed | Returns -6 (Z_VERSION_ERROR) unless the version string starts with '1' and the stream size is 0x38. Returns -2 for a null stream. It installs the default alloc… |
| `010ee1a0` | inflate | confirmed | zlib 1.2.3 inflate(strm, flush). Reads next_in/avail_in (strm+0, +4) and writes next_out/avail_out (+0xc, +0x10), msg (+0x18) and state (+0x1c; mode is the fir… |
| `010efee0` | compress2 | confirmed | zlib compress2(dest, destLen, source, sourceLen, level). Builds a z_stream on the stack (next_in=source, avail_in=sourceLen, next_out=dest, avail_out=*destLen,… |
| `010f2330` | inflate_fast | confirmed | zlib inffast.c inflate_fast(strm, start): fast LEN/DIST decode loop used by inflate when enough input and output remain. Uses the bit buffer hold (state word 0… |
| `01539dc0` | zlib_uncompress | confirmed | zlib-style uncompress: calls 010ee080 (inflateInit-like, given version string '1.2.3' and a 0x38-byte stream), 010ee1a0 (inflate with flush 4 = Z_FINISH) and 0… |
| `006f6ed0` | InflateTreesFixed | likely | Fastcall with EAX=bl, ECX=bd, EDX=tl and the fourth argument td on the stack: writes 9 to *bl and 5 to *bd, stores the fixed literal/length table DAT_017e2c58 … |
| `006f6ef0` | InflateFlush | likely | Copies bytes from the window read pointer (+0x30) to the write pointer (+0x34), or to the window end (+0x2c) when the write pointer is behind, into z->next_out… |
| `006f6fe0` | InflateCodesNew | likely | Allocates a 0x1c-byte codes state through z->zalloc (EAX=z, items 1). On success it stores the bl argument byte at +0x10 and the bd argument byte at +0x11, zer… |
| `006f75e0` | ZStreamFree | likely | Calls z->zfree(z->opaque, p) with z in EAX and p in ECX: loads zfree from +0x24 and opaque from +0x28, pushes both with p, and returns after cleaning the stack… |
| `006f81c0` | InflateBlocksFree | likely | Calls InflateBlocksReset(0), then frees the window (+0x28), the hufts table (+0x24) and the block state itself through z->zfree. Returns 0. |
| `006f8240` | InflateEnd | likely | With the z_stream in ESI, returns -2 unless z, z->state and z->zfree are all set. Otherwise it frees the block layer through InflateBlocksFree when present, fr… |
| `006f8670` | Adler32 | likely | Computes the Adler-32 of buf[0..len) starting from seed (low 16 bits a, high 16 bits b). It processes 0x15b0-byte chunks with the inner loop unrolled 16 at a t… |
| `006f87b0` | ZCalloc | likely | Computes items*size and calls 006bd320 with (opaque, byte count, &items), the third argument being the address of the caller's items slot. Returns the block po… |
| `006f8800` | GzipReadHeader | likely | Reads the gzip member header through the source helpers 006bb340, 006bb380, 006bb5d0, 006bb690 and 006bcff0. It requires the magic bytes 0x1f 0x8b, method 8 an… |
| `006f8950` | InflateStreamRelease | likely | Calls InflateEnd on the embedded z_stream at this+0xc, then zeroes the reader's bookkeeping fields at this+0, +4, +8, +0xc, +0x10, +0x18, +0x1c, +0x2c, +0x30 a… |
| `006f8980` | GzipStreamRewind | likely | Seeks the source (006bb340 with the source at this+0 and the handle at this+0x44) back to its start. On success it calls InflateReset, points the input chunk p… |
| `006f89d0` | GzipStreamFillInput | likely | Refills the 0x1000-byte input chunk at this+0x48. If the source has a read callback at +0x14 it is called with (source, offset, chunk, 0x1000). Otherwise min(r… |
| `006f8a50` | GzipStreamInflateStep | likely | Loops: refills the input through GzipStreamFillInput when avail_in is zero (returning its error), calls Inflate(z, 0), and returns 0x55 on any Inflate error. O… |
| `006f8ac0` | GzipStreamSkip | likely | Consumes buffered output in steps: each pass takes min(EAX, buffered) bytes, adds the count to the position at +0x2048 and to the read cursor at +0x204c, decre… |
| `006f8b10` | GzipStreamReadAt | likely | Compares the EAX argument with the position at +0x2048: below it calls GzipStreamRewind, above it calls GzipStreamSkip. It then copies up to the requested leng… |
| `006f8c50` | InflateInit2_ | likely | Returns -6 (0xfffffffa) unless the version string starts with '1' and stream_size is 0x38 (sizeof z_stream). Returns -2 for a null stream. It allocates the 0x1… |
| `006f8de0` | GzipStreamOpen | likely | Zeroes a fresh reader object, parses the member header with GzipReadHeader, allocates a 0x2054-byte reader (006bbdf0) and initialises it through GzipStreamInit… |
| `010ec210` | ZlibPutShortMSB | likely | Fastcall: the 16-bit value is in ECX and the deflate state in EAX. Appends the high byte at pending_buf (state+8) + pending (state+0x14), increments pending, t… |
| `010ec240` | ZlibFlushPending | likely | Copies min(strm avail_out at +0x10, state pending count at state+0x14) bytes from state pending_out (state+0x10) to strm next_out (+0xc) with memcpy. It advanc… |
| `010eca70` | ZlibDeflateEnd | likely | Returns -2 (Z_STREAM_ERROR) when strm or strm->state (+0x1c) is null or the state status (+4) is not one of 42, 69, 73, 91, 103, 113 or 666. Otherwise it frees… |
| `010ecb50` | ZlibDeflateCopy | likely | Returns -2 when either stream or the source state (+0x1c) is null. Copies the 0x38-byte z_stream header from source to dest, allocates a 0x16c0-byte deflate st… |
| `010ecd60` | ZlibLmInit | likely | Sets window_size (+0x3c) to 2 * w_size (+0x2c), clears the hash head table at +0x44 with memset (length +0x4c*2-2), and copies u16 values from the 12-byte leve… |
| `010ecf70` | ZlibRleMatchLength | likely | Compares the window at strstart with the window at param_1 (the candidate position) byte by byte, 8 at a time, up to 258 bytes (0x102). When at least 3 bytes m… |
| `010edfb0` | ZlibInflateInit2 | likely | Returns -6 (Z_VERSION_ERROR) unless the version string starts with '1' (code tests *param_3 != '1') and the stream size is 0x38. Installs the default allocator… |
| `010ef7a0` | inflateEnd | likely | Returns 0xfffffffe (-2) when strm, strm->state (+0x1c) or strm->zfree (+0x24) is null. Otherwise, when state->window (state+0x34) is non-null, calls zfree(opaq… |
| `010efbc0` | adler32 | likely | zlib adler32(adler, buf, len). Returns 1 when buf is null. With len 1 adds the single byte to the first sum. Otherwise sums bytes in blocks of at most 5552 (16… |
| `010eff90` | compress | likely | Calls 010efee0 (compress2) with its four arguments and level 0xffffffff, then returns the int result unchanged. |
| `010effe0` | init_block | likely | Clears the 286 dyn_ltree frequency words (state+0x94, stride 4), the 30 dyn_dtree words (+0x988) and the 19 bl_tree words (+0xa7c). Sets dyn_ltree[256].Freq (+… |
| `010f0050` | pqdownheap | likely | Sifts heap entry k down the Huffman heap (state in EAX, tree in EDI, k as the stack argument). The heap array is at state+0xb5c and heap_len at state+0x1450. C… |
| `010f0120` | gen_bitlen | likely | Assigns bit lengths for one tree from the heap built by build_tree. Desc pointer in ECX (tree +0, max_code +4, stat_desc +8 with static tree, extra bits, extra… |
| `010f0430` | send_tree | likely | Fastcall: state in EAX, max_code in ECX, tree in EDX. Writes the run-length coded code lengths of tree nodes 0..max_code as bl_tree codes (code at state+0xa7c+… |
| `010f0940` | send_all_trees | likely | Writes the dynamic block header: lcodes-257 in 5 bits, dcodes-1 in 5 bits and blcodes-4 in 4 bits through bi_buf (+0x16b8) and bi_valid (+0x16bc). Then writes … |
| `010f0c40` | compress_block | likely | Replays the buffered symbols (l_buf at state+0x1698, d_buf at +0x16a4, count last_lit at +0x16a0). Literals are written with the literal tree Code (+0) and Len… |
| `010f1050` | detect_data_type | likely | Second argument is the state. Counts zero-frequency words in dyn_ltree (state+0x94, stride 4), then scans further slots for a non-zero entry. Stores into strm-… |
| `010f1140` | bi_flush | likely | Moves complete bytes out of the bit buffer. When bi_valid (state+0x16bc) equals 16, writes bi_buf (+0x16b8) as two bytes to the pending buffer (+8, index +0x14… |
| `010f11c0` | bi_windup | likely | State in EAX. When bi_valid (state+0x16bc) is above 8, writes bi_buf (+0x16b8) as two bytes; otherwise, when bi_valid is positive, writes one byte. Then clears… |
| `010f1240` | StoredBlockAlignCopy | likely | Helper with count in ECX, source pointer in EDX, header flag on the stack, state in EAX. Calls bi_windup (010f11c0) and sets last_eob_len (+0x16b4) to 8. When … |
| `010f12d0` | _tr_init | likely | Points the literal (+0xb18), distance (+0xb24) and bit-length (+0xb30) tree descriptors at dyn_ltree (+0x94), dyn_dtree (+0x988) and bl_tree (+0xa7c). Sets the… |
| `010f1340` | gen_codes | likely | Assigns canonical Huffman codes. Tree in EDI, max_code in EBX, bl_count in EDX. Builds next_code[1..15] in a 16-word stack array by code = (code + bl_count[bit… |
| `010f13d0` | build_tree | likely | Builds one Huffman tree from a descriptor pointer on the stack, with state in ESI. Collects non-zero-frequency leaves into the heap (heap_len at state+0x1450, … |
| `010f15d0` | build_bl_tree | likely | Runs scan_tree (010f0340) over the literal and distance trees, then build_tree (010f13d0) on the bl descriptor at state+0xb30. Scans bl_order entries (table 01… |
| `010f16a0` | _tr_stored_block | likely | Writes the 3-bit stored block header, value = last (block type 0), into bi_buf (+0x16b8) and bi_valid (+0x16bc). Flushes 16 bits to the pending buffer when bi_… |
| `010f1740` | _tr_align | likely | Sends an empty static block: 3-bit header value 2, the 7-bit END_BLOCK static code (zero bits), then bi_flush (010f1140). Sends the empty block a second time w… |
| `010f1930` | _tr_flush_block | likely | Four stack arguments: state, buf, stored_len, last. When level (state+0x84) is above 0, calls detect_data_type (010f1050) if strm->data_type is 2, build_tree (… |
| `010f1b50` | crc32_byfour | likely | CRC-32 body, fastcall: buf in ECX, byte count in EDX, crc in EAX. Inverts the crc, handles unaligned leading bytes one at a time, then 32-byte blocks (eight ta… |
| `010f1df0` | crc32_big | likely | Big-endian table variant of the CRC-32 body, thiscall: buf in ECX, byte count in EAX, crc on the stack. Byte-swaps the incoming crc, inverts it, processes the … |
| `010f22a0` | crc32 | likely | crc32(crc, buf, len). Returns 0 when buf (second argument) is null. Otherwise jumps to 010f1b50 with crc in EAX, buf in ECX and len in EDX. |
| `010f2300` | zcalloc | likely | zlib zcalloc(opaque, items, size): imul items*size, pushes the product and calls _malloc (00d8a681). Returns the malloc result in EAX with no NULL check; the o… |
| `010f2320` | zcfree | likely | zlib zcfree(opaque, ptr): pushes the pointer (second argument) and calls _free (00d8a9d8). Returns nothing. |
| `010f27f0` | inflate_table | likely | Builds a DEFLATE canonical Huffman decode table: counts code lengths per symbol in a 16-entry array, finds the max and min used lengths, and fills 4-byte table… |

## Source shards

[[S0099|S0099]] (33), [[S0141|S0141]] (28), [[S0171|S0171]] (4), [[S0173|S0173]] (7), [[S0195|S0195]] (5), [[S0284 009f8470-009feec0|S0284]] (5), [[S0324 00ad8880-00adee10|S0324]] (2), [[S0437 00d297c0-00d2f870|S0437]] (8), [[S0438 00d2f8c0-00d34430|S0438]] (41), [[S0445 00d54b90-00d59cb0|S0445]] (5), [[S0529|S0529]] (42), [[S0530|S0530]] (60), [[S0534|S0534]] (29), [[S0540|S0540]] (1), [[S0541|S0541]] (7), [[S0600|S0600]] (17), [[S0601|S0601]] (29), [[S0602|S0602]] (1), [[S0797|S0797]] (1)
