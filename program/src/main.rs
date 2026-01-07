extern crate ceno_rt;

pub use ceno_common::fib;

fn main() {
    let n: u32 = ceno_rt::read_owned();

    let result = fib(n);

    // public statement = [n, fib(n)]
    let public_io = vec![n, result];
    ceno_rt::commit(&public_io);
}
