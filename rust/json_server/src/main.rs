use std::thread;
use std::os::unix::net::{UnixStream, UnixListener};

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use serde_json::{Value, Error};
use serde_json::map::Map;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Boop {
    #[serde(default)]
    uint64: u64,
    double: f64,
    string: String,
    //arbitrary_list: Vec<serde_json::Value>,
    //arbitrary_object: Map<String, serde_json::Value>,
    nullable: Option<String>,
}

#[allow(dead_code)]
fn json_example() {
    let s = r#"{
        "extra": 333,
        "uint64": 333,
        "double": 3.4444e3,
        "string": "beep boop meow"
    }"#;
    match serde_json::from_str::<Boop>(&s) {
        Ok(v) => {
            println!("json succeded {:?}", serde_json::to_string(&v).unwrap());
        },
        Err(e) => {
            println!("json failed {:?}", e);
        }
    }
}

fn get_json(stream: UnixStream) {
}

fn main() {
    // TODO: actually handle error, use /var/run or take in arg
    let listener = UnixListener::bind("/tmp/boop").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                thread::spawn(|| get_json(s));
            },
            Err(e) => println!("connection failed {:?}", e),
        }
    }
}
