# 6 Databases: Starting with Agenda
We turn now to "real problems." Our intention is to investigate three programs in detail: a natural agenda-parallel, a natural result-parallel

```text
and finally a natural specialist-parallel example. The final *solutions* won't fall into these three categories, of course; the starting points will.
```
In each case, we'll arrive at an efficient agenda-parallel solution in the end. But our travels from the starting point to the goal will follow
three very different routes. (Chapter 9, by the way, presents a program which does *not* get transformed into an agenda parallel version.)
Our intention in each chapter is to describe a problem, explain how a parallel solution was developed, and consider the performance of the
solution. In concluding sections and in the exercises, we discuss the application of related techniques to other problems. A real problem isn't

```text
necessarily a complicated one; the problem we discuss in this chapter is quite simple. The problems in succeeding chapters are admittedly
```
somewhat more complicated, but easily encompassed nonetheless in a single chapter each. Parallelism as a *real problem-solving technique* isn't restricted to enormous codes or complex problems. And great progress can be made through the use of simple techniques, so long as the
techniques are chosen well and applied carefully.
### 6. 1 Big Issues

```text
(We begin this and each of the following two chapters by distinguishing the *major themes* common to all three from the *special
```
considerations* that distinguish each problem.)*Main themes:*
*Load balance*is crucial to good performance. The master-worker structure is a strong basis for building parallel applications
with good load-balance characteristics.*Special considerations:*
*Watermark techniques*can meter the flow of data from a large database to a parallel application.*Large variance in task size*calls for special handling in the interests of good load balance. An ordered task stream is one possible approach.
*Hardware and operating-system support for parallel I/O would be nice,* but can't be assumed. We need to develop

```text
techniques that don't rely on it; but programmers should also know how to take advantage of it.
```
### 6.2 Problem: database search with a complex search criterion
Suppose users need to search or examine every element in a database, and a fair amount of computing is required to examine each element .
The example we will discuss involves DNA sequences. When a new sequence is discovered, geneticists may need to find out which
previously-unraveled sequences the new one resembles. To do so, they may need to examine a large database of existing sequences.
Obviously, the time necessary to examine the database increases with the size of the database. As the database grows, examination time may
go from seconds to minutes to hours, crossing a series of cost and inconvenience thresholds along the way.
Comparing two long sequences is time-consuming, and such comparisons can be parallelized. However, geneticists usually require not a
single comparison, but an extensive series of comparisons in which the target is compared against all or a major portion of a large database.
It's clear, then, that we should consider parallelizing not the sequence-to-sequence comparison, but the database search—our program can
perform many *sequential *comparisons* simultaneously*. This is the approach we'll discuss here. Parallelizing the comparison of two

```text
sequences (and forgetting about the rest of the database) is an interesting approach also; we discuss it in the next chapter.
```
The DNA database problem closely resembles others in many domains. There are lots of reasons to walk through every element of a large
database, performing computations along the way. Consider one of the paradigm problems of commercial data processing, preparing bills.
Phone companies, for example, devote enormous amounts of computing power (typically in the form of large mainframes) to a perpetual
slog through swamps of customer records. Bills may go out to each customer monthly, but the bill-preparation process may go on full-time,

```text
around the clock. (Such problems can be dominated by sequential; but I/O as well as computation can be parallelized.) There are many
```
related examples in commercial data processing. Many types of library searches fall into this category too—you might have a database of
chemical abstracts, or images, or news stories, and you need to apply some comparison-and-retrieval function to each one. The
DNA-database problem itself is increasingly significant, as the Human Genome project adds enormously to the volume of our genetic
knowledge. The larger point, of course, has nothing to do with this particular problem, nor even with the large class of related problems (like

```text
bill preparation); we are treating database search merely as one example of natural agenda parallelism. But the database problem does have
```
wide applicability.
### 6.3 The problem description, and the agenda approach

```text
You are given a function *compare*; it accepts two character strings representing DNA sequences as arguments; it yields a positive integer
```
result. The closer the biological resemblance between sequences *s *and* t*, the greater the result yielded by *compare(s,t)*. Given a target
sequence *T*, you must compute *compare(s, T)* for every sequence *s* in a database of sequences. You return an ordered list of all sequences
whose closeness to *T* is greater than some cutoff value (or the single closest match, as the case may be).
Recall that we posed essentially this problem earlier, in discussing agenda parallelism and its manifestation in master-worker programs :
Suppose we have a database of employee records, and we need to identify the employee with (say) the lowest ratio of salary
to dependents. Given a record *Q*, the function *r(Q)* computes this ratio. The agenda is simple: "apply function *r* to all

```text
records in the database; return the identity of the record for which *r* is minimum." We can structure this application as a
```
master-worker program in a natural way: the master fills a bag with data objects, each representing one employee record.
Each worker repeatedly withdraws a record from the bag, computes *r* and sends the result back to the master. The master
keeps track of the minimum-so-far, and when all tasks are complete, reports the answer.
This is exactly the kind of solution we'll describe here.
### 6.4 The sequential starting point
For the moment, let's forget about parallelism. How do we solve this kind of problem sequentially? The basic control structure is a loop in
which the "important" computation is independent from one iteration to the next. But there is some auxiliary set up (*e.g*.: setting the value of
the iteration variable) and clean up (*e.g*.: collating results from the computation) that may involve inter-iteration dependencies: swinging
through the loop for the *nth* time may require that we have already swung through it for the *n -*1 *st* time. In this particular case, we iterate
over all sequences in the database. The set-up extracts one sequence from a data base, the computation assesses its similarity to the target
and the clean-up updates the "best" result if the current result exceeds the best so far:

```text
while (TRUE) {
 done = get\_next\_seq(seq\_info);
 if (done) break;
 result = compare(target\_info, seq\_info);
 update\_best(result);
```
}

```text
output\_best();
```
We assume appropriate definitions for the data structures and functions. That we *can be* oblivious to this information, and in particular to the
exact details of the comparison function, is an important feature of this approach. The parallel code that we will develop treats the
comparison function as a "black box." This means not only that we can develop the coordination framework without worrying about the

```text
computations to be embedded within it; it also makes it possible to offer the user a *choice* of computations. A number of different
```
sequence-comparison functions are available. The user might be invited to plug any function he likes into our framework. For that matter,
virtually any kind of exhaustive database search can be handled using the general approach we're describing.
Agenda parallelism is a natural for this domain: our focus is drawn to the "how," not the "who" (specialists) or the "what" (result). We
conceive of this problem in terms of a simple agenda of tasks (look at every record in the database) which many workers can clearly attack
simultaneously. As we'll see, the natural solution is also an efficient one. It requires some tuning and optimization, but no basic
transformation.
As usual, though, real problems are a bit more complicated than ideal ones. Our sequential code fragment presents two possible barriers to a
simultaneous attack by identical workers. We must examine the following question: how are the *ith *and* i*+ 1 *st* iterations of get\_next\_seq()
and update\_result() related? We can do lots of comparisons simultaneously, but we can't necessarily do lots of gets and updates
simultaneously. (In principle, we might also worry about simultaneous invocations of compare() itself. But in this and many related
domains, it is in fact okay to perform many simultaneous comparisons.)
get\_next\_seq(seq\_info) seems in fact, for our application, to be inherently sequential. If we have one file on a conventional file-system
device, we can't do many *read* s simultaneously. We must do them one at a time. We'll accept this limitation for now (we will discuss ways
of relaxing it later on). Thus, our agenda contains one task that should be broken off and attacked sequentially by a single worker. One
process in our ensemble will be devoted to reading the database record-by-record, making its contents available to the other workers.
update\_result() finds the largest result (or the *n* largest) in the set of all results. There is no reason to believe that the outcome will be
sensitive to the order in which the results are processed. For the problem at hand, there is in fact no such sensitivity. It is conceivable,
however, that the update process might be asked to log all results in a file. While the actual order in which results appear in the file could be
unimportant, the fact that they are all being written to the same file suggests (on analogy with data input) that invocations of
update\_result() be executed serially, and for the time being we will work under this assumption. Thus, we will commission another process
to perform result-updating sequentially, result-by-result. Later on we will be a bit more careful to distinguish between the "real" work of this

```text
routine (arriving at an overall best value) and its possible role in generating output.
```
### 6.5 A first parallel version
Armed with this understanding of the dependencies between iterations of the basic loop, we can sketch our first parallel version. We will
have sequential input process (a master), a swarm of comparison processes (workers), and a sequential output process (standing on its own
or, conceivably, subsumed by the master process). We can now expand our sequential code into skeletons for three kinds of processes:**Input**while (get\_new\_seq(seq\_info)) {

```text
 generate\_task(seq\_info);
```
 }**Comparison**while (get\_new\_task(task\_info)) {

```text
 generate\_result(compare(target\_info, task\_info);
```
 }**Output**while (get\_result(result\_info)) {
 update\_result(result\_info))
 }
The reader is entitled to feel a little misled—aren't we adding a dash of specialist-parallelism to our agenda approach? Yes. It's reasonable to
describe the *input* and the *output* processes precisely as specialists. This kind of heterogeneity is crucial to real programming: the point is to
find a good solution, not to impose a dogmatic framework. It's important to note, though, that the bulk of the computation will be handled by
the multiple workers attacking the *comparison* tasks. What we've sketched is only a small variation on the standard master-worker
coordination framework.
In fact, returning to the issue of input and output, the output task is trivial for our application. There is no good reason to create two separate

```text
processes for input and output; it's more reasonable to have the master do both.
```
With this modification, we can now present the code (figures 6.1 and 6.2). Several details will help in understanding it. All records in the

```text
database have a fixed-size header followed by the (variable length) sequence; dbe holds the entire record, while dbs is an offset that points
to the sequence proper. The header includes information that identifies the sequence; this information is reduced to some unique integer
```
label by get\_db\_id(). The actual similarity assessment is accomplished by the routine similarity(). Don't worry about its seemingly strange
collection of arguments for now. This routine is recycled for further use in the next chapter, and the meaning of its arguments will become

```text
clear later on.**Figure 6.1****Database search: First version (master)****char**dbe[MAX + HEADER], target[MAX];**char**\* dbs = dbe+HEADER;
real\_main(argc, argv)**char**\*\* argv;
```
{

```text
 t\_length = get\_target(argv[1], target);
 open\_db(argv[2]);
 num\_workers = atoi(argv[3]);*/\* Set up. \*/***for**(i = 0; i < num\_workers; ++i)**eval**("worker", compare());**out**("target", target:t\_length);
```
 /\* *Loop putting sequences into tuples* \*/

```text
 tasks = 0**while**(d\_length = get\_seq(dbe)) {**out**("task", get\_db\_id(dbe), dbs:d\_length);
 ++tasks;
```
 }

```text
 close\_db();*/\* Get results. \*/* real\_max = 0;**while**(tasks--) {**in**("result", ? db\_id, ? max);**if**(real\_max < max) {
 real\_max = max;
 real\_max\_id = db\_id;
```
 }

```text
 }*/\* Poison tasks. \*/***for**(i = 0; i < num\_workers; ++i)**out**("task", 0, "":0);
 print\_max(db\_id, real\_max);
}**Figure 6.2****Database search: First version (worker)****char**dbe[MAX + HEADER], target[MAX];**char**\* dbs = dbe+HEADER;*/\* Work space for a vertical slice of the similarity matrix\*/* ENTRY\_TYPE col\_0[MAX+2], col\_1[MAX+2], \*cols[2]={col\_0,col\_1};
compare()
```
{

```text
 SIDE\_TYPE left\_side, top\_side;**rd**("target", ? target:t\_length);
left\_side.seg\_start = target;
left\_side.seg\_end = target + t\_length;
top\_side.seg\_start = dbs;**while**(1) {**in**("task", ? db\_id, ? dbs:d\_length);**if**(!d\_length)**break**;*/\* Zero out column buffers. \*/***for**(i=0; i <= t\_length+1; ++i) cols[0][i]=cols[1][i]=ZERO\_ENTRY;
 top\_side.seg\_end = dbs + d\_length;
 max = 0;
 similarity(&top\_side, &left\_side, cols, 0, &max);**out**("result", db\_id, max);
```
 }**out**("worker done")
}
This code uses the simple but important "poison pill" technique to shut itself down. When the master has gathered the results it needs, it
dumps a task tuple with a special value into tuple space. A worker ingesting this death tuple spits it back out into tuple space, then quietly
falls on its sword.* About the program examples in this and in subsequent chapters:*the code given in the figures is derived from, but not
identical to, working code. In order to clarify the presentation (and in particular to fit each module onto a single page),
we've omitted int declarations, void function declarations, struct definitions and #define constants. We've also omitted
various sanity checks for error conditions that, while necessary for robust code, are logically irrelevant. Finally, in times of
acute need, we have resorted to replacing statement conditionals by expression conditionals. Thus, instead of

```text
if (p) then s1 else s2,
```
we have

```text
(p) ? e1 : e2.
```
e1 and e2 are ordinarily the same as s1 and s2. This transformation typically allows us to collapse four lines into one, but
the effect may be jarring to readers unfamiliar with C. They can find an explanation of expression conditionals (should they
feel the need for one) in any good C text.
The transformation from a sequential to a parallel program has been easy (so far)—almost trivial. But this simple example illustrates many
aspects of the typical candidate for agenda parallelism. Often, problems in this class are (or can be viewed as) iterative, with each iteration
being largely independent of the others. Where there are dependencies, they often occur in the parceling-out of input data or the collation of
the results, not in the computation that transforms a bit of data from input to result. When starting from a sequential code that exploits this
structure, it's often extremely simple to convert the sequential control loop into parallel control loops, as we've just done. (In fact, the
transformation is so simple that in many cases it can be performed automatically by a parallelizing compiler, or expressed via primitive
syntactic constructs like "execute all iterations of this loop simultaneously"—a PARDO, DOALL, forall or some other construct as the case
may be. But these alternatives grow far less effective when the going gets a bit tougher than this, as it will—when the required coordination
framework isn't quite so simple. General-purpose languages like C-Linda can handle the simple*and *the complicated cases. And as we've
seen, the easy cases are easily handled in C-Linda.)
### 6.6 Second version: Adding "Flow Control"
So far, we've kept the problem specification simple. It's now time to confront issues that will occur when we try to solve real problems using
this program. In the process, we'll expose some deficiencies in our first-cut solution.

```text
In reality, we will want to search large databases. (The genetics databases of interest are currently tens of megabytes large; in the future they
```
will contain hundreds, potentially thousands of megabytes of data.) Furthermore, the databases to be searched contain records that vary
enormously in size. (Genetics databases currently contain sequences ranging in size from tens to ten-thousands of bases.) Both of these facts
present challenges to our parallel code. We'll attack the large-database problem here and the large-variance problem in the following section.
Clearly, we can't assume that a large database will fit*in toto *into main memory. But our current program rests implicitly on exactly this
assumption. There is nothing to prevent the input process from running so far ahead of the comparison workers that it dumps the entire
database into tuple space before the workers have even made a dent in the accumulating pile of sequences.
This problem didn't arise in our sequential version. The sequential code required merely that the target and one database sequence be

```text
inspected (hence be present in memory) at any given time. In the abstract, we could do something analogous in the parallel version. We
```
could allow exactly one sequence per worker into tuple space at any given time. But this scheme forces a greater degree of synchronization
between the input process and the workers. There is no*a priori *reason why the input process should have any idea what the comparison

```text
workers are up to; keeping it informed complicates the coordination framework and adds overhead.* Pop Quiz:*Sketch a solution that works in this way: at any given time, tuple space holds at most*n *sequences, where*n *is the
```
number of workers.
There are other ways of achieving the same end, but they are also problematic. We could have every worker read sequences directly for
itself as needed. This will only be acceptable, however, if we have parallel I/O capabilities on our machine: we must have the hardware and
the software to allow many simultaneous accesses to a file. Some parallel-programming environments provide this kind of support, but most
don't (yet). It isn't safe to assume that we have it.
The case for an oblivious input process is strong, but the problem remains: this process could potentially flood tuple space with sequence
data. Given that it's harder to force vendors to supply adequate parallel file access capability than it is to modify our input/worker
synchronization, we choose the latter strategy. We use a "high watermark/low watermark" approach to control the amount of sequence data
in tuple space. (See for example Comer [Com84] for a discussion of watermark control in operating systems.) The idea is to maintain the
number of sequences between an upper and a lower limit (the high and low watermarks). The upper limit ensures that we don't flood tuple
space, the lower limit that we don't parch workers (that workers will be guaranteed to find a sequence tuple whenever they look for one).

```text
(Strictly speaking, we should be concentrating not on the number of sequences but on the total*length *of the sequences in tuple space. We
```
don't do this simply because in practice, we don't need the extra "accuracy" of the latter approach.)
To carry this scheme out, we need some extra synchronization. Whenever a worker completes a comparison, it dumps a done tuple into
tuple space. To start out, the input process outs*h *sequences, where*h *is the high-water mark. It then collects*d *done tuples, where*d *is the

```text
difference between the high and low watermarks; whereupon it resumes pumping sequences out. Both limits are fuzzy. The input process's
```
count of outstanding sequences is approximate—it's always *≥* the "true" number. In practice this presents no difficulties: watermark
schemes, this one included, rarely require that upper or lower bounds be hit precisely.
The code for the watermark version is given in figure 6.3. Only the master required modification, and so we give its code only. The workers

```text
are the same as they were in the previous version.**Figure 6.3****The modified master, with watermarking****char**dbe[MAX + HEADER], target[MAX];**char**\* dbs = dbe+HEADER;
real\_main(argc, argv)**char**\*\* argv;
```
{

```text
 t\_length = get\_target(argv[1], target);
```

```text
 open\_db(argv[2]);
 num\_workers = atoi(argv[3]);
 lower\_limit = atoi(argv[4]);
 upper\_limit = atoi(argv[5]);*/\* Set up. \*/***for**(i = 0; i < num\_workers; ++i)**eval**("worker", compare());**out**("target", target:t\_length);
```

```text
/\**Loop putting sequences into tuples* \*/ real\_max = 0; tasks =
0;**while**(d\_length = get\_seq(dbe)) {**out**("task", get\_db\_id(dbe),
dbs:d\_length);**if**(++tasks >upper\_limit)*/\* Too many tasks, get some
results. \*/***do { in**("result", ? db\_id, ? max);**if**(real\_max < max) {
real\_max = max; real\_max\_id = db\_id;**} } while**(--tasks > lower\_limit); }
close\_db();*/\* Get remaining results. \*/***while**(tasks--) {**in**("result",
? db\_id, ? max);**if**(real\_max < max) { real\_max = max real\_max\_id =
db\_id; } }*/\* Poison tasks. \*/***for**(i = 0; i < num\_workers;
++i)**out**("task", 0, "":0); print\_max(db\_id, real\_max); }* Pop Quiz:*Sketch
```
a program in which the input process always knows the*exact *number of
outstanding sequences.
### 6.7 Third Parallel Version: Ordering Sequences to
Improve Load Balancing Having addressed the issue of fitting a large database
into a small space (an issue of interest for*all *database applications), let's
tackle the implications of wide variance in sequence length. The first question
is "what's the problem?" It comes down to*load balance *. Suppose we have lots
of short sequences and one very long one. Suppose that the last task chosen by
the last worker involves the very long one. While every other worker sits idly

```text
(all other tasks having been completed), this one unfortunate worker hacks away
```
at his final, extra-long task. For example: suppose we have a database
consisting, in aggregate, of 106 bases, that the longest single sequence is 104
bases and the rest are of roughly equal length. (Say they're 100 bases each.)
Now assume that we have deployed 20 comparison workers to search this database.
If the long sequence is picked up near the end of the computation, we will have
one comparison worker scanning ~ 60,000 bases (* i.e.*an extra ten thousand)
while the rest scan ~ 50,000 bases each. Assuming that each worker's compute
time is proportional to the total number of database bases it scans, we see that
we will achieve a speedup of ~ 17, significantly less than the ideal 20.* Pop
Quiz:*Why? Justify the claimed 17-times speedup. If, on the other hand, the long
sequence is picked up early, the natural load balancing properties of our
program will result in all workers doing about the same amount of work, and
together achieving a speedup close to the ideal. This last observation suggests
an obvious attack on the problem. As part of the work done in preparing the
database, order it longest sequence first. Then, instead of a bag of sequences,
maintain an ordered list of sequences in a one-source multiple-sink stream. The
comparison workers process this stream in order, ensuring that the longest
sequences are done first, not last. Once again, we solve a problem by
introducing additional synchronization. This time it isn't between the input and
comparison processes

```text
(recall there is no interprocess synchronization for the writer of a
```
one-source/many-sink stream), but between the comparison processes themselves,
in the form of the tuple used to index the stream. Here, synchronization is
added to address an efficiency concern—but adding synchronization to solve an
efficiency problem is extremely non-intuitive. We've balanced the load across
the workers, but at the cost of introducing a potential bottleneck. Each worker
needs to access the index tuple to acquire a sequence (a task). Suppose that the
time required to do a task is too short to allow all other workers to access the
index tuple. In this case, when the first worker (now done with the last task)
reaches for the index tuple again, it's likely that at least one other worker
will also want to claim the index tuple. The more workers in the field, the more
time will be spent contending for the index tuple. For example: suppose that
updating the index tuple takes 1 unit of time and*every *comparison takes the
same amount of time, say 100 units. Now if we start 10 workers at once, the time
at which they actually start doing comparisons will be staggered: at time 0, one
worker updates the index tuple (it will begin computing at time 1), some other
worker grabs the index tuple at time 1 (and start computing at time 2) and so
on. The last worker gets his shot at the index tuple at time 9 and starts
computing at time 10. As of time step 10 all workers are busy, and they will
remain so until time step 101. At this point the first worker will have finished
its first task. It proceeds to grab the index tuple in order to learn the
identity of its next sequence. The process repeats, with the result that all

```text
workers (except for a brief start-up and shut-down phase) are kept busy with
```
only a 1% task-assignment overhead per task. If we have 200 workers, however, we
have a problem. The first round of work will not have been played out until time
200, but the first worker will be ready for more work at time 101! On average,
half our workers will be idle at any given time awaiting access to the index
tuple. Note that, under the ideal circumstances assumed here, the performance of
the code will improve in a well-behaved way up through 100 workers, then
abruptly go flat. (In the less-than-perfect real world, performance may degrade
instead of flattening out—heavy contention for the index tuple may result in
increased access time, making things even worse. The effect will likely kick in
sooner than the ratio of average comparison time to access time would predict.)
If the user's understanding was based on an purely empirical study of the
program's performance, there would be no warning that a performance collapse was
imminent. This problem needs to be dealt with pragmatically. Actual problems

```text
must be solved; problems that are merely "potential" need not be (and*should
```
*not be, to the extent that a solution complicates the code or increases
overhead at runtime). As we discuss in the performance section, for typical
genetics database searches using a modest number of workers (under 100), there
is no index-tuple bottleneck. For other forms of this problem, in other machine
environments, there might be. A solution is obvious: use many index tuples
instead of one.* Pop Quiz:*Develop a multiple-index program. Workers might be
assigned to a sequence stream at initialization time, or they might choose
streams dynamically. All streams can be fed by the same master or input process.
We've arrived at a realistic, practical solution to our search problem.
Realistic, because we've used a strategy that will enable this code to handle
virtually any size database. Practical because our program is highly parallel,
and we've taken pains to ensure a reasonably good load balance. We can install
one final refinement, however. The actual constraints on update\_result() are
less severe than we've planned for. We collapsed two distinct jobs, result
collation and output, into this one function. In the actual case of interest, we
don't need to generate output for every result—just for the best (or*n *best)
results, where*n *is much smaller than the total number of sequences. Hence we
don't need to serialize the invocations of update\_result() (or at least, we
don't need to serialize them completely). We can migrate the update
functionality from update\_result() to compare() by having the latter keep track
of the best result (or best*n *results) its worker has seen so far. When a
worker's last task is complete, it reports its best results to the master. The
master collects these local results and reduces them to one global result. The
goal, of course, is to reduce the volume of data exchanged between the workers
and the master (and the attendant overhead), and to parallelize the best-result
computation. Figures 6.4 and 6.5 present the final database code. Workers use
the index tuple to manage the task stream. The result tuple has now become two
tuples, labeled task done and worker done. The first signals task completion to

```text
the master, for watermarking purposes; the second holds each worker's local
```
maximum.**Figure 6.4****Database search: Final version, using streams

```text
(master)****char**dbe[MAX + HEADER], target[MAX];**char**\* dbs = dbe+HEADER;
real\_main(argc, argv)**char**\*\* argv; { t\_length = get\_target(argv[1],
target); open\_db(argv[2]); num\_workers = atoi(argv[3]); lower\_limit =
atoi(argv[4]); upper\_limit = atoi(argv[5]);*/\* Set up. \*/***for**(i = 0; i <
num\_workers; ++i)**eval**("worker", compare());**out**("target",
target:t\_length);**out**("index", 1);*/\* Loop putting sequences into tuples
\*/* tasks = 0; task\_id = 0;**while**(d\_length = get\_seq(dbe))
{**out**("task", ++task\_id, get\_db\_id(dbe), dbs:d\_length);**if**(++tasks >
```
upper\_limit)*/\* Too many tasks, get some results. \*/***do in**("task

```text
done");**while**(--tasks > lower\_limit); }*/\* Poison tasks. \*/***for**(i = 0;
i < num\_workers; ++i)**out**("task", ++task\_id, 0, "":0); close
db();**while**(tasks--)**in**("task done");*/\* Clean up\*/* real\_max =
0**for**(i = 0; i < num\_workers; ++i) {*/\* Get results \*/***in**("worker
done", ?, db\_id, ? max);**if (**real\_max < max) { real\_max = max;
real\_max\_id = db\_id; } } print\_max(db\_id, real\_max); }**Figure
```
6.5****Database search: Final version (worker)****char**dbe[MAX + HEADER],

```text
target[MAX];**char**\* dbs = dbe+HEADER;*/\* Work space for a vertical slice of
```
the similarity matrix\*/* ENTRY\_TYPE col\_0[MAX+2], col\_1[MAX+2],

```text
\*cols[2}={col\_0,col\_1}; compare() { SIDE\_TYPE left\_side,
top\_side;**rd**("target", ? target:t\_length); left\_side.seg\_start = target;
left\_side.seg\_end = target + t\_length; top\_side.seg\_start = dbs; local\_max
= 0;**while**(1) {**in**("index", ? task\_id,);**out**("index",
task\_id+1);**in**("task", task\_id, ? db\_id, ? dbs:d\_length);*/\* If poison
task, dump local max and exit. \*/***if**(!d\_length)**break;***/\* Zero out
column buffers. \*/***for**(i=0; i <= t\_length+1; ++i)
cols[0][i]=cols[1][i]=ZERO\_ENTRY; top\_side.seg\_end = dbs + d\_length; max =
0;
```

```text
similarity(&top\_side, &left\_side, cols, 0, &max);**out**("task done
");**if**(max > local\_max) { local\_max = max; local\_max\_id = db\_id; }
}**out**("worker done", local\_max\_id, local\_max); }
```
### 6.8 Performance
analysis While developing this code, we've made some isolated observations about
various aspects of its performance. If we are to be reasonably sure that the
code is working as designed, and in order to have some confidence in projecting
its performance to other machines, we need to develop a model of its
performance. We will proceed informally here, but the reader can carry this out
to a surprisingly exact level of detail. (While much is made of nondeterminacy
effects in parallel codes, the sort of codes being developed here, the
underlying Linda implementations, and the hardware architectures are usually
capable of delivering repeatable performance [within a few %]. This is more than
sufficient to permit study in enough detail to diagnose problems in the program
[and even the occasional hardware glitch—a dead node on a 20 node shared memory
machine is not immediately obvious but does become evident in careful
performance testing].) The serial code has two major components, the I/O and the
comparisons. The I/O cost is essentially a linear function of the length of the

```text
database (we can ignore the target sequence for all "real" problems). The
```
comparison cost is linear in the product of the lengths of the target and the

```text
database. We've made no attempt to parallelize the I/O* per se *; rather, all
```
I/O is done by the master in parallel with the worker's attack on the comparison
tasks. Thus the amount of time spent on I/O is essentially the same in the
serial and the parallel programs, but it will overlap with the computations (of
course, it probably won't be *exactly* the same, since the I/O pattern has
changed from an input/compute cycle to input/out cycle with flow control).
Likewise, each comparison takes the same amount of time it used to take (in the
sequential version), but many comparisons will proceed in parallel. The parallel
version bears the added cost of the Linda operations. If *D* is the number of
sequences in the database and there are *K* workers, we'll need to do *D* outs
in series, *D*+*K* in/outs of the index tuple in series (although some of the in
costs can be overlapped with other ins and with one out), *D* ins of database

```text
sequences (in parallel), and *K* out/ins of result summaries (again overlapped
```
to some extent). Assuming that *D* dominates *K*, we can lump all of these
synchronization costs together into a single term, *TSynch*, which will grow
with* D.*We assume that each worker ends up processing about the same
total*length *of database sequences. (This assumption should be checked by
collecting this datum for each worker and reporting it at the end of the
computation.) Thus the runtime for the parallel version will be the maximum of
three terms:* i) tIO *, time needed to do the I/O (including outing the
sequences and ining the results).* ii) tSeq */K + t *TO*(D/K), the parallelized
execution time. *tSeq* is the the sequential runtime. *tTO* is the
*parallelizable* part of the overhead associated with each task assignment. *D*
is the number of tasks (that is, the number of sequences), and *K* is the number
of workers (which must be ≤*D*).* iii) tSynchD *, the non-parallelizable
synchronization costs, where *tSynch* is the per task* non-parallelizable
*overhead. Empirically, *ii* dominates the first and third expressions for
"moderate" numbers of workers. We've discussed the program modifications

```text
(multiple index-tuples and, possibly, parallel I/O) that are called for when
```
large numbers of workers are deployed. Figure 6.6 shows the speedup achieved by
the program in figures 6.4 and 6.5. The graph shows speedup achieved by a given
number of *workers*(not processes: the master is excluded) relative to a
comparable sequential program (* i.e.*a C, not a C-Linda program) running on a
single processor of the machine at issue. The graph shows data obtained on an
18-processor Encore Multimax and a 64-processor Intel iPSC/2. (Recall that the
Encore is a shared-memory and the Intel a distributed memory, hypercube-shaped
parallel computer.) The test database contained 731 sequences totaling roughly

```text
170,000 bases. The longest sequence was 468 bases; hence no single sequence
```
dominates the computation on machines with fewer than 360 processors. On the
Multimax, we used a target sequence of length 200 bases. The sequential time was

```text
661 seconds; our C-Linda program ran in 42 seconds using 16 workers. The iPSC/2
```
runs used a target of 800 bases: since this machine is about 4 times larger than
the Multimax, we increased the computation size by a factor of 4 to establish a
comparable timing base when running with a full machine. The sequential time was

```text
2023 seconds; the C-Linda program using 60 workers ran in 34 seconds.
```

One final significant performance issue must be raised here, but can only be
resolved in the next chapter. We've discussed the ordering of tasks by
length-of-sequence in the interests of a good load balance. It's possible,
though, that we might search a partition of the database whose character is so
pathological that a good load balance is*impossible *using the methods discussed
in this chapter. It may be that the cost of comparing a single very long
sequence to the target dominates the cost of the entire remainder of the search.
That is, if the first worker starts on this first comparison, and the rest set
out through the remainder of the database, the first worker is still at it when
the rest are finished. This situation is atypical for our database (as for most
others), but under unlucky conditions it*can *occur. We discuss it further in
the next chapter.
### 6.9 Conclusions Problems with large quantities of obvious
parallelism are often conveniently treated using the agenda paradigm. Such
problems may be "embarrassingly parallel" (naturally endowed with very generous
quantities of easily-recognized parallelism), but they may also be vitally
important to significant computational efforts. For example,**Ray tracing**.
This is an important technique for producing images in computer graphics. The
general idea is to trace the path of a representative collection of light rays
from the viewer's eyes backwards to the objects viewed and back again to the
light source. Ray tracing can be parallelized in several ways—for example, by
building separate pieces of the image (separate scan lines for a raster-display,
for example) independently and simultaneously. Many other graphics and imaging
techniques can also be parallelized.**Monte Carlo simulations**allow you to
study the performance of a system (say a microchip exposed to potentially
damaging radiation) by summing the effects of a large number of
statistically-representative trials rather than by solving an equation that
characterizes the system as a whole. Separate trials can often be computed
simultaneously and independently.**Parameter sensitivity analyses**test a

```text
model's sensitivity to fluctuations in each of its input parameters; many trial
```
runs using different parameter sets can often be computed simultaneously. The
"model" might predict anything from the shape of a rocket plume to the behavior
of the bond market.**Linkage analysis**is used to determine the most likely site
on a chromosome for the genetic determinant of some trait, given information
about how the trait is inherited relative to the inheritance of known genetic
markers, and the layout of the chromosome. Many candidate sites can be
investigated simultaneously. This list could be extended nearly* ad infinitum *.
The only thing these particular examples have in common is the fact that Linda
has been used effectively to attack each of them. Numerical computations that
require repeated updates to a matrix are another set of good candidates for
agenda parallelism.**LU decomposition**, for example, is a major step in the
standard approach to solving sets of linear equations. It requires that a

```text
collection of matrix rows or columns be recomputed repeatedly; at each step in
```
the iteration, all of these column-update computations can proceed
simultaneously. The parallelization of direct methods for solving sparse systems
of equations is a far more complicated problem, but it too can be approached
using agenda parallelism [ACG89].**Linear programming**using the simplex
algorithm resembles LU decomposition in requiring a series of matrix updates
which can be computed in parallel at each iteration. This is a small sample

```text
based (once again) on problems that have been solved using Linda; there are many
```
other examples.
### 6.10 Exercises The first four exercises involve a database
search of the sort we've been discussing. To do them, you need a database and a
search criterion. One interesting and easily-obtainable kind of database is a
collection of text records. You can use files of email, for example, or of

```text
network news stories; or you can define a record as a single paragraph, and
```
simply concatenate an arbitrary bunch of text files. Use any strategy you want,
but build yourself a text database somehow or other. Next, you need a search
criterion. One standard possibility is a scan for keywords or key phrases. Your
search function accepts a predicate over words or phrases: for example, "find
all records that include the phrases* gross national product *,* trade deficit
*,* big seven,**Japanese prime minister *or* speaking of capybaras...*" Keyword
searches are usually implemented in terms of an elaborate index structure (which
allows you to focus your attention on likely records only, avoiding an

```text
exhaustive scan); but they can be (and sometimes are) implemented in terms of an
```
exhaustive search as well. Our assumption here is, of course, that you will

```text
perform an exhaustive search of the database; in this case, you'd return 1 when
```
the predicate is satisfied and 0 otherwise. A better possibility is a search
against an "interesting phrases (or words)" list. This might be an extensive
list, kept in a file. Your search criterion returns a score: a higher score
means a greater number of phrase matches. 1. Implement a search program against
a text database. The user supplies a value that determines how many matches will
be returned. If your criterion returns a score (not merely 1 or 0), records with

```text
the *n* best values are identified; otherwise *any**n* records that match the
```
predicate are identified. If the user supplies some distinguished value (say
-1), *all* matches are returned. (If your criterion returns a score, they should
be returned in sorted order.) Use a watermark scheme, but order the database by
length only if performance requires. If your records are short and your search
criterion very simple, performance will be bad. (Why?) Try to build a
better-performing version by using *clumping*: each task consists of searching a
bunch of records instead of a single one. 2. Change your program so that it
performs a cut-off search. There are two variants. If your criterion returns
only 1 or 0, stop the program as soon as you've found *n* matches. (Execution
should be shut down as expeditiously as possible. Note that it's not enough to
out a poison pill that workers may just *happen* to ingest. They should swallow
it at the earliest reasonable moment.) If your criterion returns a score, you'll

```text
have to examine every record in the database; but you should abort any search
```
that is guaranteed to produce a worse value than the *n* best known so far ("so
far" meaning "at the start of this comparison," not "right now"). 3. Some
vendors are now producing parallel machines with parallel I/O capabilities. Many
more such machines will be available in future. Parallel I/O capabilities entail

```text
spreading a database over many disks, each attached to a different processor;
```
hence we can perform many operations on the database simultaneously. Such
capabilities are an obvious match to exactly the kind of parallel database
search we've discussed in this chapter. (In fact, when Intel Scientific
introduced its parallel I/O system in early 1989, they used a version of the
Linda program described in this chapter to demo it.) One way to adapt to the
possibilities of parallel I/O is to build a database searcher with multiple

```text
masters (or input processes). Each master runs on a processor with direct access
to the database; all masters can read the database in parallel; they all feed
```
the common bag or stream of tasks. Implement this version of your search
program. (You won't necessarily have a parallel I/O system in fact, but you can
experiment with this kind of logical structure regardless.)*Massive
parallelism:* 4. In the foreseeable future, asynchronous parallel machines
encompassing tens of thousands of powerful processors (and more) will become
available. Clearly, we can't feed thousands of workers from a single task

```text
*stream*; we've discussed multiple-stream search programs. It won't be practical
```
to feed thousands of workers from the same task *bag* either. Although a bag
requires no synchronization on access, current implementation techniques will
break down in the face of tens of thousands of concurrent accesses to the same
distributed data structure. One interesting approach to the problem of massively
parallel search involves a *hierarchy* of task bags or pools, with higher-level
pools cascading into lower-level ones. The pools at the top of the hierarchy are
fed by the master (or more likely by a collection of input processes, as

```text
discussed above). A cluster of sub-masters hovers beneath the top pool; each one
```
is charged with replenishing its own second-level pool from the top pool as
needed. Each second-level pool has a cluster of sub-sub-masters (brown belts?)
clustered below *it*, and so on. The workers are located at the bottom of the

```text
hierarchy; each worker is assigned to some task pool. There are, of course, many
```
such pools, each fed by its own local "master." Such a program structure lends
itself to *multiple tuple space* versions of Linda, in which each pool can exist
within a separate tuple space. But it's easily implemented in "standard" Linda
as well. Implement it. For concreteness, assume that there is a single input

```text
process (a single top-level master, in other words, performing all file I/O),
```
and at least two levels of sub-pools beneath the top pool.

This hierarchical approach can be applied to stream-based task management as
well. A top-level ordered stream can feed second-level ordered streams, and so
on. Implement this version as well. ° 5. The oldest (probably) and
best-established (certainly) notation for concurrency is the musical staff. The
staff might be taken as a way of specifying *synchronous* parallelism only, but
that's not quite so. True, tempo and (ordinarily) meter are the same for every
line in the ensemble, and in a sense all threads are driven by a single clock.
But of course, the separate musical lines in an ensemble do *not* proceed in
lock-step. (*a*) Describe a notation for parallel programs that is based on
musical notation. (Clearly you need to abstract some basic principles, not do a
literal adaptation.) (*b*) Your notation provides a graceful way to express both
specialist and agenda parallelism. Explain. (*c*) Use your notation to describe
the relationship of the master to the workers in the database search program

```text
(with watermarking) described in this chapter.
```
