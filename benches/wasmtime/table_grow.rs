use criterion::{criterion_group, criterion_main, Criterion};
use std::time::Duration;

#[cfg(target_os = "macos")]
const MEASUREMENT_TIME: u64 = 10;
#[cfg(target_os = "linux")]
const MEASUREMENT_TIME: u64 = 2;

const SAMPLE_SIZE: usize = 20;

/// Lengths used for benchmarking.
const LENGTHS: &[usize] = &[
  0,
  1,
  2,
  5,
  10,
  20,
  30,
  50,
  60,
  70,
  80,
  90,
  100,
  200,
  300,
  400,
  500,
  600,
  700,
  800,
  900,
  1_000,
  2_000,
  3_000,
  4_000,
  5_000,
  6_000,
  7_000,
  8_000,
  9_000,
  10_000,
  20_000,
  30_000,
  40_000,
  50_000,
  60_000,
  70_000,
  80_000,
  90_000,
  100_000,
  200_000,
  300_000,
  400_000,
  500_000,
  600_000,
  700_000,
  800_000,
  900_000,
  1_000_000,
  2_000_000,
  3_000_000,
  4_000_000,
  5_000_000,
  6_000_000,
  7_000_000,
  8_000_000,
  9_000_000,
  10_000_000,
  20_000_000,
  30_000_000,
  40_000_000,
  50_000_000,
  60_000_000,
  70_000_000,
  80_000_000,
  90_000_000,
  100_000_000,
  200_000_000,
  300_000_000,
  400_000_000,
  500_000_000,
  600_000_000,
  700_000_000,
  800_000_000,
  900_000_000,
  1_000_000_000,
];

/// Initial size of the benchmarked table.
const INITIAL: usize = 1;

const TEMPLATE: &str = r#"
(module
  (table (export "tab") <INITIAL> funcref)
  (elem func $f1 $f2)
  (func $f1)
  (func $f2)
  (func (export "warm"))
  (func (export "fun") (result i32)
    ref.func <FUN>
    i32.const <GROW>
    table.grow 0
  )
)
"#;

fn wat_source(grow: usize, fun: bool) -> String {
  let fun = if fun { "$f1" } else { "$f2" };
  TEMPLATE
    .replace("<INITIAL>", &INITIAL.to_string())
    .replace("<GROW>", &grow.to_string())
    .replace("<FUN>", fun)
}

fn make_config() -> Criterion {
  Criterion::default()
    .without_plots()
    .measurement_time(Duration::new(MEASUREMENT_TIME, 0))
    .sample_size(SAMPLE_SIZE)
    .configure_from_args()
}

/// Checks if the benchmarked Wasm code works.
fn precheck() {
  let mut fun_switch = false;
  for length in LENGTHS {
    let wasm_bytes = wat::parse_str(wat_source(*length, fun_switch)).unwrap();
    fun_switch = !fun_switch;
    let compiler = wasmer::sys::Singlepass::default();
    let mut store = wasmer::Store::new(compiler);
    let module = wasmer::Module::from_binary(&store, &wasm_bytes).unwrap();
    let instance = wasmer::Instance::new(&mut store, &module, &wasmer::imports! {}).unwrap();
    let tab = instance.exports.get_table("tab").unwrap();
    assert_eq!(INITIAL, tab.size(&store) as usize);
    let fun = instance.exports.get_typed_function::<(), i32>(&store, "fun").unwrap();
    assert_eq!(INITIAL as i32, fun.call(&mut store).unwrap());
    assert_eq!(INITIAL + length, tab.size(&store) as usize);
  }
}

fn _0001(c: &mut Criterion) {
  precheck();
  let mut group = c.benchmark_group("t.grow");
  let mut fun_switch = false;
  for length in LENGTHS {
    let wasm_bytes = wat::parse_str(wat_source(*length, fun_switch)).unwrap();
    fun_switch = !fun_switch;
    let compiler = wasmer::sys::Singlepass::default();
    let store = wasmer::Store::new(compiler);
    let module = wasmer::Module::from_binary(&store, &wasm_bytes).unwrap();
    group.bench_with_input(format!("{length}"), &length, |b, _| {
      b.iter_batched_ref(
        || {
          let mut store = wasmer::Store::new(wasmer::sys::Singlepass::default());
          let instance = wasmer::Instance::new(&mut store, &module, &wasmer::imports! {}).unwrap();
          let warm = instance.exports.get_typed_function::<(), ()>(&store, "warm").unwrap();
          warm.call(&mut store).unwrap();
          let fun = instance.exports.get_typed_function::<(), i32>(&store, "fun").unwrap();
          (store, fun)
        },
        |(store, fun)| {
          fun.call(store).unwrap();
        },
        criterion::BatchSize::LargeInput,
      );
    });
  }
}

criterion_group!(name = table_grow; config = make_config(); targets = _0001);
criterion_main!(table_grow);
