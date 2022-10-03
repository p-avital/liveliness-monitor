# A Liveliness Monitor for your Asynchronous Runtimes

Asynchronous Rust allows us to schedule lots of tasks without the cost of allocating a full thread for each of them.

This is done by splitting these tasks into smaller ones at each `.await` point, which marks the task yielding to the executor.

However, by mixing synchronous code into async code, it's very easy to park executor threads on accident. Since most executors
use a finite thread pool, the opportunity for a special kind of deadlock occurs, where all executor threads are parked, but the events
that would unpark them are triggered by tasks that are waiting to be executed.

This kind of "soft" deadlock, where adding threads to the executor could (but isn't necessarily) result in the application returning 
to normal is often referred to as "executor stalls".

## What is a Liveliness Monitor

It's a very trivial thing, really: it's an asynchronous task that will run in a loop, updating a shared value to show liveliness
before immediately yielding back to the executor. Meanwhile, an independent thread may poll that variable to check for signs of stalling,
i.e. the value not being updated for a long time, and possibly try to remedy the situation.

## How does this one work?

Here, the shared value is an atomic i64, which is wrapped to allow treating it as an atomic `std::time::Instant`.

The task holds a weak reference to this shared value, and will stop if no strong references exist on it anymore.
When creating this task, a strong reference to the shared value is handed back to you to treat as you see fit.

## Should I include this in my asynchronous library?

Maybe: asynchronous library authors are typically more aware of these sorts of issues than the average. Exposing an easy way to use
a liveliness monitor to your users could let them discover what they might be doing wrong.

However, the end-user should be the one handling the returned monitor (typically by spawning a watcher thread with low priority),
as only they know the full behaviour of their application, and how it could/couldn't recover from executor stalls.

## Should I include this in my executable?

Definitely. If your application is _extremely_ performance bound, you might want to make it optional, mostly due to the fact that
a liveliness monitor must reside on a thread that is independent of the asynchronous executor. If you can spare a low priority thread
and a low overhead task, you should probably enable it by default, as executor stalls tend to be Heisenbugs, and you'll need any help
you can get if your application gets plagued by them.