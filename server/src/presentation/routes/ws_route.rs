use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use crate::{app_log, info_log};
use crate::session::WsGameSession;

/// WebSocket接続を処理するエンドポイント
pub async fn ws_index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    info_log!("Received WebSocket connection: {:?}", req);

    let session = WsGameSession::new();

    let resp = ws::start(session, &req, stream)?;
    Ok(resp)
}