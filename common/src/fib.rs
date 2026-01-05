// Compute the n-th fibonacci number.
// This function is shared among guest and host.
pub fn fib(n: u32) -> u32 {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        let c = (a + b) % 7919; // Modulus to prevent overflow.
        a = b;
        b = c;
    }
    b
}
