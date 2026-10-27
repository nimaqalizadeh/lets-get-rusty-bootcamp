// =====================================================================
//  A tiny HTTP server, written by hand, annotated layer by layer.
//
//  Every section is tagged with who does this job in a real Axum app:
//
//    [OS]     the kernel: sockets, file descriptors, syscalls
//    [TOKIO]  the async runtime: non-blocking sockets, tasks, scheduling
//    [HYPER]  the HTTP library: parsing requests, writing responses
//    [TOWER]  middleware layer (timeouts, logging, compression...)
//    [AXUM]   routing, extractors, handlers, IntoResponse
//    [TOWER-HTTP ServeDir]  static file serving
//
//  Run:   rustc main.rs && ./main      (or put it in a cargo project)
//  Open:  http://127.0.0.1:7878
// =====================================================================

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

// ---------------------------------------------------------------------
//  The "request" our parser produces.
//
//  [HYPER] equivalent: `http::Request<Incoming>`. Hyper gives Axum a
//  structured request: method, URI, version, a HeaderMap, and a body
//  stream. Axum's extractors (Path, Query, Json, HeaderMap...) all read
//  from this struct. Here we keep only what we need.
// ---------------------------------------------------------------------
struct Request {
    method: String,
    path: String,
    version: String,
}

// ---------------------------------------------------------------------
//  The "response" a handler produces.
//
//  [AXUM] equivalent: anything implementing `IntoResponse`. A handler in
//  Axum can return a String, Html<...>, Json<...>, (StatusCode, body),
//  etc. Axum converts it into `http::Response<Body>`, and Hyper turns
//  that into bytes on the socket. Here we build it by hand.
// ---------------------------------------------------------------------
struct Response {
    status: &'static str,       // e.g. "200 OK"
    content_type: &'static str, // e.g. "text/html; charset=utf-8"
    body: Vec<u8>,              // bytes, not String, so images etc. also work
}

/// Handles a single client connection: parse one request, route it,
/// send one response, close.
///
/// [TOKIO + HYPER] equivalent: for every accepted socket, axum::serve
/// does `tokio::spawn(async move { hyper::serve_connection(socket, app) })`.
/// Hyper then loops: parse request -> call the Axum router -> write the
/// response -> repeat on the same socket (keep-alive) until it closes.
fn handle_http_connection(mut stream: TcpStream) -> std::io::Result<()> {
    // [OS] The socket is a file descriptor. peer_addr() is the
    // getpeername() syscall: who is on the other end.
    let client_addr = stream
        .peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    println!("Handling connection from {}", client_addr);

    // [OS] Reading from a socket is the read()/recv() syscall, same as
    // reading a file. BufReader reads big chunks into memory so we don't
    // make one syscall per byte.
    //
    // [HYPER] Hyper also keeps a read buffer, but it parses headers in
    // place (zero-copy via the `httparse` crate) instead of copying each
    // line into a new String like we do.
    let mut reader = BufReader::new(&stream);

    // ================================================================
    //  1. READ AND PARSE THE REQUEST LINE   e.g. "GET /about.html HTTP/1.1"
    //
    //  [HYPER] does this. It also rejects malformed lines with
    //  400 Bad Request, and caps line length so a client can't send
    //  gigabytes without a newline.
    // ================================================================
    let mut request_line = String::new();
    match reader.read_line(&mut request_line) {
        Ok(0) => return Ok(()), // client connected and closed without sending anything
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to read request line from {}: {}", client_addr, e);
            return Ok(());
        }
    }
    println!("Request from {}: {}", client_addr, request_line.trim());

    // Split "GET /about.html HTTP/1.1" into its three parts.
    let mut parts = request_line.split_whitespace();
    let request = match (parts.next(), parts.next(), parts.next()) {
        (Some(m), Some(p), Some(v)) => Request {
            method: m.to_string(),
            path: p.to_string(),
            version: v.to_string(),
        },
        _ => {
            // [HYPER] would answer 400 Bad Request automatically.
            let resp = Response {
                status: "400 Bad Request",
                content_type: "text/plain; charset=utf-8",
                body: b"Bad Request".to_vec(),
            };
            return send_response(&mut stream, &client_addr, resp);
        }
    };

    // ================================================================
    //  2. READ (AND IGNORE) THE HEADERS
    //
    //  Headers are "Name: value\r\n" lines, ending with an empty line.
    //  We must consume them even if unused.
    //
    //  [HYPER] parses them into a `HeaderMap` (case-insensitive keys),
    //  limits their total size and count (protection against abuse),
    //  and uses Content-Length / Transfer-Encoding to know how to read
    //  the BODY that follows. We never read a body, so POST requests
    //  with data would not work here.
    //
    //  [AXUM] lets a handler see them with the `HeaderMap` or
    //  `TypedHeader` extractors.
    // ================================================================
    let mut header_line = String::new();
    loop {
        header_line.clear();
        match reader.read_line(&mut header_line) {
            Ok(0) => {
                eprintln!("Client disconnected during header read.");
                return Ok(());
            }
            Ok(_) => {
                if header_line.trim().is_empty() {
                    break; // blank line = end of headers
                }
                // ignored in this simple server
            }
            Err(e) => {
                eprintln!("Error reading headers: {}", e);
                return Err(e);
            }
        }
    }

    // ================================================================
    //  3. ROUTING: decide which "handler" runs for this request.
    //
    //  [AXUM] equivalent:
    //
    //      Router::new()
    //          .route("/",           get(index))
    //          .route("/index.html", get(index))
    //          .route("/about.html", get(about))
    //          .fallback(not_found);
    //
    //  Axum stores routes in a radix tree (the `matchit` crate), so
    //  lookup is fast no matter how many routes exist, and it supports
    //  parameters like "/users/:id" which the `Path` extractor reads.
    //  It also routes on METHOD separately: a POST to a GET-only route
    //  gets 405 Method Not Allowed instead of 404.
    // ================================================================
    let response = if request.method != "GET" {
        Response {
            status: "405 Method Not Allowed",
            content_type: "text/plain; charset=utf-8",
            body: b"Method Not Allowed".to_vec(),
        }
    } else {
        match request.path.as_str() {
            "/" | "/index.html" => serve_file("index.html", "200 OK"),
            "/about.html" => serve_file("about.html", "200 OK"),
            _ => not_found(),
        }
    };

    let _ = &request.version; // a real server would check HTTP/1.0 vs 1.1 here

    // ================================================================
    //  4. SEND THE RESPONSE
    // ================================================================
    send_response(&mut stream, &client_addr, response)?;
    println!("Response sent to {}. Closing connection.", client_addr);

    // [OS] When `stream` is dropped at the end of this function, Rust
    // calls close() on the file descriptor and the TCP connection ends.
    //
    // [HYPER] would NOT close here: HTTP/1.1 is keep-alive by default,
    // so Hyper goes back to step 1 and waits for the next request on
    // the same socket. That saves a TCP handshake per request.
    Ok(())
}

// ---------------------------------------------------------------------
//  "HANDLERS": functions that produce a response.
//
//  [AXUM] a handler is just an async fn:
//
//      async fn about() -> Html<String> { ... }
//
//  For static files you wouldn't write these at all; you'd use
//  [TOWER-HTTP]  `.fallback_service(ServeDir::new("static"))`, which:
//    - reads files with tokio::fs (async, doesn't block the thread)
//    - STREAMS large files in chunks instead of loading them into memory
//    - guesses Content-Type from the extension (.png -> image/png)
//    - sets ETag / Last-Modified so browsers can cache
//    - supports Range requests (needed for video seeking)
//    - rejects path traversal like "/../../etc/passwd"
//
//  If the page needed DATABASE data, the handler is where it happens:
//
//      async fn user(Path(id): Path<i64>, State(db): State<PgPool>)
//          -> Json<User>
//      {
//          let user = sqlx::query_as!(User, "SELECT ... WHERE id = $1", id)
//              .fetch_one(&db).await.unwrap();
//          Json(user)
//      }
//
//  Axum itself does nothing special for databases: the handler awaits
//  the query, and Axum turns the return value into a response.
// ---------------------------------------------------------------------

/// Read a file from disk and wrap it in a response; fall back to 404.
fn serve_file(filename: &str, status: &'static str) -> Response {
    // [OS] fs::read is open() + read() + close() on a file descriptor,
    // the same kind of object as the socket. "Everything is a file."
    //
    // We use fs::read (bytes) instead of read_to_string: read_to_string
    // fails on anything that isn't valid UTF-8, like images.
    match fs::read(filename) {
        Ok(body) => Response {
            status,
            content_type: content_type_for(filename),
            body,
        },
        Err(_) => {
            println!("File '{}' not found. Sending 404.", filename);
            not_found()
        }
    }
}

/// [AXUM] equivalent: the `.fallback(...)` handler on the Router.
fn not_found() -> Response {
    let body = fs::read("404.html").unwrap_or_else(|_| {
        b"<h1>404 Not Found</h1><p>The requested resource was not found.</p>".to_vec()
    });
    Response {
        status: "404 Not Found",
        content_type: "text/html; charset=utf-8",
        body,
    }
}

/// [TOWER-HTTP] ServeDir uses the `mime_guess` crate for this.
fn content_type_for(filename: &str) -> &'static str {
    match filename.rsplit('.').next() {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        _ => "application/octet-stream",
    }
}

// ---------------------------------------------------------------------
//  Serialize a Response into raw HTTP bytes and write it to the socket.
//
//  [HYPER] does this from the http::Response that Axum returns. It
//  writes the status line and headers, then the body (with
//  Content-Length, or chunked encoding if the length is unknown, e.g.
//  a streamed file). It also adds headers like `Date` automatically.
// ---------------------------------------------------------------------
fn send_response(stream: &mut TcpStream, client_addr: &str, resp: Response) -> std::io::Result<()> {
    // HTTP response format:
    //   HTTP/1.1 200 OK\r\n
    //   Header: value\r\n
    //   ...\r\n
    //   \r\n            <- blank line ends the headers
    //   <body bytes>
    let head = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        resp.status,
        resp.content_type,
        resp.body.len() // length in BYTES, which is what HTTP expects
    );

    // [OS] write_all loops over the write()/send() syscall until every
    // byte is handed to the kernel's TCP send buffer. The kernel then
    // sends it over the network on its own schedule.
    //
    // [TOKIO] uses non-blocking writes: if the send buffer is full, the
    // task yields and the thread serves other connections until the
    // OS (via epoll/kqueue) says the socket is writable again.
    if let Err(e) = stream
        .write_all(head.as_bytes())
        .and_then(|_| stream.write_all(&resp.body))
        .and_then(|_| stream.flush())
    {
        eprintln!("Error sending response to {}: {}", client_addr, e);
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    // --- Setup: create the HTML files the server will serve ---
    // [AXUM] you'd keep these in a "static/" folder and point ServeDir at it.
    fs::write(
        "index.html",
        "<h1>Welcome!</h1><p>This is the main page.</p><p><a href=\"/about.html\">About Us</a></p>",
    )?;
    fs::write(
        "about.html",
        "<h1>About Us</h1><p>We are a Rust learning example!</p><p><a href=\"/\">Home</a></p>",
    )?;
    fs::write(
        "404.html",
        "<h1>404 - Page Not Found</h1><p>Sorry, the page you are looking for does not exist.</p><p><a href=\"/\">Go Home</a></p>",
    )?;

    // ================================================================
    //  BIND AND LISTEN
    //
    //  [OS] TcpListener::bind performs three syscalls:
    //    socket()  -> create a file descriptor for a TCP socket
    //    bind()    -> attach it to 127.0.0.1:7878
    //    listen()  -> tell the kernel to queue incoming connections
    //  The kernel does the TCP 3-way handshake by itself and puts
    //  finished connections in a queue (the "backlog").
    //
    //  [TOKIO] equivalent: tokio::net::TcpListener::bind(addr).await,
    //  same syscalls, but the socket is set to NON-BLOCKING mode.
    // ================================================================
    let listener_address = "127.0.0.1:7878";
    let listener = TcpListener::bind(listener_address)?;
    println!("Simple HTTP Server listening on http://{}", listener_address);

    // ================================================================
    //  ACCEPT LOOP
    //
    //  [OS] incoming() calls accept(), which BLOCKS until a connection
    //  is in the backlog, then returns a NEW file descriptor just for
    //  that client. The listener keeps listening.
    //
    //  Our loop is sequential: while we serve client A, client B waits.
    //
    //  [TOKIO + AXUM] equivalent:
    //
    //      axum::serve(listener, app).await
    //
    //  which internally is roughly:
    //
    //      loop {
    //          let (socket, _) = listener.accept().await?; // doesn't block the thread
    //          tokio::spawn(async move {                    // each client = cheap task
    //              hyper::serve_connection(socket, router).await;
    //          });
    //      }
    //
    //  Tokio asks the OS (epoll on Linux, kqueue on macOS, IOCP on
    //  Windows) "tell me which of these thousands of sockets are ready",
    //  and runs only the tasks that can make progress. A few threads can
    //  then handle tens of thousands of connections at once.
    // ================================================================
    for stream_result in listener.incoming() {
        match stream_result {
            Ok(stream) => {
                println!("Main: Accepted new connection. Handling...");
                if let Err(e) = handle_http_connection(stream) {
                    eprintln!("Connection error: {}", e);
                }
            }
            Err(e) => eprintln!("Main: Failed to accept connection: {}", e),
        }
    }

    // Never reached: the loop above runs forever.
    // [AXUM] uses `.with_graceful_shutdown(signal)` to stop cleanly on Ctrl+C.
    fs::remove_file("index.html")?;
    fs::remove_file("about.html")?;
    fs::remove_file("404.html")?;
    Ok(())
}

// =====================================================================
//  THE SAME SERVER IN AXUM (Cargo.toml: axum, tokio with "full",
//  tower-http with "fs"):
//
//      use axum::Router;
//      use tower_http::services::ServeDir;
//
//      #[tokio::main]
//      async fn main() {
//          // static/ contains index.html, about.html
//          let app = Router::new()
//              .fallback_service(
//                  ServeDir::new("static")
//                      .not_found_service(ServeFile::new("static/404.html")),
//              );
//
//          let listener = tokio::net::TcpListener::bind("127.0.0.1:7878")
//              .await
//              .unwrap();
//          axum::serve(listener, app).await.unwrap();
//      }
//
//  ServeDir maps "/" to "index.html" automatically. Everything this
//  file does by hand (accept, parse, route, read file, write response)
//  happens inside those few lines, spread across Tokio, Hyper, Tower
//  and Axum.
// =====================================================================