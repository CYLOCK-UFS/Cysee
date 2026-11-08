use std::env;
use std::fs::{OpenOptions, read_to_string, write};
use std::io::{Read, Write};
use std::net::{TcpStream, UdpSocket};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const OUTBOX_PATH: &str = "/var/lib/cysee/outbox_pending.log";
const ACKED_PATH: &str = "/var/lib/cysee/outbox_acked.log";

fn main() {
    let node_id = env::var("NODE_ID").unwrap_or_else(|_| "unit-a".to_string());
    let central_addr = env::var("CENTRAL_ADDR").unwrap_or_else(|_| "172.20.101.100:50051".to_string());

    // Garante que os arquivos de persistência existam no volume
    let _ = OpenOptions::new().create(true).append(true).open(OUTBOX_PATH);
    let _ = OpenOptions::new().create(true).append(true).open(ACKED_PATH);

    // Thread Sender: tenta enviar eventos pendentes ao Central a cada 500ms (ADR-0005)
    let sender_node = node_id.clone();
    let sender_central = central_addr.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(500));
            if let Ok(content) = read_to_string(OUTBOX_PATH) {
                if !content.trim().is_empty() {
                    match TcpStream::connect_timeout(
                        &sender_central.parse().unwrap(),
                        Duration::from_secs(2),
                    ) {
                        Ok(mut stream) => {
                            if stream.write_all(content.as_bytes()).is_ok() {
                                let mut resp = [0u8; 16];
                                if let Ok(n) = stream.read(&mut resp) {
                                    if &resp[..n] == b"ACK" {
                                        let mut ack_file = OpenOptions::new()
                                            .create(true).append(true).open(ACKED_PATH).unwrap();
                                        let _ = ack_file.write_all(content.as_bytes());
                                        let _ = write(OUTBOX_PATH, ""); // Limpa pendentes após ACK
                                        println!("[{}] Lote confirmado (ACK) pelo Central!", sender_node);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!("[{}] Central inacessivel ({}). Acumulando no outbox...", sender_node, e);
                            thread::sleep(Duration::from_secs(1));
                        }
                    }
                }
            }
        }
    });

    // Trap Receiver principal na porta UDP 162
    let socket = UdpSocket::bind("0.0.0.0:162").expect("Falha ao abrir UDP 162");
    println!("[{}] cysee-agent (Rust) escutando traps SNMP na UDP 162...", node_id);

    let mut buf = [0u8; 2048];
    let mut seq: u64 = 1;
    loop {
        if let Ok((amt, src)) = socket.recv_from(&mut buf) {
            let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let event_line = format!("node={} seq={} src={} bytes={} ts={}\n", node_id, seq, src, amt, ts);

            let mut file = OpenOptions::new().create(true).append(true).open(OUTBOX_PATH).unwrap();
            file.write_all(event_line.as_bytes()).unwrap();

            println!("[{}] Trap recebido na UDP 162 e gravado no outbox: seq={} de {}", node_id, seq, src);
            seq += 1;
        }
    }
}