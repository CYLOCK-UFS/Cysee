use std::fs::{OpenOptions, read_to_string};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

const CENTRAL_DB_PATH: &str = "/var/lib/cysee/central_events.log";

fn main() {
    let _ = OpenOptions::new().create(true).append(true).open(CENTRAL_DB_PATH);

    // Thread 1: Ingest Gateway na porta 50051
    thread::spawn(|| {
        let listener = TcpListener::bind("0.0.0.0:50051").expect("Falha ao abrir porta 50051");
        println!("[central] Ingest Gateway (Rust) escutando na TCP 50051...");
        for stream in listener.incoming().flatten() {
            let mut s = stream;
            let mut buf = String::new();
            if s.read_to_string(&mut buf).is_ok() && !buf.is_empty() {
                let mut file = OpenOptions::new().create(true).append(true).open(CENTRAL_DB_PATH).unwrap();
                let _ = file.write_all(buf.as_bytes());
                let _ = s.write_all(b"ACK");
                println!("[central] Recebido lote:\n{}", buf.trim());
            }
        }
    });

    // Thread 2: API REST simples na porta 8080 para Gerência / Grafana
    let api_listener = TcpListener::bind("0.0.0.0:8080").expect("Falha ao abrir porta 8080");
    println!("[central] API REST (Rust) escutando na TCP 8080...");
    for stream in api_listener.incoming().flatten() {
        let mut s = stream;
        let mut req_buf = [0u8; 1024];
        let _ = s.read(&mut req_buf);
        let content = read_to_string(CENTRAL_DB_PATH).unwrap_or_default();
        let lines: Vec<&str> = content.lines().collect();
        let json_body = format!("{{\"total_events\": {}, \"events\": {:?}}}", lines.len(), lines);
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            json_body.len(),
            json_body
        );
        let _ = s.write_all(response.as_bytes());
    }
}