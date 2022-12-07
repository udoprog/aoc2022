My own personal overengineered helpers to solve AoC problems in Rust!

<br>

The project is organized in one crate per year under the `years` directory.

* [years/2021](years/2021)
* [years/2022](years/2022)

Each crate has an `inputs` folder, which is where the tool will look for inputs.

<br>

## How to run solutions

Of course each year can be run independently. Each solution is organized as its
own binary under the `src/bin` folder, and uses the `#[entry]` macro (custom to
this project) to define the entrypoint:

A simple way to run a single solution would be to do this:

```
cargo run -p y2022 --bin d05
```

If you want to bench a single solution, add `--bench`:

```
cargo run -p y2022 --bin d05 -- --bench
```

The following are the available arguments:

* `--bench` - Run the solution as a benchmark.
* `--verbose` - Verbose output.
* `--warmup` - Warmup period for benchmark in milliseconds (default `400`).
* `--time-limit` - Time to run the benchmark in milliseconds (default `100`).
* `--iter` - How many iterations to run for a single sample, this is determined
  by timing the solution once so that we don't try to take timings in the
  nanosecond realm which would be unreliable.
* `--json` - Output JSON which is used by the "run everything" tool below to
  collect and aggregate output. You can use it yourself if you find it
  interesting.

<br>

## The "run everything" tool

The default tool in this project is provided by the `lib` crate, and allows for
running every project at once (automatically discovered). It works with your
default `cargo run`:

```
$ cargo run -- <args> -- <solution args>
```

`<args>` is one of the following:

* `-q | --quiet` - less verbose output. * `-V | --verbose` - more verbose
output.
* `-p <project>` - only run the specified sub-project, like `y2022` for 2022
  solutions only.
* `--release` - run in release mode.
* `--no-prod` - disable the "production mode", which removes a bunch of stuff
  that is solely used to improve diagnostics during development.

How to run every solution in this repo:

```
cargo run -- -V
```

Run all benchmarks in this repo:

```
cargo run -- --release -- --bench
```
