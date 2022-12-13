use lib::prelude::*;

const CAP: usize = 256;
const HEAP_CAP: usize = 64;

#[entry(input = "d12.txt", expect = (534, 525))]
fn main(input: IStr) -> Result<(u32, u32)> {
    let mut visited = [0u128; CAP * (CAP / 128)];

    let cols = input.clone().line::<&[u8]>()?.len();
    let grid = input.as_data().as_grid_with_stride(cols, 1);

    anyhow::ensure!(grid.columns_len() < CAP, "grid size out of capacity");
    anyhow::ensure!(grid.rows_len() < CAP, "grid size out of capacity");

    let mut start = (0, 0);
    let mut end = (0, 0);

    for (y, row) in grid.rows().enumerate() {
        for (x, d) in row.into_iter().enumerate() {
            match d {
                b'S' => {
                    start = (x, y);
                }
                b'E' => {
                    end = (x, y);
                }
                _ => {}
            }
        }
    }

    let part1 = solve(&grid, start, |c, e| c > e + 1, |d| d == b'E', &mut visited)?;
    let part2 = solve(&grid, end, |c, e| e > c + 1, |d| d == b'a', &mut visited)?;
    Ok((part1, part2))
}

/// Construct an iterator over neighbours.
#[inline]
fn neigh((x, y): (usize, usize)) -> impl Iterator<Item = (usize, usize)> {
    let mut out = ArrayVec::<_, 4>::new();
    out.extend(x.checked_sub(1).map(|x| (x, y)));
    out.extend(x.checked_add(1).map(|x| (x, y)));
    out.extend(y.checked_sub(1).map(|y| (x, y)));
    out.extend(y.checked_add(1).map(|y| (x, y)));
    out.into_iter()
}

type Element = (u32, (usize, usize), u8);

fn solve<G, C, E>(
    grid: &G,
    start: (usize, usize),
    should_skip: C,
    is_end: E,
    visited: &mut [u128],
) -> Result<u32>
where
    G: Grid<u8>,
    C: Fn(u8, u8) -> bool,
    E: Fn(u8) -> bool,
{
    let mut heap = FixedHeap::<_, HEAP_CAP>::new();

    let comparer = |a: &Element, b: &Element, _: &()| a.0 < b.0;

    visited.clear_bits();

    if let Some(&b) = grid.try_get(start.1, start.0) {
        heap.push((0u32, start, to_cost(b)), &comparer, &());
        visited.set_bit(index(start));
    }

    while let Some((cost, pos, e)) = heap.pop(&comparer, &()) {
        for (pos, &d) in neigh(pos).flat_map(|(x, y)| Some(((x, y), grid.try_get(y, x)?))) {
            let c = to_cost(d);

            if should_skip(c, e) || visited.test_bit(index(pos)) {
                continue;
            }

            visited.set_bit(index(pos));

            if is_end(d) {
                return Ok(cost + 1);
            }

            if heap.push((cost + 1, pos, c), &comparer, &()).is_some() {
                anyhow::bail!("out of heap capacity");
            }
        }
    }

    Err(anyhow::anyhow!("no solution"))
}

/// Coordinates need some massaging to be translated to costs.
#[inline]
fn to_cost(b: u8) -> u8 {
    match b {
        b'S' => b'a',
        b'E' => b'z',
        b => b,
    }
}

/// Translate position to bitset index.
#[inline]
fn index(p: (usize, usize)) -> u32 {
    (p.1 * CAP + p.0) as u32
}
