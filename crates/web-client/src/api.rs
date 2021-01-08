#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to fetch response")]
    FetchError(#[from] crate::net::FetchError),
    #[error("Failed to encode request")]
    SerialiseError(#[source] serde_json::Error),
    #[error("Failed to decode response")]
    DeserialiseError(#[source] serde_json::Error),
}

pub async fn get<RespBody>(url: url::Url) -> Result<RespBody, Error>
where
    for<'a> RespBody: serde::Deserialize<'a>,
{
    Ok(crate::net::Request::build(crate::net::Method::GET, url)
        .finish()
        .fetch()
        .await?
        .json::<RespBody>()
        .map_err(Error::DeserialiseError)?)
}

pub async fn post<RespBody, RqstBody>(url: url::Url, rqst_body: RqstBody) -> Result<RespBody, Error>
where
    RqstBody: serde::Serialize,
    for<'a> RespBody: serde::Deserialize<'a>,
{
    Ok(crate::net::Request::build(crate::net::Method::POST, url)
        .json(rqst_body)
        .map_err(Error::SerialiseError)?
        .fetch()
        .await?
        .json::<RespBody>()
        .map_err(Error::DeserialiseError)?)
}

pub async fn put<RespBody, RqstBody>(url: url::Url, rqst_body: RqstBody) -> Result<RespBody, Error>
where
    RqstBody: serde::Serialize,
    for<'a> RespBody: serde::Deserialize<'a>,
{
    Ok(crate::net::Request::build(crate::net::Method::PUT, url)
        .json(rqst_body)
        .map_err(Error::SerialiseError)?
        .fetch()
        .await?
        .json::<RespBody>()
        .map_err(Error::DeserialiseError)?)
}
