use std::env;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use cargo_metadata::MetadataCommand;
use ceno_cli::sdk::CenoSDK;
use ceno_common::fib;
use ceno_emul::{Platform, Program};
use ceno_host::CenoStdin;
use ceno_zkvm::e2e::MultiProver;
use ceno_zkvm::e2e::{Preset, setup_platform};
use ff_ext::BabyBearExt4;
use mpcs::SecurityLevel::Conjecture100bits;
use mpcs::{Basefold, BasefoldRSParams, BasefoldSpec};
use openvm_native_circuit::NativeConfig;
use openvm_stark_sdk::config::baby_bear_poseidon2::BabyBearPoseidon2Config;

fn discover_workspace_root() -> PathBuf {
    if let Ok(path) = env::var("WORKSPACE_ROOT") {
        let pb = PathBuf::from(path);
        eprintln!("WORKSPACE_ROOT (env) = {}", pb.display());
        return pb;
    }

    if let Ok(metadata) = MetadataCommand::new().no_deps().exec() {
        let root = metadata.workspace_root.into_std_path_buf();
        eprintln!("WORKSPACE_ROOT (cargo-metadata) = {}", root.display());
        return root;
    }

    if let Ok(exe_path) = env::current_exe() {
        let mut dir = exe_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        loop {
            if dir.join("Cargo.lock").exists() {
                eprintln!("WORKSPACE_ROOT (inferred from exe) = {}", dir.display());
                return dir;
            }
            if !dir.pop() {
                break;
            }
        }
    }

    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    eprintln!("WORKSPACE_ROOT fallback to cwd = {}", cwd.display());
    cwd
}

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

pub const MAX_CYCLE_PER_SHARD: u64 = 1 << 29;

type PCS = Basefold<BabyBearExt4, BasefoldRSParams>;

fn main() {
    let (_, program, platform) = setup();

    // prove that fib(n) = y
    let n = 10u32;
    let fib_n = fib(n);

    let pub_io = vec![n, fib_n];
    let mut hint_in = CenoStdin::default();
    let mut pub_io_in = CenoStdin::default();

    hint_in.write(&n).expect("write hint failed");

    pub_io_in.write(&pub_io).expect("write pub io failed");

    let mut ceno_sdk: CenoSDK<BabyBearExt4, PCS, BabyBearPoseidon2Config, NativeConfig> =
        CenoSDK::new_with_app_config(
            program,
            platform,
            MultiProver::new(0, 1, (1 << 30) * 8 / 4 / 2, MAX_CYCLE_PER_SHARD),
        );

    let max_num_variables = 25;
    let security_level = Conjecture100bits;
    ceno_sdk.init_base_prover(max_num_variables, security_level);

    ceno_sdk.generate_base_proof(hint_in, pub_io_in, usize::MAX, None);
}
