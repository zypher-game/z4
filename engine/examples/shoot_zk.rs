use ark_bn254::Fr;
use ark_ed_on_bn254::EdwardsAffine;
use ark_std::rand::{CryptoRng, RngCore};
use std::collections::HashMap;
use zplonk::{
    errors::Result,
    params::{load_lagrange_params, load_srs_params, ProverParams, VerifierParams},
    poly_commit::kzg_poly_commitment::KZGCommitmentSchemeBN254,
    turboplonk::{
        constraint_system::{ConstraintSystem, TurboCS, VarIndex},
        indexer::{indexer_with_lagrange, PlonkProof},
        prover::prover_with_lagrange,
        verifier::verifier,
    },
    utils::transcript::Transcript,
};

#[derive(Clone)]
pub struct ShootOperation {
    pub from: EdwardsAffine,
    pub to: EdwardsAffine,
}

pub struct ShootResult {
    pub player: EdwardsAffine,
    pub hp: u32,
    pub bullet: u32,
}

pub struct ShootResultVar {
    pub p_x: VarIndex,
    pub p_y: VarIndex,
    pub hp: VarIndex,
    pub bullet: VarIndex,
    pub r_hp: u32,
    pub r_bullet: u32,
}

pub const TOTAL: u32 = 5;
const PLONK_PROOF_TRANSCRIPT: &[u8] = b"Plonk shoot Proof";
const N_TRANSCRIPT: &[u8] = b"Number of players";

pub(crate) fn build_cs(
    players: &[EdwardsAffine],
    inputs: &[ShootOperation],
) -> (TurboCS<Fr>, Vec<ShootResult>) {
    let mut cs = TurboCS::new();

    let mut indexs: HashMap<EdwardsAffine, usize> = HashMap::new();
    let mut results: Vec<ShootResultVar> = vec![];
    let hp = cs.new_variable(Fr::from(TOTAL));
    let bullet = cs.new_variable(Fr::from(TOTAL));

    for p in players {
        let p_x = cs.new_variable(p.x);
        let p_y = cs.new_variable(p.y);
        let r = ShootResultVar {
            p_x,
            p_y,
            hp,
            bullet,
            r_hp: TOTAL,
            r_bullet: TOTAL,
        };
        let index = results.len();
        indexs.insert(*p, index);
        results.push(r);
    }

    for input in inputs {
        let index_from = *indexs.get(&input.from).unwrap();
        let index_to = *indexs.get(&input.to).unwrap();

        let bullet_var = results[index_from].bullet;
        let hp_var = results[index_to].hp;

        let new_bullet = cs.sub(bullet_var, cs.one_var());
        let new_hp = cs.sub(hp_var, cs.one_var());

        results[index_from].bullet = new_bullet;
        results[index_to].hp = new_hp;

        results[index_from].r_bullet -= 1;
        results[index_to].r_hp -= 1;
    }

    let mut publics = vec![];
    for r in results {
        cs.prepare_pi_variable(r.p_x);
        cs.prepare_pi_variable(r.p_y);
        cs.prepare_pi_variable(r.bullet);
        cs.prepare_pi_variable(r.hp);

        publics.push(ShootResult {
            player: EdwardsAffine::new(cs.witness[r.p_x], cs.witness[r.p_y]),
            bullet: r.r_bullet,
            hp: r.r_hp,
        });
    }

    cs.pad();

    (cs, publics)
}

pub fn gen_prover_params(
    players: &[EdwardsAffine],
    operations: &[ShootOperation],
) -> Result<ProverParams> {
    let (cs, _) = build_cs(players, operations);

    let cs_size = cs.size();
    let pcs = load_srs_params(cs_size)?;
    let lagrange_pcs = load_lagrange_params(cs_size);

    let prover_params = indexer_with_lagrange(&cs, &pcs, lagrange_pcs.as_ref(), None).unwrap();

    Ok(ProverParams {
        pcs,
        lagrange_pcs,
        cs,
        prover_params,
    })
}

pub fn get_verifier_params(prover_params: ProverParams) -> VerifierParams {
    VerifierParams::from(prover_params)
}

pub fn prove_shoot<R: CryptoRng + RngCore>(
    prng: &mut R,
    prover_params: &ProverParams,
    players: &[EdwardsAffine],
    inputs: &[ShootOperation],
) -> Result<(PlonkProof<KZGCommitmentSchemeBN254>, Vec<ShootResult>)> {
    let n = players.len();

    let (mut cs, publics) = build_cs(players, inputs);
    let witness = cs.get_and_clear_witness();

    let mut transcript = Transcript::new(PLONK_PROOF_TRANSCRIPT);
    transcript.append_u64(N_TRANSCRIPT, n as u64);

    let proof = prover_with_lagrange(
        prng,
        &mut transcript,
        &prover_params.pcs,
        prover_params.lagrange_pcs.as_ref(),
        &cs,
        &prover_params.prover_params,
        &witness,
    )?;

    Ok((proof, publics))
}

pub fn verify_shoot(
    verifier_params: &VerifierParams,
    publics: &[ShootResult],
    proof: &PlonkProof<KZGCommitmentSchemeBN254>,
) -> Result<()> {
    let n = publics.len();

    let mut transcript = Transcript::new(PLONK_PROOF_TRANSCRIPT);
    transcript.append_u64(N_TRANSCRIPT, n as u64);

    let mut online_inputs = vec![];

    for p in publics.iter() {
        online_inputs.push(p.player.x);
        online_inputs.push(p.player.y);
        online_inputs.push(Fr::from(p.bullet));
        online_inputs.push(Fr::from(p.hp));
    }

    Ok(verifier(
        &mut transcript,
        &verifier_params.shrunk_vk,
        &verifier_params.shrunk_cs,
        &verifier_params.verifier_params,
        &online_inputs,
        proof,
    )?)
}
