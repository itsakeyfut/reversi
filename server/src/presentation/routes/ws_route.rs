use actix_web::{Error, HttpRequest, HttpResponse, web};
use actix_web_actors::ws;

use crate::server::GameServer;
use crate::session::WsGameSession;
use crate::{app_log, info_log};

/// WebSocket接続を処理するエンドポイント
pub async fn ws_index(
    req: HttpRequest,
    stream: web::Payload,
    server: web::Data<actix::Addr<GameServer>>,
) -> Result<HttpResponse, Error> {
    info_log!("Received WebSocket connection: {:?}", req);

    let mut session = WsGameSession::new();
    session.server_addr = Some(server.get_ref().clone());

    let resp = ws::start(session, &req, stream)?;
    Ok(resp)
}
