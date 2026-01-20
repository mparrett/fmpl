# 4 Performance Measurement and Debugging
### 4.1 Introduction
Before we set to work developing parallel programs, we need to consider two more basic questions of programming mechanics. *Debugging* parallel programs raises certain problems (centering on deadlock and non-determinism) that are new and different—although, as we discuss,
the significance of these issues is rather modest in practical terms.*Performance analysis and tuning*is a crucial issue in parallel
programming. The whole point is to write programs that run fast. Once an application is up and running, we must know how well we are
doing, and how to do better. These two basic issues are closely related, in fact, in the sense that *performance debugging*is a crucial aspect of
code development for parallelism. Once logical debugging is complete, we know that our code runs correctly. But a "performance bug" that
makes a program run too slowly is, in one sense, every bit as significant as a "logic bug" that makes it compute incorrectly. A parallel
program that doesn't run fast is, in most cases, approximately as useful as a parallel program that doesn't run at all.
### 4.2 Debugging
Debugging involves two issues—the *systems tools*you rely on to accomplish it, and the *logical questions*you expect to deal with. In the
sequential world, symbolic debuggers are the main line of defense as far as tools go. The problems posed by sequential debugging are well

```text
known. In the following discussion, we consider debugging in the coordination language framework; then we move on to the special logical
```
problems associated with parallel-program debugging.
#### 4.2.1 Introduction: software tools for debugging in the coordination language framework
The debugging of parallel programs is often held up as a complex and arcane art, but (luckily) it isn't. The concepts of *coordination language*and *coordination framework*make it possible to deal with parallel debugging as an incremental extension to ordinary sequential debugging. A
parallel program consists of some ordinary sequential programs bound together by a coordination framework. We *already know*how to debug
the sequential programs—whatever approach you take to debugging sequential programs, and whatever tools you rely on, can in principle be
carried over intact to the parallel environment. Parallel debugging reduces, accordingly, to the following question: how do we debug a
coordination framework? So far as the tools are concerned, there is a high-tech and a low-tech approach.
In the high-tech approach we use a *coordination framework debugger*that relates to conventional sequential-language symbolic debuggers
just as coordination languages relate to sequential languages. The coordination debugger shows you the *coordination state*of the program,
and gives you access to sequential debuggers focused on individual processes. Thus it organizes or "glues together" a collection of sequential
debuggers in the same way that a coordination language glues together a collection of sequential processes.
In the low-tech approach, we improvise the coordination debugger, perhaps by creating multiple windows with a sequential debugger running
in each. The low-tech approach sounds painful, and it is—particularly when you need to debug a large, diverse process ensemble. But it's
important to note that there is a crucial relationship between programming methods on the one hand and debugging on the other.
Master-worker programs have a fascinating characteristic: no matter how large they grow, they involve only *two kinds*of processes, and they
run in exactly the same way (in logical terms) whether they are executed with one worker or with ten thousand. During the logical-debugging
stage, we'll run such programs with a single master and a single worker (or two workers, to allow for any effects that appear only when there
is concurrency among workers). The extent of the actual parallelism in such a program is (obviously) modest. The logistics of such an
application are readily handled.
Simple master-worker programs (one master, many identical workers) give rise to more complicated variants. These may involve many
specialized masters or several pools of workers. But the general principle still holds.
#### 4.2.2 Tuplescope
The"high-tech" debugging/monitoring/visualization tool for Linda coordination frameworks is called Tuplescope. Tuplescope presents a
"visualization" of tuple space (figure 4.1). One of Tuplescope's major functions is the coordination of debugging agents, just as Linda
coordinates computation agents. Tuplescope initially offers the user a window on the contents of tuple space. This window is divided into
panes representing disjoint spheres of activity. Operations acting on the tuples in one pane are of no interest to operations acting in another
pane. Tuples and processes are represented iconically in the panes. As the computation unfolds, tuples appear and disappear, and processes
flit from pane to pane as their foci change.

Here, Tuplescope is focused on a program whose tuple space has been partitioned

```text
into four panes. The panes are rectangles; each is labeled with a template
```
summarizing the tuples whose activities are captured by the pane. (The nested
square icon is a resize button—the pane ("summation", INT, INT) has been
enlarged. A data tuple is represented by a round icon. A live tuple (process) is
represented by square-ish icon (labeled with the process's id): a fat arrow

```text
(upward pointing indicates the process's last Linda operation was an in;
```
downward, an out) or a diamond (the process is blocked). Tuples can be "moused"
to reveal the fields of a data tuple or the source for the last Linda operation
executed by a live tuple. Debug scripts can be written to specify breakpoints
based on a variety of conditions. The small window gives one such script. The
figure displays tuple space at the moment this condition was triggered. The icon
of the live tuple involved has been notched (Process 2) and the data tuple's
icon has been flattened into a black disk. The data tuple icon has been moused
to display its fields. A control panel, placed above the tuple space display,
allows the user to set and change parameters altering the kind of information
displayed, the amount, and the speed of update. It also allows the user to start
and stop execution as well as activate debugging scripts. Each of the objects
represented can be studied in greater detail by zooming in. A data tuple
close-up reveals the structure of the tuple: its fields and their values. A
process tuple becomes, upon closer scrutiny, a text window with Linda operations
highlighted, while zooming closer still lays bare the process's code and
environment to the prying eyes of a full-fledged debugger (a member of the dbx
family, for example.) Tuplescope is nice if you have it (or the equivalent for
some other coordination language). If you don't, there are ways to make do
without it. In either case, the logical problems of parallel debugging are the
same.
#### 4.2.3 Logical problems in parallel debugging Viewers all over the
county have noticed that, whenever the local TV news does a man-on-the-street
interview spot, and the topic is parallel code development, responses show a
striking consistency from Pasadena to Paramus. Everyone agrees that
*non-determinacy *and* deadlock* are the lurking dangers. The man-on-the-street
is right. These *are* problems that are peculiar to concurrent programs,
problems that sequential debugging rarely faces. We're glad to report, however,
that in most cases the dangers are actually quite modest.**Deadlock**. It's not
surprising that explicit concern with communication gives rise to a new class of
bugs.The most fundamental is deadlock. Suppose we have a program with two
components, processes *A* and *B*. If the program enters a state in which *A* is
waiting for a tuple from *B* and *B* is waiting for a tuple from *A*, the
program is deadlocked—no further progress will be made. Obviously more complex
cases can (and do) occur. In the general case, an arbitrarily-long cycle will
exist. The elements of the cycle are by turns *processes* and *resources*(in our
case, *tuples*). Each process in the cycle owns the previous tuple (is the only
process that can generate said tuple, in other words), and *wants* the next one.
A "feature" of deadlocks is that they cannot, in general, be distinguished from
other cases in which a program is progressing slowly, or has died for some other
reason (for example, some process has core dumped and the operating system
hasn't told anyone). Hence, the program developer must know how to *detect*
deadlock. We'll describe two approaches that have been used to detect deadlock
in Linda programs. (Similar approaches are possible, of course, in other
coordination languages.) Some implementations look for deadlock at run time. The
basis for such a method is a protocol that requires a check each time a process
is blocked on a tuple request. If the process about to block is the last one
*not* blocked, deadlock can be declared and we can all go home. All processes
exit after writing a status message identifying themselves and the operation
they are executing.

```text
The details of this mechanism are not trivial; in fact, efficient
```
implementations for some environments (* e.g.*, local area networks) remains a
research topic. This mechanism suffers from the limitation that it detects only
*total* deadlock. There are two common ways in which partial deadlock might
occur. It may be that two processes are mutually deadlocked, effectively
dropping out of the computation—and yet the computation is structured in such a
way that it can still complete. Or some processes may be blocked on something
other than a tuple space operation. If, in the simple example above, another
process was blocked waiting, say, for input, it would never appear blocked on a
tuple space operation, and thus the detection condition would not be satisfied.
It would be possible to refine the detection scheme at the cost of complexity
and reduced efficiency, but there's often a better way. Detecting deadlock using
Tuplescope is a simple matter: if all the process icons are in the blocked
state, the program is deadlocked. (If *any* process icon remains blocked, there
may be something wrong.) Luckily, deadlock in parallel applications is a
surprisingly infrequent problem. Mutual dependence is *not* a characteristic of
most parallel computations. (It's more frequently an issue in distributed

```text
systems; we discuss the problem again, from this point of view, in chapter 9.)
```
Typically, parallel applications use a sequence of tuples to represent the
evolving computation state. Updates to the state are often accomplished *via*
paired in and out operations. As long as the state variable is "created" (the
tuple representing the initial state is outed), its existence is preserved over
every update. Hence further updates and further read operations need never block
indefinitely. (Linda programmers do sometimes experience a problem that is
vaguely related to deadlock—they write ins that almost, but don't quite, match
their outs. The C-Linda linker flags in or rd statements for which no matching
tuple will ever be generated. It remains the case, though, that generating a
slightly wrong set of tuples—wrong in number or in content—can leave an in or rd
statement hanging.)**Non-determinism.***Non-determinism* refers to those aspects
of a program's behavior that can't be predicted from the source program. Linda's
in operation, for example, is defined in such a way that *some* matching tuple
is returned. We can't say which one. If *many* processes are blocked on similar
in statements, and an out creates a tuple that will match any one of them,
*some* blocked process will get the tuple and continue. But again, we can't
predict which one. This kind of "semantic non-determinism"—non-determinism that
is an explicit part of a language definition—isn't an attribute of Linda *per
se*, nor an attribute of parallel or coordination languages exclusively. It
arises in circumstances where imposing a fixed order is unnatural, hence

```text
semantically inappropriate, and a burden on a language's users; hence too, an
```
unnecessary restriction on the implementation. If many processes reach for a
tuple *simultaneously*, the simplest and most natural thing to say is that "one
of them gets it"—not that "the one in the red hat gets it," or "the one with
lowest process id gets it," or whatever. Any such rule is likely to be
arbitrary, and arbitrary (as opposed to logical) rules are best avoided. Not
bothering the user with arbitrary rules also, of course, frees the
implementation from enforcing arbitrary rules. The more freedom an
implementation has, the more opportunity it has to do the easiest or the fastest
thing at runtime. (The same may hold in sequential languages. The order in which
arguments to a function are evaluated is generally left unspecified—to choose
what is probably the most significant example.) Another kind of non-determinism
has to do not with the language definition (except in the most general sense),
but with the execution model. When processes execute asynchronously, we can't
say how each one's internal activities will relate to the others. On different
runs, the relationships might be different. If processes *P *and* Q* each
execute an out statement, sometime *P*'s tuple may be generated first and other
times *Q*'s may be (unless there is some logical dependence between the
operations). In the context of result and agenda parallelism, non-determinism of
all kinds tends to be a minor consideration, so far as debugging goes. In
result-parallel codes, execution is governed by data-dependencies. A given
process runs when and only when all the processes it depends upon have
completed. Processes execute simultaneously exactly when they have *no*
relationship at all. The one-worker debugging model for master-worker agenda
parallelism sharply constrains the extent to which non-determinism can occur.
This isn't a mere debugging trick, though—it points to a deeper issue. We can
debug with a single worker because the program *behaves* as if it had only one
worker, regardless of the number it actually *does* have. Worker processes have
*no* direct inter-relationships. Any significant non-determinism will occur in
the context of the master-worker relationship only. Specialist parallelism can
be more problematic. As we've seen, though, specialist parallelism is usually
implemented using message passing. If we use ordered message streams, one kind
of non-determinism is eliminated. Non-determinism still occurs with regard to
the order in which processes are allowed to append messages to a multi-source
stream, or remove them from a multi-sink stream. But most programs are
insensitive to such non-determinism, which can of course be eliminated, if need
be, by breaking one stream into many.
### 4.3 Performance Once a parallel
program has been written and debugged, it's incumbent upon the author to explore
the code's performance. If it doesn't run faster as more processors are made
available, at least up to a point, it's a failure. The usual measures of
parallel performance, and the ones we will concentrate on here, are speedup and
efficiency. *Speedup* is the ratio of sequential run time to parallel run time.
*Efficiency* is the ratio of speedup to number of processors. Unfortunately,
these apparently simple definitions conceal much complexity. For example, what
do we mean by "sequential run time"? In measuring a parallel application's
performance, we need to establish what we are gaining in *real* terms. A
parallel program is ordinarily costlier than a conventional, sequential version
of the same algorithm: creating and coordinating processes takes time. Running
an *efficient* parallel program on many processors allows us to recoup the
overhead and come out ahead in absolute terms. An *in* efficient parallel
program, on the other hand, may demonstrate impressive *relative* speedup—it may
run faster on many processors than on one—without ever amortizing the "overhead
of parallelization" and achieving *absolute* speedup. (Readers should be alert
to this point in assessing data on parallel programming experiments.) Clearly,
then, we need to benchmark our parallel programs against the "*comparable*"
sequential version. Establishing what's "comparable," though, can be tricky.
When a parallel application starts life as a sequential program, this sequential
program may be the natural comparison point. In fact, most "real" parallel
programs—programs that are developed for production use— *do* start life not as
blank sheets of paper but as ordinary, sequential programs that run too slowly.
But suppose that the most *natural* sequential starting point doesn't represent
the most *efficient* sequential strategy? The best sequential algorithm may

```text
parallelize poorly; another approach, worse in the sequential case, may
```
parallelize better. In these cases we are obliged, within reason, to compare the
parallel program to a *different* sequential version that represents our best
shot at good sequential performance. Once we've decided what our two comparison
programs will be, subtle questions remain. One of the most important and common
has to do with differences in problem size. Solving the same size problem
sequentially and in parallel can result in artifacts that reflect not the
programming effort *per se*, but the memory hierarchy of the machine used in the
comparison. We will run the sequential version on one processor of a parallel
machine, and the parallel version on many processors of the *same* machine, to
control for hardware as far as possible. But each sub-computation within a
parallel program will generally be "smaller" than the sequential computation in
its entirety. Because of the size difference, memory references in the parallel
sub-computations may have a better chance of hitting in cache, yielding a
performance benefit that is real but (in a sense) accidental. The aggregate
memory of the parallel machine may (for distributed-memory machines, probably
will) exceed the capacity of the node on which we execute the sequential
version. This too can lead to faster memory access for the parallel code. Ought
we to say, then, that a true comparison must compare the parallel program
against a sequential version running on a *different* machine, one whose
aggregate memory is the same size as the parallel machine's, or with a larger
cache to compensate for the larger problem sizes, and so on? If so, we are
writing a complex, burdensome and (ultimately) subjective list of requirements.
It's more reasonable to acknowledge the fact that, in practice, it's almost
impossible to avoid some element of "unfairness" in this kind of performance
testing. Scaling the problem to "fit" better for the sequential case represents
another course of action, but this leads to its own set of problems. Obviously
we can't *directly* compare a small sequential problem to a large parallel
problem. But we can't restrict parallel testing to small problems only, because
parallelism becomes more valuable as the problem size gets *larger*. The whole
point, after all, is to run *large*, computationally *expensive* problems
well—not to be able to improve the performance of *small* problems that run
adequately on conventional machines. Can we run a small sequential problem, and
use a performance model to extrapolate its behavior to the larger problem sizes
that we use in the parallel test runs? Yes, in principle. But different
components of the computation may scale differently, and achieving a
sufficiently accurate performance model to yield reliable figures will often
prove a difficult exercise. Our best bet, in sum, is to be reasonable, and to
accept the inherent imprecision of the assignment. We will compare a parallel
program with a "reasonable" sequential version. We'll run the sequential problem
on one node of a parallel machine, and parallel versions of the same size
problem on many nodes of the same machine. The next step is to study the
performance figures, and attempt to understand them.
#### 4.3.1 Modeling the
behavior of parallel programs As a first approximation, we can (or would like
to) model performance by an expression of the form *a */*k*+* b*, where *a*
represents the amount of time taken by the *parallelized component*of the
parallel program, *b* represents the time taken by the *serial component*, and
*k* is the number of processors. We assume that the parallel and sequential
programs are essentially the same algorithmically. Note, though, that *a *and* b
*might* not* sum to the sequential time *tSeq*. Both *a *and* b* may be larger
than their counterparts in the sequential version. It's possible (in fact
likely) that parallelization will add to both components—will add additional
work that can be parallelized and additional work that cannot be. Ideally, we
would like *a = tSeq ,* *b*= 0. But in practice, (parallelizable) overhead
*will* be introduced when we write the parallel version, and there will also be
some inherently serial parts of our algorithm (often including problem set up
and take down, for example), and some inherently serial aspects of our
coordination framework (for example, synchronization and certain communication
costs). Although we haven't explicitly indicated the dependence, both *a *and*
b* can (and likely will) vary both with *k* and and with the size of the
problem. Assuming for the moment that *a *and* b* are essentially independent of
*k*, consider what happens in two different cases: *k* is very large, and *k* is
small. First, suppose we are willing to buy performance by adding processors. We
don't care about efficiency, so long as each new processor reduces run time. As
*k* grows arbitrarily large, the ratio *tSeq */* b*sets an upper limit on
speedup. (This constraint is closely related to "Amdahl's Law": *speedup*= 1/1
-*f,*, where *f* is the fraction of the code that can be parallelized. If none
of the code can be parallelized,*f = 0*, and the maximum speedup is one—that is,
no speedup. If all code is parallelizable, then *f =* 1 and potential speedup
is, in the abstract, unlimited.) To reduce this absolute limit on speedup, our
only choice is to reduce *b*, the non-parallelizable part of the program.

Now, suppose that we are *not* blessed with an unlimited number of processors.
If *k* is bounded by *K* (*i.e*., we are presented with a fixed number of
processors), such that *a */*K*>>* b*, then *b* is no longer our major concern.
If we consider efficiency (achieved speedup divided by the number of
processors), we can derive a relation similar to the one for absolute speedup:
efficiency is limited by *tSeq* /*a*. Given a fixed number of processors, we can

```text
no longer tolerate ineffective use of a processor's computing time; we must
```
minimize time wasted on overhead, by attempting to make *tSeq* /*a* as close to
1 as possible—in particular, by minimizing the (parallelizable) overheads we
introduced when we turned our serial code into a parallel program. Of course we
need the make sure that, in the process of doing so, we don't increase *b*
significantly. In other words: when we have an unlimited number of processors,

```text
efficiency (in principle) doesn't matter. We can neglect efficiency in two ways.
We can leave some of our processors mostly idle; a mostly-idle processor can
```
still contribute to absolute performance if it kicks in exactly when we need it

```text
(at a particularly compute-intensive juncture), and then goes back to sleep. Or,
```
we can keep all of our processors busy full-time, but busy
*ineffectively*—spending much of their time on overhead introduced during the
process of parallelization. If processors waste most of their time on overhead,
we can still achieve good absolute performance: if performance is inadequate, we
simply hire more processors. (This is the federal bureaucracy model of parallel
computing). Performance in this setting is limited only by the unparallelizable
portion of the program (by *b*). Given a *fixed* number of processors, however,
we can no longer neglect efficiency. We must ensure that each processor wastes
as little of its time on overhead as possible, by reducing *a* as much as we
can. In practical terms, our two cases reduce to the following possibilities.
Either we have plenty of processors, in which case *b* may be a dominating

```text
factor; or processors are in short supply, in which case *a* may be a dominating
factor (so long as *b* remains under control); or we may have some
```
"intermediate" number of processors, in which we case we might concentrate on
*a* *and* *b*. If we build a parallel program and we aren't satisfied with its
performance, we must decide which situation holds, and concentrate on the
appropriate parameters. How do we go about this? First we need to accumulate

```text
some timing data; runtime for the sequential program, and runtime for the
```
parallel program under a representative sampling of *k*'s. Fitting the parallel
runtime data to a curve of the form *a/k + b*will tell us, first, whether the

```text
model even applies; second, the approximate values of *a *and* b*. We can use
```
this information to decide which case holds. At maximum *k*, are there enough
processors to make *b* the dominant term, are we still "processor-limited" (is
*a */* n*the dominant term), or does some intermediate case hold? To reduce *b*,
we need to look for ways to reduce the non-parallelizable component of the
overhead added by parallelization, *and* to parallelize more of the original
code. I/O is often one important target for the second effort. In a sequential
code, computation time may significantly outweigh I/O time—and so, naturally,
little effort is wasted on making I/O efficient. In the parallel version,
computation time may be reduced to a point at which I/O becomes a dominating
cost. If so, we need to worry about making I/O efficient. Another good way to
reduce *b* is to re-think the techniques that are used to synchronize
computations. Inexperienced programmers often assert too much control—for
example, they use queues or streams when bags might do. Bags (as we've
discussed) aren't terribly important in sequential programming, but good
parallel programmers use them whenever they can. Only one process at a time can
append-to or remove-from a queue, but many processes can reach into a bag
simultaneously. When we switch from data structures that force serialization to
data structures that allow concurrency, we reduce *b* and improve performance.
If our performance is "processor-limited," we'll concentrate on reducing
*a*(without increasing *b*). One useful technique for master-worker programs is
to reduce the cost of task acquisition. Much of this cost is parallelizable—many
workers can acquire tasks simultaneously—but the less time each worker spends at
this activity, the better. In more general terms, we try to reduce the
communication costs (typically as much a function of the *number* of
communication events as the amount of material communicated) associated with
each computation. *Increasing* *a* can lead to better performance, in a special
sense. When we increase the problem size, we obviously expect to increase total
run time. But for many problems, *a* is close to *tSeq*, and *b* grows more
slowly than *a* as the problem size increases, thus increasing *tSeq */* b*and
with it, maximum possible speed up. (To restate this strategy: if you don't get
good performance on a small problem, try a large problem.) For example: a matrix
multiplication code may spend *n* 2 time setting up, but *n* 3 time computing,
suggesting that maximum possible speedup will grow linearly with problem size.

```text
(To put this yet another way: the *f* in Amdahl's law is often a function of the
```
problem size—a fact that is frequently overlooked.) Repeated timing trials for
different values of *n*(for different problem sizes) can yield important
information about how *a *and* b* depend on *n*, and thus to what degree
altering problem size will alter performance. **Load balancing and work
starvation**. Although the *a*/* n + b *expression is useful, it isn't
guaranteed to be accurate. There are several reasons why it might fail. The most
important is* load balancing *. A problem that decomposes into four tasks (or
four specialists, or a four-element live data structure) won't achieve a speedup
of greater than four regardless of what *n* is (regardless, that is, of how many
processors we run it on). A more subtle but equally important effect occurs in
cases where, for example, we have 36 tasks in a master-worker program. If the
tasks are about equal in complexity, such a program is likely to finish no
faster on 17 processors than on 12. Finishing time is determined by the most
heavily-burdened worker. In the 12-processor case, the worker with the greatest

```text
burden (and for that matter every other worker) does three tasks. In the
```
17-processor case, the worker with the greatest burden *still* does 3 tasks. 15
of the others do only two tasks each—but thereafter, they simply idle while the
others finish up. We refer to this kind of problem as* work starvation *, a
phenomenon that occurs when a good load balance is logically impossible. Work
starvation is the extreme case of a bad load balance, but it represents (after
all) merely the end-point of a spectrum. Even where a reasonable load balance
*is* possible, achieving it may not be easy. The research literature is full of*
static scheduling *studies—recipes for achieving a good distribution of the
components of a parallel program to the processors of a parallel machine. (We
examine static scheduling in greater detail in chapter 8.) Often, however, we
can avoid load balancing problems by using the master/worker approach. This
strategy usually achieves a good load balance without even trying: because
workers grab new tasks as needed, we reach an even load distribution
dynamically.

There *are* cases in which dynamic task assignment isn't sufficient in itself.
Suppose our tasks vary greatly in complexity, and suppose that the last task to
be grabbed happens to be a complicated one. Every worker but one may sit idle

```text
(all other tasks having been completed) while the final, time-consuming task is
```
processed. Such problems are best attacked by switching from a task bag to an

```text
ordered task queue; we return to this problem in chapter 6.**Granularity
```
knobs.**The most crucial issue in parallel performance debugging is
*granularity*. A coordination framework takes time to execute. The time invested
in the coordination frame—in creating processes and moving information among
them—must (in some sense) be over-balanced by the time invested in computing. If
it isn't, the overhead of coordination overwhelms the gain of parallelism, and
we can't get good performance. Concretely, a message-passing program won't
perform well if each specialist doesn't do "enough" computing between

```text
message-exchange operations; a live data structure program won't perform well if
each live data element does "too little" computing; a master-worker program
```
won't perform well if the task size is "too small". Exceeding the granularity
bounds can lead to excessively large *a*'s or *b*'s or both. If the overhead
parallelizes, we may get an application with good relative speedup, but poor
absolute performance. If the overhead doesn't parallelize, our excessively
fine-grained program may not speed up at all. Little can be said about "correct
granularity" in absolute terms, beyond the fact that, in every hardware
environment, and for every coordination language, a "crossover point" exists
beyond which an application is too fined-grained to perform well. In most cases,
it's up to the programmer to discover this crossover point for himself. Its
rough attributes can be established on the basis of hardware. The cost of
*communication* is usually the dominating factor in determining the crossover
point. Adding a tuple to tuple space costs tens of microseconds on
current-generation shared-memory parallel computers, hundreds of microseconds on
distributed-memory parallel computers, tens of milliseconds on local area
networks. (The Linda Machine [ACGK88] is an exception. It's a distributed-memory
machine on which Linda operations run as fast as they do on shared-memory
architectures.) These differences reflect the obvious architectural
distinctions: on networks, bits must be transported *via* a slow channel, on
distributed-memory machines they go *via* a fairly fast channel, and on
shared-memory machines, they barely get transported at all. (They are merely
copied from one part of memory to another.) Exactly the same hierarchy of costs
would hold if we were discussing message passing, or any other communication
model, instead of Linda. (It's important to note that the cost of executing
Linda operations is dominated by the cost of transporting bits, not by the
high-level aspects of the Linda model. It's often conjectured that Linda must be

```text
expensive to use because of associative matching; luckily, this conjecture is
```
false. Almost all matching is handled not at runtime but at* compile time *and*
link time *, by an optimizing compile-time system which, in effect, "partially
evaluates" the runtime library with respect to a user's application
[Car87,CG90]). If the task size in any setting is* less than *the amount of time
it takes to find the task and report the results, we are paying more in overhead
than we're accomplishing in computation, and it's a good bet that our
application cannot perform well. Clearly we must avoid excessively-fine
granularity. But granularity that's too large is also bad. It can lead to
load-balance problems or (in the limit) to work starvation. Many smaller tasks
lend themselves to a more-even distribution of work than fewer, larger tasks.
The best way to deal with the granularity issue is, we believe, by building
applications with* granularity knobs *. It's desirable that granularity not be a
*fixed* attribute of a program, but rather something that can easily be
adjusted. We can use this tunability to achieve good performance on our initial
host machine. We may well need to twiddle the dial again when we port to a
different machine, particularly if the new environment has very different
communication hardware. We make this technique concrete in the next chapter.
###
4.4 Exercises 1. We've discussed the fact that tuple space is inherently

```text
non-deterministic: *some* process gets a matching tuple; a process gets *some*
```
matching tuple. Non-deterministic isn't the same as *random*: if tuple space
were *random*, *which* process or *which* tuple would be a random choice

```text
(without pattern over arbitrarily-many repetitions). (*a*) Suppose that tuple
```
space operations *were* random in their behavior. Discuss the implications from
a programmer's and also (in general terms) from an implementor's perspective.

```text
(*b*) Suppose, again, that tuple space is no longer non-deterministic; suppose
```
now that tuple space operations are defined in a time-ordered way. A process
that executes an in or a rd gets the* oldest* matching tuple (the one that was
added to tuple space at the earliest point). If many processes are awaiting a
tuple, the oldest blocked process (the one that started waiting at the earliest
point) gets the tuple. Discuss the implications, again, from a programmer's and
an implementor's perspective. 2. Read the "timing and tracing tools" section of
the appendix (section 5). Now, do some measurements: how long do out, in and rd
require on you implementation? How do their execution times vary with the size
of the tuple? With the matching pattern specified by in or rd? With the number
of active processes? 3. How long does it take to add a tuple to and remove it
from a bag in your implementation? How long does it take to append elements to
and remove them from in-streams (all three variants) and read streams? How long
does it take to look elements up and add them to the hash table that was
described in exercise 2 in the previous chapter? 4. How do granularity issues
arise in human organizations? To what extent have higher communication rates
altered the limits of acceptable granularity? To what extent does "finer task
granularity" mean less worker autonomy?
