use criterion::{Criterion, criterion_group, criterion_main};
use std::time::Duration;

#[cfg(target_os = "macos")]
const MEASUREMENT_TIME: u64 = 2;
#[cfg(target_os = "linux")]
const MEASUREMENT_TIME: u64 = 2;

const SAMPLE_SIZE: usize = 10;

/// Lengths used for benchmarking.
const LENGTHS: &[usize] = &[1, 10, 100, 1_000, 10_000, 100_000, 1_000_000, 10_000_000];

/// Initial sizes of the benchmarked table.
const INITIALS: &[usize] = &[1, 10, 100, 1_000, 10_000, 100_000, 1_000_000, 10_000_000];

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

fn wat_source(initial: usize, grow: usize, fun: bool) -> String {
  let fun = if fun { "$f1" } else { "$f2" };
  TEMPLATE
    .replace("<INITIAL>", &initial.to_string())
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
  for initial in INITIALS {
    for length in LENGTHS {
      let wasm_bytes = wat::parse_str(wat_source(*initial, *length, fun_switch)).unwrap();
      fun_switch = !fun_switch;
      let mut config = wasmtime::Config::new();
      config.strategy(wasmtime::Strategy::Winch);
      let engine = wasmtime::Engine::new(&config).unwrap();
      let mut store = wasmtime::Store::new(&engine, ());
      let module = wasmtime::Module::from_binary(&engine, &wasm_bytes).unwrap();
      let instance = wasmtime::Instance::new(&mut store, &module, &[]).unwrap();
      let tab = instance.get_table(&mut store, "tab").unwrap();
      assert_eq!(*initial, tab.size(&store) as usize);
      let fun = instance.get_typed_func::<(), i32>(&mut store, "fun").unwrap();
      assert_eq!(*initial as i32, fun.call(&mut store, ()).unwrap());
      assert_eq!(initial + length, tab.size(&mut store) as usize);
    }
  }
}

fn _0001(c: &mut Criterion) {
  precheck();
  let mut config = wasmtime::Config::new();
  config.strategy(wasmtime::Strategy::Winch);
  let mut group = c.benchmark_group("tg");
  let mut fun_switch = false;
  for initial in INITIALS {
    for length in LENGTHS {
      let wasm_bytes = wat::parse_str(wat_source(*initial, *length, fun_switch)).unwrap();
      fun_switch = !fun_switch;
      let engine = wasmtime::Engine::new(&config).unwrap();
      let module = wasmtime::Module::from_binary(&engine, &wasm_bytes).unwrap();
      group.bench_with_input(format!("{initial}x{length}"), &length, |b, _| {
        b.iter_batched_ref(
          || {
            let mut store = wasmtime::Store::new(&engine, ());
            let instance = wasmtime::Instance::new(&mut store, &module, &[]).unwrap();
            let warm = instance.get_typed_func::<(), ()>(&mut store, "warm").unwrap();
            warm.call(&mut store, ()).unwrap();
            let fun = instance.get_typed_func::<(), i32>(&mut store, "fun").unwrap();
            (store, fun)
          },
          |(store, fun)| {
            fun.call(store, ()).unwrap();
          },
          criterion::BatchSize::LargeInput,
        );
      });
    }
  }
}

criterion_group!(name = table_grow; config = make_config(); targets = _0001);
criterion_main!(table_grow);
