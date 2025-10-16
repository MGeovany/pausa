use rand::{distributions::Alphanumeric, Rng};
use sha2::{Digest, Sha256};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

#[derive(Clone)]
pub struct PkcePair { pub verifier: String, pub challenge: String }

pub fn generate() -> PkcePair {
    let verifier: String = rand::thread_rng()
        .sample_iter(&Alphanumeric).take(64).map(char::from).collect();
    let digest = Sha256::digest(verifier.as_bytes());
    let challenge = URL_SAFE_NO_PAD.encode(digest);
    PkcePair { verifier, challenge }
}
