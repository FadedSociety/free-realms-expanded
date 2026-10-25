//! T517: byte-level codec tests for the housing rating directory stub.
//!
//! Layouts come from the 2014 client: request writer `0x00abfe70` (strings through `0x00911c30`),
//! reply reader `0x00c85530`, list reader `0x00c84e90` and record reader `0x00c83d90`. The
//! meaning of the reply's three strings is reconstructed (R); their order is the client's.
use osfr_tools::housing_rating::{
    CandidateInfoReply, CandidateInfoRequest, CandidateRatingInfo, HOUSING_SYSTEM, MAX_STRING,
};

/// One wire string: u32 byte length then UTF-8 bytes (client `0x00911c30`).
fn string(value: &str) -> Vec<u8> {
    let mut out = (value.len() as u32).to_le_bytes().to_vec();
    out.extend_from_slice(value.as_bytes());
    out
}

/// 174/16 as the client writer lays it out: header, system, candidate id, owner, correlation.
fn request(system: &str, candidate_id: &str, owner: u64, correlation: u64) -> Vec<u8> {
    let mut inner = vec![174, 0, 16];
    inner.extend(string(system));
    inner.extend(string(candidate_id));
    inner.extend(owner.to_le_bytes());
    inner.extend(correlation.to_le_bytes());
    inner
}

#[test]
fn the_native_34_byte_request_decodes_field_by_field() {
    let inner = request("Housing", "", 0x1122_3344_5566_7788, 0x99aa_bbcc_ddee_ff00);
    assert_eq!(inner.len(), 34, "2 + 1 + 11 + 4 + 8 + 8");
    assert_eq!(
        CandidateInfoRequest::decode(&inner),
        Some(CandidateInfoRequest {
            system: HOUSING_SYSTEM.to_owned(),
            candidate_id: String::new(),
            owner_guid: 0x1122_3344_5566_7788,
            correlation: 0x99aa_bbcc_ddee_ff00,
        })
    );
}

#[test]
fn malformed_requests_are_refused() {
    let valid = request("Housing", "", 7, 9);
    let mut trailing = valid.clone();
    trailing.push(0);
    let mut wrong_family = valid.clone();
    wrong_family[0] = 173;
    let mut wrong_subtype = valid.clone();
    wrong_subtype[2] = 15;
    let mut short = valid.clone();
    short.truncate(valid.len() - 1);

    let mut oversize = vec![174, 0, 16];
    oversize.extend(257u32.to_le_bytes());
    oversize.extend(vec![b'A'; 257]);

    let mut oversize_candidate = vec![174, 0, 16];
    oversize_candidate.extend(string("Housing"));
    oversize_candidate.extend(257u32.to_le_bytes());
    oversize_candidate.extend(vec![b'A'; 257]);

    let mut bad_utf8 = vec![174, 0, 16];
    bad_utf8.extend(7u32.to_le_bytes());
    bad_utf8.extend([0xff; 7]);

    // The client reader treats a negative length as an error; so does this decoder.
    let mut negative = vec![174, 0, 16];
    negative.extend((-1i32).to_le_bytes());

    for (what, inner) in [
        ("truncated", short),
        ("trailing byte", trailing),
        ("wrong family", wrong_family),
        ("wrong subtype", wrong_subtype),
        ("system over the bound", oversize),
        ("candidate id over the bound", oversize_candidate),
        ("invalid utf-8", bad_utf8),
        ("negative length", negative),
        ("empty", Vec::new()),
    ] {
        assert_eq!(
            CandidateInfoRequest::decode(&inner),
            None,
            "{what}: {inner:02x?}"
        );
    }
}

#[test]
fn an_empty_reply_is_the_header_the_correlation_and_a_zero_count() {
    let reply = CandidateInfoReply {
        correlation: 0x0102_0304_0506_0708,
        candidates: Vec::new(),
    };
    let mut expected = vec![174, 0, 17];
    expected.extend(0x0102_0304_0506_0708u64.to_le_bytes());
    expected.extend(0u32.to_le_bytes());
    assert_eq!(reply.encode_inner().unwrap(), expected);
    assert_eq!(CandidateInfoReply::decode(&expected), Some(reply));
}

#[test]
fn a_one_entry_reply_follows_the_client_reader_order() {
    let candidate = CandidateRatingInfo {
        candidate_id: "house-1".to_owned(),
        owner_name: "owner".to_owned(),
        house_name: "condo".to_owned(),
        rating: 4.5,
        votes: 7,
        has_rating: true,
        can_vote: false,
    };
    let reply = CandidateInfoReply {
        correlation: 9,
        candidates: vec![candidate],
    };
    let mut expected = vec![174, 0, 17];
    expected.extend(9u64.to_le_bytes());
    expected.extend(1i32.to_le_bytes());
    for text in ["house-1", "owner", "condo"] {
        expected.extend((text.len() as i32).to_le_bytes());
        expected.extend(text.as_bytes());
    }
    expected.extend(4.5f32.to_le_bytes());
    expected.extend(7i32.to_le_bytes());
    expected.push(1);
    expected.push(0);
    assert_eq!(reply.encode_inner().unwrap(), expected);
    assert_eq!(CandidateInfoReply::decode(&expected), Some(reply));
}

#[test]
fn malformed_replies_are_refused() {
    let valid = CandidateInfoReply {
        correlation: 1,
        candidates: Vec::new(),
    }
    .encode_inner()
    .unwrap();
    let mut trailing = valid.clone();
    trailing.push(0);
    let mut wrong_subtype = valid.clone();
    wrong_subtype[2] = 16;

    // A count larger than the body can hold is refused before any allocation.
    let mut too_many = vec![174, 0, 17];
    too_many.extend(1u64.to_le_bytes());
    too_many.extend(i32::MAX.to_le_bytes());

    // One record of the one-entry reply is cut short.
    let mut truncated_record = vec![174, 0, 17];
    truncated_record.extend(1u64.to_le_bytes());
    truncated_record.extend(1i32.to_le_bytes());
    truncated_record.extend(4i32.to_le_bytes());
    truncated_record.extend(b"id-1");

    for (what, inner) in [
        ("trailing byte", trailing),
        ("wrong subtype", wrong_subtype),
        ("count larger than the body", too_many),
        ("truncated record", truncated_record),
        ("empty", Vec::new()),
    ] {
        assert_eq!(
            CandidateInfoReply::decode(&inner),
            None,
            "{what}: {inner:02x?}"
        );
    }
}

#[test]
fn an_oversize_string_is_not_encoded() {
    let reply = CandidateInfoReply {
        correlation: 0,
        candidates: vec![CandidateRatingInfo {
            candidate_id: "id".to_owned(),
            owner_name: "owner".to_owned(),
            house_name: "A".repeat(MAX_STRING + 1),
            rating: 0.0,
            votes: 0,
            has_rating: false,
            can_vote: false,
        }],
    };
    assert!(reply.encode_inner().is_err());
}
