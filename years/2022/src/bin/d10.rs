use lib::prelude::*;

#[entry(input = "d10.txt", expect = (14420, "RGLRBZAU"))]
fn main(mut input: IStr) -> Result<(i32, ArrayString<8>)> {
    let mut x = 1i32;
    let mut cycle = 1i32;
    let mut part1 = 0;

    let mut screen = [b'.'; 40 * 6];

    while let Some((W(line), arg)) = input.try_line::<(W<&str>, Option<i32>)>()? {
        let ops = match (line, arg) {
            ("noop", _) => [Some(0), None],
            ("addx", Some(n)) => [Some(0), Some(n)],
            (other, _) => {
                bail!(other)
            }
        };

        for op in ops.into_iter().flatten() {
            let index = cycle - 1;

            if (index % 40 - x).unsigned_abs() < 2 {
                screen[index as usize] = b'#';
            }

            cycle += 1;
            x += op;

            if (20..=220).contains(&cycle) && (cycle - 20) % 40 == 0 {
                part1 += cycle * x;
            }
        }
    }

    let screen = screen.as_grid(40);

    if cfg!(aoc_print) {
        for row in screen.rows() {
            for b in row {
                print!("{}", *b as char);
            }

            println!();
        }
    }

    let output = read_lcd(screen)?;
    Ok((part1, output))
}

const A: &[u8] = b".##.#..##..######..##..#";
const B: &[u8] = b"###.#..####.#..##..####.";
const C: &[u8] = b".##.#..##...#...#..#.##.";
const D: &[u8] = b"###.#..##..##..##..####.";
const E: &[u8] = b"#####...###.#...#...####";
const F: &[u8] = b"#####...###.#...#...#...";
const G: &[u8] = b".##.#..##...#.###..#.###";
const H: &[u8] = b"#..##..##..######..##..#";
const I: &[u8] = b"#...#...#...#...#...#...";
const J: &[u8] = b"..##...#...#...##..#.##.";
const K: &[u8] = b"#..##.#.##..#.#.#.#.#..#";
const L: &[u8] = b"#...#...#...#...#...####";
const O: &[u8] = b".##.#..##..##..##..#.##.";
const P: &[u8] = b"###.#..##..####.#...#...";
const R: &[u8] = b"###.#..##..####.#.#.#..#";
const T: &[u8] = b"####.#...#...#...#...#..";
const U: &[u8] = b"#..##..##..##..##..#.##.";
const Z: &[u8] = b"####...#..#..#..#...####";

// utility function to read the LCR screen.
fn read_lcd<G>(grid: G) -> Result<ArrayString<8>>
where
    G: Grid<u8>,
{
    let width = grid.columns_len();

    let mut output = ArrayString::new();
    let mut line = ArrayVec::<u8, 24>::new();

    for start in (0..width).step_by(5) {
        line.clear();

        for row in grid.rows() {
            for b in row.into_iter().skip(start).take(4) {
                line.try_push(*b)?;
            }
        }

        let c = match line.as_ref() {
            A => 'A',
            B => 'B',
            C => 'C',
            D => 'D',
            E => 'E',
            F => 'F',
            G => 'G',
            H => 'H',
            I => 'I',
            J => 'J',
            K => 'K',
            L => 'L',
            O => 'O',
            P => 'P',
            R => 'R',
            T => 'T',
            U => 'U',
            Z => 'Z',
            other => {
                let other = BStr::new(other);
                anyhow::bail!("unknown char: {other:?}");
            }
        };

        output.try_push(c)?;
    }

    Ok(output)
}
