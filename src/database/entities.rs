#[derive(Debug, sqlx::FromRow)]
pub struct StickerTag {
    pub sticker_id: String,
    pub file_id: String,
    pub user_id: String,
    pub tag_name: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct StickerStat {
    pub user_id: String,
    pub sticker_id: String,
    pub count: i64,
    pub last_used: i64,
}
