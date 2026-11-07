//! A character's identity entering its progression save.
use super::*;

impl Gameplay {
    pub fn apply_identity(&mut self, identity: &CharacterIdentity) -> io::Result<bool> {
        self.remember_identity_look(identity);
        self.set_character_name(identity.display_name());
        if self.profiles.is_none() {
            return Ok(false);
        }
        identity.validate()?;
        self.gender = identity.gender;
        self.creation_clothes = identity
            .attachments
            .iter()
            .map(|attachment| (attachment.definition, attachment.tint_id))
            .collect();
        let fingerprint = format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(identity).map_err(invalid)?)
        );
        if let Some(saved) = &self.identity_fingerprint {
            if saved != &fingerprint {
                return Err(invalid(
                    "character identity differs from this progression save",
                ));
            }
            return Ok(false);
        }
        for attachment in &identity.attachments {
            self.inventory
                .grant_with_tint(0, attachment.definition, 1, attachment.tint_id)?;
            self.inventory
                .equip_owned_definition(attachment.definition, 1)?;
        }
        self.identity_fingerprint = Some(fingerprint);
        Ok(true)
    }
}
