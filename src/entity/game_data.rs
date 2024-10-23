//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.0

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "game_data")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub game_id: String,
    pub game_time: Decimal,
    #[sea_orm(column_type = "Text")]
    pub game_data: String,
    pub summoner_name: Option<String>,
    pub champion_name: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::games::Entity",
        from = "Column::GameId",
        to = "super::games::Column::GameId",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Games,
}

impl Related<super::games::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Games.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
