use std::env;
use std::io::{stdin, stdout, Write};
use std::process::Command;

#[derive(Debug)]
struct Station {
    name: String,
    scanning: bool,
}

fn main() {
    let stations = get_network_stations();
    let station: &Station = stations.first().unwrap();

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "disconnect" => {
                Command::new("iwctl")
                    .arg("station")
                    .arg(station.name.clone())
                    .arg("disconnect")
                    .output()
                    .expect("Error");
            }
            _ => {}
        }
    } else {
        connect_process();
    }
}

fn connect_process() {
    let binding = get_network_stations();
    if binding.is_empty() {
        return println!("No stations found");
    }
    let station: &Station = if binding.len() == 1 {
        &binding[0]
    } else {
        for (i, station) in binding.iter().enumerate() {
            println!("{}: {}", i, station.name);
        }

        let input = take_input("Select your station");
        let number: usize = input.parse::<usize>().unwrap_or(0);
        if number > binding.len() {
            return println!("Not a station buddy");
        }
        &binding[number]
    };
    if !station.scanning {
        start_scan(station)
    }
    let networks = get_networks(station);
    for (i, network) in networks.iter().enumerate() {
        println!("{}: {}", i, network);
    }

    let input = take_input("Select your network");
    let number: usize = input.parse::<usize>().unwrap_or(0);
    if number > networks.len() {
        return println!("Not a network buddy");
    }
    let selected_network = &networks[number];
    println!("You selected: {}", selected_network);
    let is_protected = take_input("Is there a password? Y/n");
    if is_protected == "y" || is_protected == "Y" || is_protected.is_empty() {
        let password = take_input("Password");
        connect_to_network(station, selected_network.clone(), Some(password));
    } else if is_protected == "n" || is_protected == "N" {
        connect_to_network(station, selected_network.clone(), None);
    }
}

fn take_input(promt: &str) -> String {
    let mut input = String::new();
    print!("{}: ", promt);
    let _ = stdout().flush();
    stdin().read_line(&mut input).expect("error");
    if let Some('\n') = input.chars().next_back() {
        input.pop();
    }
    if let Some('\r') = input.chars().next_back() {
        input.pop();
    }
    input
}

fn connect_to_network(station: &Station, network_ssid: String, passphrase: Option<String>) -> bool {
    match &passphrase {
        Some(_string) => {
            let output = Command::new("iwctl")
                .arg("station")
                .arg(station.name.clone())
                .arg("connect")
                .arg(network_ssid)
                .arg("--passphrase")
                .arg(passphrase.unwrap())
                .output()
                .expect("Failed to connect");

            if output.stdout.is_empty() {
                return false;
            }
            true
        }
        None => {
            let output = Command::new("iwctl")
                .arg("station")
                .arg(station.name.clone())
                .arg("connect")
                .arg(network_ssid)
                .output()
                .expect("Failed to connect");
            if output.stdout.is_empty() {
                return false;
            }
            true
        }
    }
}

fn get_networks(station: &Station) -> Vec<String> {
    let mut list: Vec<String> = vec![];
    let output = Command::new("iwctl")
        .arg("station")
        .arg(station.name.clone())
        .arg("get-networks")
        .output()
        .expect("Failed");
    let text = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = text.split('\n').collect();
    for (i, line) in lines.iter().enumerate() {
        if line.len() < 26 {
            continue;
        }

        match i {
            0..=3 => continue,
            4 => {
                if &line[13..14] == ">" {
                    list.push(String::from(&line[21..53]));
                } else {
                    list.push(String::from(&line[10..42]));
                }
            }
            _ => {
                list.push(String::from(&line[6..36]));
            }
        }
    }
    for (i, list_item) in list.clone().iter().enumerate() {
        let currentlist: &Vec<&str> = &list_item.split("  ").collect();
        list[i] = String::from(currentlist[0]);
    }
    list
}

fn get_network_stations() -> Vec<Station> {
    let oputput = Command::new("iwctl")
        .arg("station")
        .arg("list")
        .output()
        .expect("Failed");

    let text = String::from_utf8(oputput.stdout).unwrap();
    let lines: Vec<&str> = text.split('\n').collect();
    let mut list: Vec<Station> = vec![];
    for (i, line) in lines.iter().enumerate() {
        if line.len() < 26 {
            continue;
        }
        let is_scanning: bool = line.contains("scanning");
        if i < 4 {
            continue;
        }
        let name = String::from(&line[6..20]);
        list.push(Station {
            name: name.trim().to_string(),
            scanning: is_scanning,
        });
    }
    list
}

fn start_scan(station: &Station) {
    println!("Scanning");
    let _ = Command::new("iwctl")
        .arg("station")
        .arg(station.name.clone())
        .arg("scan")
        .output()
        .expect("Failed");
}
