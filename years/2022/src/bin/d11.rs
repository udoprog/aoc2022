use lib::prelude::*;

const QUEUE: usize = 16;

/// The idea is the following:
///
/// Since each branch is only triggered in a specific collection of divisors, we
/// only need to ensure that the factors that make up those divisors are
/// preserved.
///
/// I initially considered prime factors, but since we have addition as one of
/// our operators that wouldn't work, since adding a number would have different
/// effects if the prime factors are `3` (`3 + 2 == 5`) vs `3 ^ 2` (`9 + 2 ==
/// 11`).
///
/// Then we can instead reach for modular arithmetic. Because if `n // 19` then
/// `n % m // 19` if `19` is a factor of `m`.
#[entry(input = "d11.txt", expect = (50830, 14399640002))]
fn main(mut input: IStr) -> Result<(u64, u64)> {
    let mut monkeys = ArrayVec::<Monkey>::new();

    let mut factors = 1;

    while let Some(..) = input.try_next::<(W, W, Ws)>()? {
        let (_, _, Split(items)) = input.line::<(W, W, Split<',', ArrayRingBuffer<_, QUEUE>>)>()?;
        let (_, _, _, _, op, operand) = input.next::<(W, W, W, W, Op, Operand)>()?;
        let (_, _, _, div) = input.next::<(W, W, W, _)>()?;
        let (_, if_true) = input.next::<((W, W, W, W, W), usize)>()?;
        let (_, if_false) = input.next::<((W, W, W, W, W), usize)>()?;

        factors *= div;

        monkeys.push(Monkey {
            items,
            op,
            operand,
            div,
            if_true,
            if_false,
        });
    }

    let solve = |end: usize, stress: u64, monkeys: &mut [Monkey]| -> Result<u64> {
        let mut levels = ArrayVec::<u64>::new();

        for _ in 0..monkeys.len() {
            levels.try_push(0)?;
        }

        for _ in 0..end {
            for n in 0..monkeys.len() {
                while let Some(item) = monkeys[n].items.dequeue() {
                    let operand = match monkeys[n].operand {
                        Operand::Old => item,
                        Operand::Value(n) => n,
                    };

                    let op = monkeys[n].op;

                    let checked_op = match op {
                        Op::Mul => u64::checked_mul,
                        Op::Add => u64::checked_add,
                    };

                    let Some(mut item) = checked_op(item, operand) else {
                        anyhow::bail!("{item} {op} {operand}: overflow");
                    };

                    item = (item % factors) / stress;

                    let t = if item % monkeys[n].div == 0 {
                        monkeys[n].if_true
                    } else {
                        monkeys[n].if_false
                    };

                    monkeys[t].items.push(item);
                    levels[n] += 1;
                }
            }
        }

        levels.sort();
        Ok(levels.into_iter().rev().take(2).product())
    };

    let mut monkeys1 = monkeys.clone();
    let part1 = solve(20, 3, &mut monkeys1)?;
    let part2 = solve(10000, 1, &mut monkeys)?;
    Ok((part1, part2))
}

#[derive(Clone)]
struct Monkey {
    items: ArrayRingBuffer<u64, QUEUE>,
    op: Op,
    operand: Operand,
    div: u64,
    if_true: usize,
    if_false: usize,
}

#[derive(Debug, Clone, Copy)]
enum Op {
    Mul,
    Add,
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Op::Mul => write!(f, "*"),
            Op::Add => write!(f, "+"),
        }
    }
}

lib::from_input! {
    |W(v): W<&'static str>| -> Op {
        Ok(match v {
            "*" => Op::Mul,
            "+" => Op::Add,
            c => bail!(c),
        })
    }
}

#[derive(Debug, Clone)]
enum Operand {
    Value(u64),
    Old,
}

lib::from_input! {
    |W(v): W<&'static str>| -> Operand {
        Ok(match v {
            "old" => Operand::Old,
            index => Operand::Value(index.parse()?),
        })
    }
}
