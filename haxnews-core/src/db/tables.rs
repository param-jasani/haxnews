use redb::TableDefinition;

/// Tables using String keys and JSON-serialized values
pub const FEEDS_TABLE: TableDefinition<'static, &str, &[u8]> = 
    TableDefinition::new("feeds");

pub const ITEMS_TABLE: TableDefinition<'static, &str, &[u8]> = 
    TableDefinition::new("items");