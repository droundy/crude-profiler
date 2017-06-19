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

fn add_to_map<K: std::hash::Hash + std::cmp::Eq>(m: &mut HashMap<K, std::time::Duration>,
                                                 k: K, d: std::time::Duration) {
    match m.entry(k) {
        Entry::Occupied(mut o) => {
            *o.get_mut() += d;
        },
        Entry::Vacant(v) => {
            v.insert(d);
        },
    }
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
            add_to_map(&mut self.times, self.stack.clone(), d);
        }
    }
}

lazy_static! {
    static ref PROFILE: Mutex<Profile> = Mutex::new(Profile::new());
}

/// A `Guard` causes a task to end when it is dropped.
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

/// Push a task to the stack of tasks.  The task will continue until
/// the `Guard` is dropped.
pub fn push(task: &'static str) -> Guard {
    let now = std::time::Instant::now();
    let mut m = PROFILE.lock().unwrap();
    m.add_time(now);
    m.stack.push(task);
    Guard {}
}

/// Replace the last task pushed with a new one.
pub fn replace(task: &'static str) -> Guard {
    let now = std::time::Instant::now();
    let mut m = PROFILE.lock().unwrap();
    m.add_time(now);
    m.stack = vec![task];
    Guard {}
}

/// Forget any prior timings.
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

/// Create a string that holds a report of time used.
pub fn report() -> String {
    let now = std::time::Instant::now();
    let mut m = PROFILE.lock().unwrap();
    m.add_time(now);
    let mut out = String::new();
    let mut total_time = std::time::Duration::from_secs(0);
    for &v in m.times.values() {
        total_time += v;
    }
    let mut keys: Vec<_> = m.times.keys().collect();
    keys.sort();
    let mut cum: HashMap<&'static str, std::time::Duration> = HashMap::new();
    for &k in keys.iter() {
        for &s in k.iter() {
            add_to_map(&mut cum, s, m.times[k]);
        }
    }
    let mut shortkeys: Vec<_> = cum.keys().collect();
    shortkeys.sort();
    let total_f64 = duration_to_f64(total_time);
    for s in shortkeys {
        let percent = 100.0*duration_to_f64(cum[s])/total_f64;
        out.push_str(&format!("{:.1}% {}: {}\n", percent, &s, duration_to_f64(cum[s])));
        let mut ways: HashMap<Vec<&'static str>, std::time::Duration> = HashMap::new();
        for &k in keys.iter().filter(|&k| k.contains(s)) {
            let mut vv = Vec::from(k.split(|&ss| ss == *s).next().unwrap());
            vv.push(s);
            add_to_map(&mut ways, vv, m.times[k]);
        }
        let mut waykeys: Vec<_> = ways.keys().collect();
        waykeys.sort();
        if waykeys.len() > 1 {
            for &k in waykeys.iter().filter(|&k| k.contains(s)) {
                let percent = 100.0*duration_to_f64(ways[k])/total_f64;
                out.push_str(&format!("    {:.1}% {} {}\n",
                                      percent, &pretty_stack(k),
                                      duration_to_f64(ways[k])));
            }
            out.push_str("\n");
        }
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
        println!("\n{}", rep);
        assert!(rep.contains("hello world"));
    }
    #[test]
    fn nesting() {
        clear();
        let _a = push("hello");
        let _b = push("world");
        let rep = report();
        println!("\n{}", rep);
        assert!(rep.contains("hello:world"));
    }
    #[test]
    fn replace_works() {
        clear();
        let _a = push("hello");
        let _b = replace("world");
        let rep = report();
        println!("\n{}", rep);
        assert!(!rep.contains("hello:world"));
        assert!(rep.contains("world"));
        assert!(rep.contains("hello"));
    }
}
