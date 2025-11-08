use axum::{extract::Request, http::header, middleware::Next, response::Response};
use uuid::Uuid;

const SESSION_COOKIE_NAME: &str = "rapidfab_session";

/// Middleware that reads `rapidfab_session` cookie or creates one
pub async fn session_middleware(mut request: Request, next: Next) -> Response {
    let session_id = request
        .headers()
        .get(header::COOKIE)
        .and_then(|h| h.to_str().ok())
        .and_then(|cookies| {
            cookies
                .split(';')
                .find(|c| c.trim().starts_with(SESSION_COOKIE_NAME))
                .and_then(|c| c.split('=').nth(1))
                .map(|value| value.trim().to_string())
        })
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    request
        .extensions_mut()
        .insert(SessionId(session_id.clone()));

    let mut response = next.run(request).await;

    response.headers_mut().append(
        header::SET_COOKIE,
        format!("{SESSION_COOKIE_NAME}={session_id}; Path=/; HttpOnly; SameSite=Lax")
            .parse()
            .expect("valid cookie header"),
    );

    response
}

#[derive(Clone, Debug)]
pub struct SessionId(pub String);
