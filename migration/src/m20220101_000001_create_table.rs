use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;


#[derive(DeriveIden)]
enum StickerTag {
    Table,

    StickerId,
    FileId,
    UserId,
    TagName,
}

#[derive(DeriveIden)]
enum StickerStat {
    Table,

    UserId,
    StickerId,
    Count,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .create_table(
                Table::create()
                    .table(StickerTag::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(StickerTag::StickerId).string().not_null())
                    .col(ColumnDef::new(StickerTag::FileId).string().not_null())
                    .col(ColumnDef::new(StickerTag::UserId).string().not_null())
                    .col(ColumnDef::new(StickerTag::TagName).string().not_null())
                    .primary_key(
                        Index::create()
                            .col(StickerTag::StickerId)
                            .col(StickerTag::UserId)
                            .col(StickerTag::TagName),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(StickerStat::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(StickerStat::UserId).string().not_null())
                    .col(ColumnDef::new(StickerStat::StickerId).string().not_null())
                    .col(ColumnDef::new(StickerStat::Count).integer().not_null())
                    .primary_key(
                        Index::create()
                            .col(StickerStat::UserId)
                            .col(StickerStat::StickerId),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .drop_table(Table::drop().table(StickerTag::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(StickerStat::Table).to_owned())
            .await?;

        Ok(())
    }
}
