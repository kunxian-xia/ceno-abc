use ceno_common::fib;
use ceno_host::CenoStdin;

fn main() {
    // prove that fib(n) = y
    let n = 10u32;
    let fib_n = fib(n);

    let pub_io = vec![n, fib_n];
    let mut hint_in = CenoStdin::default();
    let mut pub_io_in = CenoStdin::default();

    hint_in
        .write(&n)
        .expect("write hint failed");

    pub_io_in
        .write(&pub_io)
        .expect("write pub io failed");

    let mut ceno_sdk: ceno_sdk::CenoSDK<_, _, BabyBearPoseidon2Config, NativeConfig> =
        ceno_sdk::CenoSDK::new_with_app_config(
            program,
            platform,
            MultiProver::new(0, 1, (1 << 30) * 8 / 4 / 2, MAX_CYCLE_PER_SHARD),
        );

    ceno_sdk.init_base_prover(max_num_variables, security_level);
}
