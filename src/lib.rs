//! A library for some simple manual profiling.
//!
//! This is a slightly silly profiling library, which requires you to
//! manually annotate your code to obtain profiling information.  On
//! the plus side, this means you aren't overwhelmed with detail, and
//! that you can profile portions of functions, or just the functions
//! you care about.  On the down side, you end up having to insert a
//! bunch of annotations just to get information.  You also need to
//! write the profiling information to a file or stdout on your own.
//!
//! One possible use is to print out a simple table of where time was
//! spent, e.g. 5% initializing, 95% computing.  Another would be if
//! you want to profile while ensuring that all time spent sleeping
//! (or waiting for another process) is accounted for.
//!
//! # Example
//!
//! ```
//! let _g = crude_profiler::push("test one");
//! // ... do some work here
//! _g.replace("test two");
//! println!("{}", crude_profiler::report());
//! ```

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
        m.started = std::time::Instant::now();
    }
}

impl Guard {
    /// Replace the last task pushed (or replaced) with a new one.
    ///
    /// # Example
    ///
    /// ```
    /// let _g = crude_profiler::push("test one");
    /// std::thread::sleep(std::time::Duration::from_secs(2));
    /// _g.replace("test two");
    /// std::thread::sleep(std::time::Duration::from_secs(2));
    /// println!("{}", crude_profiler::report());
    /// ```
    pub fn replace(&self, task: &'static str) {
        let now = std::time::Instant::now();
        let mut m = PROFILE.lock().unwrap();
        m.add_time(now);
        m.stack.pop();
        m.stack.push(task);
        m.started = std::time::Instant::now();
    }
}

/// Push a task to the stack of tasks.  The task will continue until
/// the `Guard` is dropped.
///
/// # Example
///
/// ```
/// let _g = crude_profiler::push("test one");
/// println!("{}", crude_profiler::report());
/// ```
pub fn push(task: &'static str) -> Guard {
    let now = std::time::Instant::now();
    let mut m = PROFILE.lock().unwrap();
    m.add_time(now);
    m.stack.push(task);
    m.started = std::time::Instant::now();
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

/// Create a string that holds a report of time used.  This is
/// currently the *only* way to extract timings data, so obviously it
/// isn't very automation-friendly.
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
    shortkeys.sort_by_key(|&s| cum[s]);
    shortkeys.reverse();
    let total_f64 = duration_to_f64(total_time);
    for s in shortkeys {
        let mut ways: HashMap<Vec<&'static str>, std::time::Duration> = HashMap::new();
        for &k in keys.iter().filter(|&k| k.contains(s)) {
            let mut vv = Vec::from(k.split(|&ss| ss == *s).next().unwrap());
            vv.push(s);
            add_to_map(&mut ways, vv, m.times[k]);
        }
        let mut waykeys: Vec<_> = ways.keys().collect();
        waykeys.sort_by_key(|&k| ways[k]);
        waykeys.reverse();
        let percent = 100.0*duration_to_f64(cum[s])/total_f64;
        if waykeys.len() > 1 {
            out.push_str(&format!("{:4.1}% {} {:.1}s\n",
                                  percent, &s, duration_to_f64(cum[s])));
            for &k in waykeys.iter().filter(|&k| k.contains(s)) {
                let percent = 100.0*duration_to_f64(ways[k])/total_f64;
                out.push_str(&format!("      {:4.1}% {} {:.1}s\n",
                                      percent, &pretty_stack(k),
                                      duration_to_f64(ways[k])));
            }
        } else {
            out.push_str(&format!("{:4.1}% {} {:.1}s\n", percent,
                                  &pretty_stack(waykeys[0]), duration_to_f64(cum[s])));
        }
    }
    // out.push_str(&format!("{:?}", m.times));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    lazy_static! {
        static ref TEST_LOCK: Mutex<()> = Mutex::new(());
    }

    #[test]
    fn it_works() {
        let mut _m = TEST_LOCK.lock().unwrap();
        clear();
        push("hello world");
        let rep = report();
        println!("\n{}", rep);
        assert!(rep.contains("hello world"));
    }
    #[test]
    fn nesting() {
        let mut _m = TEST_LOCK.lock().unwrap();
        clear();
        {
            let _a = push("hello");
            let _b = push("world");
        }
        let rep = report();
        println!("\n{}", rep);
        assert!(rep.contains("hello:world"));
    }
    #[test]
    fn replace_works() {
        let mut _m = TEST_LOCK.lock().unwrap();
        clear();
        {
            let _a = push("first");
            let _b = push("greet");
            _b.replace("world");
        }
        {
            let _a = push("second");
            let _b = push("greet");
            _b.replace("world");
        }
        let rep = report();
        println!("\n{}", rep);
        assert!(!rep.contains("hello:world"));
        assert!(rep.contains("first:world"));
        assert!(rep.contains("first:greet"));
        assert!(rep.contains("second:world"));
        assert!(rep.contains("second:greet"));
    }
    #[test]
    fn replace_timings() {
        let mut _m = TEST_LOCK.lock().unwrap();
        clear();
        {
            let _a = push("first");
            std::thread::sleep(std::time::Duration::from_secs(2));
            let _b = push("hello");
            std::thread::sleep(std::time::Duration::from_secs(3));
            _b.replace("world");
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        {
            let _a = push("second");
            std::thread::sleep(std::time::Duration::from_secs(1));
            let _b = push("greet");
            std::thread::sleep(std::time::Duration::from_secs(4));
            _b.replace("world");
            std::thread::sleep(std::time::Duration::from_secs(3));
        }
        let rep = report();
        println!("\n{}", rep);
        assert!(!rep.contains("hello:world"));
        assert!(rep.contains("first:world"));
        assert!(rep.contains("first:hello"));
        assert!(rep.contains("second:world"));
        assert!(rep.contains("second:greet"));
        assert!(rep.contains("first: 6"));
        assert!(rep.contains("first:hello: 3"));
        assert!(rep.contains("first:world: 1"));
        assert!(rep.contains("second: 8"));
        assert!(rep.contains("second:greet: 4"));
        assert!(rep.contains("second:world: 3"));
        assert!(rep.contains("world 4"));
    }
}
