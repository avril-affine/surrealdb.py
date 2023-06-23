//! Defines the core functions for creating records. These functions should not be called directly
//! from the Python API but rather from the TCP connection in the runtime module. In this
//! module we can do the following:
//! 
//! * Create a record in the database
use serde_json::value::Value;
use crate::connection::state::{
    WrappedConnection
};
use surrealdb::opt::Resource;
use tokio::sync::mpsc::Sender;
use crate::connection::state::ConnectionMessage;
use crate::connection::state::{get_connection, return_connection};


type DbResult = Result<surrealdb::sql::Value, std::string::String>;


/// Creates a record in the database.
/// 
/// # Arguments
/// * `connection_id` - The unique ID for the connection
/// * `table_name` - The name of the table to create the record in
/// 
/// # Returns
/// * `Ok(())` - The record was created successfully
pub async fn create(connection_id: String, table_name: String, data: Value, tx: Sender<ConnectionMessage>) -> Result<(), String> {
    let mut connection = get_connection(connection_id.clone(), tx).await?;

    let resource = Resource::from(table_name.clone());

    let used_connection: WrappedConnection;
    let db_result: DbResult;
    match connection.value.unwrap() {
        WrappedConnection::WS(ws_connection) => {
            db_result = ws_connection.create(resource).content(data).await.map_err(|e| e.to_string());
            used_connection = WrappedConnection::WS(ws_connection);
        },
        WrappedConnection::HTTP(http_connection) => {
            db_result = http_connection.create(resource).content(data).await.map_err(|e| e.to_string());
            used_connection = WrappedConnection::HTTP(http_connection);
        },
    }   
    connection.value = Some(used_connection);
    return_connection(connection).await?;
    db_result?;
    Ok(())
}
