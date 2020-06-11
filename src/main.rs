extern crate prometheus_exporter_base;
extern crate serde;
extern crate ssh2;

mod ssh;

use std::net::TcpStream;
//use std::path::Path;
use ssh2::Session;
//use std::io::Read;
use std::str;

use serde::Deserialize;
use std::error::Error;
//use std::io;
use std::process;
//use std::marker::Copy;

use std::collections::LinkedList;
use std::fs::File;
//use std::io::{BufRead, BufReader, Write};
use std::io::BufReader;

use prometheus_exporter_base::{render_prometheus, MetricType, PrometheusMetric};
use std::fs::read_dir;

/*fn exec(_command: &str, _session: &Session) -> String {
    let mut channel = _session.channel_session().unwrap();
    channel.exec(_command).unwrap();
    let mut s = String::new();
    channel.read_to_string(&mut s).unwrap();
    channel.wait_close();
    return s;
}*/

// https://github.com/BurntSushi/rust-csv/blob/master/examples/cookbook-read-serde.rs
// By default, struct field names are deserialized based on the position of
// a corresponding field in the CSV data's header record.
#[derive(Debug, Deserialize, Default)]
struct Endpoint {
    identifier: String,
    hostname: String,
    port: i32,
    username: String,
    password: String,
}

fn example() -> Result<(), Box<dyn Error>> {
    let mut result: LinkedList<Endpoint> = LinkedList::new();

    println!("====example====");

    // let input = io::stdin();
    let endpoints = "/Users/pa/Nextcloud/basteln/rust-ssh-client/endpoints.csv";

    let input = File::open(endpoints).unwrap();

    let buffered = BufReader::new(input);

    let mut rdr = csv::Reader::from_reader(buffered);
    println!("====example====");
    for entry in rdr.deserialize() {
        let record: Endpoint = entry?;
        //result.push_back(Endpoint::new());
        //result.push_back(record);
        // .push_back(record);
        println!("endpoint: {:?}", record);
    }

    Ok(())
}

fn calculate_file_size(path: &str) -> Result<u64, std::io::Error> {
    let mut total_size: u64 = 0;
    for entry in read_dir(path)? {
        let p = entry?.path();
        if p.is_file() {
            total_size += p.metadata()?.len();
        }
    }

    Ok(total_size)
}

/*
#[derive(Debug, Clone)]
struct MyOptions {}

#[tokio::main]
async fn main() {
    let addr = ([0, 0, 0, 0], 32221).into();
    println!("starting exporter on {}", addr);

    render_prometheus(addr, MyOptions::default(), |request, options| {
        async move {
            println!(
                "in our render_prometheus(request == {:?}, options == {:?})",
                request, options
            );

            let total_size_log = calculate_file_size("/var/log").unwrap();

            let pc =
                PrometheusMetric::new("folder_size", MetricType::Counter, "Size of the folder");
            let mut s = pc.render_header();

            let mut attributes = Vec::new();
            attributes.push(("folder", "/var/log/"));
            s.push_str(&pc.render_sample(Some(&attributes), total_size_log));

            Ok(s)
        }
    })
    .await;
}
*/
fn main() {
    let hostname = "mail.strigal.net:22";
    let username = "monitor";
    let port = 22;
    // escaped \ -> \\
    let password = "#5YW!\\\\3=x<g";

    // Connect to the local SSH server
    let mut sess = ssh::connect(hostname, &port, username, password);

    /*
    let tcp = TcpStream::connect(hostname).unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();

    sess.userauth_password(username, password).unwrap();
    assert!(sess.authenticated());
    */

    let processes = ssh::exec("ps -aux", &sess);
    println!("processes:\n====\n{}====", processes);

    let loadavg = ssh::exec("cat /proc/loadavg", &sess);
    println!("loadavg:\n====\n{}====", loadavg);

    // Wert 1: Anzahl der Prozesse im Status R (lauffähig / runnable) oder D (auf I/O wartend / disk sleep) in der Run Queue als Durchschnitt über 1 Minute
    // Wert 2: Analog zu Wert 1, allerdings als Durchschnitt über 5 Minuten
    // Wert 3: Analog zu Wert 1, allerdings als Durchschnitt über 15 Minuten
    // Wert 4: Der Feld vor dem Schrägstrich enthält die aktuell lauffähigen (runnable) Prozesse / Threads (Kernel Scheduling Entities).
    // Das Feld danach enthält die Anzahl der Kernel Scheduling Entities im System.
    // Wert 5: PID des jüngsten im System erzeugten Prozesses
    let mut parts = loadavg.split_whitespace();
    let one = parts.next().unwrap();
    let five = parts.next().unwrap();
    let fifteen = parts.next().unwrap();

    println!("1: {}", one);
    println!("5: {}", five);
    println!("15: {}", fifteen);

    if let Err(err) = example() {
        println!("error running example: {}", err);
        process::exit(1);
    }

    /*

        let mut channel = sess.channel_session().unwrap();
        channel.exec("cat /proc/loadavg > /home/monitor/loadavg.tmp").unwrap();
        let mut s = String::new();
        channel.read_to_string(&mut s).unwrap();
        println!("{}", s);
        channel.wait_close();
        println!("{}", channel.exit_status().unwrap());
    */
    //
    // /proc/loadavg
    /*
        let (mut remote_file, stat) = sess.scp_recv(Path::new("/home/monitor/loadavg.tmp")).unwrap();
        println!("remote file size: {}", stat.size());
        let mut contents = Vec::new();
        remote_file.read_to_end(&mut contents).unwrap();
        // println!("remote file contents: {:?}",contents);
        // remote file contents: [48, 46, 48, 48, 32, 48, 46, 48, 49, 32, 48, 46, 48, 51, 32, 49, 47, 49, 51, 55, 32, 52, 48, 54, 10]

        let tmp = str::from_utf8(&contents).unwrap();
        println!("remote file contents: {}",tmp);
    */
    // Wert 1: Anzahl der Prozesse im Status R (lauffähig / runnable) oder D (auf I/O wartend / disk sleep) in der Run Queue als Durchschnitt über 1 Minute
    // Wert 2: Analog zu Wert 1, allerdings als Durchschnitt über 5 Minuten
    // Wert 3: Analog zu Wert 1, allerdings als Durchschnitt über 15 Minuten
    // Wert 4: Der Feld vor dem Schrägstrich enthält die aktuell lauffähigen (runnable) Prozesse / Threads (Kernel Scheduling Entities).
    // Das Feld danach enthält die Anzahl der Kernel Scheduling Entities im System.
    // Wert 5: PID des jüngsten im System erzeugten Prozesses
}
