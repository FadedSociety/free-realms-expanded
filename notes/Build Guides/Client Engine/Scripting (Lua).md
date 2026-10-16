---
type: build-guide
section: Client Engine
updated: 2026-10-12
---

# Scripting (Lua)

Part of [[Build Guides]] › Client Engine. Related: [[UI Framework (GFC and XML)]].

<!-- NARRATIVE START -->
## How it works

_Not written yet. An agent fills this from the shard findings, labels and video evidence: what the feature does, its data formats and packets, what the server must do, and open questions._
<!-- NARRATIVE END -->

## Mapped functions (generated)

457 accepted functions: 96 confirmed, 180 likely, 118 speculative, 63 unknown. Network-linked (`wire`): 9.

Subsystems: `script.lua` (257), `lib.lua.api` (73), `lib.lua.debug` (28), `lib.lua.aux` (27), `lib.lua.do` (22), `script.clientlua` (17), `lib.lua.object` (8), `script` (5), `script.metrics` (4), `lib.lua.loadlib` (4), `script.minigame` (3), `script.lobby` (3), `script.gameevents` (2), `script.quest` (1), `script.event` (1), `script.welcome` (1), `lib.lua.vm` (1)

### `lib.lua.api`

**(no class)** (73)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e3a5a0` | lua_typename | confirmed | Returns the string 'no value' for tag -1; otherwise returns the entry at index t of the global type-name pointer array (PTR_PTR_018541d8). |
| `00e3a180` | index2adr | likely | Maps a Lua stack index in EAX to a TValue address: positive index to base+idx-1 (or the nil object 01853f50 if past top); negative index to top+idx; LUA_REGIST… |
| `00e3a250` | lua_checkstack | likely | Fails (returns 0) when size is >= 0x1f41 or the used stack plus size reaches 0x1f41. Otherwise, for size > 0, grows the stack through luaD_growstack (00e3d5b0)… |
| `00e3a2c0` | lua_xmove | likely | If from and to differ: moves param_3 values from the top of from to the top of to, lowering from->top by n*0x10 and copying each TValue in order, then raising … |
| `00e3a330` | lua_atpanic | likely | Returns the previous panic function stored at global_State+0x58 and stores param_2 in its place. |
| `00e3a350` | lua_newthread | likely | Runs the GC check (luaC_step 00e3f070 when the threshold is crossed), creates a thread through 00e3b960, pushes it with tag 8 (LUA_TTHREAD) and raises top. |
| `00e3a390` | lua_gettop | likely | Returns the number of values in the current frame: (L->top - L->base) >> 4 (TValue is 16 bytes). |
| `00e3a3a0` | lua_settop | likely | For a negative index, moves top down by (idx+1)*16 (pops). For a non-negative index, sets top to base + idx*16, nil-filling new slots when growing. |
| `00e3a3f0` | lua_remove | likely | Resolves idx through index2adr (00e3a180), shifts every later TValue down one slot, then lowers top by 0x10. |
| `00e3a440` | lua_insert | likely | Resolves idx through index2adr (00e3a180), shifts the values from idx up to the top one slot higher, and stores the old top value at idx. |
| `00e3a550` | lua_pushvalue | likely | Copies the TValue at idx (via index2adr 00e3a180) to the stack top and advances top by 0x10. |
| `00e3a580` | lua_type | likely | Resolves idx through index2adr (00e3a180). Returns -1 when the slot is the nil object 01853f50 (no value); otherwise returns the tag dword at +8. |
| `00e3a5c0` | lua_iscfunction | likely | Returns 1 when the value at idx is tag 6 (function) and its closure isC flag at +6 is nonzero; otherwise 0. |
| `00e3a5f0` | lua_isnumber | likely | Returns 1 when the value at idx is a number (tag 3) or converts to one through luaV_tonumber (00e3f340); otherwise 0. |
| `00e3a630` | lua_isstring | likely | Returns 1 when idx is a valid value whose tag is 4 (string) or 3 (number, convertible); otherwise 0. |
| `00e3a690` | lua_rawequal | likely | Resolves both indices with index2adr (00e3a180); returns 0 if either is the nil object, otherwise the result of the raw-equality test 00e3cdb0. |
| `00e3a770` | lua_tonumber | likely | Returns the number at idx; when the tag is not 3 it converts through luaV_tonumber (00e3f340), returning 0 on failure. |
| `00e3a7f0` | lua_toboolean | likely | Returns 1 unless the value is nil (tag 0) or the boolean false (tag 1 with value 0); returns 0 otherwise. |
| `00e3a820` | lua_tolstring | likely | Resolves idx; for a non-string converts in place through luaV_tostring (00e3f390) with a GC check and returns NULL (writing len 0) when conversion fails. For a… |
| `00e3a890` | lua_objlen | likely | Resolves idx (00e3a180). Strings return the length at +0xc; userdata return the size at +0x10; tables return luaH_getn (00e41a60); numbers are converted with l… |
| `00e3a900` | lua_tocfunction | likely | For a function value whose closure isC flag is set returns the function pointer stored at closure+0x10; otherwise 0. |
| `00e3a930` | lua_touserdata | likely | Returns the light-userdata pointer (tag 2) directly, the full-userdata data pointer (tag 7, +0x18) or 0 for other tags. |
| `00e3a960` | lua_tothread | likely | Returns the lua_State pointer for a tag-8 (thread) value, else 0. |
| `00e3a980` | lua_topointer | likely | Switches on the tag: userdata (7) returns the data pointer (+0x18); light userdata (2) and tables, functions and threads (5, 6, 8) return the object pointer; o… |
| `00e3a9f0` | lua_pushnil | likely | Stores nil (tag 0) at the stack top and advances top by one slot. |
| `00e3aa10` | lua_pushnumber | likely | Stores the double argument at the stack top with tag 3 and advances top by one slot. |
| `00e3aa30` | lua_pushinteger | likely | Converts the int argument to double, stores it at the stack top with tag 3 and advances top by one slot. |
| `00e3aa50` | lua_pushlstring | likely | Runs the GC check (luaC_step 00e3f070 when the threshold is crossed), interns len bytes through luaS_newlstr (00e420b0), stores the string at the top with tag … |
| `00e3aa90` | lua_pushstring | likely | If the pointer is NULL pushes nil (tag 0) and returns. Otherwise computes strlen, runs the GC check, interns the string through luaS_newlstr (00e420b0) and pus… |
| `00e3ab00` | lua_pushvfstring | likely | Runs the GC check, then calls luaO_pushvfstring (00e3cf10) with the format and a va_list argument. |
| `00e3ab30` | lua_pushfstring | likely | Runs a GC check, then calls luaO_pushvfstring (00e3cf10) with the format and the variadic arguments taken from the stack. |
| `00e3ab60` | lua_pushcclosure | likely | Runs a GC check, creates a C closure with n upvalues through luaF_newCclosure (00e42250, env from the current closure or the global table), stores the function… |
| `00e3ac00` | lua_pushboolean | likely | Stores (param_2 != 0) as a boolean at the stack top with tag 1 and advances top by one slot. |
| `00e3ac20` | lua_pushlightuserdata | likely | Stores the pointer argument at the stack top with tag 2 (light userdata) and advances top by one slot. |
| `00e3ac40` | lua_pushthread | likely | Pushes the current lua_State as a thread value (tag 8) and returns whether it is the main thread (global_State+0x70 equals L). |
| `00e3ac70` | lua_gettable | likely | Resolves the table at idx (00e3a180) and calls luaV_gettable (00e3f5e0) with the key at top-1, replacing the key with the result. |
| `00e3aca0` | lua_getfield | likely | Resolves idx through index2adr (00e3a180), pushes the key string (luaS_newlstr 00e420b0 of k with the length computed by a strlen loop, tag 4) and calls luaV_g… |
| `00e3ad00` | lua_rawget | likely | Resolves the table at idx (00e3a180), looks up the key at top-1 with luaH_get (00e41850) and writes the result back into the key slot, replacing the key. |
| `00e3ad40` | lua_rawgeti | likely | Resolves the table at idx (00e3a180) and reads the array slot for integer key param_3 through luaH_getnum (00e41770), then pushes the copied value with its tag. |
| `00e3ad80` | lua_createtable | likely | Runs the GC check, creates a table with luaH_new (00e41680) and pushes it with tag 5 (LUA_TTABLE). |
| `00e3adc0` | lua_getmetatable | likely | Resolves idx (00e3a180) and reads the metatable: table and userdata at +8, otherwise the global per-type metatable at G+0x98 plus 4*tag. If it is NULL returns … |
| `00e3ae20` | lua_getfenv | likely | For a function or userdata at idx pushes its environment table (value at +0xc) with tag 5; for a thread pushes a copy of its global-table TValue at +0x48; any … |
| `00e3ae90` | lua_settable | likely | Resolves the table at idx (00e3a180) and calls luaV_settable (00e3f6d0) with the key at top-2 and the value at top-1, then pops two values. |
| `00e3aec0` | lua_setfield | likely | Resolves idx (00e3a180), interns the key string (00e420b0), stores it as a string TValue below top and calls luaV_settable (00e3f6d0) with table, key and the v… |
| `00e3af20` | lua_rawset | likely | Resolves the table at idx (00e3a180), stores the value at top-1 into the slot returned by luaH_set (00e41900) for the key at top-2, runs a write barrier (00e3f… |
| `00e3af90` | lua_rawseti | likely | Resolves the table at idx (00e3a180), stores the value at top-1 into the slot for integer key param_3 through luaH_setnum (00e41970), runs a write barrier (00e… |
| `00e3b000` | lua_setmetatable | likely | Pops the metatable (or nil) from the top. For tables and userdata stores it at +8 of the object (with a write barrier when collectable); for other tags stores … |
| `00e3b0b0` | lua_setfenv | likely | Resolves idx (00e3a180). For a function or userdata stores the top table at +0xc; for a thread stores it in the thread global slot (+0x48) with tag 5 at +0x50;… |
| `00e3b140` | lua_call | likely | Calls luaD_call (00e3de60) on the function at top-(nargs+1) with nresults; when nresults is -1 (LUA_MULTRET) raises ci->top to top if it is below. |
| `00e3b1a0` | lua_pcall | likely | Converts the errfunc index (if nonzero) to a stack offset, then runs luaD_pcall (00e3e000) with the f_call body (00e3b180) on the function at top-(nargs+1) and… |
| `00e3b210` | f_Ccall | likely | Protected body for lua_cpcall: takes a two-field block (C function pointer, user data), creates a C closure with zero upvalues (luaF_newCclosure 00e42250) whos… |
| `00e3b280` | lua_cpcall | likely | Stores the function and user-data pointers in a two-field block and runs luaD_pcall (00e3e000) with f_Ccall (00e3b210) as the protected body, passing the curre… |
| `00e3b2c0` | lua_load | likely | Initialises a ZIO stream from the reader and data (luaZ_init 00e42700), then runs luaD_protectedparser (00e3e100) on it with the chunk name; a NULL name become… |
| `00e3b300` | lua_dump | likely | If the value on top is a Lua function (tag 6 with isC clear at +6) calls luaU_dump (00e42df0) with its prototype at +0x10, the writer and data; otherwise retur… |
| `00e3b340` | lua_status | likely | Returns the byte at +6 of the lua_State, the thread status field. |
| `00e3b350` | lua_gc | likely | Switches on the what code: 0 sets the GC threshold to a stop value, 1 restarts it, 2 runs a full collection (00e3f0f0), 3 and 4 return the allocated size in KB… |
| `00e3b450` | lua_error | likely | Calls luaG_errormsg (00e3c900) on the top error value, which does not return, then returns 0. |
| `00e3b460` | lua_next | likely | Resolves the table at idx (00e3a180) and calls luaH_next (00e41270) with the key at top-1; on success the key and value pair is left on the stack (top raised b… |
| `00e3b4a0` | lua_concat | likely | For n greater than 1 runs luaV_concat (00e3fbd0) over the top n values, after a GC check. For n equal to 0 pushes the empty string (luaS_newlstr 00e420b0 with … |
| `00e3b570` | lua_newuserdata | likely | Runs the GC check, creates a userdata of the requested size through luaS_newudata (00e421f0) with the environment of the current closure (or the global table),… |
| `00e3b6b0` | lua_setupvalue | likely | For a function value at funcindex with a valid upvalue number n: pops the value on top into the upvalue slot (C closure upvalue or Lua closure upvalue), runs a… |
| `00e3b8d0` | close_state | likely | Releases the state: calls luaF_close (00e42420) and the state-teardown routine 00e3ecb0, frees a block through luaM_realloc_ (00e42ea0), and finally calls the … |
| `00e3b960` | luaE_newthread | likely | Allocates a 0x78-byte lua_State through luaM_realloc_ (00e42ea0), links it to the GC list with luaC_link (00e3f1c0, tag 8), copies the global-state fields from… |
| `00e3ba00` | luaE_freethread | likely | Closes open upvalues (luaF_close 00e42420), frees the CallInfo block (size_ci * 0x18), the value stack (stacksize * 16) and the 0x78-byte lua_State through lua… |
| `00e3ba60` | lua_newstate | likely | Allocates a 0x178-byte global state through the allocator argument (param_1(ud, NULL, 0, 0x178)), zeroes and wires its fields (mainthread, tag bytes, string ta… |
| `00e3bba0` | lua_close | likely | Switches to the main thread (global_State+0x70), closes its upvalues (luaF_close 00e42420), separates userdata with finalizers (00e3e290), then repeatedly runs… |
| `00e3a220` | luaA_pushobject | speculative | Copies the three dwords of the TValue at param_2 onto the stack top and advances L->top by 0x10. No other effect. |
| `00e3a310` | CopyNCcallsField | speculative | Copies the 16-bit field at +0x34 (nCcalls) from the lua_State at param_1 to the state at param_2. |
| `00e3a490` | SetFunctionEnvFromTop | speculative | With idx -10001 (LUA_ENVIRONINDEX) and the frame at base_ci raises 'no calling environment' through luaG_runerror (00e3c990). Otherwise stores the value at top… |
| `00e3a730` | lua_equal | speculative | Resolves both indices (00e3a180); returns 0 if either is invalid, otherwise the result of luaV_equalval (00e3f9c0) with metamethods. |
| `00e3a7b0` | — | unknown | Same shape as lua_tonumber: uses the number directly when tag is 3, else converts through luaV_tonumber (00e3f340); returns the value truncated to int, or 0 on… |
| `00e3b620` | — | unknown | For a function value at funcindex (tag 6) with a valid upvalue number n: for a C closure returns the empty name (DAT_0175b400) and pushes upvalue n-1 at closur… |
| `00e3b760` | — | unknown | Without arguments (state in ESI): allocates a 0x140-byte value stack and a base CallInfo through luaM_realloc_ (00e42ea0) and links them; sets stack limits, st… |

### `lib.lua.aux`

**(no class)** (27)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e39660` | errfile | confirmed | Reads errno and strerror text, pushes the message 'cannot <what> <file>: <strerror>' formatted with lua_pushfstring (00e3ab30) using the file name at fnameinde… |
| `00e396a0` | luaL_loadfile | confirmed | Pushes '=stdin' (or '@<name>') as the chunk name, opens the file (_fopen, or stdin when the name is NULL), skips a first line starting with '#', and when the n… |
| `00e399b0` | luaL_argerror | confirmed | Gets the calling frame (lua_getstack 00e3bd30) and name info (lua_getinfo 00e3cb10). If there is no frame, raises 'bad argument #%d (%s)'. For a method call de… |
| `00e39aa0` | typeerror | confirmed | Gets the type name of arg narg (lua_type 00e3a580, lua_typename 00e3a5a0), formats '%s expected, got %s' with tname via lua_pushfstring (00e3ab30) and raises i… |
| `00e39c00` | luaL_checkany | confirmed | If lua_type(narg) (00e3a580) equals -1 (LUA_TNONE, no value) raises 'value expected' through luaL_argerror (00e399b0). Otherwise returns. |
| `00e3a0b0` | luaL_checkoption | confirmed | Reads the option name (default when given and nil/none, else luaL_checklstring 00e39c30), then walks the NULL-terminated option list comparing with the name; r… |
| `00e390a0` | luaL_getmetafield | likely | Calls lua_getmetatable(L,obj) (00e3adc0); returns 0 if none. Otherwise pushes the event string (00e3aa90), lua_rawget(L,-2) (00e3ad00); if the result is nil it… |
| `00e39100` | luaL_callmeta | likely | Converts a negative obj index to absolute (param_2+1+lua_gettop) when needed, calls luaL_getmetafield (00e390a0); if it returns 0 returns 0. Otherwise pushes a… |
| `00e39330` | luaL_prepbuffer | likely | Pushes the pending buffer bytes (B->p - (B+0xc)) as a string when non-empty (lua_pushlstring), increments B->lvl, resets B->p to the buffer, runs adjuststack (… |
| `00e39370` | luaL_addlstring | likely | Copies param_3 bytes from param_2 into the luaL_Buffer one at a time; when the write pointer reaches the end of the 0x200-byte inline buffer (B+0x20c) it flush… |
| `00e39400` | luaL_pushresult | likely | Flushes the pending buffer bytes as a string when non-empty (lua_pushlstring), calls lua_concat(B->L, B->lvl) (00e3b4a0) and sets B->lvl to 1. |
| `00e39440` | luaL_addvalue | likely | Converts the value at the top (lua_tolstring 00e3a820) to bytes with its length; if it fits in the free part of the luaL_Buffer (B+0x20c minus B->p) copies it … |
| `00e394d0` | luaL_buffinit | likely | Stores the lua_State pointer at B+8, sets B->p to B+0xc (the inline buffer) and B->lvl to 0. No calls. |
| `00e39600` | getF | likely | lua_load reader for a FILE: if the extraline flag at +0 is set it clears it and returns the static newline string (DAT_01770548) with size 1; if feof is set re… |
| `00e398b0` | luaL_loadbuffer | likely | Builds a two-field load block {buff, size} on the stack and calls lua_load(L, getS reader at 00e39890, &block, name). The name argument is passed through as th… |
| `00e398e0` | luaL_loadstring | likely | Computes strlen(s) (loop, stored in a local), then calls lua_load(L, getS reader at 00e39890, block, s) with s as the chunk name. |
| `00e39980` | luaL_newstate | likely | Calls lua_newstate(00e39920 allocator, NULL userdata) (00e3ba60); when non-null installs the panic function at 00e39950 with lua_atpanic (00e3a330) and returns… |
| `00e39b20` | luaL_checkudata | likely | lua_touserdata(narg) (00e3a930); when non-null and it has a metatable (00e3adc0) compares that metatable with the registry entry tname (lua_getfield 00e3aca0 o… |
| `00e39bb0` | luaL_checktype | likely | If lua_type(narg) (00e3a580) differs from the expected type t, builds '%s expected, got %s' from the type names and raises it through luaL_argerror (00e399b0). |
| `00e39c30` | luaL_checklstring | likely | Calls lua_tolstring(narg, len) (00e3a820); when the result is NULL, builds '%s expected, got %s' with the string type name (lua_typename of tag 4) and raises i… |
| `00e39c90` | luaL_optlstring | likely | If lua_type(narg) (00e3a580) is nil or none, stores the default length (strlen(def) or 0) through len when len is non-null and returns def; otherwise returns l… |
| `00e39cf0` | luaL_checknumber | likely | Calls lua_tonumber (00e3a770); if the result is 0 and lua_isnumber (00e3a5f0) is false, raises a type error with LUA_TNUMBER (3) as the expected type through l… |
| `00e39d70` | luaL_optnumber | likely | If lua_type(narg) (00e3a580) is nil or none returns the default double; otherwise returns luaL_checknumber (00e39cf0). |
| `00e39da0` | luaL_checkinteger | likely | Calls lua_tointeger (00e3a7b0); if the result is 0 and lua_isnumber (00e3a5f0) is false, raises a number type error (LUA_TNUMBER) through luaL_argerror. Return… |
| `00e39e10` | luaL_optinteger | likely | If lua_type(narg) (00e3a580) is nil or none returns the default; otherwise returns luaL_checkinteger (00e39da0). |
| `00e39f60` | luaL_gsub | likely | Builds a luaL_Buffer (0x200 inline) that copies s with every occurrence of p (strstr 00d8b390) replaced by r, appending pieces with luaL_addlstring (00e39370),… |
| `00e392b0` | — | unknown | When buffer level B->lvl exceeds 1, sums lua_objlen of the top string pieces and, while the level count is under 10 and the next piece is not longer than the r… |

### `lib.lua.debug`

**(no class)** (28)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e3bdd0` | findlocal | confirmed | For a Lua frame with a valid proto, asks luaF_getlocalname (00e42610) for the name of local n; when there is none and n is within the frame's live stack return… |
| `00e3c620` | getobjname | confirmed | Walks back from pc with symbexec (00e3c110) to the instruction that last set register reg. Returns 'local' when luaF_getlocalname (00e42610) names it; for a GE… |
| `00e3c870` | addinfo | confirmed | If the current frame is a Lua function, builds its short source name with luaO_chunkid (00e3d200, 0x3c bytes), obtains currentline (00e3bc80) and pushes the me… |
| `00e3c9c0` | auxgetinfo | confirmed | Fills the lua_Debug record for the requested option letters: S (funcinfo 00e3bee0), l (currentline 00e3bc80), n (name via getfuncname 00e3c7a0, or NULL), u (up… |
| `00e3cbc0` | luaG_typeerror | confirmed | Builds 'attempt to <op> <type> ...' with the variable kind and name from getobjname (00e3c620) when the value is a named register, then raises it through luaG_… |
| `00e3cc50` | luaG_concaterror | confirmed | Passes the string 'concatenate' with the operand to luaG_typeerror (00e3cbc0), selecting the non-string operand. |
| `00e3cc80` | luaG_aritherror | confirmed | If the first operand is not convertible to a number (luaV_tonumber 00e3f340) it is the bad operand; passes 'perform arithmetic on' with it to luaG_typeerror (0… |
| `00e3ccc0` | luaG_ordererror | confirmed | Compares the type names of the two operands: when equal raises 'attempt to compare two <t> values', otherwise 'attempt to compare <t1> with <t2>' through luaG_… |
| `00e3bcd0` | lua_sethook | likely | Clears the hook when the function or mask is zero; stores the hook function at L+0x44, the base count at +0x3c (and the live count at +0x40), the mask byte at … |
| `00e3bd00` | lua_gethook | likely | Returns the hook function stored at L+0x44. |
| `00e3bd10` | lua_gethookmask | likely | Returns the byte at L+0x38 (the hook mask). |
| `00e3bd20` | lua_gethookcount | likely | Returns the dword at L+0x3c (the base hook count). |
| `00e3bd30` | lua_getstack | likely | Walks the CallInfo list down from L->ci by level frames (decrementing by the CallInfo size 0x18 and skipping Lua frames by their extra counts). When the level … |
| `00e3be40` | lua_getlocal | likely | Resolves the local name for slot n of the frame in ar (via findlocal 00e3bdd0); when a name is found pushes a copy of the slot value (lua_pushobject 00e3a220) … |
| `00e3be90` | lua_setlocal | likely | Resolves the local name for slot n with findlocal (00e3bdd0); when found stores the value at the top into the slot and pops it. Always pops one value and retur… |
| `00e3bee0` | funcinfo | likely | Fills the debug record from a closure: for a C function sets source to the constant '=[C]' (DAT_01853e5c), linedefined and lastlinedefined to -1 and what to 'C… |
| `00e3bfa0` | CollectActiveLines | likely | For a Lua function, creates an empty table (luaH_new 00e41680) and sets each of the proto line-info entries as a key with value true (luaH_setnum 00e41970). Fo… |
| `00e3c7a0` | getfuncname | likely | Given a CallInfo, returns NULL when the frame is a tail call or the caller frame is not Lua; otherwise reads the calling instruction at the caller's pc and, wh… |
| `00e3c900` | luaG_errormsg | likely | If L+0x74 (errfunc) is set, copies the error value and calls the handler (luaD_call 00e3de60, one result) on it; then throws with luaD_throw (00e3dc00) using s… |
| `00e3c990` | luaG_runerror | likely | Formats the message with luaO_pushvfstring (00e3cf10) from the format and variadic arguments, prefixes source position through addinfo (00e3c870), then raises … |
| `00e3cb10` | lua_getinfo | likely | Selects the frame from the stack when what starts with '>' (popping the function) or from ar+0x60; calls auxgetinfo (00e3c9c0). For 'f' pushes the function val… |
| `00e3c030` | MatchPendingReturnOp1E | speculative | Tests a proto-like record: its byte fields and the stack limit, then whether the last instruction at pc-1 has opcode 0x1e (30, RETURN in Lua 5.1). Returns 1 wh… |
| `00e3c090` | MatchCallOrReturnOpcode | speculative | Takes an instruction word: opcode is the low 6 bits; returns true for opcodes 0x1c..0x1e and 0x22 when the high 9 bits (0xff800000 mask) are clear, false other… |
| `00e3c0c0` | CheckOperandMode | speculative | Switches on the EAX mode (0, 2 or 3) and checks param_1 against the proto limits: mode 0 requires zero; mode 2 requires param below the upvalue/stack limit at … |
| `00e3c5d0` | SymbexecLastPcCheck | speculative | Returns true when symbexec (00e3c110) finds a setter for register 0xff at the proto's saved pc (+0x2c). |
| `00e3c5f0` | ConstStringOrDefault | speculative | With the operand in EAX: if bit 8 (constant flag) is set and the constant at index (operand & 0xfeff) in the proto constant table is a string (tag 4), returns … |
| `00e3bc80` | — | unknown | Takes the CallInfo in ECX and the state in the stack argument. For a Lua closure (tag 6, isC clear) returns the source line of the saved pc: (savedpc - code)/4… |
| `00e3c110` | — | unknown | Scans the instructions of a proto from pc 0 up to lastpc and returns the last pc that sets register reg, using an opcode-mode table at 01855b64 (set-A bit 0x40… |

### `lib.lua.do`

**(no class)** (22)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e3d360` | luaD_seterrorobj | confirmed | Sets the error object for a failed protected call: for codes 2 and 3 copies the top value to the target slot; code 4 pushes 'not enough memory' and code 5 push… |
| `00e3da20` | luaD_yield | confirmed | Raises the yield-across-boundary error through luaG_runerror (00e3c990) when the C-call count exceeds the base; otherwise records the result base, sets the sta… |
| `00e3dc50` | luaD_growCI | confirmed | When the CallInfo count exceeds 20000 throws through luaD_throw (00e3dc00); grows the CallInfo array (luaD_reallocCI 00e3d530), advances L->ci by one record an… |
| `00e3de60` | luaD_call | confirmed | Increments the C-call counter (+0x34) and raises 'C stack overflow' at 200 nested calls (luaG_runerror 00e3c990) or throws at above 224 (luaD_throw 00e3dc00); … |
| `00e3df60` | lua_resume | confirmed | Refuses to resume a thread that is running or dead with resume_error (00e3d9a0) and message 'cannot resume non-suspended coroutine'; limits the nesting to 200,… |
| `00e3d3e0` | luaD_rawrunprotected | likely | Links a jump buffer into the state's error chain (+0x70), runs setjmp (__setjmp3), and on normal return calls the protected function with the state and user da… |
| `00e3d4d0` | luaD_reallocstack | likely | Reallocates the value stack to param_2 slots plus 6 (luaM_realloc_ 00e42ea0 with the old size), or luaM_toobig (00e42e80) for huge sizes; rebases top, base, Ca… |
| `00e3d530` | luaD_reallocCI | likely | Reallocates the CallInfo array to param_2 entries of 0x18 bytes (luaM_realloc_ 00e42ea0) or luaM_toobig (00e42e80) when the size is too large, and updates the … |
| `00e3d5b0` | luaD_growstack | likely | Doubles the value stack when the request n fits in the current size, otherwise grows it by n, through luaD_reallocstack (00e3d4d0). |
| `00e3d5e0` | luaD_callhook | likely | If a hook is installed (L+0x44) and hooks are allowed (L+0x39), records the event and line in a local block, reserves stack space (luaD_reallocstack 00e3d4d0),… |
| `00e3d6d0` | adjust_varargs | likely | Nil-fills the fixed parameters that were not supplied; when the proto flag 4 (vararg with arg table) is set creates a table (luaH_new 00e41680) holding the ext… |
| `00e3d820` | tryfuncTM | likely | Makes a non-function value callable: looks up the __call metamethod (luaT_gettmbyobj 00e3f2e0); when the value is not a function raises through luaG_typeerror … |
| `00e3d8b0` | callretphook | likely | Fires the return hook (luaD_callhook 00e3d5e0) and then adjusts the frame chain while the ci hook-count field is set and the hook flags request it; returns the… |
| `00e3d9a0` | resume_error | likely | Sets the thread's top to its base, pushes the message string (luaS_newlstr 00e420b0, tag 4), and returns 2 (LUA_ERRRUN) after growing the stack if needed. |
| `00e3da60` | f_parser | likely | Protected parser body: looks at the first byte of the input stream (luaZ_lookahead 00e426a0); if it is the bytecode signature 0x1b calls luaU_undump (00e504f0)… |
| `00e3db80` | resetstack | likely | Resets the thread after an error: ci back to base_ci, base set to its base, closes upvalues (luaF_close 00e42420), sets the error object (luaD_seterrorobj 00e3… |
| `00e3dc00` | luaD_throw | likely | With an active jump buffer (L+0x70) stores the error code and longjmps to it. Otherwise sets the status byte and, when the global panic function (G+0x58) is se… |
| `00e3dc90` | luaD_precall | likely | Prepares a call at func. A non-function callee goes through tryfuncTM (00e3d820). For a Lua proto: grows the stack if needed, takes a new CallInfo (00e3dc50), … |
| `00e3dee0` | resume | likely | Continues a coroutine: when it is not yielded calls luaD_precall (00e3dc90) on the function below the arguments and, for a Lua frame, runs luaV_execute (00e3ff… |
| `00e3e000` | luaD_pcall | likely | Saves the frame and stack positions and the error-handler field, runs luaD_rawrunprotected (00e3d3e0) on the protected function; on error closes upvalues (luaF… |
| `00e3e100` | luaD_protectedparser | likely | Builds a parser block {stream, name} and runs f_parser (00e3da60) through luaD_pcall (00e3e000) with the current stack offset and the error-handler field; then… |
| `00e3d910` | — | unknown | If the return hook mask (L+0x38 bit 2) is set, runs callretphook (00e3d8b0). Pops the current CallInfo, moves the result values down to the function slot and f… |

### `lib.lua.loadlib`

**(no class)** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e39e40` | luaL_openlib | confirmed | Counts the luaL_Reg entries; when libname is non-null, runs findtable (00e39180) on _LOADED in the registry, reads or creates the module table via lua_getfield… |
| `00e39180` | findtable | likely | Requires 0x200 free stack slots (00e3a250). For each dot-separated part of fname (split with strchr '.'), pushes the part, lua_rawget on the table at idx, and … |
| `00e3a160` | Wrapper_00e3a160 | likely | Forwards its three arguments to luaL_openlib (00e39e40) with nup = 0 and returns nothing. |
| `00e3bc00` | luaL_openlibs | likely | Walks a table of (name, luaopen function) pairs. For each entry pushes the function as a C closure with no upvalues (00e3ab60), pushes its name (00e3aa90) and … |

### `lib.lua.object`

**(no class)** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e3d200` | luaO_chunkid | confirmed | Builds a printable source name in a buffer of the given size: '=' names are copied (truncated), '@' names keep the file part, and other sources become [string … |
| `00e3cd20` | luaO_int2fb | likely | Encodes an unsigned integer into the 8-bit floating byte form: divides by two (rounding up) while the value is 16 or more counting exponent steps, returns the … |
| `00e3cd60` | luaO_fb2int | likely | Decodes the floating-byte form: exponent is (x>>3)&31; zero exponent returns x, otherwise returns ((x&7)+8) shifted left by exponent-1. |
| `00e3cd80` | luaO_log2 | likely | Returns floor(log2(x)) for a 32-bit value by byte-shifting and looking up the remaining byte in the 256-entry log2 table at 01853f60. |
| `00e3ce10` | luaO_str2d | likely | Converts a string to a double: strtod-like conversion (FUN_00d866d0), a hex branch for a 0x/0X prefix through strtoul base 16, then accepts only trailing white… |
| `00e3cec0` | pushstr | likely | Interns a NUL-terminated string (luaS_newlstr 00e420b0) and stores it at the current string slot with tag 4 then advances top; grows the stack through luaD_gro… |
| `00e3cf10` | luaO_pushvfstring | likely | Pushes an empty string then scans the format for '%': copies literal pieces with luaS_newlstr (00e420b0) and handles %s and %c through pushstr (00e3cec0), %d a… |
| `00e3d1e0` | luaO_pushfstring | likely | Forwards the format and its variadic arguments to luaO_pushvfstring (00e3cf10). |

### `lib.lua.vm`

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e3cdb0` | luaV_rawequalobj | likely | Compares two TValues: unequal tags give 0; equal nil tags give 1; for numbers compares the doubles; for other tags compares the value words. |

### `script`

**(no class)** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a1a7d0` | InventoryHandler::OnIsItemUsable6 | confirmed | Creates or reuses a Scaleform parameter array, writes six arguments including a normalized boolean first argument, dispatches InventoryHandler:OnIsItemUsable, … |
| `00a4e130` | Script_InvokeNamedMethod | likely | Builds a SoeUtil::StringFixed<256> from param_1 (constructor FUN_00725ee0, which stores the StringFixed<256> vtable) and invokes the global Lua/script bridge F… |
| `00903b40` | LuaCallNameWithIntArg | speculative | Stack args param_1 (Lua function name, passed to FUN_009797a0) and param_2 (integer). Builds a one-entry Lua argument array with FUN_0070ec40, stores tag 1 and… |
| `00904060` | LuaCallNameWithIntArg | speculative | Stack args param_1 (Lua function name) and param_2 (integer). Byte-for-byte the same body as 00903b40 apart from its SEH handler address: builds a one-entry Lu… |
| `00a1a4b0` | — | unknown | Creates or reuses a Scaleform parameter array, writes five arguments with tag 1, dispatches InventoryHandler:OnIsItemUsable, and cleans up the temporary data. |

### `script.clientlua`

**AchievementDataSource** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ce4c30` | AchievementDataSource::SetTotalPoints | confirmed | Virtual slot [14] of AchievementDataSource::vftable (no stack arguments): zeroes the total at this+0x274, walks the node list headed at the object at this+0x25… |

**ClientLuaOwner** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008181d0` | ClientLuaOwner::DtorBody | confirmed | Destructor body. Stores ClientLuaOwner::vftable at *this and nothing else. Called from two EH unwind funclets. |
| `008181e0` | ClientLuaOwner::ScalarDeletingDtor | confirmed | Thiscall scalar deleting destructor, vtable slot 0 of ClientLuaOwner. Stores ClientLuaOwner::vftable at *this, then if the low bit of the flag argument is set … |

**(no class)** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00818280` | ClientLuaSetIsModalUp | confirmed | Parameterless wrapper with one stack argument. Calls 009763c0 (ClientLuaSetClientIntField) with the string name IsModalUp and the argument. |
| `008182a0` | ClientLuaGetIsModalUp | confirmed | Parameterless. Calls 00976630 (ClientLuaReadClientIntField) with the field name IsModalUp and a local dword; returns the local when the read succeeds and 0 whe… |
| `008182c0` | ClientLuaSetModalResult | confirmed | Parameterless wrapper with one stack argument. Calls 009763c0 (ClientLuaSetClientIntField) with the field name ModalResult and the argument. |
| `008182e0` | ClientLuaGetModalResult | confirmed | Parameterless. Calls 00976630 (ClientLuaReadClientIntField) with the field name ModalResult and a local dword; returns the local when the read succeeds and 0 w… |
| `008ed9c0` | ChatHandlerStartChatBind | confirmed | Fastcall on this. Resolves the "ChatHandler" name through 00977740 into a local string; when that returns nonzero it appends ":StartChat" (00402770) and calls … |
| `007121e0` | TaggedValueSet | likely | Dispatches on the source tag at [param_2]: tag 1 calls 007071f0; tag 2 calls 00707290 with the 8-byte payload at param_2+8; tag 4 calls 00707240 with the dword… |
| `00713b70` | TaggedValueCtorFrom | likely | Sets [this] to 0 and then calls TaggedValueSet (007121e0) on this with the stack argument as the source value; returns this (ret 4). |
| `00713cf0` | TaggedArray0xA0_Reallocate | likely | Reallocates the 0xa0-byte element buffer of the array at ECX (vtable at [ECX], buffer [ECX+4], count [ECX+8], capacity [ECX+0xc]): allocates the new buffer thr… |
| `010cb770` | FireOnDataChanged | likely | Thiscall with three stack arguments (selector S, value V, string T; ret 0xc): calls the owner's vtable slot 9 (+0x24) for a name, appends _OnDataChanged to a c… |
| `00cb6da0` | IStringClientLuaArrayCtor | speculative | Stores an IString<char> vtable at param_1[0], sets param_1[1] to the empty-string static and clears [2],[3]; calls 00402ce0 with param_2; stores the Array<Clie… |
| `00cb6e60` | CtorWrapperDerefArgs | speculative | Thiscall wrapper: ECX is the object to construct; dereferences the four pointer arguments (*param_2..*param_5), calls 00cb6da0 with ECX unchanged and those fou… |
| `010cb3a0` | — | unknown | Thiscall (ECX = owner, no stack arguments): calls the owner's vtable slot 9 (+0x24) to get a name string, copies it into a local SoeUtil::IString<char>, append… |
| `010cb4e0` | — | unknown | Thiscall (ECX = owner, no stack arguments): calls the owner's vtable slot 9 (+0x24) for a name, copies it into a local SoeUtil::IString<char>, appends _OnDataR… |
| `010cb620` | — | unknown | Thiscall (ECX = owner, no stack arguments): calls the owner's vtable slot 9 (+0x24) for a name, copies it into a local SoeUtil::IString<char>, appends _OnDataM… |

### `script.event`

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c722f0` | RaiseFormattedScriptEvent | speculative | If the C string argument is non-empty: builds a temporary StringFixed<256> (004029b0), calls 00977740 with it, and when that returns non-zero formats the templ… |

### `script.gameevents`

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008ff1c0` | UpdateAssetsLoadingEvent | confirmed | fastcall (ECX=this, /GS frame). Computes a pending-size value: 00769710 result's vtable slot +0x3c plus 00ad3270, stored as a local count (iStack_1e8). When th… |
| `00900c20` | SetGameTimeScalar | confirmed | Stores param_2 (a dword) at this+0x798, builds a one-entry Lua argument array (FUN_0070ec40) whose entry is the float at this+0x798 converted to double (tag 2)… |

### `script.lobby`

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a241e0` | FireLobbyGameIsStartingEvent | speculative | Appends the literal "Lobby:GameIsStarting" to the global script string at 01be670c with SoeUtil::IStringAppend and then runs it through the script/Lua invocati… |
| `00a242e0` | FireLobbyGameIsLaunchingEvent | speculative | Appends the literal "Lobby:GameIsLaunching" to the global script string at 01be670c with SoeUtil::IStringAppend and invokes the script routine 009797a0 on it. |
| `00a243e0` | FireLobbyShowEvent | speculative | Builds an IString over the global script string 01be670c with the literal "Lobby:Show", invokes the script routine 009797a0 on it, and then releases the string… |

### `script.lua`

**ClientLuaInternals** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00979120` | ClientLuaInternals::ClientLuaInternals | confirmed | Installs the CommandProcessor::CommandEventHandler vtable then the ClientLuaInternals vtable at this+0, calls FUN_00fc4580(4000) on this+8, stores param_2 at +… |
| `009791c0` | ClientLuaInternals::DtorBody | confirmed | Stores the ClientLuaInternals vtable at this+0, calls FUN_00986c10 with this as its argument, closes the Lua state at +0x94 with FUN_00e3bba0 when it is non-nu… |
| `00979260` | ClientLuaInternals::ScalarDeletingDtor | confirmed | Calls the ClientLuaInternals destructor body 009791c0 on this, then frees the object with _free when bit 0 of the flag argument is set; returns this. |
| `00976cb0` | ClientLuaInternalsInitState | likely | Creates the Lua state with FUN_00e39980 and stores it at this+0x94 (returns 0 when that is null). Zeroes this+0x1ac, sets this+0x1b8 = 1000, stores a clock rea… |

**ExecuteLuaEvent** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009c5290` | ExecuteLuaEventScalarDeletingDtor | confirmed | Calls the event teardown routine, frees the allocation only when the deleting flag is set, and otherwise returns the original object pointer. |
| `00c4fb60` | ExecuteLuaEvent::Run | confirmed | Calls 009797a0 with fields at this+0x48 and this+0x54, then returns the result with its low byte cleared. |
| `00c4fb80` | ExecuteLuaEvent::Dtor | confirmed | Restores the event vtable, destroys its ClientLuaParam array and aligned buffer, releases its string buffer, and tears down the base object. |
| `009c51d0` | — | unknown | Initializes the Lua event base, its string member and parameter array, then reserves array elements based on the supplied input array size. |

**Factory** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bfecb0` | Factory::RequestEquipTool | speculative | Checks for the required Lua argument, extracts its value, obtains the factory context, and invokes 009e8cd0 to construct/send the tool equip request. |

**Mount** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bffbc0` | Mount::Upgrade 🌐 | confirmed | Requires two Lua arguments, builds a sub-opcode 14 mount packet, reads mount and item-definition ids, conditionally sends it, and logs both ids. |
| `00bff8e0` | Mount::Despawn 🌐 | likely | For one Lua argument, checks mount state; if active, logs despawn, builds and conditionally sends PacketDismountRequest, otherwise invokes the mount button cal… |
| `00bff6f0` | — 🌐 | unknown | For one Lua argument, checks mount state; if no mount is active it builds and conditionally sends dismount and spawn packets, records the selected mount id, lo… |
| `00bffa60` | — 🌐 | unknown | Checks mount state; when active, logs dismount, builds and conditionally sends PacketDismountRequest, otherwise refreshes the mount button callback. |

**PlayClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008baab0` | PlayClient::InvokeGuiShutdownCallback | confirmed | If a global GUI flag is set, calls FUN_00995b80; then checks for Lua callback GuiOnShutdown and invokes it with two null arguments when registered. |

**SocialShare** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bfe710` | SocialShare::RequestLoginURL | confirmed | Requires a Lua stack of two arguments and provider ID 1 or 2, formats a diagnostic string, calls the provider login-URL helper, and returns zero. |
| `00bfe600` | SocialShare::GetProviderStatus | likely | Requires a Lua stack of two arguments and provider ID 1 or 2, formats a diagnostic string, invokes the provider status callback, and returns zero. |
| `00bfe7e0` | SocialShare::Callback | speculative | Requires three Lua arguments with types accepted by the checks, extracts values from positions two and three, and calls 00adffe0 for provider ID 1 or 2. |

**SoeUtil::Array<ClientLuaParam,0,1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008ebdd0` | — | unknown | Initializes the destination fixed-string member and ClientLuaParam array vtable, zeros array count/capacity fields, and assigns the supplied name. If a source … |

**SoeUtil::Array<ClientLuaParam,20,1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a18960` | Array_ClientLuaParam_BufferHook | confirmed | Virtual slot 1 of the Lua-parameter array: tail-jumps to the generic grow routine 00708950 when the requested count exceeds 0x14, otherwise reports capacity 0x… |
| `00a188c0` | Array_ClientLuaParam_Dtor | likely | Destructor of the Lua-parameter array: stores the 20-inline-element vftable, destroys the live elements through FUN_0070b4e0 (which releases the variant payloa… |

**(no class)** (237)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00976010` | ClientLuaRegisterGlobalFunctions | confirmed | Void, one lua_State* argument. Walks the (name, function) pairs at 0180b1d0 (8 bytes each, terminated by a zero function pointer). For each pair: lua_pushcclos… |
| `009763c0` | ClientLuaSetClientIntField | confirmed | Thiscall on a holder whose pointer is at this+4 (its first dword is the ClientLuaInternals object X; the Lua state is *(X+0x94)), with two stack arguments (nam… |
| `00976490` | ClientLuaSetClientFloatField | confirmed | Thiscall on the same holder at this+4 with two stack arguments: name and a float. Under the internals lock: lua_getfield of the global 'Client', table check (0… |
| `00976560` | ClientLuaSetClientStringField | confirmed | Thiscall on the same holder at this+4 with two stack string arguments (key, value). Under the internals lock: lua_getfield of the global 'Client', table check … |
| `00976630` | ClientLuaReadClientIntField | confirmed | Thiscall on the holder at this+4 (its first dword is the ClientLuaInternals object X; Lua state *(X+0x94)) with two stack arguments (name, out int*); returns b… |
| `00977740` | ClientLuaReadClientStringField | confirmed | Resets the IString output param_3 (releases its current buffer through the reference count test, then points it at the empty static string with length 0). If t… |
| `009797a0` | ClientLuaCallByName | confirmed | Copies the name in param_2 (up to 0x1ff bytes) and resolves it: 'Object:Method' (method call, flag 1) and 'Object.Method' (field call, flag 0) are looked up as… |
| `00a8de60` | CombatGroupHandlerSetHealth | confirmed | When the associated state gate permits, builds four Lua parameters, updates a retained state reference, invokes CombatGroupHandler:SetHealth, then destroys the… |
| `00a8e000` | CombatGroupHandlerSetPower | confirmed | When the associated state gate permits, builds three Lua parameters, updates a retained state reference, invokes CombatGroupHandler:SetPower, then destroys the… |
| `00a8e170` | GroupHandlerAddPlayer | confirmed | Builds parameters and retained state for CombatGroupHandler:AddPlayer; under a separate client flag it also calls SoccerGroupHandler:AddPlayer, and links a cha… |
| `00a8e3e0` | GroupHandlerRemovePlayer | confirmed | Builds parameters from the supplied pair, calls CombatGroupHandler:RemovePlayer and conditionally SoccerGroupHandler:RemovePlayer, then links the pair when it … |
| `00a8ec00` | GroupHandlerAddAbilityEffect | confirmed | Updates two retained state values, invokes GroupHandler:AddAbilityEffect with the supplied identifiers, then initializes and releases a temporary string object. |
| `00a8ee30` | GroupHandlerRemoveAbilityEffect | confirmed | Updates retained state for the supplied identifiers, invokes GroupHandler:RemoveAbilityEffect, then initializes and releases a temporary string object. |
| `00bfe520` | RegisterAdventurersJournalBindings | confirmed | Passes the AdventurersJournal binding name and SetSelectedRegion callback table to 00e39e40, with a zero flag, then returns zero. |
| `00bfe540` | RegisterClientNudgeBindings | confirmed | Passes the ClientNudge binding name and LuaCallbackNudgeHasBeenShown callback table to 00e39e40, with a zero flag, then returns zero. |
| `00bfe5e0` | RegisterSocialShareBindings | confirmed | Passes the SocialShare binding name and GetProviderStatus callback table to 00e39e40, with a zero flag, then returns zero. |
| `00bfea50` | RegisterLuaUtilBindings | confirmed | Passes the LuaUtil binding name and RandomUint32 callback table to 00e39e40, with a zero flag, then returns zero. |
| `00bfec70` | RegisterFactoryBindings | confirmed | Passes the Factory binding name and RequestToolData callback table to 00e39e40, with a zero flag, then returns zero. |
| `00bff1b0` | RegisterCharacterCreateServerUtilsBindings | confirmed | Passes CharacterCreateServerUtils and its ServerListQuery callback table to 00e39e40, with a zero flag, then returns zero. |
| `00bff1d0` | RegisterClientEffectBindings | confirmed | Passes ClientEffect and its ScaleUp callback table to 00e39e40, with a zero flag, then returns zero. |
| `00bff480` | RegisterMountBindings | confirmed | Passes the Mount binding name and Spawn callback table to 00e39e40, with a zero flag, then returns zero. |
| `00bffca0` | RegisterQuickChatBindings | confirmed | Passes the QuickChat binding name and RequestPossibleMatches callback table to 00e39e40, with a zero flag, then returns zero. |
| `00bffd80` | RegisterCoinStoreScreenBindings | confirmed | Passes the CoinStoreScreen binding name and CanBuyItem callback table to 00e39e40, with a zero flag, then returns zero. |
| `00c00dc0` | RegisterClaimScreenBindings | confirmed | Passes ClaimScreen and its StartClaim callback table to 00e39e40, with a zero flag, then returns zero. |
| `00c01000` | RegisterCinematicBindings | confirmed | Passes Cinematic and its Start callback table to 00e39e40, with a zero flag, then returns zero. |
| `00c01dd0` | RegisterLuaDataSourceMetatable | confirmed | Creates a Lua metatable, registers __index and __metatable entries using stack index -3, then pops the constructed metatable. |
| `00c01f70` | RegisterRecipeBindings | confirmed | Passes Recipe and its SelectRecipe callback table to 00e39e40, with a zero flag, then returns zero. |
| `00c01fd0` | RegisterMarketplacePortalScreenBindings | confirmed | Passes MarketplacePortalScreen and its OnMarketplacePortalScreenOpened callback table to 00e39e40, with a zero flag, then returns zero. |
| `00c021d0` | RegisterPromotionalScreenBindings | confirmed | Passes PromotionalScreen and its IsCodeRedemptionEnabled callback table to 00e39e40, with a zero flag, then returns zero. |
| `00c02370` | RegisterInGamePurchaseStoreBindings | confirmed | Passes InGamePurchaseStoreScreen and IsMarketplaceEnabled callback table to 00e39e40, with a zero flag, then returns zero. |
| `00c04950` | RegisterTradingCardGameBindings | confirmed | Passes TradingCardGame and its StartTutorial callback table to 00e39e40, with a zero flag, then returns zero. |
| `00c04a80` | RegisterOnlineHelpWindowBindings | confirmed | Passes OnlineHelpWindow and its callback table to 00e39e40, with a zero flag, then returns zero. |
| `00c04ff0` | RegisterGameActionQueueBindings | confirmed | Passes GameActionQueue and its Respond callback table to 00e39e40, with a zero flag, then returns zero. |
| `00c05150` | RegisterVideoUploadBindings | confirmed | Passes VideoUpload and its VideoUploadInit callback table to 00e39e40, with a zero flag, then returns zero. |
| `00c056a0` | RegisterVideoCaptureBindings | confirmed | Passes VideoCapture and its GetVideoCaptureOutputFile callback table to 00e39e40, with a zero flag, then returns zero. |
| `00c1a4d0` | CastLuaValueToGuiSurfaceCtrl | confirmed | Reads one argument, converts it, then performs an RTTI cast from GObject to GuiSurfaceCtrl and returns the cast result or null. |
| `00c1b1c0` | ReadyForActivity | confirmed | Calls the unsigned-GUID parser for argument 1 and discards its result, reads argument 2, selects the first node with byte +0xb5 != 1 when that value is zero or… |
| `00c24f30` | GetGfxCtrl | confirmed | Checks Lua stack capacity, retrieves the GfxCtrl userdata at the requested index, raises a type error if absent, then raises a null-object error or returns its… |
| `00c251f0` | RegisterGfxCtrlMetatable | confirmed | Registers the GfxCtrl userdata metatable, installs its __index and __metatable fields, and closes the Lua stack scope. |
| `00c25c50` | PushDsTable | confirmed | Pushes one value onto the Lua stack, registers the DsTable metatable, then creates and returns a DsTable userdata wrapper. |
| `00c25c90` | GetDsTable | confirmed | Checks Lua stack capacity, retrieves DsTable userdata at the requested index, raises type or null-object errors when invalid, and returns its stored pointer. |
| `00c260b0` | RegisterDsTableMetatable | confirmed | Registers the DsTable userdata metatable and installs its __index and __metatable fields. |
| `00c26160` | DsTable_GetData | confirmed | Validates DsTable and row/column arguments, checks table bounds, fetches the cell, converts its value to Lua, and reports signature or range errors. |
| `00c269c0` | RegisterImageControlMetatable | confirmed | Registers ImageControl userdata and its methods, including getSize, then installs __index and __metatable. |
| `00c893f0` | DispatchFirstTimeEventScript | confirmed | Splits an object-and-method string at the period, rebuilds the call target with a colon, converts integer arguments to Lua parameters, and invokes the Lua brid… |
| `00976070` | ClientLuaRegisterAllBindings | likely | Void, ECX = the ClientLuaInternals object. Calls 56 native registration routines in sequence (for example 00c28e60, 00c288d0, 00c28470, ... 00bfe520), each wit… |
| `00976330` | ClientLuaSetGlobalNewTable | likely | Thiscall with one stack argument (the name). Creates an empty table with 00e3ad80(L, 0, 0) on the Lua state at this+0x94, then stores it with lua_setfield (00e… |
| `00976360` | ClientLuaInternalsCloseLuaState | likely | Fastcall (ECX = this), returns void. Calls 00986c10 on this, then, when the Lua state at this+0x94 is non-null, closes it with 00e3bba0 and zeroes this+0x94. |
| `00976740` | ClientLuaReadClientFloatField | likely | Reads field param_2 of the Lua global Client: lua_getfield on the globals index for Client, pushes the key, gets Client[key]; if the value is type 3 (number) o… |
| `00976860` | ClientLuaToggleClientFlag | likely | If the holder at this+4 is non-null: calls FUN_00976630(param_2, &param_2), which reads Client[name] as a number or string into the out parameter. When that re… |
| `00976900` | ClientLuaIsGlobalFunction | likely | Pushes the Lua global named by param_2 (lua_getfield on the globals index, Lua state at this+0x94), reads its type with lua_type on the top slot, pops it with … |
| `009769b0` | ClientLuaIsGlobalFunctionGuarded | likely | Bytes: mov eax,[ecx+4]; test eax,eax; je; cmp dword [eax],0; je; mov ecx,[eax]; jmp 00976900. When the holder at +4 is null or its first dword (the internals p… |
| `00976d50` | ClientLuaLoadRunLogErrorStatus | likely | If the holder at this+4 is non-null: records a clock reading in DAT_01be5220/5224, loads the file named by param_2 with FUN_00e396a0 (luaL_loadfile-shaped: its… |
| `00976e90` | ClientLuaLoadRunLogErrorVoid | likely | If the holder at this+4 is non-null: records a clock reading in DAT_01be5220/5224, calls FUN_00e396a0(L, param_2) (file loader, luaL_loadfile-shaped) and ignor… |
| `00976fa0` | ClientLuaCollectGarbageRateLimited | likely | Reads a clock value (FUN_007027f0) and computes its elapsed time since the stored pair at this+0x1b0/+0x1b4 (clamped to 0x7fffffff). When the limit at this+0x1… |
| `00977020` | ClientLuaCollectGarbageForwarder | likely | Bytes: mov eax,[ecx+4]; test eax,eax; je to ret; mov ecx,[eax]; jmp 00976fa0. When the holder at this+4 is non-null it loads the internals pointer (first dword… |
| `00977890` | ClientLuaPushTaggedValue | likely | Pushes one tagged value onto the Lua state param_2 by switching on its tag dword: 1 pushes the int at +8 as a number, 2 pushes the double at +8, 3 and 8 push t… |
| `00978f80` | ClientLuaBindSetNamedString | likely | Lua C-function shaped body: gets the argument count with FUN_00e3a390 and requires exactly 2. It requires both arguments to pass FUN_00e3a630 (string test), re… |
| `00979280` | ClientLuaInternalsCreate | likely | Allocates a 0x1c0-byte block with FUN_00d833a6(0x1c0, ...). When the allocation is non-null it constructs a ClientLuaInternals in it with 00979120(param_2); ot… |
| `009792f0` | TaggedValueLessThan | likely | Compares tagged values this and param_2: when the tags differ it returns (tag of this < tag of param_2). Same tag: 1 compares the ints at +8 (0 when absent), 2… |
| `00979590` | ClientLuaParamReadStackValue | likely | If the Lua state has a value (FUN_00e3a390 nonzero), reads the type of the top value (lua_type at index -1) and converts it: 0, 2, 6, 7 and 8 call FUN_007071b0… |
| `00979b70` | ClientLuaParamArrayFromTypeString | likely | Initialises a local SoeUtil::Array<ClientLuaParam,0,1> (vtable store, ArrayAssign with the local, clear with FUN_0070b4e0(0), free of the old buffer with FUN_0… |
| `00a8fa80` | ForwardRecordToGroupHelper | likely | When the input record is nonnull, forwards its first two fields, address of its remaining data, and field at index 0x24 to 00a8e170. |
| `00a90790` | ForwardFirstFieldToCallback | likely | Loads the dword at the third argument, then forwards it with the first argument to 00a8f3b0; returns after the call. |
| `00a907b0` | ForwardFirstFieldToCallbackVariant | likely | Loads the dword at the third argument, then forwards it with the first argument to 00a8f3b0; returns after the call. |
| `00a907d0` | ForwardFirstFieldToCallbackVariant2 | likely | Loads the dword at the third argument, then forwards it with the first argument to 00a8f3b0; returns after the call. |
| `00bfead0` | FormatAndPushLuaString | likely | Formats param_2 and param_3 with the global format string, passes the resulting string to 00e3aa90 for param_1, then releases the temporary string. |
| `00bfeb90` | FormatAndPushLuaString | likely | Formats param_2 and param_3 with the global format string, passes the resulting string to 00e3aa90 for param_1, then releases the temporary string. |
| `00c17c00` | PushDerivedBooleanThenZero | likely | When exactly one argument is present and its value converts, calls 008a8de0 and passes the returned nonzero test to 00e3aa30; then unconditionally calls 00e3aa… |
| `00c17cf0` | LuaSetCameraAutoAdjustment | likely | For exactly one argument, converts argument 1 to a number, normalizes nonzero to true, calls 009178e0 with that boolean, and returns 0. |
| `00c17f80` | LuaGetActivityCountByName | likely | For one valid argument, converts argument 1, checks the client activity-list field, calls 008a8da0 with the converted value and passes its result to the Lua ou… |
| `00c18000` | LuaGetActivityTimeRemainingByName | likely | For one valid argument, converts argument 1, checks the client activity-list field, calls 008a8dc0 with the converted value and passes its result to the Lua ou… |
| `00c196a0` | InvokeRowSelectionRenumbering | likely | For exactly one argument, converts argument 1, obtains the client context and calls 00b8c4a0 with the converted value; returns 0. |
| `00c1ada0` | GetGameSceneLocalPosition | likely | Looks up GameScene, conditionally casts it to GCtrl, calls 0041d5df with a zero argument, pushes two returned coordinates and returns count two. |
| `00c1af80` | SetQuickChatChannelNinePair 🌐 | likely | When argument -1 is present, looks up the selected GUID pair and compares it with two global dwords. If they differ, reads argument -1 with that pair and calls… |
| `00c1b000` | InvokePositionInputWithDefaults | likely | For exactly one argument, parses it as an unsigned GUID, copies four dwords beginning at DAT_01cb53d0 into a local vector, obtains the client singleton and cal… |
| `00c1b310` | PushClientFieldF78UnlessLookupReturnsZero | likely | With fewer than two arguments, obtains the client field at +0xf78. With two or more, parses argument 2 and looks up its key pair; it obtains that field only wh… |
| `00c1b7c0` | ParseDelimitedIntegerPairForDisplayResolution | likely | For exactly one argument, gets its string, parses an integer before delimiter 0x78 (x), then a second integer before delimiter 0x20 (space). If the second valu… |
| `00c1c8f0` | ExecuteConsoleCommandArgument | likely | When argument 1 converts successfully, converts it to UTF-16 and passes the resulting command line to the console executor, then releases the temporary string. |
| `00c1e520` | ApplyClientStringWhenFlag62CSet | likely | With exactly two arguments, gets argument 1 and argument 2, assigns argument 2 to a temporary IString, checks the client field at +0x62c and calls 0098e5d0 wit… |
| `00c1e610` | PushKeybindingUpdateResult | likely | If fewer than two arguments are present, passes 0 to the output helper and returns. Otherwise rounds argument 1 to an integer, converts argument 2 into a Strin… |
| `00c1ef10` | FormatDurationString | likely | For a valid numeric argument, decomposes seconds into days, hours, minutes and seconds, formats the result through 0097afe0 or 0097ae80 depending on a flag, an… |
| `00c21a90` | RefreshCurrentClientLuaParams | likely | Reads a source string array, parses delimited triples into Lua parameter arrays, marks each entry current when two global values match, and calls the Lua dispa… |
| `00c253f0` | GfxCtrl_SetProperty | likely | Reads three Lua values, obtains or converts the target object and property index, then calls the object's virtual setter with the index and value. |
| `00c25710` | GfxCtrl_GetProperty | likely | Validates GfxCtrl userdata and a property key, invokes the property's virtual getter, converts the result to a Lua value, and reports argument, userdata, or mi… |
| `00c25d70` | DsTable_GetRowCount | likely | Validates DsTable, invokes the table interface method at offset 0x30, and pushes its result. |
| `00c25dc0` | DsTable_GetColumnCountAtIndex | likely | Validates DsTable and one integer argument, compares the index with the result of interface slot +0 at object offset +0x30, calls interface slot +4 for that in… |
| `00c25ee0` | DsTable_FindRowByUserData | likely | Validates DsTable and one user-data argument, scans each row's user-data value, then pushes the matching row index or -1. |
| `00c26800` | ImageControl_ApplyPairOperation5 | likely | Validates ImageControl, reads two numbers, and calls 004072e2 with the values in the final numeric positions and operation selector five. |
| `00c26850` | ImageControl_ApplyPairOperation6 | likely | Validates ImageControl, reads two numbers, and calls 004072e2 with the values in the first numeric positions and operation selector six. |
| `00975970` | ClientLuaLockInternalsReturnArg | speculative | Fastcall with one argument (ECX, saved in ESI). Calls 00fc45f0 with ECX = [DAT_01be51a8] + 8, the acquire routine of the lock at +8 of the ClientLuaInternals o… |
| `009769f0` | ClientLuaCounterIsNonZero | speculative | Reads the dword at +0x1ac of the global internals pointer DAT_01be51a8 and returns whether it is nonzero (bool in eax), inside the SEH and stack-cookie prolog … |
| `00976a60` | ClientLuaCounterIncrement | speculative | Increments the dword at +0x1ac of the global internals pointer DAT_01be51a8 by one, inside the SEH and stack-cookie prolog and epilog. |
| `00976ad0` | ClientLuaCounterDecrement | speculative | Decrements the dword at +0x1ac of the global internals pointer DAT_01be51a8 by one, inside the SEH and stack-cookie prolog and epilog. |
| `00977140` | TaggedValue160ArrayAppendRaw | speculative | Appends one 0xa0-byte slot to the array at this+4 (count at +8, capacity at +0xc, growth through FUN_00713cf0 when count+1 exceeds capacity), increments the co… |
| `009773f0` | TaggedValue160ArrayAppendInt | speculative | Appends one 0xa0-byte slot to the array at this+4 with the same growth as 00977140, then stores tag 1 in its first dword and copies the dword *param_2 into slo… |
| `00977480` | TaggedValue160ArrayAppendDouble | speculative | Appends one 0xa0-byte slot with the same growth as 00977140, stores tag 2 in its first dword and copies the 8-byte value *param_2 into slot+8. Returns the slot… |
| `00977510` | TaggedValue160ArrayAppendPair | speculative | Appends one 0xa0-byte slot with the same growth as 00977140, stores tag 0 in its first dword, and passes the two dwords of *param_2 to FUN_00707240 with the sl… |
| `009775d0` | TaggedValue160ArrayAppendBool | speculative | Appends one 0xa0-byte slot with the same growth as 00977140, stores tag 5 in its first dword and copies the byte *param_2 into slot+8. Returns the slot pointer… |
| `00977c10` | TaggedValue160ArrayAppendCopy | speculative | Appends one 0xa0-byte slot to the array at this+4 (count at +8, capacity at +0xc, growth through FUN_00713cf0), sets the slot tag dword to 0, and assigns the a… |
| `00978e20` | TaggedValue160ArrayAppendAssignC620 | speculative | Appends one 0xa0-byte slot to the array at this+4 (count at +8, capacity at +0xc, growth through FUN_00713cf0), sets the slot tag dword to 0, and assigns the a… |
| `00979450` | TaggedValueEquals | speculative | Returns false when the tags differ. With equal tags: 1 compares the ints at +8 (0 when absent), 2 compares the doubles, 3 and 8 compare the strings through the… |
| `00979ac0` | TaggedValue160ArrayAppendAssignC118 | speculative | Appends one 0xa0-byte slot to the array at this+4 (growth through FUN_00713cf0 when count+1 exceeds capacity), zeroes its tag dword, and assigns param_2 into t… |
| `00bfea90` | ParseLuaUnsignedGuid | speculative | Reads a Lua string argument, rejects null or strings beginning with '-', parses the text as a signed 64-bit value, and returns zero on rejection. |
| `00bff060` | GetManagerValuePair | speculative | When the manager exists, calls 0097c8c0 with a local output integer; on nonzero result pushes its return value and that integer and returns two. |
| `00bff150` | GetManagerValuePair | speculative | When the manager exists, calls 0097c910 with a local output integer; on nonzero result pushes its return value and that integer and returns two. |
| `00bfffe0` | CallHelperAndPushArgumentResult | speculative | Requires one Lua argument and an available manager, calls 00c6f210 with that argument, pushes the result, and returns one. |
| `00c00050` | LookupAndPushManagerValue | speculative | Requires one Lua argument and an available manager, passes its value to 00c703f0, pushes the returned value, and returns one. |
| `00c000c0` | CallHelperAndPushArgumentResult | speculative | Requires one Lua argument and an available manager, calls 00c6f250 with that argument, pushes the result, and returns one. |
| `00c01060` | ApplyVectorAndRadius | speculative | Requires three numeric Lua arguments, reads three float values and optional radius and boolean, calls 008c6110 with those values, then pushes its result. |
| `00c01b90` | CreateLuaClosureWithCapturedValue | speculative | Allocates four bytes in the Lua state, stores param_2 in that slot, creates a closure using the registry key and upvalue -2, then returns the allocated slot. |
| `00c01bd0` | GetLuaDataSourceObject | speculative | Sets a Lua stack top, looks up the named registry value, returns zero if absent, and otherwise dereferences its object pointer; logs an error when that pointer… |
| `00c01c60` | CallLuaDataSourceMethodTrue | speculative | Fetches the registered data-source object, clears its registry entry if missing, otherwise calls vtable offset 0x14 with argument 1, then returns zero. |
| `00c01ca0` | CallLuaDataSourceMethodFalse | speculative | Fetches the registered data-source object, clears its registry entry if missing, otherwise calls vtable offset 0x14 with argument 0, then returns zero. |
| `00c01e60` | IsLuaGuiDataSourceValue | speculative | Reads Lua argument one, converts it through 008195a0, checks the resulting object against GuiDataSource and GuiDsValue RTTI, and pushes a generated string when… |
| `00c026e0` | DispatchSeventeenLuaValues | speculative | Validates 17 Lua arguments and converted values, checks manager and required nonzero fields, calls 00b33bc0 with the validated values, then pushes its result. |
| `00c03cf0` | CallHelperWithTwoIntegers | speculative | Requires two numeric Lua arguments and an active manager, reads both integers, calls 00b37850, pushes one on success and zero otherwise. |
| `00c03d80` | CallHelperWithInteger | speculative | Requires one numeric Lua argument and an active manager, reads the integer, calls 00b23cb0, and pushes one on success or zero otherwise. |
| `00c03e60` | CallHelperWithTwoValues | speculative | Requires two numeric Lua arguments and an active manager, extracts both values, calls 00b37b90, and returns zero. |
| `00c03ed0` | CallHelperWithThreeValues | speculative | Requires three numeric Lua arguments and an active manager, extracts all three values, calls 00b3b3b0, and pushes zero. |
| `00c05170` | ForwardNonemptyLuaString | speculative | For one Lua argument, reads a string and if it is non-null and nonempty passes it to 008ef350 after obtaining the global manager; returns zero. |
| `00c051e0` | ForwardLuaInteger | speculative | For one Lua argument, converts it to an integer, obtains the global manager, calls 008ef510 with the value, and returns zero. |
| `00c05250` | ForwardLuaInteger | speculative | For one Lua argument, converts it to an integer, obtains the global manager, calls 008ef740 with the value, and returns zero. |
| `00c05510` | LookupValueForLuaString | speculative | For one nonempty Lua string, obtains the manager, passes the string to 008a8700, pushes its result, and returns one; otherwise pushes -1. |
| `00c17600` | InvokeWildcardArgForwarder | speculative | Checks the wildcard argument conversion using index -1; if successful, converts index -1 again and passes the value to 009ad440. |
| `00c176a0` | SetClientFlag658FromArg | speculative | If the wildcard conversion succeeds, converts the wildcard value and stores its nonzero test at client field offset 0x658. |
| `00c176e0` | SetClientFlag659FromArg | speculative | If the wildcard conversion succeeds, converts the wildcard value and stores its nonzero test at client field offset 0x659. |
| `00c17750` | InvokeTwoFloatArgs | speculative | Requires two present arguments, converts both values, casts them to floats and passes them to 008b9a40. |
| `00c177e0` | InvokeTwoFloatArgs2 | speculative | Requires two present arguments, converts both values to floating point and passes them to 008b9a80. |
| `00c17860` | InvokeTwoFloatArgs3 | speculative | Requires two present arguments, converts both values, casts them to floats and passes them to 008a6be0. |
| `00c178f0` | InvokeOneFloatArg | speculative | Requires one present argument, converts index 1 to floating point and passes it to 008a6c20. |
| `00c17960` | SetVectorFromArgs | speculative | Accepts one or more arguments, chooses defaults from two globals for missing components, reads later components when present, then stores the first converted a… |
| `00c17a10` | InvokeIndexedCallbackA | speculative | When at least one argument exists and index 1 converts, passes its converted value or zero to callback 00a870b0. |
| `00c17a60` | InvokeIndexedCallbackB | speculative | When at least one argument exists and index 1 converts, passes its converted value or zero to callback 00a870d0. |
| `00c17bb0` | RequestPointOfInterestTeleport 🌐 | speculative | For exactly one argument, converts argument 1 and calls 00b991d0, which validates the local point-of-interest id and sends PacketZoneTeleportRequest; returns 0. |
| `00c17d30` | InvokeFourFloatArgs | speculative | Requires four present arguments, converts indexes 1 through 4 to floats and passes all four values to 008fe620. |
| `00c17df0` | RequestGroupMapPing | speculative | Requires two present arguments, converts each to a value and passes both to 00938650. |
| `00c18310` | SetClientFlagFromBoundedValue | speculative | For exactly one argument, converts index 1 and, when the result minus one is below 0x39, calls 00a46010 with the value and zero. |
| `00c184f0` | SetOutputFromSelectedValue | speculative | Calls the helper that selects a client value using the converted argument when one argument is present, then writes the selected value through the output helpe… |
| `00c185f0` | SetOutputFromPairField | speculative | For a valid single argument, converts its value, calls 00a10700, and writes the returned value through the output helper; otherwise writes zero. |
| `00c18840` | SetOutputFromBooleanizedValue | speculative | Converts the first argument if present, tests the converted result for nonzero, passes that boolean to 00bd4fa0 and writes the returned byte through the output… |
| `00c188f0` | RequestStripEffect 🌐 | speculative | For a positive argument count with argument 1 present, converts it and calls 00934740, which constructs and sends PlayerUpdatePacketRequestStripEffect; returns… |
| `00c18940` | GetTwoClientValues | speculative | Initializes two local outputs to zero, asks 009ad890 to fill them, writes each output to the argument stack and returns count 2. |
| `00c189a0` | RequestLeaderboard | speculative | For exactly one argument, converts the wildcard argument and passes the value to ClientLeaderBoardManager::RequestLeaderBoard. |
| `00c189e0` | InvokeTwoClientValues | speculative | For more than one argument, converts indexes 1 and 2 and passes both to 00a28d00. |
| `00c18aa0` | StartQuestBreadcrumb | speculative | For exactly one argument, converts the wildcard value and passes it to 00a97f90. |
| `00c18b10` | GetOutputFromValueA | speculative | For one present argument, converts index 1 to a value, calls 008a8d30, passes the result through 01044d80 and writes it to the argument output. |
| `00c18b70` | InvokeTwoClientValuesA | speculative | Requires two arguments, converts the first and second values, calls 008a8d30, then passes both converted values to 01044d60. |
| `00c18be0` | GetOutputFromValueB | speculative | For one present argument, converts index 1 to a value, calls 008a8d30, passes the result through 01044d90 and writes it to the argument output. |
| `00c18c80` | SelectPlayerTitle 🌐 | speculative | For a positive argument count with argument 1 present, converts it and calls 00b4a880 only when client field +0x3e0 is nonzero; that helper constructs and send… |
| `00c18d00` | SetJobCustomizationSlot | speculative | Requires two arguments, converts argument 1 and argument 2, and passes both to 00a845a0. |
| `00c18d70` | ClearJobCustomizationSlot | speculative | For one present argument, converts index 1 and passes the value to 00a84be0. |
| `00c18de0` | SelectJobCustomizationRow | speculative | For one present argument, converts index 1 and passes it to 00a849a0. |
| `00c18e30` | GetOutputFromClientValueA | speculative | For one argument, converts index 1 with the typed argument helper and passes the result to 009c5590. |
| `00c18e80` | GetOutputFromClientValueB | speculative | For one argument, converts index 1 with the typed argument helper and passes the result to 009c5720. |
| `00c192e0` | GetThreeDefaultValues | speculative | Initializes three local values, calls 00702670 and 007026b0 to populate them, pushes the three values to the output stack and returns count 3. |
| `00c19350` | SetClientFlagFromArg | speculative | For exactly one argument, converts it to an integer and writes whether it equals 1 into a client boolean at a nested offset. |
| `00c19600` | SetQuickChatEmoteFavoriteMode | speculative | For one argument, converts index 1, calls 00b8c440 with the value, then invokes 00b96170. |
| `00c19650` | SetQuickChatEmoteFavoriteMode2 | speculative | For one argument, converts index 1, calls 00b8c470 with the value, then invokes 00b96170. |
| `00c19a10` | ApplyOneOrTwoClientValues | speculative | For at least one argument, converts index 1 and optionally argument 2, then passes the first value and optional second value to 00a1f8a0. |
| `00c19f60` | ForwardWildcardArgumentA | speculative | For exactly one argument, converts the wildcard argument and forwards it through thunk 00bb85c0 to 00bb84f0. |
| `00c19fa0` | ForwardWildcardArgumentB | speculative | For exactly one argument, converts the wildcard argument and passes it to 00bb8220. |
| `00c1a170` | SetClientFlagFromPositiveArg | speculative | For exactly one argument, converts index 1 and stores whether it is positive in the client field at offset 0x8c, then writes zero to the argument output. |
| `00c1a460` | RegisterGetNameBinding | speculative | Calls 00e39e40 with the receiver, a global object pointer, the exact method name GetName, and a zero argument. |
| `00c1ad60` | GetClientPairAndStoreOutput | speculative | Calls 00a23120 to fill two local words, passes the first and second word to 00bfeb90 with the output context, and returns one result. |
| `00c1aef0` | GetTwoPositionCoordinates | speculative | Gets an input object from the wildcard argument, initializes two output coordinates to -1, calls 008ba830, pushes either default or returned coordinates, and r… |
| `00c1b160` | SendMemberQuitFromArgs | speculative | Calls the unsigned-GUID parser for argument 1 and discards its result, reads argument 2, selects the first node with byte +0xb5 != 1 when that value is zero or… |
| `00c1bff0` | GetOutputStringFromClientResource | speculative | For one valid argument, converts its identifier, checks the client resource pointer at offset 0x62c, resolves a string through 0098e620 and pushes it to output… |
| `00c1c1d0` | LookupStringForMappedClientCode | speculative | Calls 00a22640 to obtain a mapped code, initializes a temporary SoeUtil IString, invokes a client virtual method at vtable offset +8 with the code and output s… |
| `00c1c2a0` | LookupClientRecordStringA | speculative | Converts an optional first argument into a record key, looks up a client record using 00a226a0, invokes a virtual string conversion and writes the resulting st… |
| `00c1c3a0` | LookupClientRecordStringB | speculative | Converts an optional first argument into a record key, looks up a client record using 00a22720, invokes a virtual string conversion and writes the resulting st… |
| `00c1c4a0` | ConvertClientValueToString | speculative | For one valid argument, converts the value through 00a34e00 into a temporary IString, pushes that string to output and releases the temporary buffer. |
| `00c1c790` | GetWideStringArgumentA | speculative | Converts the wildcard argument to a UTF-16 string and passes it to the client object's virtual method at offset 0x70, then releases the temporary string. |
| `00c1c840` | GetWideStringArgumentB | speculative | Converts the wildcard argument to a UTF-16 string and passes it to the client object's virtual method at offset 0x74, then releases the temporary string. |
| `00c1cd20` | CallStringHelperWithFourArguments | speculative | With exactly four arguments, converts all four, initializes a temporary IString, calls 0097b0e0 with the string and four values, passes the resulting string to… |
| `00c1ce60` | CallStringHelperWithOneArgument | speculative | With exactly one argument, converts it, initializes a temporary IString, calls 0097b1e0 with the string and value, passes the result to the output helper, rele… |
| `00c1cf70` | CallStringHelperWithTwoArguments | speculative | With exactly two arguments, converts both, initializes a temporary IString, calls 0097b2d0 with the string and both values, passes the result to the output hel… |
| `00c1d3c0` | CreateGameSceneObject | speculative | For a valid argument, checks whether its name already exists; if absent, allocates 0x2e0 bytes, initializes the object, looks up GameScene, calls its vtable sl… |
| `00c1d4c0` | CopyArgumentStringToOutput | speculative | Converts argument 1, copies up to 0x200 bytes into an IString buffer, resets the temporary string on copy failure, pushes the resulting string and releases the… |
| `00c1de10` | InvokeClientVirtualWithArguments | speculative | Requires a client object and one argument, converts remaining arguments by repeating four typed conversion patterns into local slots, then calls the object's v… |
| `00c1e090` | InvokeClientVirtualWithOneValue | speculative | For exactly one argument, converts the wildcard value, builds a temporary IString, calls the client object's virtual method at offset 0x10 with the value and s… |
| `00c1e400` | InvokeClientVirtualWithValueToString | speculative | For one argument, converts its value, invokes a client virtual method at offset 0x8 with a temporary IString, then pushes the string result to output. |
| `00c1e720` | SendQuickChatWhisper 🌐 | speculative | For two arguments, verifies argument one and a non-null string argument two, builds an IString, converts the first argument and calls the accepted quick-chat w… |
| `00c1f0a0` | InvokeFieldAdjustedClientMethod | speculative | For one valid argument, converts it to a string, resolves the current client object, invokes thunk 009aaf50 with a temporary IString, and releases that string. |
| `00c24f90` | GfxCtrl_Action | speculative | For a positive argument count, validates GfxCtrl at index one and calls 0083db00 when the userdata is non-null. |
| `00c25100` | GfxCtrl_Action | speculative | Requires exactly one Lua argument, validates GfxCtrl at index one, and calls 00406607 with zero when the userdata is non-null. |
| `00c25330` | GfxCtrl_ApplyFlags | speculative | Validates GfxCtrl, initializes an output record, calls its virtual method at slot offset 0x114, then invokes 004d93a3 when output bits masked by 0x24924 are se… |
| `00c25e60` | DsTable_EnableFlag | speculative | Validates DsTable and invokes its virtual method at offset 0x14 with argument one. |
| `00c25ea0` | DsTable_DisableFlag | speculative | Validates DsTable and invokes its virtual method at offset 0x14 with argument zero. |
| `00977dc0` | — | unknown | Switches on the tag dword at this: for tags 3, 6, 7 and 8 it calls the function pointer stored at this+8 with argument 0. It then stores 8 in the tag dword, an… |
| `00978ab0` | — | unknown | Initialises the IString param_2 as empty, then formats the tagged value *param_1 into it by tag: 0 assigns a constant string, 1 calls FUN_00821d00 on the int, … |
| `009c5350` | — | unknown | Increments the timer-event counter, builds four tagged parameter values, and constructs a ClientTimerEvent only when lookup 00d833a6 returns nonzero; then disp… |
| `00bfed30` | — | unknown | Reads Lua argument one, converts it through 009e1a50 to a boolean, calls 00e3aa30 with that value, and returns one. |
| `00bffda0` | — | unknown | Requires two truthy Lua arguments and an available manager, calls 00c70fe0 with both values, pushes the boolean result, and returns one. |
| `00bfff70` | — | unknown | Requires one Lua argument and an available manager, calls 00c6f200 with that argument, pushes the result, and returns one. |
| `00c01660` | — | unknown | Validates arguments, parses two 64-bit identifiers and optional scalar or boolean arguments, builds two temporary target objects, calls 008c6170, and cleans th… |
| `00c02a50` | — | unknown | Validates 16 Lua arguments, converts numeric and boolean fields, checks required values and manager, calls 00b33e60, then pushes its result. |
| `00c02e50` | — | unknown | Validates five Lua arguments and the manager, extracts two strings and three numeric values, calls 00b37670, and pushes its result. |
| `00c02f60` | — | unknown | Validates Lua arguments one and three, reads them as values, optionally reads integer arguments two and four, checks the manager, then calls 00b33900. |
| `00c03520` | — | unknown | For one truthy Lua argument, obtains its value and calls 00b37b80 when the manager is present; returns zero. |
| `00c03860` | — | unknown | For one truthy Lua argument, reads its string and calls 00b20100 when both the string and manager are present; returns zero. |
| `00c038b0` | — | unknown | For one truthy Lua argument, reads its string and calls 00b20100 when both the string and manager are present; returns zero. |
| `00c039f0` | — | unknown | Reads one Lua 64-bit identifier, compares it to a global identifier, and when the active account object has the expected low flag and manager, calls 00b376d0; … |
| `00c03df0` | — | unknown | Requires one numeric Lua argument and an active manager, reads the integer, calls 00b37a10, and pushes one on success or zero otherwise. |
| `00c03f70` | — | unknown | Requires one string argument and active manager, copies the string to a temporary object, calls 00b37bb0 with the resulting value and zero, then pushes its res… |
| `00c04500` | — | unknown | Requires five numeric Lua arguments and a manager, checks a purchase record with 00c03ba0, invokes 00b2a4d0, calls a vtable method with three values, formats a… |
| `00c04bc0` | — | unknown | For one truthy Lua argument, reads its boolean value and stores whether the corresponding integer is nonzero at global object offset 0x18; returns zero. |
| `00c04c10` | — | unknown | For one nonempty Lua string, converts it to a temporary wide-string representation, calls 00995780 with that result, then destroys temporary data. |
| `00c17390` | — | unknown | Reads one argument through the Lua helper and, when its conversion succeeds, passes it with constant 1 to the next helper and sends the result to the global st… |
| `00c174a0` | — | unknown | Requires one converted argument, calls the local helper, converts the argument with mode 1 and flag 1, then passes that result and flag 1 to 009bd590. |
| `00c174e0` | — | unknown | Requires one converted argument, calls the local helper, converts the argument with mode 1 and flag 0, then passes that result and flag 0 to 009bd590. |
| `00c17520` | — | unknown | Requires two converted arguments, converts indexes 1 and 2, then passes both values to 009bd450. |
| `00c17570` | — | unknown | Requires two converted arguments, converts indexes 1 and 2, then passes both values to 009bd4f0. |
| `00c175c0` | — | unknown | Requires one converted argument, invokes the global helper, converts argument index 1 and passes that value to 00a976a0. |
| `00c17640` | — | unknown | Requires one converted argument, converts index 1 and passes the value to 008a7d30. |
| `00c17af0` | — | unknown | For exactly one argument, converts index 1, tests the resulting object for non-null and passes that boolean to 008ee600. |
| `00c17b70` | — | unknown | For exactly one argument, converts index 1, tests the resulting object for non-null and passes that boolean to 008ee710. |
| `00c186a0` | — | unknown | Converts indexes 1 and 2 when available, calls 008c6b60 with both float values and constant 1 when index 2 exists, then outputs whether index 2 converted. |
| `00c18a60` | — | unknown | When the argument count is positive, converts argument 1 and passes it to 008a8c90; that helper normalizes it to 0 or 1 and stores the byte at (*(this+0x644))+… |
| `00c18fc0` | — | unknown | Requires two arguments, converts a first typed value and a second integer, resolves the client target at offset 0xb9c and passes both to 00cea9d0. |
| `00c19030` | — | unknown | Requires two arguments, converts a first typed value and a second integer, resolves the client target at offset 0xb9c and passes both to 00ceaa00. |
| `00c190a0` | — | unknown | For one argument, converts index 1 with the typed helper, resolves the client target at offset 0xb9c and passes the result to 00ceaa20. |
| `00c190f0` | — | unknown | For one valid argument, converts argument 1 into ESI, resolves the client subobject through 008a6720, loads a pointer from that result into ECX and calls thunk… |
| `00c19140` | — | unknown | Requires two arguments, converts both and resolves the client target at offset 0xb9c before passing the pair to 00cea940. |
| `00c191b0` | — | unknown | Requires two arguments, converts both and resolves the client target at offset 0xb9c before passing the pair to 00cea980. |
| `00c19220` | — | unknown | For one argument, converts index 1 with the typed helper, resolves the client target at offset 0xb9c and passes the result to 00cea9b0. |
| `00c19270` | — | unknown | For one argument, converts index 1 with the typed helper, resolves the client target at offset 0xb9c and passes the result to 00ce9ad0. |
| `00c19820` | — | unknown | Requires arguments at indexes -1 through -4, reads three float values and one output pointer, calls 00a88480 with the pointer and vector, then writes its boole… |
| `00c1b3d0` | — | unknown | Reads two nested client setting fields and, when either differs from saved globals, passes the settings and four-word default data to helpers, then calls 00a39… |
| `00c1b960` | — | unknown | Walks the client list from its head at +0x6e4 through next links at node+0x78, matching node+0x74 against client+0x5a4. For a match it forms the stored time fr… |
| `00c1cbb0` | — | unknown | When the argument count is positive, converts argument 1 and queries through thunk 00a12830. If the record exists, passes selected record fields through the Lu… |
| `00c1ec70` | — | unknown | For exactly five arguments, converts the values, resolves a client object through 00984960, creates an IString argument and calls its virtual method at offset … |
| `00c25a90` | — | unknown | Validates GfxCtrl and a destination object, builds a GuiGfxParam array from remaining Lua arguments as numbers or strings, then invokes the native parameter up… |
| `00c25d20` | — | unknown | Validates DsTable, invokes the table interface method at offset 0x38, and pushes its result. |
| `00c263d0` | — | unknown | Resolves a named data source, applies optional row and column bounds, builds a delimited header and rows from cell values, then pushes the serialized string. |
| `00c267a0` | — | unknown | Checks Lua stack capacity, retrieves ImageControl userdata at the requested index, raises type or null-object errors when invalid, and returns its stored point… |
| `00c268a0` | — | unknown | Validates three arguments, converts the property name to an index, then calls a virtual setter at offset 0x140 with that index and supplied value. |

### `script.metrics`

**SoeUtil::HashList<ClientLuaMetrics::Entry,20,-1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01044db0` | HashList_ClientLuaMetricsEntry::Ctor | confirmed | Fastcall, ECX = this. Stores SoeUtil::HashListMap<int,ClientLuaMetrics::Entry,20,-1>::vftable at *this, zeroes this+0x10, +8, +0xc and +4, memsets 0x50 bytes a… |
| `01044e30` | HashList_ClientLuaMetricsEntry::ScalarDeletingDtor | confirmed | Thiscall with a flag byte param_2. Calls 0091d980 (the ClientLuaMetrics hash container destructor body), then if bit 0 of the flag is set calls _free on the ob… |

**SoeUtil::HashListMap<int,ClientLuaMetrics::Entry,20,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01044bb0` | HashListMap_ClientLuaMetricsEntry::Ctor | confirmed | Fastcall, ECX = this. Stores SoeUtil::HashListMap<int,ClientLuaMetrics::Entry,20,-1>::vftable at *this, zeroes this+0x10, +0x8, +0xc and +0x4, memsets 0x50 byt… |

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01044e50` | HashList_ClientLuaMetricsEntry::CtorClearFlag | likely | Fastcall, ECX = this. Calls 01044db0 on this (the Ghidra cookie argument is not a real argument), then stores byte 0 at this+0x64. |

### `script.minigame`

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009aae60` | MiniGameGroup_SetPreselectedGameId | confirmed | Calls FUN_0098e690 with the literal name MiniGameGroup.PreselectedGameId and param_1, i.e. publishes param_1 under that client-side property name. |
| `009b6490` | EmitUnlockUiEvent | confirmed | Builds a one-element Lua parameter array. For a tagged value with tag 3, 6, 7 or 8 it invokes the stored virtual release method, resets the local tag and paylo… |
| `009b6590` | EmitMiniGameFlashKeyPressed | confirmed | Builds a MiniGameFlash:OnKeyPressed Lua event, stores the supplied key value in event state, dispatches the event, and releases its string. |

### `script.quest`

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a87090` | QuestHandlerCloseHelper | confirmed | Issues the named script/UI callback "QuestHandler:CloseHelper" by calling FUN_009797a0 with that name and two null arguments on the global callback manager at … |

### `script.welcome`

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ad0000` | NotifyShowWelcomeScreen | confirmed | When the pending flag at +0x10 is set, calls Lua OnShowWelcomeScreen with a sign value and the two dwords at +0x14/+0x18, then clears the flag. |

## Source shards

[[S0146 00712020-00714060|S0146]] (3), [[S0195|S0195]] (6), [[S0223 008b7790-008bcf40|S0223]] (1), [[S0232 008e82e0-008ec1d0|S0232]] (1), [[S0233 008ec2a0-008f0cc0|S0233]] (1), [[S0236|S0236]] (1), [[S0237 008ff3d0-00904e70|S0237]] (3), [[S0260|S0260]] (9), [[S0261 00976740-0097c910|S0261]] (34), [[S0269 009a6cf0-009aceb0|S0269]] (1), [[S0271 009b2210-009b7170|S0271]] (2), [[S0274 009c2cc0-009c8d80|S0274]] (3), [[S0288 00a13190-00a18a10|S0288]] (2), [[S0289 00a18a90-00a1dd30|S0289]] (2), [[S0290 00a1de60-00a243e0|S0290]] (3), [[S0298 00a4b480-00a53380|S0298]] (1), [[S0308 00a81470-00a87b30|S0308]] (1), [[S0310 00a8ce60-00a91b00|S0310]] (10), [[S0322 00acd6d0-00ad3250|S0322]] (1), [[S0382 00bfcfe0-00c056a0|S0382]] (69), [[S0385 00c17390-00c1f0a0|S0385]] (100), [[S0386 00c1f190-00c26aa0|S0386]] (25), [[S0394|S0394]] (2), [[S0400 00c6df10-00c73b60|S0400]] (1), [[S0404 00c84f00-00c89db0|S0404]] (1), [[S0413 00cb6d20-00cbc7f0|S0413]] (2), [[S0421 00ce0340-00ce4c30|S0421]] (1), [[S0485|S0485]] (163), [[S0571 01044480-0104a2b0|S0571]] (4), [[S0594|S0594]] (4)
