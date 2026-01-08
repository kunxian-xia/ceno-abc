use ceno_cli::sdk::CenoSDK;
use ceno_common::fib;
use ceno_emul::{Platform, Program};
use ceno_host::CenoStdin;
use ceno_zkvm::e2e::{MultiProver, run_e2e_verify};
use ceno_zkvm::e2e::{Preset, setup_platform};
use ff_ext::BabyBearExt4;
use mpcs::SecurityLevel::Conjecture100bits;
use mpcs::{Basefold, BasefoldRSParams};
use openvm_native_circuit::NativeConfig;
use openvm_stark_sdk::config::baby_bear_poseidon2::BabyBearPoseidon2Config;
use std::path::PathBuf;
use std::sync::LazyLock;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

mod utils;

use utils::discover_workspace_root;

pub const MAX_CYCLE_PER_SHARD: u64 = 1 << 29;
type Pcs = Basefold<BabyBearExt4, BasefoldRSParams>;

static WORKSPACE_ROOT: LazyLock<PathBuf> = LazyLock::new(discover_workspace_root);

fn setup() -> (Vec<u8>, Program, Platform) {
    let stack_size = 128 * 1024 * 1024;
    let heap_size = 128 * 1024 * 1024;
    // public io is [u8; 32] and will be serialized to [u32; 32] + some meta data
    // so the overall we need >= 256 bytes
    let pub_io_size_in_byte = 512;
    println!(
        "stack_size: {stack_size:#x}, heap_size: {heap_size:#x}, pub_io_size: {pub_io_size_in_byte:#x}"
    );

    println!("workspace root: {}", WORKSPACE_ROOT.display());

    let elf_path = WORKSPACE_ROOT
        .join("..")
        .join("program")
        .join("target")
        .join("riscv32im-ceno-zkvm-elf")
        .join("release")
        .join("ceno-guest");
    let elf = std::fs::read(elf_path).unwrap();
    let program = Program::load_elf(&elf, u32::MAX).unwrap();
    let platform = setup_platform(
        Preset::Ceno,
        &program,
        stack_size,
        heap_size,
        pub_io_size_in_byte,
    );
    (elf, program, platform)
}

fn main() {
    let _ = tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .try_init();
    let (_, program, platform) = setup();

    // prove that fib(n) = y
    let n = 10000u32;
    let fib_n = fib(n);

    let pub_io = vec![n, fib_n];

    let mut hint_in = CenoStdin::default();
    let mut pub_io_in = CenoStdin::default();

    hint_in.write(&n).expect("write hint failed");
    pub_io_in.write(&pub_io).expect("write pub io failed");

    let mut ceno_sdk: CenoSDK<BabyBearExt4, Pcs, BabyBearPoseidon2Config, NativeConfig> =
        CenoSDK::new_with_app_config(
            program,
            platform,
            MultiProver::new(0, 1, (1 << 30) * 8 / 4 / 2, MAX_CYCLE_PER_SHARD),
        );

    // initialize the app prover
    let max_num_variables = 25;
    let security_level = Conjecture100bits;
    ceno_sdk.init_base_prover(max_num_variables, security_level);

    let max_steps = usize::MAX;
    let app_proofs = ceno_sdk.generate_base_proof(hint_in, pub_io_in, max_steps, None);
    assert_eq!(app_proofs.len(), 1);

    let app_verifier = ceno_sdk.create_zkvm_verifier();
    run_e2e_verify(&app_verifier, app_proofs, Some(0), max_steps);
}
