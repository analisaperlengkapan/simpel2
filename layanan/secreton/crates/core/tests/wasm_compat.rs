#[cfg(feature = "wasm")]
#[test]
fn test_wasm_instantiation() {
    use wasmtime::{Engine, Instance, Module, Store};

    // Minimal WASM module: (module (func (export "evaluate") (result i32) (i32.const 1)))
    let wasm: &[u8] = &[
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, // Header
        0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f, // Type section: 1 type, () -> i32
        0x03, 0x02, 0x01, 0x00, // Function section: 1 func, type 0
        0x07, 0x0c, 0x01, 0x08, 0x65, 0x76, 0x61, 0x6c, 0x75, 0x61, 0x74, 0x65, 0x00,
        0x00, // Export "evaluate"
        0x0a, 0x06, 0x01, 0x04, 0x00, 0x41, 0x01, 0x0b, // Code section: i32.const 1, end
    ];

    let engine = Engine::default();
    let module = Module::new(&engine, wasm).expect("Failed to create module");
    let mut store = Store::new(&engine, ());

    let instance = Instance::new(&mut store, &module, &[]).expect("Failed to instantiate");

    let func = instance
        .get_func(&mut store, "evaluate")
        .expect("Failed to get export");

    let mut results = [wasmtime::Val::I32(0)];
    func.call(&mut store, &[], &mut results)
        .expect("Failed to call");

    assert_eq!(results[0].i32(), Some(1));
}
