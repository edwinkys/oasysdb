use super::*;
use protos::data_node_server::DataNode as ProtoDataNode;
use regex::Regex;

type NodeName = Box<str>;

/// Data node server definition.
///
/// Data nodes are responsible for managing the records and data processing
/// capabilities of OasysDB clusters. Various data nodes can register with a
/// coordinator node.
#[derive(Debug)]
pub struct DataNode {
    name: NodeName,
    params: NodeParameters,
    database_url: DatabaseURL,
    schema: DataSchema,
}

impl DataNode {
    /// Create a new data node instance.
    pub async fn new(
        name: impl Into<NodeName>,
        params: impl Into<NodeParameters>,
        database_url: impl Into<DatabaseURL>,
    ) -> Self {
        let name = name.into();
        let params = params.into();
        let database_url = database_url.into();

        // Validate node name: lowercase, alphanumeric, and underscores only.
        if !Regex::new("^[a-z0-9_]+$").unwrap().is_match(name.as_ref()) {
            let action = "Use lowercase letters, numbers, and underscores.";
            panic!("Invalid node name: {action}");
        }

        let mut connection = PgConnection::connect(database_url.as_ref())
            .await
            .expect("Failed to connect to Postgres database");

        let schema = DataSchema::new(name.as_ref());
        schema.create_schema(&mut connection).await;
        schema.create_all_tables(&mut connection).await;

        Self { name, params, database_url, schema }
    }

    /// Return the name configured for this data node.
    pub fn name(&self) -> &NodeName {
        &self.name
    }

    /// Return the parameters configured for the data node.
    pub fn params(&self) -> &NodeParameters {
        &self.params
    }

    /// Return the configured database URL.
    pub fn database_url(&self) -> &DatabaseURL {
        &self.database_url
    }

    /// Return the schema configured for the data node.
    pub fn schema(&self) -> &DataSchema {
        &self.schema
    }
}

#[async_trait]
impl ProtoDataNode for Arc<DataNode> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::postgres::test_utils;

    const DATA_SCHEMA: &str = "data_";

    #[tokio::test]
    async fn test_data_node_new() {
        let node_name = "49babacf";
        let schema_name = format!("{DATA_SCHEMA}{node_name}");

        let params = test_utils::node_parameters();
        let db = test_utils::database_url();

        let mut conn = PgConnection::connect(&db.to_string())
            .await
            .expect("Failed to connect to Postgres database");

        test_utils::drop_schema(&mut conn, &schema_name).await;
        DataNode::new(node_name, params, db).await;
        test_utils::assert_table_count(&mut conn, &schema_name, 2).await;
    }
}
