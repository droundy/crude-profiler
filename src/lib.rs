#[macro_use]
extern crate lazy_static;

use std::sync::Mutex;
use std::collections::HashMap;
use std::collections::hash_map::{Entry};

struct Profile {
    times: HashMap<Vec<&'static str>, std::time::Duration>,
    stack: Vec<&'static str>,
    started: std::time::Instant,
}

impl Profile {
    fn new() -> Profile {
        Profile {
            times: HashMap::new(),
            started: std::time::Instant::now(),
            stack: Vec::new(),
        }
    }
    fn add_time(&mut self, now: std::time::Instant) {
        if now > self.started {
            let d = now.duration_since(self.started);
            match self.times.entry(self.stack.clone()) {
                Entry::Occupied(mut o) => {
                    *o.get_mut() += d;
                },
                Entry::Vacant(v) => {
                    v.insert(d);
                },
            }
        }
    }
}

lazy_static! {
    static ref PROFILE: Mutex<Profile> = Mutex::new(Profile::new());
}

pub struct Guard {
}

impl Drop for Guard {
    fn drop(&mut self) {
        let now = std::time::Instant::now();
        let mut m = PROFILE.lock().unwrap();
        m.add_time(now);
        m.stack.pop();
    }
}

pub fn push(task: &'static str) -> Guard {
    let now = std::time::Instant::now();
    let mut m = PROFILE.lock().unwrap();
    m.add_time(now);
    m.stack.push(task);
    Guard {}
}

fn pretty_stack(v: &Vec<&'static str>) -> String {
    let mut out = String::new();
    for s in v {
        out.push_str(s);
        out.push_str(":");
    }
    out
}

pub fn report() -> String {
    let m = PROFILE.lock().unwrap();
    let mut out = String::new();
    for (k,v) in m.times.iter() {
        out.push_str(&pretty_stack(k));
        out.push_str(&format!(" {:?}\n", v));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        push("hello world");
        let rep = report();
        println!("{}", rep);
        assert!(rep.contains("hello world"));
    }
}
