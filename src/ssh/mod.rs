extern crate ssh2;

use ssh2::Session;
use std::io::Read;
use std::net::TcpStream;
use log::{debug};


pub fn connect(_hostname: &str, _port: &i32, _username: &str, _password: &str) -> Session {
    let addr = format!("{}:{}", _hostname, _port);
    let tcp = TcpStream::connect(addr).unwrap();
    let mut sess = Session::new().unwrap();

    debug!("connecting to: {}", _hostname);

    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();

    sess.userauth_password(_username, _password).unwrap();
    assert!(sess.authenticated());
    return sess;
}

pub fn exec(_command: &str, _session: &Session) -> String {
    let mut channel = _session.channel_session().unwrap();
    channel.exec(_command).unwrap();
    let mut s = String::new();
    channel.read_to_string(&mut s).unwrap();
    channel.wait_close().map_err(|err| println!("{:?}", err)).ok();
    return s;
}

pub fn du(_session: &Session, _dir: &str) -> u32
{
    let command = format!("du -s {}", _dir);
    //println!("command: {}", command);
    let output = exec(&command, &_session);

    if output.len()==0
    {
        return 0;
    }

    //println!("output: {}", output);
    let mut parts = output.split_whitespace();
    let size = parts.next().unwrap();
    return size.parse::<u32>().unwrap();
}

#[derive(Debug, Default)]
pub struct LoadAverage {
    pub load1: f32,
    pub load5: f32,
    pub load15: f32,
}

pub fn loadavg(_session: &Session) -> LoadAverage
{
    let loadavg = exec("cat /proc/loadavg", &_session);
    //println!("loadavg:\n====\n{}====", loadavg);

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

    let res = LoadAverage{
        load1: one.parse::<f32>().unwrap(),
        load5: five.parse::<f32>().unwrap(),
        load15: fifteen.parse::<f32>().unwrap()
    };

    return res;
}


#[derive(Debug, Default)]
pub struct UsageEntry {
    pub folder: String,
    pub size: u32
}

#[derive(Debug, Default)]
pub struct Usage {
    pub attributes: Vec<UsageEntry>
}

pub fn usage(_session: &Session, _basedir: &str) -> Usage
{
    // basedir = /srv/mail/mysql/data
    // ls -ld /srv/mail/mysql/data/*/
    // find /srv/mail/mysql/data -type d -depth 1
    // _basedir
    let command = format!("ls -1d {}/*/", _basedir);
    let inner_folders = exec(&command, &_session);
    //println!("inner_folders:\n====\n{}====", inner_folders);
    
    let mut usages: Vec<UsageEntry> = Vec::new();
    for folder in inner_folders.lines() {
        let du = du(_session, folder);
        //println!("folder: {} ", folder);
        //println!("diskusage: {} ", du);
        let mut f = folder.to_string();
        if f.ends_with("/"){f.pop();};
        usages.push(UsageEntry{folder: f,size:du});
    } 

    let du = du(_session, _basedir);
    //println!("folder: {} ", _basedir);
    //println!("diskusage: {} ", du);
    usages.push(UsageEntry{folder: _basedir.to_string(),size:du});
    let res = Usage{
        attributes: usages
    };
    return res;
}

#[derive(Debug, Default)]
pub struct MemoryEntry {
    pub name: String,
    pub size: u64
}

#[derive(Debug, Default)]
pub struct Memory {
    pub attributes: Vec<MemoryEntry>
}
pub fn meminfo(_session: &Session) -> Memory
{
    let meminfo = exec("cat /proc/meminfo", &_session);
    //println!("meminfo:\n====\n{}====", meminfo);
    // MemTotal:        8002772 kB
    // MemFree:          422012 kB
    // MemAvailable:    6554824 kB
    // Buffers:          202756 kB
    // Cached:          5468388 kB
    // SwapCached:            8 kB

    let mut usages: Vec<MemoryEntry> = Vec::new();
    for line in meminfo.lines()
    {
        let mut parts = line.split_whitespace();
        // label: 'MemAvailable:'
        let mut label = parts.next().unwrap();
        // label: 'MemAvailable'
        label = &label[..label.len()-1];
        let size = parts.next().unwrap();
        //println!("{:?}: {:?}", label, size);

        usages.push(MemoryEntry{
            name: String::from(label),
            size: size.parse::<u64>().unwrap()
        });
    }
    return Memory{attributes: usages};
}


// --output: FIELD_LIST  is  a  comma-separated  list of columns to be included.  Valid field names are: 'source', 'fstype', 'itotal', 'iused',
//        'iavail', 'ipcent', 'size', 'used', 'avail', 'pcent', 'file' and 'target' (see info page).

// df -h -t ext4 -t vfat -l --output=size,used,avail,target
// df -t ext4 -t vfat -l --output=size,used,avail,target
/*

1K-Blöcke Benutzt    Verf. Eingehängt auf
 29292924 5002084 22779800 /
    64366   21530    42836 /boot

 */

