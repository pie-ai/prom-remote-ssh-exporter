extern crate ssh_prometheus_exporter;

//use clap::{crate_authors, crate_name, crate_version, Arg};
// use climake::{Argument, CLIMake, DataType};

use ssh_prometheus_exporter::ssh;
use log::{info, trace};
//, error};
use prometheus_exporter_base::{render_prometheus, MetricType, PrometheusMetric};
use serde::Deserialize;
use std::env;
use std::fs::File;
use std::io::BufReader;

use ssh::LoadAverage;
//use std::path::PathBuf;
//use std::convert::TryInto;
//use std::any::Any;
//use std::borrow::Borrow;

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
    /*
     let matches = clap::App::new(crate_name!())
        .version(crate_version!())
        //.about(crate_description!())
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
        .arg(
            Arg::with_name("endpoints")
                .short("e")
                .help("endpoints csv file (identifier,hostname,port,username,password,usage)")
                .takes_value(true),
        )
        .get_matches();
    let tmp = matches.value_of("endpoints");
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

    let mut endpoints: &str = "foo.txt";//matches.value_of("endpoints").unwrap();

    if matches.is_present("endpoints") {
        endpoints = tmp.unwrap();
    }
*/


    /*
    let arg_endpoints = Argument::new(
        &['e'],
        &["endpoints"],
        Some("endpoints csv file"),
        DataType::Files,
    ).unwrap();
    let arg_verbose =  Argument::new(
        &['v'],
        &["verbose"],
        Some("verbose mode on"),
        DataType::None,
    ).unwrap();
    let args = &[arg_endpoints,arg_verbose];

    let cli = CLIMake::new(args, Some("PA's Rust Prometheus Exporter"), None).unwrap();
    let args = cli.parse();
    let mut verbose = false;
    let mut endpoints: Vec<PathBuf> = Vec::new();
    for arg in &args {
        let argument arg.argument;
        match argument
        {
            arg_verbose=> verbose = true,
            arg_endpoints=>
                {
                    match arg.passed_data
                    {
                        File => verbose = false
                    }
                }
        }


        println!("{}", arg.argument.help.unwrap());
    }

    println!("Args used: {:#?}", cli.parse());
    */
    let mut args = pico_args::Arguments::from_env();
    let args = Args {
        help: args.contains(["-h", "--help"]),
        verbose: args.contains(["-v", "--verbose"]),
        endpoints: args.value_from_str("--endpoints").unwrap(),
        port: 6660
    };

    if args.verbose
    {
        env::set_var(
            "RUST_LOG",
            format!("folder_size=trace,{}=trace", "prometheus-exporter"),
        );
    } else {
        env::set_var(
            "RUST_LOG",
            format!("folder_size=info,{}=info", "prometheus-exporter"),
        );
    }

    env_logger::init();

    //info!("using matches: {:?}", &matches);

    //let bind = "8080";//matches.value_of("port").unwrap();
    //let bind = u16::from_str_radix(&bind, 10).expect("port must be a valid number");
    let addr = ([0, 0, 0, 0], args.port).into();

    info!("starting exporter on {}", addr);

    //let endpointsAvailable = matches.is_present("endpoints");
    //let endpoints = matches.value_of("endpoints");
    /*if endpoints == ''
    {
        error!("specify endpoints file");
        //std::process::exit(1);
        return Err("missing endpoints file")
    }*/
//    let endpoints: &'static str = args.endpoints.as_str();
        //let endpoints: &'static str = matches.value_of("endpoints").unwrap();
    //let endpoints = endpoints.unwrap();
    let endpoints = args.endpoints.clone();

    render_prometheus(addr, MyOptions::default(), move |request, options| {
        async move {
            trace!(
                "in our render_prometheus(request == {:?}, options == {:?})",
                request,
                options
            );

            //let endpoints = "/Users/pa/Nextcloud/basteln/rust-ssh-client/endpoints.csv";
    //    //

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
    Ok(())
}