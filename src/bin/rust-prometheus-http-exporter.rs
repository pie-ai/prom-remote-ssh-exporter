extern crate ssh_prometheus_exporter;

use ssh_prometheus_exporter::ssh;
use log::{info, trace, debug};
use prometheus_exporter_base::{render_prometheus, MetricType, PrometheusMetric};
use serde::Deserialize;
use std::env;
use std::fs::File;
use std::io::BufReader;
use ssh::LoadAverage;

#[derive(Debug, Clone, Default)]
struct MyOptions {}

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
    usage: String
}

struct Args {
    help: bool,
    verbose: bool,
    endpoints: String,
    port: u16
}

// https://www.dev-notes.eu/2020/03/Return-a-Result-Type-from-the-Main-Function-in-Rust/
#[tokio::main]
async fn main() -> Result<(), &'static str>{
    let mut args = pico_args::Arguments::from_env();
    let args = Args {
        help: args.contains(["-h", "--help"]),
        verbose: args.contains(["-v", "--verbose"]),
        endpoints: args.value_from_str("--endpoints").unwrap(),
        port: 6660
    };

    if args.verbose
    {
        env::set_var("RUST_LOG","trace");
    } else {
        env::set_var("RUST_LOG","info");
    }

    env_logger::init();

    let addr = ([0, 0, 0, 0], args.port).into();
    info!("starting exporter on {}", addr);
    let endpoints = args.endpoints.clone();

    render_prometheus(addr, MyOptions::default(), move |request, options| {
        async move {
            trace!(
                "in our render_prometheus(request == {:?}, options == {:?})",
                request,
                options
            );
            let input = File::open(endpoints.as_str()).unwrap();
            let buffered = BufReader::new(input);
            let mut rdr = csv::Reader::from_reader(buffered);
            //println!("====example====");
            let load_1_metric = PrometheusMetric::new("load_1", MetricType::Counter, "system load 1 minute");
            let mut load_1_buf = load_1_metric.render_header();

            let load_5_metric = PrometheusMetric::new("load_5", MetricType::Counter, "system load 5 minute");
            let mut load_5_buf = load_5_metric.render_header();
            
            let load_15_metric = PrometheusMetric::new("load_15", MetricType::Counter, "system load 15 minute");
            let mut load_15_buf = load_15_metric.render_header();

            let usage_metric = PrometheusMetric::new("filesystem", MetricType::Counter, "fs usage");
            let mut usage_buf = usage_metric.render_header();

            let memory_metric = PrometheusMetric::new("memory", MetricType::Gauge, "memory usage");
            let mut memory_buf = memory_metric.render_header();

            for entry in rdr.deserialize() {
                let record: Endpoint = entry?;
                //println!("endpoint: {:?}", record.identifier);

                debug!("connecting to {} via ssh", &record.hostname);

                let sess = ssh::connect(&record.hostname,&record.port, &record.username, &record.password);
                //let processes = ssh::exec("ps -aux", &sess);
                //println!("processes:\n====\n{}====", processes);

                let mut attributes: Vec<(&str, &str)> = Vec::new();
                attributes.push(("host", &record.identifier));
                
                let load: LoadAverage = ssh::loadavg(&sess);
                load_1_buf.push_str(&load_1_metric.render_sample(Some(attributes.as_slice()), load.load1));
                load_5_buf.push_str(&load_5_metric.render_sample(Some(attributes.as_slice()), load.load5));
                load_15_buf.push_str(&load_15_metric.render_sample(Some(attributes.as_slice()), load.load15));

                for u in record.usage.split("|")
                {
                    let usage = ssh::usage(&sess,&u);

                    for entry in usage.attributes
                    {
                        let mut usage_attributes: Vec<(&str, &str)> = Vec::new();
                        usage_attributes.push(("host", &record.identifier));
                        usage_attributes.push(("folder",&entry.folder));
                        usage_buf.push_str(&usage_metric.render_sample(Some(usage_attributes.as_slice()),entry.size));
                    }
                }

                // meminfo
                let meminfo = ssh::meminfo(&sess);
                for entry in meminfo.attributes
                {
                    let mut memory_attributes: Vec<(&str, &str)> = Vec::new();
                    memory_attributes.push(("type", &entry.name));
                    memory_attributes.push(("host", &record.identifier));
                    memory_buf.push_str(&memory_metric.render_sample(Some(memory_attributes.as_slice()),entry.size));
                }
            }

            let mut s = load_1_buf;
            s.push_str(&load_5_buf);
            s.push_str(&load_15_buf);
            s.push_str(&usage_buf);
            s.push_str(&memory_buf);
            Ok(s)
        }
    })
    .await;
    Ok(())
}