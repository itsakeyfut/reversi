use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use crate::{app_log, info_log};
use crate::session::WsGameSession;
use crate::server::GameServer;


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