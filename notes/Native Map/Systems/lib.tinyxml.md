# System: lib.tinyxml

10 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009f86b0` | `TiXmlVisitor::Dtor` | TiXmlVisitor | Resets the base visitor subobject during destruction. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009f8740` | `TiXmlVisitor::ScalarDeletingDtor` | TiXmlVisitor | Scalar deleting destructor for TiXmlVisitor. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009f9b70` | `TiXmlPrinter::Ctor` | TiXmlPrinter | Initializes a TinyXML printer object and its output strings. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `009f9c00` | `TiXmlPrinter::ScalarDeletingDtor` | TiXmlPrinter | Scalar deleting destructor for TiXmlPrinter. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |
| `00d54fd0` | `TiXmlAttribute::Parse` | TiXmlAttribute | Parses one XML attribute name=value pair into a TiXmlAttribute. | [[Native Map/Shards/S0445 00d54b90-00d59cb0|S0445]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009f9170` | `TiXmlVisitor::Dtor ? ⚠rename` | TiXmlVisitor | Releases visitor-owned buffers and resets the visitor base during destruction. | [[Native Map/Shards/S0284 009f8470-009feec0|S0284]] |

## ⚪ unknown (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00d54b90` | `— ⚠reject` | TiXmlElement | XML element start-tag parser of the TinyXML-derived parser used by the client's XML reader. | [[Native Map/Shards/S0445 00d54b90-00d59cb0|S0445]] |
| `00d54f10` | `— ⚠reject` | TiXmlComment | Parses an XML comment node from '<!--' through '-->'. | [[Native Map/Shards/S0445 00d54b90-00d59cb0|S0445]] |
| `00d55170` | `— ⚠reject` | TiXmlText | Parses XML character data and CDATA sections for a TiXmlText node. | [[Native Map/Shards/S0445 00d54b90-00d59cb0|S0445]] |
| `00d55300` | `— ⚠reject` | TiXmlDeclaration | Parses the <?xml ... ?> declaration of an XML document (version, encoding, standalone). | [[Native Map/Shards/S0445 00d54b90-00d59cb0|S0445]] |

