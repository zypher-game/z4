use ark_ed_on_bn254::EdwardsProjective;
use poker_core::{
    cards::{CryptoCard, EncodingCard, ENCODING_CARDS_MAPPING},
    combination::CryptoCardCombination,
    play::{PlayAction, PlayerEnvBuilder},
    schnorr::KeyPair,
    task::Task,
};
use poker_methods::{POKER_METHOD_ELF, POKER_METHOD_ID};
use rand_chacha::{rand_core::SeedableRng, ChaChaRng};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use std::collections::HashMap;
use zshuffle::{
    reveal::{reveal, unmask, verify_reveal},
    Ciphertext,
};

pub fn prove_task(task: &Task) {
    let env = ExecutorEnv::builder()
        .write(&task)
        .unwrap()
        .build()
        .unwrap();

    let prover = default_prover();

    let start = std::time::Instant::now();
    let receipt = prover.prove(env, POKER_METHOD_ELF).unwrap();
    println!("prover time: {:.2?}", start.elapsed());

    println!("I can prove it!");
}

fn main() {
    let task = mock_task();

    prove_task(&task);
}

pub fn mock_task() -> Task {
    let mut rng = ChaChaRng::from_seed([0u8; 32]);
    let a = r##"
    {"private_key":[199,118,160,104,194,165,207,217,144,164,212,71,181,14,72,177,120,188,233,202,221,172,188,248,216,225,221,164,66,122,141,0],"public_key":[198,139,168,239,228,144,7,82,187,206,88,35,111,79,134,63,62,56,231,98,241,53,113,86,165,66,74,181,254,47,228,162]}
     "##;
    let b = r##"
     {"private_key":[159,254,8,205,249,43,125,141,199,65,215,232,162,202,51,1,249,136,244,62,94,79,52,54,199,126,33,11,174,2,3,5],"public_key":[94,53,185,211,206,160,229,182,98,91,232,221,106,21,242,162,153,250,55,49,138,11,184,221,250,42,225,150,125,187,176,46]}
      "##;
    let c = r##"
      {"private_key":[210,122,124,112,222,6,173,188,209,175,27,133,180,55,104,153,48,93,57,111,150,225,178,193,249,87,191,109,19,153,228,5],"public_key":[59,196,247,52,176,21,214,60,244,109,153,119,203,235,57,221,223,53,180,218,93,220,193,243,153,117,147,135,103,58,27,175]}
       "##;

    let alice: KeyPair = serde_json::from_str(&a).unwrap();
    let bob: KeyPair = serde_json::from_str(&b).unwrap();
    let charlie: KeyPair = serde_json::from_str(&c).unwrap();

    let card_serialized = r##"
    [{"e1":[219,58,243,156,152,45,248,138,226,69,19,157,16,79,127,1,71,151,115,58,211,166,35,72,118,140,150,160,66,245,198,37],"e2":[247,217,241,56,131,190,206,15,169,71,35,123,185,239,57,35,47,241,97,44,80,186,101,96,60,68,55,190,12,249,221,16]},{"e1":[207,102,119,246,100,212,13,184,140,220,33,147,180,255,19,34,156,23,49,60,249,133,184,121,79,182,230,235,102,253,109,134],"e2":[194,245,22,18,238,42,192,56,246,37,112,244,208,47,72,146,251,142,199,153,251,208,226,44,127,234,243,174,151,166,83,172]},{"e1":[249,83,189,36,148,115,32,162,44,195,113,252,99,210,5,40,15,62,16,196,130,159,185,119,150,203,138,16,255,31,1,138],"e2":[139,220,87,43,40,224,172,69,195,168,200,184,130,1,196,130,169,132,181,77,122,126,98,60,198,190,173,70,120,58,14,144]},{"e1":[101,82,206,163,239,98,31,221,45,83,92,53,118,125,71,73,191,206,145,103,92,141,171,222,179,167,86,174,64,119,172,170],"e2":[78,81,172,240,175,5,108,157,109,220,181,83,216,255,221,15,197,142,10,196,155,31,139,157,144,138,76,209,183,246,173,41]},{"e1":[163,165,88,236,221,85,95,3,183,194,245,156,211,72,117,169,35,172,147,170,183,182,254,168,7,131,208,15,45,212,113,41],"e2":[155,94,12,124,204,136,31,178,206,82,91,147,182,65,26,72,27,22,248,134,71,174,52,75,231,5,99,88,190,117,224,42]},{"e1":[1,204,216,71,255,2,96,226,60,173,111,21,227,75,164,233,80,236,15,38,210,231,45,141,89,201,147,162,58,120,130,129],"e2":[93,167,47,240,4,222,91,184,21,179,66,169,90,14,235,228,223,10,202,66,184,109,208,199,178,9,210,112,125,18,152,37]},{"e1":[37,44,12,19,185,1,236,76,18,28,71,34,169,17,103,155,139,207,172,171,193,229,15,222,165,249,55,9,130,77,253,31],"e2":[145,41,183,179,250,33,96,77,246,128,97,151,239,243,121,131,6,170,224,44,157,122,183,24,23,59,73,158,210,189,7,15]},{"e1":[119,64,57,10,251,196,177,197,12,212,94,161,189,28,159,203,164,159,217,7,83,200,78,118,107,254,118,236,82,254,69,140],"e2":[4,17,87,132,207,46,81,244,31,215,62,144,144,203,251,11,98,72,100,185,26,218,239,241,169,114,150,149,182,104,166,156]},{"e1":[197,136,75,202,38,131,145,3,42,22,252,171,82,38,49,137,181,3,201,112,101,123,220,66,96,2,46,238,98,23,52,21],"e2":[168,12,155,149,82,226,36,200,197,202,119,193,93,194,72,91,52,203,151,154,76,136,188,127,255,237,248,55,182,197,71,24]},{"e1":[120,233,153,112,98,226,69,144,88,97,129,196,24,208,203,57,79,118,60,123,26,184,174,97,11,81,61,206,158,52,184,27],"e2":[217,208,127,159,0,246,181,116,92,222,91,6,226,80,69,42,146,228,228,252,101,227,73,221,65,84,216,203,100,72,174,37]},{"e1":[71,244,192,115,182,2,57,208,249,147,56,29,16,10,142,148,211,214,61,242,31,94,134,90,164,87,89,230,228,65,186,10],"e2":[201,246,255,186,40,119,107,72,103,39,201,12,183,28,150,8,176,137,176,59,205,96,175,208,229,70,180,192,109,96,114,141]},{"e1":[179,228,133,50,78,51,33,221,43,170,167,96,128,149,127,39,4,182,22,123,183,220,150,123,92,92,171,55,92,166,91,32],"e2":[41,163,15,126,131,29,38,71,29,123,24,114,161,51,215,233,122,122,169,163,53,156,185,63,203,110,95,37,195,11,91,161]},{"e1":[203,145,170,163,62,39,178,141,106,132,167,206,255,167,146,117,2,233,171,206,229,42,221,117,141,123,73,246,36,34,245,145],"e2":[173,67,220,231,251,172,56,166,214,119,218,15,154,99,202,34,174,199,255,78,246,128,30,127,111,147,83,23,98,157,144,5]},{"e1":[229,69,40,101,185,48,251,247,160,210,55,186,65,175,28,235,184,150,23,132,249,181,125,2,118,167,220,206,92,168,67,27],"e2":[249,234,221,97,17,202,67,69,232,125,61,35,241,213,55,219,191,13,190,216,187,86,76,167,187,196,45,244,122,238,1,130]},{"e1":[188,30,58,89,104,206,85,49,153,122,203,65,153,179,181,213,120,21,44,5,208,208,47,202,79,186,187,11,253,107,27,147],"e2":[237,73,198,131,2,107,19,101,229,76,191,57,12,249,199,192,129,240,38,222,59,153,229,199,231,170,221,250,180,12,168,141]},{"e1":[211,238,197,195,38,141,145,223,129,224,41,79,37,174,127,97,250,210,10,147,89,228,54,153,244,74,10,205,120,146,179,144],"e2":[167,202,138,97,239,33,3,144,120,45,230,129,72,79,199,154,201,148,61,116,134,76,30,25,164,31,38,51,0,82,94,19]},{"e1":[93,143,4,200,147,164,76,54,34,175,137,14,107,204,186,190,213,73,96,173,84,158,40,125,168,55,20,200,61,78,49,9],"e2":[184,84,166,167,63,59,176,158,137,111,128,113,176,107,202,21,184,217,209,238,109,74,32,218,112,27,174,161,235,33,8,42]},{"e1":[2,68,67,146,107,207,73,205,214,186,32,175,23,46,159,185,35,250,31,207,119,53,172,236,88,216,214,136,230,154,241,6],"e2":[254,129,85,122,90,35,216,193,174,216,157,72,57,165,84,158,129,33,78,123,90,13,85,184,83,193,42,1,203,145,111,39]},{"e1":[3,43,9,168,219,94,191,124,194,66,113,81,24,238,216,138,150,176,38,84,110,10,213,86,125,35,30,226,209,5,105,28],"e2":[67,169,252,180,47,53,118,166,76,19,140,7,78,33,97,193,126,142,230,188,214,175,153,111,105,179,153,172,137,168,224,21]},{"e1":[226,69,70,71,174,169,152,142,133,244,16,192,146,205,216,232,163,155,151,232,229,113,88,151,74,173,162,243,47,15,84,128],"e2":[37,182,2,122,11,112,29,100,128,151,53,36,228,189,4,141,28,56,77,72,145,17,97,254,29,192,159,191,152,28,242,21]},{"e1":[85,17,118,217,190,34,84,42,202,245,159,211,142,119,89,238,166,217,1,197,69,22,134,177,139,62,115,174,62,19,70,164],"e2":[68,10,152,248,98,38,79,37,207,120,49,120,117,224,150,174,95,82,56,182,170,225,36,230,76,28,12,203,4,68,203,10]},{"e1":[247,99,112,152,3,71,242,26,16,75,72,106,199,213,136,110,217,14,120,93,248,199,81,104,137,90,148,241,6,123,7,134],"e2":[11,219,175,46,90,76,78,220,52,145,243,240,3,30,72,133,36,64,103,68,191,49,190,235,15,215,56,244,93,198,113,165]},{"e1":[10,3,73,204,7,40,122,8,150,108,188,238,134,177,46,248,60,59,119,41,247,1,115,95,13,216,221,76,147,248,205,156],"e2":[117,208,56,10,105,7,227,177,6,67,19,50,248,137,39,245,91,53,203,95,44,117,229,119,243,135,227,103,149,50,83,151]},{"e1":[100,151,85,236,237,103,212,64,141,236,121,75,98,126,9,154,68,128,73,199,47,223,82,84,98,136,15,104,87,171,90,176],"e2":[222,1,67,2,227,120,32,51,123,38,197,47,249,52,226,202,139,10,195,254,213,190,188,36,168,252,233,59,80,64,154,158]},{"e1":[14,3,120,80,91,152,225,233,188,104,193,126,6,42,65,210,125,218,161,112,250,164,124,89,131,128,21,3,116,184,222,43],"e2":[185,184,150,147,94,86,106,162,220,166,198,34,234,146,6,154,221,83,84,43,155,204,140,192,151,205,174,60,69,53,254,148]},{"e1":[205,255,20,58,220,95,188,185,115,101,89,94,149,159,41,15,115,180,17,199,92,210,116,184,26,152,197,229,94,41,153,150],"e2":[21,32,195,81,229,31,173,42,21,46,181,34,22,221,145,203,104,20,14,136,210,4,93,220,100,58,230,105,253,216,131,26]},{"e1":[10,139,219,27,82,121,138,74,195,170,188,98,196,190,115,136,178,235,113,20,86,61,124,111,70,67,99,254,98,186,157,38],"e2":[141,120,147,49,107,215,107,126,148,82,240,249,52,105,171,196,157,173,87,237,159,119,249,74,177,57,80,34,190,74,156,0]},{"e1":[209,46,110,251,14,171,75,46,71,196,110,247,170,133,166,49,175,188,240,57,253,98,73,92,164,157,116,139,197,13,76,155],"e2":[98,27,197,123,37,48,101,147,98,162,123,239,157,103,47,193,221,239,34,8,171,51,130,139,178,57,108,77,19,252,12,152]},{"e1":[167,88,129,163,192,148,88,43,196,62,212,86,33,44,3,7,149,70,218,193,235,45,56,243,214,66,224,173,98,157,18,41],"e2":[170,134,160,255,171,37,217,110,184,214,4,134,27,164,16,76,182,210,246,196,19,64,243,41,187,203,235,21,139,135,171,128]},{"e1":[191,42,133,165,66,241,95,112,211,43,248,43,215,30,88,233,150,192,137,97,197,104,192,207,100,94,10,241,95,229,84,134],"e2":[230,143,223,128,63,178,203,39,171,104,210,27,234,50,0,21,139,77,187,71,110,22,114,198,174,165,54,153,200,40,251,155]},{"e1":[222,169,113,115,2,176,186,56,170,88,117,186,78,45,63,245,172,197,92,69,205,167,247,127,28,117,156,60,14,133,53,176],"e2":[132,67,227,156,90,25,191,206,0,74,14,217,28,182,130,163,184,143,142,253,148,82,219,197,192,62,131,94,249,172,41,29]},{"e1":[100,105,80,174,132,26,182,72,181,169,2,24,20,235,234,173,247,28,82,49,73,182,168,207,52,224,160,168,77,251,18,8],"e2":[35,209,80,171,32,0,146,241,187,54,131,218,190,28,19,108,22,37,115,191,253,142,157,79,118,109,56,250,169,55,45,135]},{"e1":[105,120,228,230,47,200,129,242,29,67,127,87,244,143,211,218,101,133,148,178,140,156,250,43,13,181,84,210,122,144,95,6],"e2":[157,249,102,81,112,129,176,42,222,24,86,65,32,123,83,230,187,143,119,182,207,163,99,164,250,199,86,150,218,138,75,44]},{"e1":[5,100,202,192,130,231,224,217,71,87,167,119,53,144,18,78,153,21,1,148,187,183,161,25,175,30,50,140,224,75,251,169],"e2":[189,162,121,45,77,138,110,23,72,5,77,127,195,156,78,97,212,240,26,29,38,163,231,0,94,213,36,16,146,180,124,128]},{"e1":[35,69,104,234,80,98,70,158,2,101,35,54,72,146,105,77,147,211,97,110,165,209,40,50,162,54,134,83,30,242,240,15],"e2":[245,254,219,92,106,172,84,17,36,132,120,206,107,95,17,200,60,172,65,89,238,207,92,28,168,75,250,63,198,245,1,143]},{"e1":[96,197,254,195,124,20,127,53,103,116,88,200,144,29,185,22,147,205,2,111,167,237,217,118,21,36,120,231,29,137,189,138],"e2":[7,207,145,74,250,54,131,106,158,105,227,178,236,134,200,204,158,0,47,108,124,113,154,143,33,189,81,108,19,86,91,156]},{"e1":[132,244,161,18,42,33,241,41,213,50,29,255,20,100,105,179,106,144,127,203,32,158,218,133,179,111,79,117,19,65,127,152],"e2":[171,62,3,9,249,171,67,232,241,145,116,205,26,143,46,143,148,17,172,117,52,89,48,151,221,145,208,106,137,29,7,132]},{"e1":[164,23,101,47,147,80,101,110,62,88,47,244,223,126,207,243,200,147,231,41,20,171,70,55,2,27,59,141,226,109,32,40],"e2":[244,144,141,154,19,224,229,3,85,197,7,242,200,185,165,220,219,179,236,157,18,10,55,135,152,39,0,244,232,87,236,140]},{"e1":[151,123,247,122,214,110,67,141,112,137,67,55,197,61,29,193,33,159,143,52,42,159,176,102,145,44,27,74,160,155,211,151],"e2":[199,151,195,133,221,66,126,6,141,186,231,162,120,202,248,163,41,37,170,127,86,139,2,171,67,63,245,204,40,161,44,139]},{"e1":[168,155,44,194,96,88,112,85,176,49,47,176,249,81,13,14,216,118,210,223,37,30,60,6,253,25,81,185,79,250,219,137],"e2":[216,203,64,105,59,145,41,194,23,32,199,108,188,190,54,189,252,216,76,18,38,123,219,135,192,185,27,160,213,129,16,47]},{"e1":[64,39,199,229,53,44,136,178,83,127,154,30,94,251,237,4,61,98,234,94,60,99,142,68,244,102,185,197,39,57,172,38],"e2":[146,247,92,13,78,220,58,45,82,100,180,87,16,228,105,150,147,124,19,42,137,232,248,229,222,225,126,189,81,105,122,16]},{"e1":[108,177,86,155,140,170,184,126,137,141,88,215,120,99,163,6,25,254,137,147,230,206,85,247,24,254,131,143,54,41,153,166],"e2":[161,217,126,56,3,204,174,58,0,76,209,215,108,195,200,90,179,253,235,8,42,255,166,188,84,96,239,254,234,155,26,133]},{"e1":[160,40,222,110,212,126,219,182,63,235,16,80,28,188,12,235,200,18,172,221,51,14,225,210,56,142,215,100,166,65,119,162],"e2":[181,76,25,99,95,179,126,37,56,26,19,71,31,143,51,135,105,228,163,164,80,28,138,72,185,202,219,164,167,183,231,128]},{"e1":[135,148,70,46,120,24,188,116,201,201,234,6,204,68,84,239,186,130,200,74,226,95,110,126,154,121,248,96,168,52,153,43],"e2":[208,111,139,9,28,157,63,73,123,154,99,168,88,137,74,69,187,35,209,93,247,203,86,56,53,40,170,144,16,137,7,18]},{"e1":[169,132,21,47,234,14,180,208,112,54,202,161,181,229,249,232,217,222,55,56,56,119,186,203,4,4,119,62,178,82,58,6],"e2":[175,1,173,86,143,119,93,12,246,102,16,139,30,115,206,90,222,9,70,52,154,80,20,153,104,104,222,87,167,37,161,22]},{"e1":[205,32,152,250,34,43,18,175,136,60,207,123,37,122,150,220,29,229,16,113,165,205,11,188,92,99,235,110,79,115,181,22],"e2":[67,233,130,51,31,249,189,11,170,88,114,232,230,50,58,79,211,248,244,4,99,122,201,23,141,73,22,206,46,249,57,3]},{"e1":[207,45,11,247,74,158,123,55,153,182,244,60,66,108,147,160,109,142,216,12,119,127,251,27,95,249,115,195,229,41,151,158],"e2":[87,116,194,9,198,67,179,161,147,133,172,158,89,54,59,46,69,212,6,108,131,38,167,244,85,64,73,118,194,6,105,13]},{"e1":[13,146,83,47,250,76,231,216,135,173,209,69,252,237,79,219,185,3,174,115,128,25,201,107,231,71,186,25,172,138,239,175],"e2":[50,250,232,113,243,26,164,108,82,32,19,55,6,244,109,198,198,228,129,24,156,221,11,143,20,197,104,237,201,67,26,147]},{"e1":[42,241,93,124,115,131,60,157,138,235,194,131,196,11,75,70,15,99,239,216,2,15,67,209,217,115,194,147,220,83,178,42],"e2":[147,189,164,237,57,217,226,68,140,167,142,234,83,154,123,62,53,74,182,142,174,69,82,237,178,238,247,44,50,40,135,20]},{"e1":[65,62,65,74,104,233,168,176,115,131,31,56,39,27,251,126,247,116,146,185,196,137,2,212,83,172,126,210,3,161,199,174],"e2":[165,65,135,233,192,80,188,63,41,36,95,224,145,160,123,212,192,34,211,164,229,43,242,186,69,28,154,121,62,136,15,172]},{"e1":[120,175,118,215,48,110,204,47,113,10,18,161,243,23,196,119,196,88,196,108,57,240,0,14,111,72,188,255,196,175,143,143],"e2":[8,250,207,44,201,115,196,16,93,213,175,230,100,152,100,53,1,112,180,169,121,15,83,192,205,211,3,9,90,133,92,144]},{"e1":[89,133,176,107,245,109,147,95,142,126,212,87,205,153,132,93,170,179,181,91,228,0,0,145,159,152,115,83,218,236,156,144],"e2":[118,50,122,45,59,176,203,228,157,133,68,2,111,101,243,181,194,70,84,212,114,96,245,15,166,130,29,12,126,227,167,131]}]
     "##;

    let shuffle_cards: Vec<Ciphertext<EdwardsProjective>> =
        serde_json::from_str(&card_serialized).unwrap();

    let alice_deck = &shuffle_cards[..18];
    let bob_deck = &shuffle_cards[18..35];
    let charlie_deck = &shuffle_cards[35..52];

    let alice_z: zshuffle::keygen::Keypair = alice.clone().into();
    let bob_z: zshuffle::keygen::Keypair = bob.clone().into();
    let charlie_z: zshuffle::keygen::Keypair = charlie.clone().into();

    let mut a_card = vec![];
    let mut b_card = vec![];
    let mut c_card = vec![];

    let mut reveal_proofs = HashMap::new();

    for card in alice_deck.iter() {
        let (reveal_card_b, reveal_proof_b) = reveal(&mut rng, &bob_z, card).unwrap();
        let (reveal_card_c, reveal_proof_c) = reveal(&mut rng, &charlie_z, card).unwrap();
        let (reveal_card_a, reveal_proof_a) = reveal(&mut rng, &alice_z, card).unwrap();
        verify_reveal(&bob_z.public, card, &reveal_card_b, &reveal_proof_b).unwrap();
        verify_reveal(&charlie_z.public, card, &reveal_card_c, &reveal_proof_c).unwrap();
        verify_reveal(&alice_z.public, card, &reveal_card_a, &reveal_proof_a).unwrap();

        let reveals = vec![reveal_card_b, reveal_card_a, reveal_card_c];
        let unmasked_card = unmask(card, &reveals).unwrap();

        let opened_card = ENCODING_CARDS_MAPPING.get(&unmasked_card).unwrap();
        a_card.push(opened_card);

        reveal_proofs.insert(
            card,
            (
                vec![
                    (
                        EncodingCard(reveal_card_c),
                        reveal_proof_c,
                        charlie.get_public_key(),
                    ),
                    (
                        EncodingCard(reveal_card_b),
                        reveal_proof_b,
                        bob.get_public_key(),
                    ),
                ],
                (
                    EncodingCard(reveal_card_a),
                    reveal_proof_a,
                    alice.get_public_key(),
                ),
            ),
        );
    }

    for card in bob_deck.iter() {
        let (reveal_card_c, reveal_proof_c) = reveal(&mut rng, &charlie_z, card).unwrap();
        let (reveal_card_a, reveal_proof_a) = reveal(&mut rng, &alice_z, card).unwrap();
        let (reveal_card_b, reveal_proof_b) = reveal(&mut rng, &bob_z, card).unwrap();
        verify_reveal(&bob_z.public, card, &reveal_card_b, &reveal_proof_b).unwrap();
        verify_reveal(&charlie_z.public, card, &reveal_card_c, &reveal_proof_c).unwrap();
        verify_reveal(&alice_z.public, card, &reveal_card_a, &reveal_proof_a).unwrap();

        let reveals = vec![reveal_card_b, reveal_card_a, reveal_card_c];
        let unmasked_card = unmask(card, &reveals).unwrap();

        let opened_card = ENCODING_CARDS_MAPPING.get(&unmasked_card).unwrap();
        b_card.push(opened_card);

        reveal_proofs.insert(
            card,
            (
                vec![
                    (
                        EncodingCard(reveal_card_c),
                        reveal_proof_c,
                        charlie.get_public_key(),
                    ),
                    (
                        EncodingCard(reveal_card_a),
                        reveal_proof_a,
                        alice.get_public_key(),
                    ),
                ],
                (
                    EncodingCard(reveal_card_b),
                    reveal_proof_b,
                    bob.get_public_key(),
                ),
            ),
        );
    }

    for card in charlie_deck.iter() {
        let (reveal_card_c, reveal_proof_c) = reveal(&mut rng, &charlie_z, card).unwrap();
        let (reveal_card_a, reveal_proof_a) = reveal(&mut rng, &alice_z, card).unwrap();
        let (reveal_card_b, reveal_proof_b) = reveal(&mut rng, &bob_z, card).unwrap();
        verify_reveal(&bob_z.public, card, &reveal_card_b, &reveal_proof_b).unwrap();
        verify_reveal(&charlie_z.public, card, &reveal_card_c, &reveal_proof_c).unwrap();
        verify_reveal(&alice_z.public, card, &reveal_card_a, &reveal_proof_a).unwrap();

        let reveals = vec![reveal_card_b, reveal_card_a, reveal_card_c];
        let unmasked_card = unmask(card, &reveals).unwrap();

        let opened_card = ENCODING_CARDS_MAPPING.get(&unmasked_card).unwrap();
        c_card.push(opened_card);

        reveal_proofs.insert(
            card,
            (
                vec![
                    (
                        EncodingCard(reveal_card_b),
                        reveal_proof_b,
                        bob.get_public_key(),
                    ),
                    (
                        EncodingCard(reveal_card_a),
                        reveal_proof_a,
                        alice.get_public_key(),
                    ),
                ],
                (
                    EncodingCard(reveal_card_c),
                    reveal_proof_c,
                    charlie.get_public_key(),
                ),
            ),
        );
    }

    let players_order = vec![
        alice.get_public_key(),
        bob.get_public_key(),
        charlie.get_public_key(),
    ];

    let mut players_hand = HashMap::new();
    players_hand.insert(
        alice.get_public_key(),
        alice_deck.iter().map(|x| CryptoCard(*x)).collect(),
    );
    players_hand.insert(
        bob.get_public_key(),
        bob_deck.iter().map(|x| CryptoCard(*x)).collect(),
    );
    players_hand.insert(
        charlie.get_public_key(),
        charlie_deck.iter().map(|x| CryptoCard(*x)).collect(),
    );

    //  ---------------round 0--------------------
    let alice_play_0_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(0)
        .turn_id(0)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::ThreeWithOne(
            CryptoCard(alice_deck[8]),
            CryptoCard(alice_deck[12]),
            CryptoCard(alice_deck[14]),
            CryptoCard(alice_deck[4]),
        )))
        .others_reveal(&[
            reveal_proofs.get(&alice_deck[8]).unwrap().0.clone(),
            reveal_proofs.get(&alice_deck[12]).unwrap().0.clone(),
            reveal_proofs.get(&alice_deck[14]).unwrap().0.clone(),
            reveal_proofs.get(&alice_deck[4]).unwrap().0.clone(),
        ])
        .owner_reveal(&[
            reveal_proofs.get(&alice_deck[8]).unwrap().1.clone(),
            reveal_proofs.get(&alice_deck[12]).unwrap().1.clone(),
            reveal_proofs.get(&alice_deck[14]).unwrap().1.clone(),
            reveal_proofs.get(&alice_deck[4]).unwrap().1.clone(),
        ])
        .build_and_sign(&alice, &mut rng)
        .unwrap();

    let bob_play_0_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(0)
        .turn_id(0)
        .action(PlayAction::PAAS)
        .build_and_sign(&bob, &mut rng)
        .unwrap();

    let charlie_play_0_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(0)
        .turn_id(0)
        .action(PlayAction::PAAS)
        .build_and_sign(&charlie, &mut rng)
        .unwrap();

    let round_0 = vec![alice_play_0_0, bob_play_0_0, charlie_play_0_0];

    //  ---------------round 1--------------------
    let alice_play_1_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(1)
        .turn_id(0)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::Pair(
            CryptoCard(alice_deck[6]),
            CryptoCard(alice_deck[17]),
        )))
        .others_reveal(&[
            reveal_proofs.get(&alice_deck[6]).unwrap().0.clone(),
            reveal_proofs.get(&alice_deck[17]).unwrap().0.clone(),
        ])
        .owner_reveal(&[
            reveal_proofs.get(&alice_deck[6]).unwrap().1.clone(),
            reveal_proofs.get(&alice_deck[17]).unwrap().1.clone(),
        ])
        .build_and_sign(&alice, &mut rng)
        .unwrap();

    let bob_play_1_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(1)
        .turn_id(0)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::Pair(
            CryptoCard(bob_deck[0]),
            CryptoCard(bob_deck[10]),
        )))
        .others_reveal(&[
            reveal_proofs.get(&bob_deck[0]).unwrap().0.clone(),
            reveal_proofs.get(&bob_deck[10]).unwrap().0.clone(),
        ])
        .owner_reveal(&[
            reveal_proofs.get(&bob_deck[0]).unwrap().1.clone(),
            reveal_proofs.get(&bob_deck[10]).unwrap().1.clone(),
        ])
        .build_and_sign(&bob, &mut rng)
        .unwrap();

    let charlie_play_1_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(1)
        .turn_id(0)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::Pair(
            CryptoCard(charlie_deck[2]),
            CryptoCard(charlie_deck[11]),
        )))
        .others_reveal(&[
            reveal_proofs.get(&charlie_deck[2]).unwrap().0.clone(),
            reveal_proofs.get(&charlie_deck[11]).unwrap().0.clone(),
        ])
        .owner_reveal(&[
            reveal_proofs.get(&charlie_deck[2]).unwrap().1.clone(),
            reveal_proofs.get(&charlie_deck[11]).unwrap().1.clone(),
        ])
        .build_and_sign(&charlie, &mut rng)
        .unwrap();

    let alice_play_1_1 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(1)
        .turn_id(1)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::Pair(
            CryptoCard(alice_deck[7]),
            CryptoCard(alice_deck[13]),
        )))
        .others_reveal(&[
            reveal_proofs.get(&alice_deck[7]).unwrap().0.clone(),
            reveal_proofs.get(&alice_deck[13]).unwrap().0.clone(),
        ])
        .owner_reveal(&[
            reveal_proofs.get(&alice_deck[7]).unwrap().1.clone(),
            reveal_proofs.get(&alice_deck[13]).unwrap().1.clone(),
        ])
        .build_and_sign(&alice, &mut rng)
        .unwrap();

    let bob_play_1_1 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(1)
        .turn_id(1)
        .action(PlayAction::PAAS)
        .build_and_sign(&bob, &mut rng)
        .unwrap();

    let charlie_play_1_1 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(1)
        .turn_id(1)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::Pair(
            CryptoCard(charlie_deck[5]),
            CryptoCard(charlie_deck[13]),
        )))
        .others_reveal(&[
            reveal_proofs.get(&charlie_deck[5]).unwrap().0.clone(),
            reveal_proofs.get(&charlie_deck[13]).unwrap().0.clone(),
        ])
        .owner_reveal(&[
            reveal_proofs.get(&charlie_deck[5]).unwrap().1.clone(),
            reveal_proofs.get(&charlie_deck[13]).unwrap().1.clone(),
        ])
        .build_and_sign(&charlie, &mut rng)
        .unwrap();

    let alice_play_1_2 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(1)
        .turn_id(2)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::Pair(
            CryptoCard(alice_deck[5]),
            CryptoCard(alice_deck[11]),
        )))
        .others_reveal(&[
            reveal_proofs.get(&alice_deck[5]).unwrap().0.clone(),
            reveal_proofs.get(&alice_deck[11]).unwrap().0.clone(),
        ])
        .owner_reveal(&[
            reveal_proofs.get(&alice_deck[5]).unwrap().1.clone(),
            reveal_proofs.get(&alice_deck[11]).unwrap().1.clone(),
        ])
        .build_and_sign(&alice, &mut rng)
        .unwrap();

    let bob_play_1_2 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(1)
        .turn_id(2)
        .action(PlayAction::PAAS)
        .build_and_sign(&bob, &mut rng)
        .unwrap();

    let charlie_play_1_2 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(1)
        .turn_id(2)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::Pair(
            CryptoCard(charlie_deck[3]),
            CryptoCard(charlie_deck[6]),
        )))
        .others_reveal(&[
            reveal_proofs.get(&charlie_deck[3]).unwrap().0.clone(),
            reveal_proofs.get(&charlie_deck[6]).unwrap().0.clone(),
        ])
        .owner_reveal(&[
            reveal_proofs.get(&charlie_deck[3]).unwrap().1.clone(),
            reveal_proofs.get(&charlie_deck[6]).unwrap().1.clone(),
        ])
        .build_and_sign(&charlie, &mut rng)
        .unwrap();

    let alice_play_1_3 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(1)
        .turn_id(3)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::Pair(
            CryptoCard(alice_deck[2]),
            CryptoCard(alice_deck[3]),
        )))
        .others_reveal(&[
            reveal_proofs.get(&alice_deck[2]).unwrap().0.clone(),
            reveal_proofs.get(&alice_deck[3]).unwrap().0.clone(),
        ])
        .owner_reveal(&[
            reveal_proofs.get(&alice_deck[2]).unwrap().1.clone(),
            reveal_proofs.get(&alice_deck[3]).unwrap().1.clone(),
        ])
        .build_and_sign(&alice, &mut rng)
        .unwrap();

    let bob_play_1_3 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(1)
        .turn_id(3)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::Pair(
            CryptoCard(bob_deck[6]),
            CryptoCard(bob_deck[8]),
        )))
        .others_reveal(&[
            reveal_proofs.get(&bob_deck[6]).unwrap().0.clone(),
            reveal_proofs.get(&bob_deck[8]).unwrap().0.clone(),
        ])
        .owner_reveal(&[
            reveal_proofs.get(&bob_deck[6]).unwrap().1.clone(),
            reveal_proofs.get(&bob_deck[8]).unwrap().1.clone(),
        ])
        .build_and_sign(&bob, &mut rng)
        .unwrap();

    let charlie_play_1_3 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(1)
        .turn_id(3)
        .action(PlayAction::PAAS)
        .build_and_sign(&charlie, &mut rng)
        .unwrap();

    let alice_play_1_4 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(1)
        .turn_id(4)
        .action(PlayAction::PAAS)
        .build_and_sign(&alice, &mut rng)
        .unwrap();

    let round_1 = vec![
        alice_play_1_0,
        bob_play_1_0,
        charlie_play_1_0,
        alice_play_1_1,
        bob_play_1_1,
        charlie_play_1_1,
        alice_play_1_2,
        bob_play_1_2,
        charlie_play_1_2,
        alice_play_1_3,
        bob_play_1_3,
        charlie_play_1_3,
        alice_play_1_4,
    ];

    //  ---------------round 2--------------------
    let bob_play_2_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(2)
        .turn_id(0)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::Single(CryptoCard(
            bob_deck[12],
        ))))
        .others_reveal(&[reveal_proofs.get(&bob_deck[12]).unwrap().0.clone()])
        .owner_reveal(&[reveal_proofs.get(&bob_deck[12]).unwrap().1.clone()])
        .build_and_sign(&bob, &mut rng)
        .unwrap();

    let charlie_play_2_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(2)
        .turn_id(0)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::Single(CryptoCard(
            charlie_deck[15],
        ))))
        .others_reveal(&[reveal_proofs.get(&charlie_deck[15]).unwrap().0.clone()])
        .owner_reveal(&[reveal_proofs.get(&charlie_deck[15]).unwrap().1.clone()])
        .build_and_sign(&charlie, &mut rng)
        .unwrap();

    let alice_play_2_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(2)
        .turn_id(0)
        .action(PlayAction::PAAS)
        .build_and_sign(&alice, &mut rng)
        .unwrap();

    let bob_play_2_1 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(2)
        .turn_id(1)
        .action(PlayAction::PAAS)
        .build_and_sign(&bob, &mut rng)
        .unwrap();

    let round_2 = vec![bob_play_2_0, charlie_play_2_0, alice_play_2_0, bob_play_2_1];

    //  ---------------round 3--------------------
    let charlie_play_3_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(3)
        .turn_id(0)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::Straight(vec![
            CryptoCard(charlie_deck[7]),
            CryptoCard(charlie_deck[1]),
            CryptoCard(charlie_deck[10]),
            CryptoCard(charlie_deck[4]),
            CryptoCard(charlie_deck[9]),
        ])))
        .others_reveal(&[
            reveal_proofs.get(&charlie_deck[7]).unwrap().0.clone(),
            reveal_proofs.get(&charlie_deck[1]).unwrap().0.clone(),
            reveal_proofs.get(&charlie_deck[10]).unwrap().0.clone(),
            reveal_proofs.get(&charlie_deck[4]).unwrap().0.clone(),
            reveal_proofs.get(&charlie_deck[9]).unwrap().0.clone(),
        ])
        .owner_reveal(&[
            reveal_proofs.get(&charlie_deck[7]).unwrap().1.clone(),
            reveal_proofs.get(&charlie_deck[1]).unwrap().1.clone(),
            reveal_proofs.get(&charlie_deck[10]).unwrap().1.clone(),
            reveal_proofs.get(&charlie_deck[4]).unwrap().1.clone(),
            reveal_proofs.get(&charlie_deck[9]).unwrap().1.clone(),
        ])
        .build_and_sign(&charlie, &mut rng)
        .unwrap();

    let alice_play_3_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(3)
        .turn_id(0)
        .action(PlayAction::PAAS)
        .build_and_sign(&alice, &mut rng)
        .unwrap();

    let bob_play_3_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(3)
        .turn_id(0)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::Straight(vec![
            CryptoCard(bob_deck[5]),
            CryptoCard(bob_deck[16]),
            CryptoCard(bob_deck[15]),
            CryptoCard(bob_deck[1]),
            CryptoCard(bob_deck[2]),
        ])))
        .others_reveal(&[
            reveal_proofs.get(&bob_deck[5]).unwrap().0.clone(),
            reveal_proofs.get(&bob_deck[16]).unwrap().0.clone(),
            reveal_proofs.get(&bob_deck[15]).unwrap().0.clone(),
            reveal_proofs.get(&bob_deck[1]).unwrap().0.clone(),
            reveal_proofs.get(&bob_deck[2]).unwrap().0.clone(),
        ])
        .owner_reveal(&[
            reveal_proofs.get(&bob_deck[5]).unwrap().1.clone(),
            reveal_proofs.get(&bob_deck[16]).unwrap().1.clone(),
            reveal_proofs.get(&bob_deck[15]).unwrap().1.clone(),
            reveal_proofs.get(&bob_deck[1]).unwrap().1.clone(),
            reveal_proofs.get(&bob_deck[2]).unwrap().1.clone(),
        ])
        .build_and_sign(&bob, &mut rng)
        .unwrap();

    let charlie_play_3_1 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(3)
        .turn_id(1)
        .action(PlayAction::PAAS)
        .build_and_sign(&charlie, &mut rng)
        .unwrap();

    let alice_play_3_1 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(3)
        .turn_id(1)
        .action(PlayAction::PAAS)
        .build_and_sign(&alice, &mut rng)
        .unwrap();

    let round_3 = vec![
        charlie_play_3_0,
        alice_play_3_0,
        bob_play_3_0,
        charlie_play_3_1,
        alice_play_3_1,
    ];

    //  ---------------round 4--------------------
    let bob_play_4_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(4)
        .turn_id(0)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::FourWithTwoSingle(
            CryptoCard(bob_deck[3]),
            CryptoCard(bob_deck[13]),
            CryptoCard(bob_deck[14]),
            CryptoCard(bob_deck[11]),
            CryptoCard(bob_deck[4]),
            CryptoCard(bob_deck[7]),
        )))
        .others_reveal(&[
            reveal_proofs.get(&bob_deck[3]).unwrap().0.clone(),
            reveal_proofs.get(&bob_deck[13]).unwrap().0.clone(),
            reveal_proofs.get(&bob_deck[14]).unwrap().0.clone(),
            reveal_proofs.get(&bob_deck[11]).unwrap().0.clone(),
            reveal_proofs.get(&bob_deck[4]).unwrap().0.clone(),
            reveal_proofs.get(&bob_deck[7]).unwrap().0.clone(),
        ])
        .owner_reveal(&[
            reveal_proofs.get(&bob_deck[3]).unwrap().1.clone(),
            reveal_proofs.get(&bob_deck[13]).unwrap().1.clone(),
            reveal_proofs.get(&bob_deck[14]).unwrap().1.clone(),
            reveal_proofs.get(&bob_deck[11]).unwrap().1.clone(),
            reveal_proofs.get(&bob_deck[4]).unwrap().1.clone(),
            reveal_proofs.get(&bob_deck[7]).unwrap().1.clone(),
        ])
        .build_and_sign(&bob, &mut rng)
        .unwrap();

    let charlie_play_4_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(4)
        .turn_id(0)
        .action(PlayAction::PAAS)
        .build_and_sign(&charlie, &mut rng)
        .unwrap();

    let alice_play_4_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(4)
        .turn_id(0)
        .action(PlayAction::PAAS)
        .build_and_sign(&alice, &mut rng)
        .unwrap();

    let round_4 = vec![bob_play_4_0, charlie_play_4_0, alice_play_4_0];

    //  ---------------round 5--------------------

    let bob_play_5_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(5)
        .turn_id(0)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::Single(CryptoCard(bob_deck[9]))))
        .others_reveal(&[reveal_proofs.get(&bob_deck[9]).unwrap().0.clone()])
        .owner_reveal(&[reveal_proofs.get(&bob_deck[9]).unwrap().1.clone()])
        .build_and_sign(&bob, &mut rng)
        .unwrap();

    let charlie_play_5_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(5)
        .turn_id(0)
        .action(PlayAction::PAAS)
        .build_and_sign(&charlie, &mut rng)
        .unwrap();

    let alice_play_5_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(5)
        .turn_id(0)
        .action(PlayAction::PAAS)
        .build_and_sign(&alice, &mut rng)
        .unwrap();

    let round_5 = vec![bob_play_5_0, charlie_play_5_0, alice_play_5_0];

    let players_env = vec![round_0, round_1, round_2, round_3, round_4, round_5];

    let task = Task {
        room_id: 1,
        num_round: 6,
        players_order,
        players_env,
        players_hand,
    };

    task
}
