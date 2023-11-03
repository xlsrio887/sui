module 0x42::m {

    public enum ABC<T> has drop {
        A(T),
        B,
        C(T)
    }

    fun fib(x: u64): u64 {
        match (x) {
            0 => 1,
            1 => 1,
            x => fib(x-1) + fib(x-2),
        }
    }

    // fun t0(): u64 {
    //     match (ABC::C(0)) {
    //         ABC::C(x) => x,
    //         ABC::A(x) => x,
    //         ABC::B => 1,
    //     }
    // }

}
