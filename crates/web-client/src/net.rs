pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
}

impl AsRef<str> for Method {
    fn as_ref(&self) -> &str {
        match self {
            Self::GET => "GET",
            Self::POST => "POST",
            Self::PUT => "PUT",
            Self::DELETE => "DELETE",
        }
    }
}

// TODO borrow rather than own? or cow?
#[derive(Clone, Debug)]
pub enum Body {
    None,
    Json(String),
}

pub enum StatusCode {
    Continue,
    SwitchingProtocols,
    Processing,
    Ok,
    Created,
    Accepted,
    NonAuthoritativeInformation,
    NoContent,
    ResetContent,
    PartialContent,
    MultiStatus,
    AlreadyReported,
    IMUsed,
    MultipleChoices,
    MovedPermanently,
    Found,
    SeeOther,
    NotModified,
    UseProxy,
    TemporaryRedirect,
    PermanentRedirect,
    BadRequest,
    Unauthorized,
    PaymentRequired,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    NotAcceptable,
    ProxyAuthenticationRequired,
    RequestTimeout,
    Conflict,
    Gone,
    LengthRequired,
    PreconditionFailed,
    PayloadTooLarge,
    RequestURITooLong,
    UnsupportedMediaType,
    RequestedRangeNotSatisfiable,
    ExpectationFailed,
    MisdirectedRequest,
    UnprocessableEntity,
    Locked,
    FailedDependency,
    UpgradeRequired,
    PreconditionRequired,
    TooManyRequests,
    RequestHeaderFieldsTooLarge,
    ConnectionClosedWithoutResponse,
    UnavailableForLegalReasons,
    ClientClosedRequest,
    InternalServerError,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
    GatewayTimeout,
    HTTPVersionNotSupported,
    VariantAlsoNegotiates,
    InsufficientStorage,
    LoopDetected,
    NotExtended,
    NetworkAuthenticationRequired,
    NetworkConnectTimeoutError,
}

impl std::convert::TryFrom<u16> for StatusCode {
    type Error = ();

    fn try_from(raw: u16) -> Result<Self, ()> {
        match raw {
            100 => Ok(Self::Continue),
            101 => Ok(Self::SwitchingProtocols),
            102 => Ok(Self::Processing),
            200 => Ok(Self::Ok),
            201 => Ok(Self::Created),
            202 => Ok(Self::Accepted),
            203 => Ok(Self::NonAuthoritativeInformation),
            204 => Ok(Self::NoContent),
            205 => Ok(Self::ResetContent),
            206 => Ok(Self::PartialContent),
            207 => Ok(Self::MultiStatus),
            208 => Ok(Self::AlreadyReported),
            226 => Ok(Self::IMUsed),
            300 => Ok(Self::MultipleChoices),
            301 => Ok(Self::MovedPermanently),
            302 => Ok(Self::Found),
            303 => Ok(Self::SeeOther),
            304 => Ok(Self::NotModified),
            305 => Ok(Self::UseProxy),
            307 => Ok(Self::TemporaryRedirect),
            308 => Ok(Self::PermanentRedirect),
            400 => Ok(Self::BadRequest),
            401 => Ok(Self::Unauthorized),
            402 => Ok(Self::PaymentRequired),
            403 => Ok(Self::Forbidden),
            404 => Ok(Self::NotFound),
            405 => Ok(Self::MethodNotAllowed),
            406 => Ok(Self::NotAcceptable),
            407 => Ok(Self::ProxyAuthenticationRequired),
            408 => Ok(Self::RequestTimeout),
            409 => Ok(Self::Conflict),
            410 => Ok(Self::Gone),
            411 => Ok(Self::LengthRequired),
            412 => Ok(Self::PreconditionFailed),
            413 => Ok(Self::PayloadTooLarge),
            414 => Ok(Self::RequestURITooLong),
            415 => Ok(Self::UnsupportedMediaType),
            416 => Ok(Self::RequestedRangeNotSatisfiable),
            417 => Ok(Self::ExpectationFailed),
            421 => Ok(Self::MisdirectedRequest),
            422 => Ok(Self::UnprocessableEntity),
            423 => Ok(Self::Locked),
            424 => Ok(Self::FailedDependency),
            426 => Ok(Self::UpgradeRequired),
            428 => Ok(Self::PreconditionRequired),
            429 => Ok(Self::TooManyRequests),
            431 => Ok(Self::RequestHeaderFieldsTooLarge),
            444 => Ok(Self::ConnectionClosedWithoutResponse),
            451 => Ok(Self::UnavailableForLegalReasons),
            499 => Ok(Self::ClientClosedRequest),
            500 => Ok(Self::InternalServerError),
            501 => Ok(Self::NotImplemented),
            502 => Ok(Self::BadGateway),
            503 => Ok(Self::ServiceUnavailable),
            504 => Ok(Self::GatewayTimeout),
            505 => Ok(Self::HTTPVersionNotSupported),
            506 => Ok(Self::VariantAlsoNegotiates),
            507 => Ok(Self::InsufficientStorage),
            508 => Ok(Self::LoopDetected),
            510 => Ok(Self::NotExtended),
            511 => Ok(Self::NetworkAuthenticationRequired),
            599 => Ok(Self::NetworkConnectTimeoutError),
            _ => Err(()),
        }
    }
}

pub struct Response {
    status_code: StatusCode,
    body: Vec<u8>,
}

impl Response {
    pub fn json<'a, V>(&'a self) -> serde_json::Result<V>
    where
        V: serde::Deserialize<'a>,
    {
        serde_json::from_slice(&self.body)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct JsError(String);

impl From<wasm_bindgen::JsValue> for JsError {
    fn from(v: wasm_bindgen::JsValue) -> Self {
        Self(v.as_string().unwrap_or(String::from("Unknown")))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("Failed to send {0}")]
    SendError(url::Url, #[source] JsError),
    #[error("Failed to receive {0}")]
    ReceiveError(url::Url, #[source] ReceiveError),
}

pub struct Request {
    method: Method,
    url: url::Url,
    body: Body,
}

impl Request {
    pub fn build(method: Method, url: url::Url) -> RequestBuilder {
        RequestBuilder {
            method: method,
            url,
        }
    }

    pub fn get(url: url::Url) -> RequestBuilder {
        Request::build(Method::GET, url)
    }

    pub async fn fetch(self) -> Result<Response, FetchError> {
        use wasm_bindgen::JsCast;

        let window = web_sys::window().unwrap();

        let mut request_init = web_sys::RequestInit::new();
        request_init.method(self.method.as_ref());

        let headers = web_sys::Headers::new().unwrap();
        match self.body {
            Body::Json(json) => {
                headers.append("Content-Type", "application/json").unwrap();
                request_init.body(Some(&js_sys::Uint8Array::new(unsafe {
                    &js_sys::Uint8Array::view(json.as_ref())
                })));
            }
            Body::None => {}
        }
        request_init.headers(&headers);

        let url = self.url;

        match wasm_bindgen_futures::JsFuture::from(
            window.fetch_with_str_and_init(url.as_str(), &request_init),
        )
        .await
        {
            Ok(response) => receive(response.unchecked_into::<web_sys::Response>())
                .await
                .map_err(|err| FetchError::ReceiveError(url, err)),
            Err(err) => Err(FetchError::SendError(url, JsError::from(err))),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ReceiveError {
    #[error(transparent)]
    IoError(JsError),
    #[error("Received chunk not a byte array")]
    UnexpectedChunkError(JsError),
    #[error("Bad status code '{0}'")]
    BadStatusCode(u16),
}

async fn receive(response: web_sys::Response) -> Result<Response, ReceiveError> {
    use futures::StreamExt;
    use std::convert::TryFrom;
    use wasm_bindgen::JsCast;

    let response: web_sys::Response = response.dyn_into().unwrap();

    Ok(Response {
        status_code: StatusCode::try_from(response.status())
            .map_err(|err| ReceiveError::BadStatusCode(response.status()))?,
        body: futures::stream::iter(response.body().map(|body| {
            wasm_streams::readable::ReadableStream::from_raw(
                body.unchecked_into::<wasm_streams::readable::sys::ReadableStream>(),
            )
            .into_stream()
        }))
        .flatten()
        .fold(Ok(Vec::new()), |bytes, chunk| {
            futures::future::ready(
                chunk
                    .map_err(|err| ReceiveError::IoError(JsError::from(err)))
                    .and_then(|chunk| bytes.map(|bytes| (bytes, chunk)))
                    .and_then(|(mut bytes, chunk)| {
                        let chunk = chunk.dyn_into::<js_sys::Uint8Array>().map_err(|err| {
                            ReceiveError::UnexpectedChunkError(JsError::from(err))
                        })?;

                        let len = bytes.len();
                        bytes.resize(len + chunk.byte_length() as usize, 0u8);
                        chunk.copy_to(&mut bytes[len..]);

                        Ok(bytes)
                    }),
            )
        })
        .await?,
    })
}

pub struct RequestBuilder {
    method: Method,
    url: url::Url,
}

impl RequestBuilder {
    pub fn method(&mut self, method: Method) -> &mut Self {
        self.method = method;
        self
    }

    pub fn json<B>(self, body: B) -> serde_json::Result<Request>
    where
        B: serde::Serialize,
    {
        Ok(Request {
            method: self.method,
            url: self.url,
            body: Body::Json(serde_json::to_string(&body)?),
        })
    }

    pub fn finish(self) -> Request {
        Request {
            method: self.method,
            url: self.url,
            body: Body::None,
        }
    }
}
