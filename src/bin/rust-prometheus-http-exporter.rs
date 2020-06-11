extern crate de_pa2_rust_tests;

use clap::{crate_authors, crate_name, crate_version, Arg};
use de_pa2_rust_tests::ssh;
use log::{info, trace};
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

#[tokio::main]
async fn main() {
    let matches = clap::App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .arg(
            Arg::with_name("port")
                .short("p")
                .help("exporter port")
                .default_value("32148")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .help("verbose logging")
                .takes_value(false),
        )
        .get_matches();

    if matches.is_present("verbose") {
        env::set_var(
            "RUST_LOG",
            format!("folder_size=trace,{}=trace", crate_name!()),
        );
    } else {
        env::set_var(
            "RUST_LOG",
            format!("folder_size=info,{}=info", crate_name!()),
        );
    }
    env_logger::init();

    info!("using matches: {:?}", matches);

    let bind = matches.value_of("port").unwrap();
    let bind = u16::from_str_radix(&bind, 10).expect("port must be a valid number");
    let addr = ([0, 0, 0, 0], bind).into();

    info!("starting exporter on {}", addr);

    render_prometheus(addr, MyOptions::default(), |request, options| {
        async move {
            trace!(
                "in our render_prometheus(request == {:?}, options == {:?})",
                request,
                options
            );

            let endpoints = "/Users/pa/Nextcloud/basteln/rust-ssh-client/endpoints.csv";

            let input = File::open(endpoints).unwrap();

            let buffered = BufReader::new(input);

            let mut rdr = csv::Reader::from_reader(buffered);
            //println!("====example====");
            let load_1_metric = PrometheusMetric::new("load_1", MetricType::Counter, "system load 1 minute");
            let mut load_1_buf = load_1_metric.render_header();

            let load_5_metric = PrometheusMetric::new("load_5", MetricType::Counter, "system load 5 minute");
            let mut load_5_buf = load_5_metric.render_header();
            
            let load_15_metric = PrometheusMetric::new("load_15", MetricType::Counter, "system load 15 minute");
            let mut load_15_buf = load_15_metric.render_header();

            let usage_metric = PrometheusMetric::new("usage", MetricType::Counter, "fs usage");
            let mut usage_buf = usage_metric.render_header();


            for entry in rdr.deserialize() {
                let record: Endpoint = entry?;
                println!("endpoint: {:?}", record.identifier);

                let sess = ssh::connect(&record.hostname,&record.port, &record.username, &record.password);
                //let processes = ssh::exec("ps -aux", &sess);
                //println!("processes:\n====\n{}====", processes);

                let mut attributes: Vec<(&str, &str)> = Vec::new();
                attributes.push(("host", &record.hostname));
                
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
                        usage_attributes.push(("host", &record.hostname));
                        usage_attributes.push(("folder",&entry.folder));
                        usage_buf.push_str(&usage_metric.render_sample(Some(usage_attributes.as_slice()),entry.size));
                    }
                }
            }

            let mut s = load_1_buf;
            s.push_str(&load_5_buf);
            s.push_str(&load_15_buf);
            s.push_str(&usage_buf);
            Ok(s)
        }
    })
    .await;
}