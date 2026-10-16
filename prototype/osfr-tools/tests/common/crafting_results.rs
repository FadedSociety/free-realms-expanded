pub fn vector(name: &str) -> Vec<u8> {
    let v: serde_json::Value = serde_json::from_str(include_str!(
        "../fixtures/native-crafting-results-community-vectors.json"
    ))
    .unwrap();
    v["vectors"][name]
        .as_str()
        .unwrap()
        .as_bytes()
        .chunks_exact(2)
        .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
        .collect()
}
pub fn ack(id: i32) -> Vec<u8> {
    let mut b = vec![39, 0, 7];
    for n in [id, -1, -1] {
        b.extend(n.to_le_bytes());
    }
    b
}
