//! Job-binding regression for the Harvesting candidate: every already-supported
//! regional spec binds the recovered Chef profile (45), never the game id.
//!
//! Evidence: wiki Chef rev 46136 associates Chef experience with the harvesting
//! minigame; wiki Harvesting rev 48154 separates minigame produce (usable in
//! cooking) from farm produce; no Farmer profile id is recovered. See
//! `notes/Research/Harvesting Job Binding.md` and
//! `notes/Research/Harvesting Native Launch Preflight.md`.
use osfr_tools::harvesting::{
    content, validate_content, HarvestError, CHEF_PROFILE_ID, RECOVERED_REGION_GAME_IDS,
};

#[test]
fn all_supported_regions_bind_the_chef_profile() {
    assert_eq!(CHEF_PROFILE_ID, 45);
    for game_id in RECOVERED_REGION_GAME_IDS {
        let spec = content(game_id).unwrap_or_else(|error| panic!("game {game_id}: {error}"));
        assert_eq!(
            spec.profile_id, CHEF_PROFILE_ID,
            "game {game_id} must bind the recovered Chef profile"
        );
        assert_ne!(
            spec.profile_id, game_id,
            "no game id may act as a profile id"
        );
    }
}

#[test]
fn validate_content_refuses_a_wrong_or_farmer_like_profile() {
    // The placeholder bug shape: the profile mirrors the game id.
    let mut wrong = content(111).unwrap();
    wrong.profile_id = 111;
    assert!(matches!(
        validate_content(&wrong),
        Err(HarvestError::Content(_))
    ));

    // No Farmer profile id is recovered, so any other job is refused too.
    let mut farmer_like = content(109).unwrap();
    farmer_like.profile_id = 15;
    assert!(matches!(
        validate_content(&farmer_like),
        Err(HarvestError::Content(_))
    ));

    // The recovered Chef binding stays valid.
    let mut chef = content(109).unwrap();
    chef.profile_id = CHEF_PROFILE_ID;
    validate_content(&chef).expect("Chef profile is the recovered binding");
}

#[test]
fn non_regional_profiles_still_refuse_launch() {
    // The gap-profile refusal is unchanged by the job binding.
    for game_id in [58, 110, 112, 114, 116, 118, 120, 217] {
        assert!(
            content(game_id).is_err(),
            "game {game_id} must stay unlaunchable"
        );
    }
}
