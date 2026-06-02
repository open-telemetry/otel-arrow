/// Definition of errors that could happen when exporting OTAP batches to Clickhouse
#[derive(thiserror::Error, Debug)]
pub enum ClickhouseExporterError {
    // Generic Arrow errors
    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),

    // Pdata / Otap Record related errors
    #[error("Pdata error: {0}")]
    Child(#[from] otap_df_pdata::error::Error),

    // Clickhouse client related errors
    #[error("Table creation error: {error}")]
    TableCreationError { error: String },

    #[error("Clickhouse client connection error: {error}")]
    ClientConnectionError { error: String },

    #[error("Clickhouse data insertion request error: {error}")]
    InsertRequestError { error: String },

    #[error("Clickhouse data insertion response error: {error}")]
    InsertResponseError { error: String },

    // Data processing errors
    #[error("Missing column: {name}")]
    MissingColumn { name: String },

    #[error("Coercion Error: {error}")]
    CoercionError { error: String },

    #[error("Serialization Error: {error}")]
    SerializationError { error: String },

    #[error("Unsupported Payload Type: {error}")]
    UnsupportedPayload { error: String },

    #[error("Invalid type for column: {name}, expected: {expected}, found: {found}")]
    InvalidColumnType {
        name: String,
        expected: String,
        found: String,
    },
}
