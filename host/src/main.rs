mod bindings {
    wasmtime::component::bindgen!(in "../wit");
}

fn main() {
    let component = std::env::args()
        .skip(1)
        .next()
        .expect("expected path to the component as first arg to host executable");
    let component = std::fs::read(&component).unwrap();
    let engine = wasmtime::Engine::default();
    let component = wasmtime::component::Component::new(&engine, component).unwrap();
    let mut store = wasmtime::Store::new(&engine, Data::new());
    let mut linker = wasmtime::component::Linker::new(&engine);
    wasmtime_wasi::add_to_linker_sync(&mut linker).unwrap();
    let (w, _) = bindings::SqliteWorld::instantiate(&mut store, &component, &linker).unwrap();
    let conn = w
        .component_sqlite_component_sqlite()
        .connection()
        .call_open(&mut store, "")
        .unwrap()
        .unwrap();
    w.component_sqlite_component_sqlite()
        .connection()
        .call_execute(
            &mut store,
            conn,
            "CREATE TABLE contacts (contact_id INTEGER PRIMARY KEY);",
            &[],
        )
        .unwrap()
        .unwrap();
    w.component_sqlite_component_sqlite()
        .connection()
        .call_execute(
            &mut store,
            conn,
            "INSERT INTO contacts (contact_id) values (1);",
            &[],
        )
        .unwrap()
        .unwrap();
    let result = w
        .component_sqlite_component_sqlite()
        .connection()
        .call_execute(&mut store, conn, "select * from contacts;", &[])
        .unwrap()
        .unwrap();
    println!("{:?}", result);
}

struct Data {
    ctx: wasmtime_wasi::WasiCtx,
    table: wasmtime::component::ResourceTable,
}

impl Data {
    fn new() -> Self {
        Self {
            ctx: wasmtime_wasi::WasiCtxBuilder::new().build(),
            table: wasmtime::component::ResourceTable::default(),
        }
    }
}

impl wasmtime_wasi::WasiView for Data {
    fn table(&mut self) -> &mut wasmtime_wasi::ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut wasmtime_wasi::WasiCtx {
        &mut self.ctx
    }
}
