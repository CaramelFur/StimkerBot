#[derive(Debug, sqlx::FromRow)]
pub struct EntityTag {
    pub entity_id: String,
    pub file_id: String,
    pub user_id: String,
    pub tag_name: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct EntityStat {
    pub user_id: String,
    pub entity_id: String,
    pub count: i64,
    pub last_used: i64,
}
