# Native Kart lobby gateway binding

Root registered the independently reviewed `kart_lobby_native` codec and added optional `kart_lobby.json` loading in Gameplay. This file is the typed five-table body, with unknown fields positional; supplied rows are caller-authored test content, not a recovered retail catalogue. Missing content stays inert. Loading rejects non-regular/symlink paths, malformed JSON, unknown fields and files over256KiB. The exact encoded inner reply is cached once; the gateway adds its normal tunnel envelope.

With developer tools enabled, an exact inner102/1 request (4bytes, no payload) returns102/2 through the full codec. Truncated/trailing requests are refused. Other subtypes, disabled developer tools and missing content remain inert. Serving the list does not launch a race or mutate progression/character state.

Two meaningful private integration tests failed on the preserved previous Gameplay revision (missing reply / malformed content accepted), then passed after the implementation. They exercise allfive synthetic tables, repeated requests, opt-in/off, malformed inputs and unchanged save/content. Independent review reproduced both passes and inspected envelope ownership. Logs and source hashes: `../lab/reports/kart-lobby-gameplay-lead-20261009T1626Z/receipt.json`. No captured rows or raw bytes are embedded in the tests.

Root also registered the separately reviewed opcode31 zone and native33 progress codecs and changed their tests to import the production modules. The new registration requires a combined root gate; no new release is built or deployed. Native lobby rendering, race launch, driving, lap completion, results and return remain open.

See [[Kart Native Lobby Codec]], [[Kart Native Progress Codec]], [[Native Zone Transition Codec]], [[../Reviews/Kart Progress and Gateway Catalogue Review]].

Root extension: a third private integration check passes through the actual Gateway, verifying one tunnel wrapper, developer-disabled routing, repeated requests, malformed atomic refusal and whole-snapshot conservation (`gateway-green.log`,3passed/0failed). The separately exported immutable test catalogue package is `../lab/references/opening-native-lobby-20261009T163603Z`; all45 prior content files are byte-identical and only the archived typed catalogue is added. This is prepared content, not deployed/native accepted.
