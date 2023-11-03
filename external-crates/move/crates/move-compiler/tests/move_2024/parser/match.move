module 0x42::m {
    public enum Temperature {
       Fahrenheit(u16),
       Celsius { temp: u16 },
       Unknown
    }

    fun is_temperature_fahrenheit(t: &Temperature): bool {
       match (t) {
          Temperature::Fahrenheit(_) => true,
          _ => false,
       }
    }

    fun is_temperature_boiling(t: &Temperature): bool {
       match (t) {
          Temperature::Fahrenheit(temp) if (temp >= 212) => true,
          Temperature::Celsius { temp } if (temp >= 100) => true,
          _ => false,
       }
    }

    public enum Option<T> {
      Some(T),
      None
    }

    public fun is_some_true_0(o: Option<bool>): bool {
       match (o) {
         Option::Some(true) => true,
         Option::None => false,
       }
    }

    public fun is_some_true_1(o: Option<bool>): bool {
       match (o) {
         Option::Some(true) => true,
         Option::Some(false) => false,
         Option::None => false,
       }
    }

    public fun is_some_true_2(o: Option<bool>): bool {
       match (o) {
         Option::Some(x) => x,
         Option::None => false,
       }
    }

    public fun option_default(o: Option<T>, default: T): Option<T> {
       match (o) {
         x @ Option::Some(_) => x,
         Option::None => Option::Some(default),
       }
    }

    public enum Expression {
       Done,
       Add,
       Mul,
       Num(u64),
    }

    public fun evaluate(expressions: vector<Expression>): u64 {
        use 0x42::m::Expression as E;
        let stack = vector[];
        while (!vector::is_empty(expressions)) {
            match (vector::pop_back(&mut expressions)) {
                E::Done => break,
                E::Add => {
                    let e1 = vector::pop_back(&mut stack);
                    let e2 = vector::pop_back(&mut stack);
                    vector::push_back(&mut stack, e1 + e2);
                },
                E::Mul => {
                    let e1 = vector::pop_back(&mut stack);
                    let e2 = vector::pop_back(&mut stack);
                    vector::push_back(&mut stack, e1 + e2);
                },
                E::Num(number) => {
                    vector::push_back(&mut stack, number);
                }
            }
        };
        let result = vector::pop_back(&mut stack);
        assert!(vector::is_empty(expressions), EInvalidExpression);
        assert!(vector::is_empty(stack), EInvalidExprrsion);
        result
    }

    public fun count_numbers(expressions: vector<Expression>): u64 {
        use 0x42::m::Expression as E;
        let mut n = 0;
        while (!vector::is_empty(expressions)) {
            match (vector::pop_back(&mut expressions)) {
                E::Add | E::Mul => (),
                E::Num(number) => {
                    n = n + 1;
                },
                E::Done => return n,
            }
        };
        n
    }

    public fun count_ops(expressions: vector<Expression>): u64 {
        use 0x42::m::Expression as E;
        let mut n = 0;
        while (!vector::is_empty(expressions)) {
            match (vector::pop_back(&mut expressions)) {
                E::Add | E::Mul => {
                    n = n + 1;
                },
                _ => (),
            }
        };
        n
    }

}
