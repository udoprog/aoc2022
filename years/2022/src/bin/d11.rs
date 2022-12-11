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
        let (_, Split(items)) = input.line::<([W; 2], Split<',', ArrayRingBuffer<_, QUEUE>>)>()?;
        let (_, op, operand, _, div, _, if_true, _, if_false) =
            input.next::<([W; 4], _, _, [W; 3], _, [W; 5], _, [W; 5], _)>()?;

        factors *= div;

        monkeys.push(Monkey {
            items,
            op,
            operand,
            div,
            conds: [if_false, if_true],
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
                    let Monkey {
                        op,
                        operand,
                        div,
                        conds,
                        ..
                    } = monkeys[n];

                    let operand = match operand {
                        Operand::Old => item,
                        Operand::Value(n) => n,
                    };

                    let result = match op {
                        Op::Mul => item.checked_mul(operand),
                        Op::Add => item.checked_add(operand),
                    };

                    let Some(mut item) = result else {
                        anyhow::bail!("{item} {op} {operand}: overflow");
                    };

                    item = (item % factors) / stress;
                    let t = conds[usize::from(item % div == 0)];
                    let to = monkeys.get_mut(t).context("missing monkey")?;
                    to.items.push(item);
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
    conds: [usize; 2],
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

#[derive(Debug, Clone, Copy)]
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
