use core::fmt;
use std::num::NonZeroU8;

use lib::prelude::*;

const GRID: usize = (b'Z' - b'A') as usize + 1;
const NEIGH: usize = 5;
const STACK: usize = 512;
const NODES: usize = 64;
const ACTIONS: usize = 12;

#[entry(input = "d16.txt", expect = (1850, 2306))]
fn main(mut input: IStr) -> Result<(u16, u16)> {
    let mut grid = [Node::EMPTY; NODES];

    let mut alloc = Alloc::default();

    while let Some(Split((mut a, mut b))) = input.line::<Option<Split<';', (IStr, IStr)>>>()? {
        let (_, W(pos), _, _, _, Split((_, flow))) =
            a.next::<(W, W<Point>, W, W, Ws, Split<'=', (Skip, u16)>)>()?;
        let (_, _, Split2(points)) =
            b.next::<([W; 4], Ws, Split2<',', ' ', ArrayVec<Point, NEIGH>>)>()?;

        let mut neigh = ArrayVec::new();
        let index = alloc.alloc(pos).context("ran out of ids")?;

        for p in points {
            neigh.push(alloc.alloc(p).context("ran out of ids")?);
        }

        grid[index as usize] = Node { index, flow, neigh };
    }

    let start = alloc.alloc(Point::new(0, 0)).context("missing start")?;
    let part1 = solve::<30>(&grid, start)?;
    let part2 = solve2::<26>(&grid, start)?;
    Ok((part1, part2))
}

fn solve<const LIMIT: usize>(grid: &[Node], start: u8) -> Result<u16> {
    let mut best = [[None; LIMIT]; NODES];
    let mut buf = ArrayVec::<_, ACTIONS>::new();
    let mut stack = ArrayVec::<(u8, World), STACK>::new();
    stack.try_push((start, World::default()))?;

    let mut solution = 0;

    while let Some((p, mut w)) = stack.pop() {
        w.minute += 1;
        w.flowed += w.flow;
        solution = solution.max(w.flowed);

        if w.minute as usize >= LIMIT {
            continue;
        }

        let node = grid.get(p as usize).context("missing node")?;

        for &(p, w) in actions(p, node, w, &mut buf)? {
            let best = &mut best[p as usize][(w.minute - 1) as usize];

            *best = match *best {
                Some(flowed) if flowed < w.flowed => Some(w.flowed),
                None => Some(w.flowed),
                _ => continue,
            };

            stack.try_push((p, w))?;
        }
    }

    Ok(solution)
}

fn solve2<const LIMIT: usize>(grid: &[Node], start: u8) -> Result<u16> {
    let mut best = [[None; LIMIT]; { NODES * NODES }];

    let mut stack = ArrayVec::<(u8, u8, World), STACK>::new();
    stack.try_push((start, start, World::default()))?;

    let mut buf = ArrayVec::<_, ACTIONS>::new();
    let mut buf2 = ArrayVec::<_, ACTIONS>::new();

    let mut solution = 0;

    while let Some((a, b, mut w)) = stack.pop() {
        w.minute += 1;
        w.flowed += w.flow;
        solution = solution.max(w.flowed);

        if w.minute as usize >= LIMIT {
            continue;
        }

        let node_a = grid.get(a as usize).context("missing node a")?;
        let node_b = grid.get(b as usize).context("missing node b")?;

        for &(a, w) in actions(a, node_a, w, &mut buf)? {
            for &(b, w) in actions(b, node_b, w, &mut buf2)? {
                let best = &mut best[a as usize * NODES + b as usize][(w.minute - 1) as usize];

                *best = match *best {
                    Some(flowed) if flowed < w.flowed => Some(w.flowed),
                    None => Some(w.flowed),
                    _ => continue,
                };

                stack.try_push((a, b, w))?;
            }
        }
    }

    Ok(solution)
}

/// Cheaply copyable world state.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
struct World {
    /// Bitset indicating nodes which have open valves.
    valves: u64,
    /// Current per-minute flow.
    flow: u16,
    /// Current accumulated flow.
    flowed: u16,
    /// Minutes consumed in this world.
    minute: u8,
}

/// Generate a list of possible actions a node can decide to do.
fn actions<'a, const N: usize>(
    p: u8,
    node: &Node,
    world: World,
    buf: &'a mut ArrayVec<(u8, World), N>,
) -> Result<impl Iterator<Item = &'a (u8, World)>> {
    buf.clear();

    if !world.valves.test_bit(node.index as u32) && node.flow != 0 {
        let mut copy = world;
        copy.valves.set_bit(node.index as u32);
        copy.flow += node.flow;
        buf.try_push((p, copy))?;
    }

    for &n in &node.neigh {
        buf.try_push((n, world))?;
    }

    Ok(buf.iter())
}

struct Node {
    index: u8,
    flow: u16,
    neigh: ArrayVec<u8, NEIGH>,
}

impl Node {
    const EMPTY: Self = Self {
        index: u8::MAX,
        flow: 0,
        neigh: ArrayVec::new_const(),
    };
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
struct Point {
    // NB: we do some bitbacking because the size of the point makes everything
    // very large.
    data: u16,
}

impl Point {
    #[inline]
    const fn new(x: usize, y: usize) -> Self {
        assert!(x < GRID && y < GRID);
        Self {
            data: u16::from_be_bytes([x as u8, y as u8]),
        }
    }

    #[inline]
    fn x(&self) -> usize {
        self.data.to_be_bytes()[0] as usize
    }

    #[inline]
    fn y(&self) -> usize {
        self.data.to_be_bytes()[1] as usize
    }
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Point")
            .field("x", &self.x())
            .field("y", &self.y())
            .finish()
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [a, b] = self.data.to_be_bytes();
        BStr::new(&[a + b'A', b + b'A']).fmt(f)
    }
}

lib::from_input! {
    |(B(x), B(y)): (B, B)| -> Point {
        Ok(Point::new((x - b'A') as usize, (y - b'A') as usize))
    }
}

/// Helper to allocate IDs coalescing towards zero from positions.
#[derive(Default)]
struct Alloc {
    index: u8,
    grid: [[Option<NonZeroU8>; GRID]; GRID],
}

impl Alloc {
    fn alloc(&mut self, p: Point) -> Option<u8> {
        let a = self.grid.get_mut(p.y())?.get_mut(p.x())?;

        if let Some(value) = *a {
            return Some(u8::MAX ^ value.get());
        }

        let next = self.index;
        let n = NonZeroU8::new(u8::MAX ^ next)?;
        self.index += 1;
        *a = Some(n);
        Some(next)
    }
}
