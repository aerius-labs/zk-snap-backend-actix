
pub struct BaseProofWitness {
    pub proposal_id_str: String,
    pub members_root_str: String,
    pub encryption_public_key_str: String,
}

impl BaseProofWitness {
    pub fn new(proposal_id: &str, members_root: &str, encryption_public_key: &str) -> Self {
        BaseProofWitness {
            proposal_id_str: proposal_id.to_string(),
            members_root_str: members_root.to_string(),
            encryption_public_key_str: encryption_public_key.to_string(),
        }
    }
}

pub struct ReccursiveProofWitness {
    pub proposal_id: String,
    pub witness: String,
}

impl ReccursiveProofWitness {
    pub fn new(proposal_id: &str, witness: &str) -> Self {
        ReccursiveProofWitness {
            proposal_id: proposal_id.to_string(),
            witness: witness.to_string(),
        }
    }
}
