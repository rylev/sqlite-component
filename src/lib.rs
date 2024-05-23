#[allow(warnings)]
mod bindings;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

use bindings::exports::component::sqlite_component::sqlite::{
    Connection, Error, Guest, GuestConnection, RowResult, Value,
};

struct Component;

impl Guest for Component {
    type Connection = ConnectionImpl;
}

#[derive(Clone)]
struct ConnectionImpl {
    inner: Arc<Mutex<rusqlite::Connection>>,
}

impl GuestConnection for ConnectionImpl {
    fn open(database: String) -> Result<Connection, Error> {
        use std::collections::hash_map::Entry;

        static POOL: OnceLock<Mutex<HashMap<String, ConnectionImpl>>> = OnceLock::new();
        let mut pool = POOL
            .get_or_init(|| Mutex::new(HashMap::default()))
            .lock()
            .unwrap();
        let conn_entry = pool.entry(database);
        let result = match conn_entry {
            Entry::Occupied(e) => e.get().clone(),
            Entry::Vacant(e) => {
                let inner =
                    rusqlite::Connection::open_in_memory().map_err(|e| Error::Io(e.to_string()))?;
                e.insert(ConnectionImpl {
                    inner: Arc::new(Mutex::new(inner)),
                })
                .clone()
            }
        };
        Ok(Connection::new(result))
    }

    fn execute(
        &self,
        statement: String,
        parameters: Vec<bindings::exports::component::sqlite_component::sqlite::Value>,
    ) -> Result<bindings::exports::component::sqlite_component::sqlite::QueryResult, Error> {
        let conn = self.inner.lock().unwrap();
        let mut statement = conn
            .prepare_cached(&statement)
            .map_err(|e| Error::Io(e.to_string()))?;
        let columns = statement
            .column_names()
            .into_iter()
            .map(ToOwned::to_owned)
            .collect();
        let rows = statement
            .query_map(
                rusqlite::params_from_iter(convert_params(parameters.into_iter())),
                |row| {
                    let mut values = vec![];
                    for column in 0.. {
                        let value = row.get::<usize, ValueWrapper>(column);
                        if let Err(rusqlite::Error::InvalidColumnIndex(_)) = value {
                            break;
                        }
                        let value = value?.0;
                        values.push(value);
                    }
                    Ok(RowResult { values })
                },
            )
            .map_err(|e| Error::Io(e.to_string()))?;
        let rows = rows
            .into_iter()
            .map(|r| r.map_err(|e| Error::Io(e.to_string())))
            .collect::<Result<_, Error>>()?;
        Ok(bindings::exports::component::sqlite_component::sqlite::QueryResult { columns, rows })
    }
}

fn convert_params(
    arguments: impl Iterator<Item = Value>,
) -> impl Iterator<Item = rusqlite::types::Value> {
    arguments.map(|a| match a {
        Value::Null => rusqlite::types::Value::Null,
        Value::Integer(i) => rusqlite::types::Value::Integer(i),
        Value::Real(r) => rusqlite::types::Value::Real(r),
        Value::Text(t) => rusqlite::types::Value::Text(t),
        Value::Blob(b) => rusqlite::types::Value::Blob(b),
    })
}

// A wrapper around sqlite::Value so that we can convert from rusqlite ValueRef
struct ValueWrapper(Value);

impl rusqlite::types::FromSql for ValueWrapper {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let value = match value {
            rusqlite::types::ValueRef::Null => Value::Null,
            rusqlite::types::ValueRef::Integer(i) => Value::Integer(i),
            rusqlite::types::ValueRef::Real(f) => Value::Real(f),
            rusqlite::types::ValueRef::Text(t) => {
                Value::Text(String::from_utf8(t.to_vec()).unwrap())
            }
            rusqlite::types::ValueRef::Blob(b) => Value::Blob(b.to_vec()),
        };
        Ok(ValueWrapper(value))
    }
}

bindings::export!(Component with_types_in bindings);
