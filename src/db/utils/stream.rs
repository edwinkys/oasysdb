use super::request::{Request, RequestHeaders};
use super::response::Response;
use std::collections::HashMap;
use std::env;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn read(stream: &mut TcpStream) -> Option<Request> {
    // Allow the buffer size to be set via environment variable.
    // The default buffer size is 32KB which is large enough
    // for 1536 dimensions embeddings.
    let buffer_size = env::var("OASYSDB_BUFFER_SIZE")
        .unwrap_or(String::from("32768"))
        .parse::<usize>()
        .unwrap();

    // Prepare the request for parsing.
    let mut _headers = [httparse::EMPTY_HEADER; 16];
    let mut _req = httparse::Request::new(&mut _headers);

    // Read data from the stream.
    let mut buf = vec![0; buffer_size];
    let n = stream.read(&mut buf).await.unwrap();

    // Disconnection handler.
    if n == 0 {
        return None;
    }

    // Parse the request.
    let _ = _req.parse(&buf).unwrap();

    // Parse request headers.
    let headers: RequestHeaders =
        HashMap::from_iter(_req.headers.iter().map(|header| {
            let key = header.name.to_lowercase();
            let val = String::from_utf8_lossy(header.value).to_string();
            (key, val)
        }));

    // If content length is present or more than 0, read the body.
    let _content_len = headers
        .get("content-length")
        .unwrap_or(&"0".to_string())
        .parse::<usize>()
        .unwrap_or(0);

    // Parse the request body.
    // By default, the body is an empty map, not None.
    let _body = if _content_len > 0 {
        let _buf = String::from_utf8_lossy(&buf);
        let _parts = _buf.split_once("\r\n\r\n").unwrap();
        _parts.1.replace('\0', "").clone()
    } else {
        // Create an empty body.
        "{}".to_string()
    };

    // Try to parse the body. If fail, return None.
    // This will guard against invalid JSON.
    let body = match serde_json::from_str(&_body) {
        Ok(body) => body,
        Err(_) => None,
    };

    // Returning None will cause the connection to close.
    body.as_ref()?;

    // Return request data.
    let method = _req.method.unwrap().to_lowercase();
    let route = _req.path.unwrap().to_string();
    Some(Request { method, route, headers, body: body.unwrap() })
}

pub async fn write(stream: &mut TcpStream, response: Response<String>) {
    let (parts, body) = response.into_parts();

    // Get the status code and canonical reason.
    let status = parts.status.as_str();
    let reason = parts.status.canonical_reason().unwrap();

    // HTTP response tag and header.
    let tag = format!("HTTP/1.1 {} {}", status, reason);
    let header = format!("content-length: {}", body.len());

    // Format the response as a string.
    let data = format!("{}\r\n{}\r\n\r\n{}", tag, header, body);

    // Write the response to the stream.
    stream.write_all(data.as_bytes()).await.unwrap();
}
