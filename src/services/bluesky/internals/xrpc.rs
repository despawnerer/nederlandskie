use std::error::Error;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use atrium_xrpc::{client::reqwest::ReqwestClient, HttpClient, XrpcClient};
use http::{Request, Response};

use crate::services::bluesky::entities::Session;

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
    ) -> Result<Response<Vec<u8>>, Box<dyn Error + Send + Sync + 'static>> {
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
