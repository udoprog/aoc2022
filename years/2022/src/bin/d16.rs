use core::fmt;

use lib::prelude::*;

const GRID: usize = (b'Z' - b'A') as usize + 1;
const NEIGH: usize = 5;
const STACK: usize = 1024;
const NODES: usize = 64;
const ACTIONS: usize = 12;

#[derive(Debug, Default, Clone, Copy)]
struct World {
    /// Valves which are open.
    valves: u128,
    /// Current per-minute flow.
    flow: u16,
    /// Current accumulated flow.
    flowed: u16,
    /// Minutes consumed in this world.
    minute: u8,
}

#[entry(input = "d16.txt", expect = (1850, 2306))]
fn main(mut input: IStr) -> Result<(u16, u16)> {
    let mut grid = [Node::EMPTY; { GRID * GRID }];
    let mut grid = grid.as_grid_mut(GRID);

    let mut index = 0;

    while let Some(Split((mut a, mut b))) = input.try_line::<Split<';', (IStr, IStr)>>()? {
        let (_, W(pos), _, _, _, Split((_, flow))) =
            a.next::<(W, W<Point>, W, W, Ws, Split<'=', (Skip, u16)>)>()?;
        let (_, _, Split2(neigh)) =
            b.next::<([W; 4], Ws, Split2<',', ' ', ArrayVec<Point, NEIGH>>)>()?;

        *grid.get_mut(pos.y(), pos.x()) = Node { index, flow, neigh };
        index += 1;
    }

    let part1 = solve::<_, 30>(index, &mut grid)?;
    let part2 = solve2::<_, 26>(index, &mut grid)?;
    Ok((part1, part2))
}

fn solve<G, const LIMIT: usize>(len: u32, grid: &mut G) -> Result<u16>
where
    G: Grid<Node>,
{
    let mut solution = 0;

    let mut best = ArrayVec::<[Option<u16>; LIMIT], NODES>::new();
    let mut buf = ArrayVec::<_, ACTIONS>::new();

    for _ in 0..len {
        best.try_push([None; LIMIT]).context("out of node memory")?;
    }

    let mut stack = ArrayVec::<(Point, World), STACK>::new();
    stack.try_push((Point::new(0, 0), World::default()))?;

    while let Some((p, mut w)) = stack.pop() {
        let node = grid.try_get(p.y(), p.x()).context("missing node")?;

        if w.minute > 0 {
            let best = &mut best[node.index as usize][(w.minute - 1) as usize];

            *best = match *best {
                Some(flowed) if flowed < w.flowed => Some(w.flowed),
                None => Some(w.flowed),
                _ => continue,
            };
        }

        solution = solution.max(w.flowed);

        if w.minute as usize >= LIMIT {
            continue;
        }

        w.minute += 1;
        w.flowed += w.flow;

        for &(p, w) in actions(p, node, w, &mut buf)? {
            stack.try_push((p, w))?;
        }
    }

    Ok(solution)
}

fn solve2<G, const LIMIT: usize>(len: u32, grid: &mut G) -> Result<u16>
where
    G: Grid<Node>,
{
    let mut solution = 0;

    let mut best = ArrayVec::<[[Option<u16>; LIMIT]; NODES], NODES>::new();

    for _ in 0..len {
        best.try_push([[None; LIMIT]; NODES])
            .context("out of node memory")?;
    }

    let mut stack = ArrayVec::<(Point, Point, World), STACK>::new();
    stack.try_push((Point::new(0, 0), Point::new(0, 0), World::default()))?;

    let mut buf = ArrayVec::<_, ACTIONS>::new();
    let mut buf2 = ArrayVec::<_, ACTIONS>::new();

    while let Some((a, b, mut w)) = stack.pop() {
        let na = grid.try_get(a.y(), a.x()).context("missing node")?;
        let nb = grid.try_get(b.y(), b.x()).context("missing node")?;

        if w.minute > 0 {
            let best = &mut best[na.index as usize][nb.index as usize][(w.minute - 1) as usize];

            *best = match *best {
                Some(flowed) if flowed < w.flowed => Some(w.flowed),
                None => Some(w.flowed),
                _ => continue,
            };
        }

        solution = solution.max(w.flowed);

        if w.minute as usize >= LIMIT {
            continue;
        }

        w.minute += 1;
        w.flowed += w.flow;

        for &(a, w) in actions(a, na, w, &mut buf)? {
            for &(b, w) in actions(b, nb, w, &mut buf2)? {
                stack.try_push((a, b, w))?;
            }
        }
    }

    Ok(solution)
}

/// Generate a list of possible actions a node can decide to do.
fn actions<'a, const N: usize>(
    p: Point,
    node: &Node,
    world: World,
    buf: &'a mut ArrayVec<(Point, World), N>,
) -> Result<impl Iterator<Item = &'a (Point, World)>> {
    buf.clear();

    if !world.valves.test_bit(node.index) && node.flow != 0 {
        let mut copy = world;
        copy.valves.set_bit(node.index);
        copy.flow += node.flow;
        buf.try_push((p, copy))?;
    }

    for &n in &node.neigh {
        buf.try_push((n, world))?;
    }

    Ok(buf.iter())
}

struct Node {
    index: u32,
    flow: u16,
    neigh: ArrayVec<Point, NEIGH>,
}

impl Node {
    const EMPTY: Self = Self {
        index: u32::MAX,
        flow: 0,
        neigh: ArrayVec::new_const(),
    };
}

#[derive(Clone, Copy, PartialEq, Eq)]
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
