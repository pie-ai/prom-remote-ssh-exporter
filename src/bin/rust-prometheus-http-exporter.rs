extern crate ssh_prometheus_exporter;

use ssh_prometheus_exporter::ssh;
use log::{info, trace};
use prometheus_exporter_base::{render_prometheus, MetricType, PrometheusMetric};
use serde::Deserialize;
use std::env;
use std::fs::File;
use std::io::BufReader;
use ssh::LoadAverage;
use log::{debug, error};
use ssh2::Session;
//use std::path::Path;


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
/*
            if ! Path::new(endpoints.as_str()).exists()
            {
                Ok("# could not open endpoints file");
            }
*/
            let input = File::open(endpoints.as_str()).unwrap();
            let buffered = BufReader::new(input);
            let mut rdr = csv::Reader::from_reader(buffered);

            let connectable_metric = PrometheusMetric::new("connectable", MetricType::Gauge, "system is connectable using ssh");
            let mut connectable_buf = connectable_metric.render_header();

            let load_1_metric = PrometheusMetric::new("node_load1", MetricType::Gauge, "system load 1 minute");
            let mut load_1_buf = load_1_metric.render_header();

            let load_5_metric = PrometheusMetric::new("node_load5", MetricType::Gauge, "system load 5 minute");
            let mut load_5_buf = load_5_metric.render_header();
            
            let load_15_metric = PrometheusMetric::new("node_load15", MetricType::Gauge, "system load 15 minute");
            let mut load_15_buf = load_15_metric.render_header();

            let usage_metric = PrometheusMetric::new("filesystem", MetricType::Gauge, "fs usage");
            let mut usage_buf = usage_metric.render_header();

            let memory_metric = PrometheusMetric::new("memory", MetricType::Gauge, "memory usage");
            let mut memory_buf = memory_metric.render_header();

            let machine_cpu_threads_metric = PrometheusMetric::new("machine_cpu_threads", MetricType::Gauge, "cpu thread count");
            let mut machine_cpu_threads_buf = machine_cpu_threads_metric.render_header();

            for entry in rdr.deserialize() {
                let record: Endpoint = entry?;
                let mut attributes: Vec<(&str, &str)> = Vec::new();

                debug!("connecting to {} via ssh", &record.hostname);

                let sess:(Session) = match ssh::connect(&record.hostname,&record.port, &record.username, &record.password)
                {
                    Ok(s)=>
                        {
                            connectable_buf.push_str(&*connectable_metric.render_sample(Some(attributes.as_slice()), 1));
                            s
                        }
                    Err(e)=>
                        {
                            error!("could not connect: {:?}", e);
                            connectable_buf.push_str(&*connectable_metric.render_sample(Some(attributes.as_slice()), 0));
                            continue
                        }
                };
                //let processes = ssh::exec("ps -aux", &sess);
                //println!("processes:\n====\n{}====", processes);


                attributes.push(("host", &record.identifier));
                //attributes.push(("instance", &record.identifier));

                // load
                let load: LoadAverage = ssh::loadavg(&sess);
                load_1_buf.push_str(&load_1_metric.render_sample(Some(attributes.as_slice()), load.load1));
                load_5_buf.push_str(&load_5_metric.render_sample(Some(attributes.as_slice()), load.load5));
                load_15_buf.push_str(&load_15_metric.render_sample(Some(attributes.as_slice()), load.load15));

                // cpuinfo
                let cpuinfo = ssh::cpuinfo(&sess);
                machine_cpu_threads_buf.push_str(&machine_cpu_threads_metric.render_sample(Some(attributes.as_slice()), cpuinfo.threads));

                if ! record.usage.trim().is_empty()
                {
                    for u in record.usage.split("|")
                    {
                        let usage = ssh::usage(&sess,&u);

                        for entry in usage.attributes
                        {
                            let mut usage_attributes: Vec<(&str, &str)> = Vec::new();
                            usage_attributes.push(("host", &record.identifier));
                            //usage_attributes.push(("instance", &record.identifier));
                            usage_attributes.push(("folder",&entry.folder));
                            usage_buf.push_str(&usage_metric.render_sample(Some(usage_attributes.as_slice()),entry.size));
                        }
                    }
                }

                // meminfo
                let meminfo = ssh::meminfo(&sess);
                for entry in meminfo.attributes
                {
                    let mut memory_attributes: Vec<(&str, &str)> = Vec::new();
                    memory_attributes.push(("type", &entry.name));
                    memory_attributes.push(("host", &record.identifier));
                    //memory_attributes.push(("instance", &record.identifier));
                    memory_buf.push_str(&memory_metric.render_sample(Some(memory_attributes.as_slice()),entry.size));
                }
            }

            let mut s = connectable_buf;
            s.push_str(&load_1_buf);
            s.push_str(&load_5_buf);
            s.push_str(&load_15_buf);
            s.push_str(&usage_buf);
            s.push_str(&memory_buf);
            s.push_str(&machine_cpu_threads_buf);

            Ok(s)
        }
    })
    .await;
    Ok(())
}