use rocket::FromForm;

#[derive(FromForm)]
pub struct ProofResponse {
    pub prover_id: String,
    pub ts: u128,
    pub seed: String,
    // TODO proof
}
