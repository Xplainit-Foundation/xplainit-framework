//! Xplainit Dashboard server.
//!
//! A single binary that loads a trace JSON file (an array of
//! `xplainit_core::ExecutionEvent`) and serves an interactive dashboard for
//! real time trace visualization: a timeline, a call graph, and an aggregated
//! error summary.
//!
//! The server is built on raw tokio TCP primitives. No external web framework
//! is used (none is available in this sandbox). The streaming endpoint uses
//! Server Sent Events (SSE), which needs no handshake and no extra dependency.
//!
//! Endpoints:
//! - `GET /`           the dashboard page (index.html)
//! - `GET /style.css`  stylesheet
//! - `GET /app.js`     frontend script
//! - `GET /summary`    aggregated error summary (JSON)
//! - `GET /callgraph`  derived call graph (JSON)
//! - `GET /timeline`   derived timeline (JSON)
//! - `GET /tasks`      per async-task lifecycle view (JSON)
//! - `GET /events`     SSE stream of the trace events

mod analysis;
mod http;
mod trace;

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use xplainit_core::ExecutionEvent;

/// Maximum number of request-head bytes we will buffer before giving up. This
/// bounds memory per connection and rejects abusive/oversized header blocks.
const MAX_REQUEST_HEAD_BYTES: usize = 64 * 1024;

/// How long a single connection may take to send its request head before we
/// treat it as idle and drop it. Prevents slow-loris style resource pinning.
const READ_TIMEOUT: Duration = Duration::from_secs(15);

const INDEX_HTML: &str = include_str!("../static/index.html");
const STYLE_CSS: &str = include_str!("../static/style.css");
const APP_JS: &str = include_str!("../static/app.js");

/// Command line arguments for the dashboard server.
#[derive(Parser, Debug)]
#[command(
    name = "xplainit-dashboard",
    about = "Serve an interactive dashboard for an Xplainit trace",
    version
)]
struct Args {
    /// Path to the trace JSON file (an array of ExecutionEvent).
    trace: String,

    /// Address to bind the HTTP server to.
    #[arg(long, default_value = "127.0.0.1:8080")]
    bind: String,

    /// Delay in milliseconds between streamed events (for replay pacing).
    #[arg(long, default_value_t = 0)]
    stream_delay_ms: u64,
}

/// Shared, read only server state.
struct AppState {
    events: Vec<ExecutionEvent>,
    stream_delay: Duration,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let events = trace::load_events(&args.trace)?;
    println!("Loaded {} event(s) from {}", events.len(), args.trace);

    let state = Arc::new(AppState {
        events,
        stream_delay: Duration::from_millis(args.stream_delay_ms),
    });

    let listener = TcpListener::bind(&args.bind)
        .await
        .with_context(|| format!("failed to bind to {}", args.bind))?;
    println!("Dashboard listening on http://{}", args.bind);

    loop {
        let (stream, _addr) = listener.accept().await?;
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            if let Err(err) = handle_connection(stream, state).await {
                eprintln!("connection error: {err}");
            }
        });
    }
}

/// Outcome of reading the request head from a connection.
enum RequestHead {
    /// The full request head (up to and including the blank line) was read.
    Complete(Vec<u8>),
    /// The client closed the connection before sending any bytes.
    Empty,
    /// The head exceeded `MAX_REQUEST_HEAD_BYTES` without terminating.
    TooLarge,
}

/// Read from `stream` until the end of the HTTP request head (the `\r\n\r\n`
/// separating headers from the body), accumulating across multiple reads.
///
/// This replaces a single fixed-size read so requests split across TCP
/// segments, or with headers larger than one read buffer, are parsed correctly.
/// Reading is bounded by `MAX_REQUEST_HEAD_BYTES` so a client cannot exhaust
/// memory by streaming an unbounded header block.
async fn read_request_head<R>(reader: &mut R) -> Result<RequestHead>
where
    R: AsyncRead + Unpin,
{
    let mut buf: Vec<u8> = Vec::with_capacity(1024);
    let mut chunk = [0u8; 4096];

    loop {
        let n = reader.read(&mut chunk).await?;
        if n == 0 {
            // Connection closed. If we have not read anything it is simply an
            // empty/aborted connection; otherwise return what we have so the
            // caller can still attempt to parse the request line.
            if buf.is_empty() {
                return Ok(RequestHead::Empty);
            }
            return Ok(RequestHead::Complete(buf));
        }

        buf.extend_from_slice(&chunk[..n]);

        // End of headers reached.
        if find_head_end(&buf).is_some() {
            return Ok(RequestHead::Complete(buf));
        }

        if buf.len() > MAX_REQUEST_HEAD_BYTES {
            return Ok(RequestHead::TooLarge);
        }
    }
}

/// Return the index just past the `\r\n\r\n` (or `\n\n`) that ends the head.
fn find_head_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|i| i + 4)
        .or_else(|| buf.windows(2).position(|w| w == b"\n\n").map(|i| i + 2))
}

/// Read the request, route on the request line, and write the response.
async fn handle_connection(mut stream: TcpStream, state: Arc<AppState>) -> Result<()> {
    let head = match tokio::time::timeout(READ_TIMEOUT, read_request_head(&mut stream)).await {
        Ok(result) => result?,
        Err(_elapsed) => {
            // Idle/slow client: drop the connection rather than pinning the task.
            return Ok(());
        }
    };

    let buf = match head {
        RequestHead::Complete(buf) => buf,
        RequestHead::Empty => return Ok(()),
        RequestHead::TooLarge => {
            stream
                .write_all(&http::format_response(
                    431,
                    "Request Header Fields Too Large",
                    "text/plain; charset=utf-8",
                    b"Request Header Fields Too Large",
                ))
                .await?;
            return Ok(());
        }
    };

    let request = String::from_utf8_lossy(&buf);
    let first_line = request.lines().next().unwrap_or("");
    let parsed = match http::parse_request_line(first_line) {
        Some(parsed) => parsed,
        None => {
            stream.write_all(&http::not_found()).await?;
            return Ok(());
        }
    };

    if parsed.method != "GET" {
        stream
            .write_all(&http::format_response(
                405,
                "Method Not Allowed",
                "text/plain; charset=utf-8",
                b"Method Not Allowed",
            ))
            .await?;
        return Ok(());
    }

    match parsed.path.as_str() {
        "/" | "/index.html" => {
            let body =
                http::format_response(200, "OK", "text/html; charset=utf-8", INDEX_HTML.as_bytes());
            stream.write_all(&body).await?;
        }
        "/style.css" => {
            let body =
                http::format_response(200, "OK", "text/css; charset=utf-8", STYLE_CSS.as_bytes());
            stream.write_all(&body).await?;
        }
        "/app.js" => {
            let body = http::format_response(
                200,
                "OK",
                "application/javascript; charset=utf-8",
                APP_JS.as_bytes(),
            );
            stream.write_all(&body).await?;
        }
        "/summary" => {
            let summary = analysis::build_summary(&state.events);
            write_json(&mut stream, &summary).await?;
        }
        "/callgraph" => {
            let nodes = analysis::build_call_graph(&state.events);
            let graph = analysis::call_graph_json(&nodes);
            write_json(&mut stream, &graph).await?;
        }
        "/timeline" => {
            let timeline = analysis::build_timeline(&state.events);
            write_json(&mut stream, &timeline).await?;
        }
        "/tasks" => {
            let tasks = analysis::build_task_view(&state.events);
            write_json(&mut stream, &tasks).await?;
        }
        "/events" => {
            stream_events(&mut stream, &state).await?;
        }
        _ => {
            stream.write_all(&http::not_found()).await?;
        }
    }

    stream.flush().await?;
    Ok(())
}

/// Write a JSON value as a fixed length HTTP response.
async fn write_json(stream: &mut TcpStream, value: &serde_json::Value) -> Result<()> {
    let body = serde_json::to_vec(value)?;
    let response = http::format_response(200, "OK", "application/json", &body);
    stream.write_all(&response).await?;
    Ok(())
}

/// Stream the trace events as Server Sent Events, one message per event.
async fn stream_events(stream: &mut TcpStream, state: &AppState) -> Result<()> {
    stream.write_all(&http::format_sse_headers()).await?;

    for (index, event) in state.events.iter().enumerate() {
        let payload = analysis::event_stream_payload(index, event);
        let serialized = serde_json::to_string(&payload)?;
        stream
            .write_all(&http::format_sse_message(&serialized))
            .await?;
        stream.flush().await?;

        if !state.stream_delay.is_zero() {
            tokio::time::sleep(state.stream_delay).await;
        }
    }

    // Signal completion so the client stops reconnecting.
    stream
        .write_all(&http::format_sse_message("{\"done\":true}"))
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::pin::Pin;
    use std::task::{Context as TaskContext, Poll};
    use tokio::io::ReadBuf;

    /// A minimal in-memory `AsyncRead` that yields a queued sequence of chunks,
    /// one per `poll_read`. When `pending_forever` is set and the queue is
    /// empty it returns `Poll::Pending` (never completing) so the read timeout
    /// path can be exercised without depending on an external test crate.
    struct ChunkReader {
        chunks: VecDeque<Vec<u8>>,
        pending_forever: bool,
    }

    impl ChunkReader {
        fn new(chunks: Vec<Vec<u8>>) -> Self {
            Self {
                chunks: chunks.into(),
                pending_forever: false,
            }
        }

        fn pending_after(chunks: Vec<Vec<u8>>) -> Self {
            Self {
                chunks: chunks.into(),
                pending_forever: true,
            }
        }
    }

    impl AsyncRead for ChunkReader {
        fn poll_read(
            mut self: Pin<&mut Self>,
            _cx: &mut TaskContext<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            match self.chunks.pop_front() {
                Some(chunk) => {
                    let n = chunk.len().min(buf.remaining());
                    buf.put_slice(&chunk[..n]);
                    // If the read buffer could not hold the whole chunk, push
                    // the remainder back so the next poll delivers it. This
                    // mirrors a real socket where a large payload is delivered
                    // across several reads.
                    if n < chunk.len() {
                        self.chunks.push_front(chunk[n..].to_vec());
                    }
                    Poll::Ready(Ok(()))
                }
                None if self.pending_forever => Poll::Pending,
                // Empty read signals EOF (connection closed).
                None => Poll::Ready(Ok(())),
            }
        }
    }

    #[tokio::test]
    async fn reads_head_split_across_reads() {
        // A request whose head arrives in several small chunks must be
        // accumulated into one buffer, not parsed from the first read alone.
        let request = b"GET /summary HTTP/1.1\r\nHost: localhost\r\nAccept: */*\r\n\r\n";
        let mut reader = ChunkReader::new(vec![
            request[..10].to_vec(),
            request[10..25].to_vec(),
            request[25..].to_vec(),
        ]);

        match read_request_head(&mut reader).await.unwrap() {
            RequestHead::Complete(buf) => {
                let text = String::from_utf8_lossy(&buf);
                assert!(text.starts_with("GET /summary HTTP/1.1\r\n"));
                assert!(find_head_end(&buf).is_some());
            }
            _ => panic!("expected a complete request head"),
        }
    }

    #[tokio::test]
    async fn empty_connection_is_reported() {
        let mut reader = ChunkReader::new(vec![]);
        assert!(matches!(
            read_request_head(&mut reader).await.unwrap(),
            RequestHead::Empty
        ));
    }

    #[tokio::test]
    async fn oversized_head_is_rejected() {
        // A never-terminating header block larger than the cap must be refused
        // rather than buffered without bound.
        let oversized = vec![b'a'; MAX_REQUEST_HEAD_BYTES + 4096];
        let mut reader = ChunkReader::new(vec![oversized]);
        assert!(matches!(
            read_request_head(&mut reader).await.unwrap(),
            RequestHead::TooLarge
        ));
    }

    #[tokio::test]
    async fn read_timeout_fires_for_idle_client() {
        // A client that sends a partial head and then goes silent must hit the
        // read timeout instead of pinning the task. Use a tiny timeout so the
        // test stays fast.
        let mut reader = ChunkReader::pending_after(vec![b"GET / HTTP/1.1\r\n".to_vec()]);
        let result =
            tokio::time::timeout(Duration::from_millis(50), read_request_head(&mut reader)).await;
        assert!(result.is_err(), "expected the read to time out");
    }

    #[test]
    fn find_head_end_detects_crlf_and_lf() {
        assert_eq!(find_head_end(b"GET / HTTP/1.1\r\n\r\nbody"), Some(18));
        assert_eq!(find_head_end(b"GET / HTTP/1.1\n\nbody"), Some(16));
        assert_eq!(find_head_end(b"GET / HTTP/1.1\r\n"), None);
    }
}
