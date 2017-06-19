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
        println!("drop");
    }
}

pub fn push(task: &'static str) -> Guard {
    let now = std::time::Instant::now();
    let mut m = PROFILE.lock().unwrap();
    m.add_time(now);
    m.stack.push(task);
    Guard {}
}

pub fn replace(task: &'static str) -> Guard {
    let now = std::time::Instant::now();
    let mut m = PROFILE.lock().unwrap();
    m.add_time(now);
    m.stack = vec![task];
    Guard {}
}

pub fn clear() {
    let mut m = PROFILE.lock().unwrap();
    m.times = HashMap::new();
    m.stack = Vec::new();
    m.started = std::time::Instant::now();
}

fn pretty_stack(v: &Vec<&'static str>) -> String {
    let mut out = String::new();
    for s in v {
        out.push_str(s);
        out.push_str(":");
    }
    out
}

fn duration_to_f64(t: std::time::Duration) -> f64 {
    t.as_secs() as f64 + (t.subsec_nanos() as f64)*1e-9
}

pub fn report() -> String {
    let now = std::time::Instant::now();
    let mut m = PROFILE.lock().unwrap();
    m.add_time(now);
    let mut out = String::new();
    let mut keys: Vec<_> = m.times.keys().collect();
    keys.sort();
    for k in keys {
        out.push_str(&pretty_stack(k));
        out.push_str(&format!(" {}\n", duration_to_f64(m.times[k])));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        clear();
        push("hello world");
        let rep = report();
        println!("{}", rep);
        assert!(rep.contains("hello world"));
    }
    #[test]

    fn nesting() {
        clear();
        let _a = push("hello");
        let _b = push("world");
        let rep = report();
        println!("{}", rep);
        assert!(rep.contains("hello:world"));
    }
}
