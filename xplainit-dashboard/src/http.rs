//! Minimal HTTP request parsing and response formatting helpers.
//!
//! This is a deliberately small, hand written HTTP layer built directly on
//! tokio's TCP primitives. No external web framework is used (none is
//! available in this sandbox). The streaming endpoint uses Server Sent Events
//! (SSE) rather than WebSockets, since SSE needs no handshake and no extra
//! dependency.

/// A parsed HTTP request line (method and path). Query strings are stripped
/// from the path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestLine {
    /// HTTP method, uppercased (for example `GET`).
    pub method: String,
    /// Request path with any query string removed (for example `/summary`).
    pub path: String,
}

/// Parse the first line of an HTTP request (for example `GET /events HTTP/1.1`).
///
/// Returns `None` when the line does not have at least a method and a target.
pub fn parse_request_line(line: &str) -> Option<RequestLine> {
    let mut parts = line.split_whitespace();
    let method = parts.next()?.to_uppercase();
    let target = parts.next()?;
    let path = match target.split_once('?') {
        Some((p, _query)) => p,
        None => target,
    };
    Some(RequestLine {
        method,
        path: path.to_string(),
    })
}

/// Format a complete HTTP response (status line, headers, blank line, body)
/// for a fixed length body.
pub fn format_response(status: u16, reason: &str, content_type: &str, body: &[u8]) -> Vec<u8> {
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\n\
         Content-Type: {content_type}\r\n\
         Content-Length: {len}\r\n\
         Cache-Control: no-cache\r\n\
         Connection: close\r\n\
         \r\n",
        len = body.len(),
    );
    let mut out = header.into_bytes();
    out.extend_from_slice(body);
    out
}

/// Format the response headers for a Server Sent Events stream. The body is
/// then written incrementally as SSE messages and the connection stays open.
pub fn format_sse_headers() -> Vec<u8> {
    "HTTP/1.1 200 OK\r\n\
     Content-Type: text/event-stream\r\n\
     Cache-Control: no-cache\r\n\
     Connection: keep-alive\r\n\
     \r\n"
        .to_string()
        .into_bytes()
}

/// Format a single SSE message: `data: <payload>` followed by a blank line.
///
/// If the payload contains newlines each line is prefixed with `data: ` as
/// required by the SSE specification.
pub fn format_sse_message(payload: &str) -> Vec<u8> {
    let mut out = String::new();
    for line in payload.split('\n') {
        out.push_str("data: ");
        out.push_str(line);
        out.push('\n');
    }
    out.push('\n');
    out.into_bytes()
}

/// Convenience helper for a `404 Not Found` plain text response.
pub fn not_found() -> Vec<u8> {
    format_response(404, "Not Found", "text/plain; charset=utf-8", b"Not Found")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_request_line() {
        let parsed = parse_request_line("GET /summary HTTP/1.1").unwrap();
        assert_eq!(parsed.method, "GET");
        assert_eq!(parsed.path, "/summary");
    }

    #[test]
    fn strips_query_string() {
        let parsed = parse_request_line("GET /events?replay=1 HTTP/1.1").unwrap();
        assert_eq!(parsed.path, "/events");
    }

    #[test]
    fn uppercases_method() {
        let parsed = parse_request_line("get / HTTP/1.1").unwrap();
        assert_eq!(parsed.method, "GET");
        assert_eq!(parsed.path, "/");
    }

    #[test]
    fn rejects_empty_line() {
        assert!(parse_request_line("").is_none());
        assert!(parse_request_line("GET").is_none());
    }

    #[test]
    fn formats_response_with_headers_and_body() {
        let response = format_response(200, "OK", "application/json", b"{\"ok\":true}");
        let text = String::from_utf8(response).unwrap();
        assert!(text.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(text.contains("Content-Type: application/json\r\n"));
        assert!(text.contains("Content-Length: 11\r\n"));
        assert!(text.ends_with("\r\n\r\n{\"ok\":true}"));
    }

    #[test]
    fn sse_headers_declare_event_stream() {
        let text = String::from_utf8(format_sse_headers()).unwrap();
        assert!(text.contains("Content-Type: text/event-stream\r\n"));
    }

    #[test]
    fn sse_message_prefixes_each_line() {
        let msg = String::from_utf8(format_sse_message("{\"a\":1}")).unwrap();
        assert_eq!(msg, "data: {\"a\":1}\n\n");
    }

    #[test]
    fn sse_message_handles_multiline_payload() {
        let msg = String::from_utf8(format_sse_message("line1\nline2")).unwrap();
        assert_eq!(msg, "data: line1\ndata: line2\n\n");
    }

    #[test]
    fn not_found_is_404() {
        let text = String::from_utf8(not_found()).unwrap();
        assert!(text.starts_with("HTTP/1.1 404 Not Found\r\n"));
    }
}
