use actix_web::{Error, HttpRequest, HttpResponse, web};
use actix_web_actors::ws;

use crate::session::WsGameSession;
use crate::{app_log, info_log};

/// WebSocket接続を処理するエンドポイント
pub async fn ws_index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    info_log!("Received WebSocket connection: {:?}", req);

    let session = WsGameSession::new();

    let resp = ws::start(session, &req, stream)?;
    Ok(resp)
}
