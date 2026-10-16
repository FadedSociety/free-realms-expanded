# Class: ChessMove

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c07ef0` | `ChessMove::ChessMove` | ✅ | ctor | Four-argument ChessMove constructor: stores the four byte arguments into this[0..3] (from, to, promote, special) and returns. |
| `00c07f10` | `— ⚠reject` | ⚪ | ctor | Default ChessMove constructor: writes zero into all four bytes of the move (no arguments) and returns. |
| `00c07f20` | `ChessMove::Compare` | ✅ | method | Compares two ChessMove values byte by byte (all four bytes must be equal) and returns true when they match, false otherwise. |
| `00c07f50` | `— ⚠reject` | ⚪ | method | Copy assignment (operator=) for the 4-byte ChessMove: copies the packed dword from the source into the destination and returns the destination. |
