# Crude profiler

A library for some simple manual profiling.

This is a slightly silly profiling library, which requires you to
manually annotate your code to obtain profiling information.  On the
plus side, this means you aren't overwhelmed with detail, and that you
can profile portions of functions, or just the functions you care
about.  On the down side, you end up having to insert a bunch of
annotations just to get information.  You also need to write the
profiling information to a file or stdout on your own.

One possible use is to print out a simple table of where time was
spent, e.g. 5% initializing, 95% computing.  Another would be if you
want to profile while ensuring that all time spent sleeping (or
waiting for another process) is accounted for.

# Example

```
let _g = crude_profiler::push("test one");
// ... do some work here
_g.replace("test two");
println!("{}", crude_profiler::report());
```
