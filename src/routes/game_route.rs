use crate::entity::{game_data, games};
use crate::services::game_service::calculate;
use crate::structs::game_struct::GameProps;
use crate::structs::routes_struct::{
    HTTPErrorResponse, LastByCodeRequest, LastByCodeResponse, LastByCodeResponseData,
};
use actix_web::{web, HttpResponse, Responder};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use sea_orm::{DatabaseConnection, QueryOrder};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/last", web::post().to(last_by_code));
}

pub async fn last_by_code(
    db: web::Data<DatabaseConnection>,
    body: Result<web::Json<LastByCodeRequest>, actix_web::Error>,
) -> impl Responder {
    let data = match body {
        Ok(data) => data.into_inner(),
        Err(_) => {
            return HttpResponse::BadRequest().json(HTTPErrorResponse {
                success: false,
                message: "Invalid request: Verify any missing fields",
            });
        }
    };

    let first_query = games::Entity::find()
        .filter(games::Column::GameCode.eq(data.code))
        .order_by_desc(games::Column::CreatedAt)
        .one(db.get_ref())
        .await;

    match first_query {
        Ok(Some(query_1)) => {
            let second_query = game_data::Entity::find()
                .filter(game_data::Column::GameId.eq(query_1.game_id.clone()))
                .order_by_desc(game_data::Column::GameTime)
                .one(db.get_ref())
                .await;

            match second_query {
                Ok(Some(query_2)) => match serde_json::from_str::<GameProps>(&query_2.game_data) {
                    Ok(game_props) => {
                        let calc = calculate(game_props, &data.item).await;

                        let game = match serde_json::to_string(&calc) {
                            Ok(json) => json,
                            Err(_) => {
                                return HttpResponse::InternalServerError()
                                    .json("Failed to serialize game data");
                            }
                        };
                        return HttpResponse::Ok().json(LastByCodeResponse {
                            success: true,
                            data: LastByCodeResponseData {
                                game_id: query_1.game_id,
                                summoner_name: query_1.summoner_name,
                                created_at: query_1.created_at,
                                game_code: query_1.game_code,
                                champion_name: query_1.champion_name,
                                game,
                            },
                        });
                    }
                    Err(_) => {
                        return HttpResponse::InternalServerError().json(HTTPErrorResponse {
                            success: false,
                            message: "Failed to deserialize game data",
                        });
                    }
                },
                Ok(None) => {
                    return HttpResponse::NotFound().json(HTTPErrorResponse {
                        success: false,
                        message: "No game data found with the provided code",
                    });
                }
                Err(_) => {
                    return HttpResponse::InternalServerError().json(HTTPErrorResponse {
                        success: false,
                        message: "Failed to retrieve game data from the database",
                    });
                }
            }
        }
        Ok(None) => {
            return HttpResponse::NotFound().json(HTTPErrorResponse {
                success: false,
                message: "No game found with the provided code",
            });
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(HTTPErrorResponse {
                success: false,
                message: "Failed to retrieve game from the database",
            });
        }
    }
}
