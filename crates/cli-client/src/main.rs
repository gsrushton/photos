const SUPPORTED_EXTENSIONS: [&'static str; 3] = ["jpg", "jpeg", "png"];

fn supports_extension(extension: &str) -> bool {
    return SUPPORTED_EXTENSIONS
        .iter()
        .find(|supported_extension| **supported_extension == extension)
        .is_some();
}

#[derive(Debug, thiserror::Error)]
enum ResolvedPathError {
    #[error("Operation failed on directory {0:?}")]
    Dir(std::path::PathBuf, #[source] std::io::Error),
    #[error("Operation failed on path {0:?}")]
    Unknown(std::path::PathBuf, #[source] std::io::Error),
}

enum ResolvedPath {
    Dir(std::path::PathBuf),
    File(std::path::PathBuf),
    Other(std::path::PathBuf),
}

impl ResolvedPath {
    pub fn gather_files(self) -> Vec<Result<std::path::PathBuf, ResolvedPathError>> {
        let mut files = Vec::new();
        self.gather_files_into(&mut files);
        files
    }

    fn gather_files_into(self, files: &mut Vec<Result<std::path::PathBuf, ResolvedPathError>>) {
        match self {
            Self::Dir(dir_path) => match Self::list(&dir_path) {
                Ok(path_results) => {
                    for path_result in path_results {
                        match path_result {
                            Ok(path) => path.gather_files_into(files),
                            Err(err) => files.push(Err(err)),
                        }
                    }
                }
                Err(err) => files.push(Err(err)),
            },
            Self::File(file_path) => files.push(Ok(file_path)),
            Self::Other(_) => {}
        }
    }

    fn list(
        dir_path: &std::path::Path,
    ) -> Result<Vec<Result<ResolvedPath, ResolvedPathError>>, ResolvedPathError> {
        Ok(std::fs::read_dir(dir_path)
            .map_err(|err| ResolvedPathError::Dir(dir_path.to_path_buf(), err))?
            .map(|entry_result| {
                entry_result
                    .map_err(|err| ResolvedPathError::Unknown(dir_path.to_path_buf(), err))
                    .and_then(|entry| {
                        use std::convert::TryFrom;
                        ResolvedPath::try_from(entry)
                    })
            })
            .collect())
    }

    fn try_from_path_and_metadata(
        metadata: Result<std::fs::Metadata, std::io::Error>,
        path: std::path::PathBuf,
    ) -> Result<Self, ResolvedPathError> {
        metadata
            .map_err(|err| ResolvedPathError::Unknown(path.clone(), err))
            .map(|metadata| {
                if metadata.is_dir() {
                    Self::Dir(path)
                } else if metadata.is_file() {
                    Self::File(path)
                } else {
                    Self::Other(path)
                }
            })
    }
}

impl std::convert::TryFrom<std::path::PathBuf> for ResolvedPath {
    type Error = ResolvedPathError;

    fn try_from(path: std::path::PathBuf) -> Result<Self, Self::Error> {
        Self::try_from_path_and_metadata(std::fs::metadata(&path), path)
    }
}

impl std::convert::TryFrom<std::fs::DirEntry> for ResolvedPath {
    type Error = ResolvedPathError;

    fn try_from(e: std::fs::DirEntry) -> Result<Self, Self::Error> {
        Self::try_from_path_and_metadata(e.metadata(), e.path())
    }
}

type PendingReponses =
    futures::stream::futures_unordered::FuturesUnordered<hyper::client::ResponseFuture>;

#[derive(Debug, thiserror::Error)]
enum NewClientError {
    #[error("Invalid host '{0}'")]
    InvalidHost(String, #[source] http::uri::InvalidUri),
    #[error("Invalid path")]
    InvalidPath(#[source] ResolvedPathError),
}

#[derive(Debug, thiserror::Error)]
enum MakeBodyError {
    #[error("Failed to open {0:?}")]
    FileOpenError(std::path::PathBuf, #[source] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
enum MakeRequestError {
    #[error("Failed to create body {0:?}")]
    MakeBodyError(#[from] MakeBodyError),
}

struct Client {
    http_client: hyper::client::Client<hyper::client::HttpConnector>,
    file_paths: Vec<Result<std::path::PathBuf, ResolvedPathError>>,
    responses: PendingReponses,
    uri: http::Uri,
}

impl Client {
    pub fn new(host: &str, root: std::path::PathBuf) -> Result<Self, NewClientError> {
        use std::convert::TryFrom;

        Ok(Self {
            http_client: hyper::client::Client::new(),
            responses: PendingReponses::new(),
            uri: http::uri::Builder::new()
                .scheme(http::uri::Scheme::HTTP)
                .authority(
                    http::uri::Authority::try_from(host)
                        .map_err(|err| NewClientError::InvalidHost(host.to_string(), err))?,
                )
                .path_and_query("/api/photos")
                .build()
                .unwrap(),
            file_paths: ResolvedPath::try_from(root)
                .map_err(|err| NewClientError::InvalidPath(err))?
                .gather_files()
                .into_iter()
                .filter(|file_path_result| match file_path_result {
                    Ok(file_path) => {
                        let extension = file_path
                            .extension()
                            .and_then(|extension| extension.to_str())
                            .map(|extension| extension.to_lowercase());

                        extension.is_some() && supports_extension(extension.as_ref().unwrap())
                    }
                    Err(_) => true,
                })
                .collect(),
        })
    }

    pub async fn run(&mut self) {
        use futures::StreamExt;

        async fn body_to_string(body: &mut hyper::Body) -> String {
            let mut bytes = Vec::new();
            while let Some(chunk_result) = body.next().await {
                match chunk_result {
                    Ok(chunk) => bytes.extend(chunk),
                    Err(_err) => {
                        // TODO
                        // log::warn!("{}", err);
                        break;
                    }
                }
            }
            String::from_utf8_lossy(&bytes).into_owned()
        }

        self.enqueue_requests().await;

        while let Some(response_result) = self.responses.next().await {
            match response_result {
                Ok(mut response) => {
                    let status = response.status();
                    if status.is_client_error() {
                        println!(
                            "Client error: {}",
                            status.canonical_reason().unwrap_or("Unknown reason")
                        );
                        println!("{}", body_to_string(response.body_mut()).await);
                    } else if status.is_server_error() {
                        println!(
                            "Server error: {}",
                            status.canonical_reason().unwrap_or("Unknown reason")
                        );
                        println!("{}", body_to_string(response.body_mut()).await);
                    } else if !status.is_success() {
                        unreachable!();
                    }
                }
                Err(err) => Self::log_error(&err),
            }

            self.enqueue_requests().await;
        }
    }

    async fn enqueue_requests(&mut self) {
        while !self.file_paths.is_empty() && self.responses.len() < 4 {
            let file_path = match self.file_paths.pop().unwrap() {
                Ok(file_path) => file_path,
                Err(err) => {
                    Self::log_error(&err);
                    continue;
                }
            };

            let request = match Self::make_request(self.uri.clone(), file_path).await {
                Ok(request) => request,
                Err(err) => {
                    Self::log_error(&err);
                    continue;
                }
            };

            self.responses.push(self.http_client.request(request))
        }
    }

    async fn make_body(file_path: std::path::PathBuf) -> Result<hyper::Body, MakeBodyError> {
        struct BodyFileStream {
            file: tokio::fs::File,
        }

        impl BodyFileStream {
            pin_utils::unsafe_pinned!(file: tokio::fs::File);

            pub fn new(file: tokio::fs::File) -> Self {
                Self { file }
            }
        }

        impl futures::Stream for BodyFileStream {
            type Item =
                Result<hyper::body::Bytes, Box<dyn std::error::Error + 'static + Send + Sync>>;

            fn poll_next(
                self: std::pin::Pin<&mut Self>,
                ctx: &mut futures::task::Context<'_>,
            ) -> futures::task::Poll<Option<Self::Item>> {
                use futures::task::Poll;
                use tokio::io::AsyncRead;

                let mut bytes = vec![0u8; 4096];
                let mut buffer = tokio::io::ReadBuf::new(&mut bytes);

                match self.file().poll_read(ctx, &mut buffer) {
                    Poll::Ready(result) => {
                        let read_count = buffer.filled().len();
                        drop(buffer);

                        bytes.resize(read_count, 0u8);

                        Poll::Ready(
                            result
                                .map(|_| match read_count {
                                    0 => None,
                                    _ => Some(hyper::body::Bytes::from(bytes)),
                                })
                                .map_err(|err| {
                                    Box::new(err)
                                        as Box<dyn std::error::Error + 'static + Send + Sync>
                                })
                                .transpose(),
                        )
                    }
                    Poll::Pending => Poll::Pending,
                }
            }
        }

        let file = tokio::fs::File::open(&file_path)
            .await
            .map_err(|err| MakeBodyError::FileOpenError(file_path, err))?;

        Ok(hyper::Body::wrap_stream(BodyFileStream::new(file)))
    }

    async fn make_request(
        uri: http::Uri,
        file_path: std::path::PathBuf,
    ) -> Result<hyper::Request<hyper::Body>, MakeRequestError> {
        Ok(hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri(uri)
            .body(Self::make_body(file_path).await?)
            .unwrap())
    }

    fn log_error(err: &dyn std::error::Error) {
        println!("{}", err);
    }
}

async fn run(host: &str, path: std::path::PathBuf) -> Result<(), NewClientError> {
    let mut client = Client::new(host, path)?;
    client.run().await;
    Ok(())
}

#[derive(structopt::StructOpt)]
struct CliOptions {
    host: String,
    path: std::path::PathBuf,
}

#[tokio::main]
async fn main() {
    use structopt::StructOpt;

    let cli_options = CliOptions::from_args();

    if let Err(error) = run(&cli_options.host, cli_options.path).await {
        use std::error::Error;

        println!("Error: {}", error);

        let mut current = error.source();
        if current.is_some() {
            println!("");
            println!("Caused by:");
            while let Some(error) = current {
                println!("  {}", error);
                current = error.source();
            }
        }
    }
}
