use async_trait::async_trait;
use atrium_xrpc::{client::reqwest::ReqwestClient, HttpClient, XrpcClient};
use http::{Method, Request, Response};
use std::sync::{Arc, Mutex};

use super::session::Session;

pub struct AuthenticateableXrpcClient {
    inner: ReqwestClient,
    session: Option<Arc<Mutex<Session>>>,
}

impl AuthenticateableXrpcClient {
    pub fn new(host: String) -> Self {
        Self {
            inner: ReqwestClient::new(host),
            session: None,
        }
    }

    pub fn with_session(host: String, session: Arc<Mutex<Session>>) -> Self {
        Self {
            inner: ReqwestClient::new(host),
            session: Some(session),
        }
    }
}

#[async_trait]
impl HttpClient for AuthenticateableXrpcClient {
    async fn send_http(
        &self,
        req: Request<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let (mut parts, body) = req.into_parts();

        /* NOTE: This is a huge hack because auth is currently totally broken in atrium-api */
        let is_request_to_refresh_session = parts.method == Method::POST
            && parts
                .uri
                .to_string()
                .ends_with("com.atproto.server.refreshSession");
        if let Some(token) = self.auth(is_request_to_refresh_session) {
            parts.headers.insert(
                http::header::AUTHORIZATION,
                format!("Bearer {}", token).parse()?,
            );
        }

        let req = Request::from_parts(parts, body);

        self.inner.send_http(req).await
    }
}

impl XrpcClient for AuthenticateableXrpcClient {
    fn auth(&self, is_refresh: bool) -> Option<String> {
        self.session
            .as_ref()
            .and_then(|session| session.lock().ok())
            .map(|session| {
                if is_refresh {
                    session.refresh_jwt.clone()
                } else {
                    session.access_jwt.clone()
                }
            })
    }

    fn host(&self) -> &str {
        self.inner.host()
    }
}
