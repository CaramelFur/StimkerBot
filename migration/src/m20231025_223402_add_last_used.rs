use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum StickerStat {
    Table,

    LastUsed,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add a new column to the StickerStat table to store the last used time
        manager
            .alter_table(
                Table::alter()
                    .table(StickerStat::Table)
                    .add_column(
                        ColumnDef::new(StickerStat::LastUsed)
                            .big_integer()
                            .default(0)
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove the column from the StickerStat table
        manager
            .alter_table(
                Table::alter()
                    .table(StickerStat::Table)
                    .drop_column(StickerStat::LastUsed)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
