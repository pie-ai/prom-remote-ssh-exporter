# prometheus exporter that analyses remote hosts
very simple ssh connection


# Usage
./endpoints.csv:

````
identifier,hostname,port,username,password,usage
localhorst,localhost,22,user,secret,/var/lib|/home/user/
````


````
% docker run -v ./endpoints.csv:/etc/endpoints.csv pieai/prom-remote-ssh-exporter:latest /rust-binary --endpoints /etc/endpoints.csv
````


````
% docker run -v ./endpoints.csv:/etc/endpoints.csv -p 6660 pieai/prom-remote-ssh-exporter:latest /rust-binary --endpoints /etc/endpoints.csv
  ...
% curl http://127.0.0.1:6660/metrics

# HELP node_load_1 system load 1 minute
# TYPE node_load_1 gauge
node_load_1{host="localhorst"} 1.93
# HELP node_load_5 system load 5 minute
# TYPE node_load_5 gauge
node_load_5{host="localhorst"} 1.88
# HELP load_15 system load 15 minute
# TYPE load_15 gauge
load_15{host="localhorst"} 1.89
# HELP filesystem fs usage
# TYPE filesystem gauge
filesystem{host="localhorst",folder="/home/user/Downloads"} 1580124
filesystem{host="localhorst",folder="/home/user/Nextcloud"} 9956565
filesystem{host="localhorst",folder="/var/log"} 158501
# HELP memory memory usage
# TYPE memory gauge
memory{type="MemTotal",host="localhorst"} 8178364

````

# Notes

## Debugging
VS Code Rust Debugging: https://www.forrestthewoods.com/blog/how-to-debug-rust-with-visual-studio-code

## SSH2
SSH2 Library: https://docs.rs/ssh2/0.3.3/ssh2/index.html

## Prometheus
dev.to rust prometheus exporter: https://dev.to/mindflavor/let-s-build-a-prometheus-exporter-in-rust-30pd