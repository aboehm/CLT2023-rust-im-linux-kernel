// SPDX-License-Identifier: GPL-2.0

//! Rust simple http server.

use kernel::{
    kasync::executor::{workqueue::Executor as WqExecutor, AutoStopHandle, Executor},
    kasync::net::{TcpListener, TcpStream},
    net::{self, Ipv4Addr, SocketAddr, SocketAddrV4},
    prelude::*,
    spawn_task,
    str::CString,
    sync::{Arc, ArcBorrow},
};

enum RequestType {
    Echo,
    Status,
    Other,
}

fn parse_request(buf: &[u8]) -> Result<Option<RequestType>> {
    let request = core::str::from_utf8(&buf)?;
    pr_info!("Got request: {request}");
    let mut parts = request.split(' ');

    let path = if let Some(p) = parts.nth(1) { p } else { return Ok(None); };

    let res = match path {
        "/echo"   => RequestType::Echo,
        "/status" => RequestType::Status,
        _         => RequestType::Other,
    };
    Ok(Some(res))
}

async fn write_http_response(status_code: u32, text: &str, stream: &TcpStream) -> Result {
    let text_size = text.len();
    let buf = CString::try_from_fmt(fmt!(r#"HTTP/1.1 {status_code}
Server: Linux Kernel
Content-Length: {text_size}
Content-Type: text/plain
Connection: Closed

{text}"#))?;
    stream.write_all(&buf).await?;
    Ok(())
}

async fn http_server(stream: TcpStream) -> Result {
    pr_info!("New connection");
    let mut buf = [0u8; 128];

    pr_info!("Reading request");
    let buf = {
        let n = stream.read(&mut buf).await?;
        pr_info!("Read {n} bytes from request");
        match n {
            n if n > 1024 =>  return write_http_response(400, "Request to big", &stream).await,
            n if n > 10 => &buf[..n],
            _ =>  return write_http_response(400, "Request too small", &stream).await,
        }
    };

    pr_info!("Replying");
    match parse_request(&buf)? {
        Some(RequestType::Echo)   => write_http_response(403, "not implemented", &stream).await,
        Some(RequestType::Status) => write_http_response(200, "alive", &stream).await,
        Some(RequestType::Other)  => write_http_response(404, "", &stream).await,
        _ => Ok(()),
    }
}

async fn accept_loop(listener: TcpListener, executor: Arc<impl Executor>) {
    loop {
        if let Ok(stream) = listener.accept().await {
            let _ = spawn_task!(executor.as_arc_borrow(), http_server(stream));
        }
    }
}

fn start_listener(ex: ArcBorrow<'_, impl Executor + Send + Sync + 'static>) -> Result {
    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::ANY, 8000));
    let listener = TcpListener::try_new(net::init_ns(), &addr)?;
    spawn_task!(ex, accept_loop(listener, ex.into()))?;
    Ok(())
}

struct RustHttpServer {
    _handle: AutoStopHandle<dyn Executor>,
}

impl kernel::Module for RustHttpServer {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        let handle = WqExecutor::try_new(kernel::workqueue::system())?;
        start_listener(handle.executor())?;
        Ok(Self {
            _handle: handle.into(),
        })
    }
}

module! {
    type: RustHttpServer,
    name: "rust_http_server",
    author: "Alexander Böhm",
    description: "Rust http server sample",
    license: "GPL v2",
}
