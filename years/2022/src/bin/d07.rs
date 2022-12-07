use lib::prelude::*;

#[entry(input = "d07.txt", expect = (1444896, 404395))]
fn main(mut input: IStr) -> Result<(u64, u64)> {
    let mut part1 = 0;
    let mut part2 = u64::MAX;

    let mut cd = ArrayString::<128>::new();

    let mut sizes = HashMap::<_, u64>::new();
    let mut total = 0;

    while let Some(line) = input.try_line::<&str>()? {
        if line.starts_with('$') {
            let (_, command) = line.split_once(' ').context("missing sp")?;

            match command {
                "ls" => {
                    continue;
                }
                _ => {}
            }

            let (name, arg) = command.split_once(' ').context("missing args")?;

            match name {
                "cd" => match arg {
                    "/" => {
                        cd.clear();
                    }
                    ".." => match cd.rfind('/') {
                        Some(n) => {
                            cd.truncate(n);
                        }
                        None => {
                            cd.clear();
                        }
                    },
                    _ => {
                        cd.try_push('/')?;
                        cd.try_push_str(arg)?;
                    }
                },
                name => {
                    panic!("{name}");
                }
            }

            continue;
        }

        let (prefix, _) = line.split_once(' ').context("missing ls parts")?;

        match prefix {
            "dir" => {}
            n => {
                let size = n.parse::<u64>()?;
                total += size;

                let mut cur = Some(cd.as_str());

                while let Some(d) = cur.take() {
                    *sizes.entry(d.to_owned()).or_default() += size;
                    cur = d.rfind('/').and_then(|n| d.get(..n));
                }
            }
        }
    }

    let rem = 70000000 - total;
    let needed = 30000000 - rem;

    for (_, &size) in &sizes {
        if size < 100000 {
            part1 += size;
        }

        if size >= needed {
            part2 = part2.min(size);
        }
    }

    Ok((part1, part2))
}
