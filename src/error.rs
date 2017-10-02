use hyper::Error as HttpError;
use hyper::StatusCode;
use hyper::error::UriError;
use serde_json::error::Error as SerdeError;
use std::io::Error as IoError;

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct ClientError {
    pub error_message: String,
}

error_chain! {
    errors {
        Fault {
            code: StatusCode,
            error: String,
        }
    }
    foreign_links {
        Codec(SerdeError);
        Http(HttpError);
        IO(IoError);
        Uri(UriError);
    }
}
