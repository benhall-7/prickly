use num::{Bounded, Unsigned};

pub fn compliment<I: Bounded + Unsigned + Copy>(value: I, modulo: I) -> I {
    (modulo - (value % modulo)) % modulo
}

pub fn add_mod<I: Bounded + Unsigned + Copy + PartialOrd>(augend: I, addend: I, modulo: I) -> I {
    let augend = augend % modulo;
    let addend = addend % modulo;
    let would_overflow = I::max_value() - addend < augend;
    if would_overflow {
        // if the augend + addend would overflow, their compliments won't
        // this proof is left as an exercise to the reader
        modulo - (compliment(augend, modulo) + compliment(addend, modulo))
    } else {
        (augend + addend) % modulo
    }
}

pub fn sub_mod<I: Bounded + Unsigned + Copy + PartialOrd>(
    minuend: I,
    subtrahend: I,
    modulo: I,
) -> I {
    add_mod(minuend, compliment(subtrahend, modulo), modulo)
}
