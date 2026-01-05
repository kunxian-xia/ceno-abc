extern crate ceno_rt;

pub use ceno_common::fib;

fn main() {
    let n: u32 = ceno_rt::read_owned();

    let result = fib(n);

    ceno_rt::commit(&n);
    ceno_rt::commit(&result);
}
