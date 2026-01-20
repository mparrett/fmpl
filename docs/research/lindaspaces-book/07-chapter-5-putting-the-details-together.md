# 5 Putting the Details Together
Finding all primes between 1 and *n* is a good example problem for two reasons. (1) It's not significant in itself, but there are significant

```text
problems that are similar; at the same time, primes-finding is simple enough to allow us to investigate the entire program in a series of cases.
(2) The problem can be approached naturally under several of our conceptual classes. This gives us an opportunity to consider what's natural
```
and what isn't, and how different sorts of solutions can be expressed.
### 5.1 Result parallelism and live data structures

```text
One way to approach the problem is by using result parallelism. We can define the result as an *n* element vector; *j*'s entry is 1 if *j* is prime,
```
otherwise 0. It's easy to see how we can define entry *j* in terms of previous entries: *j* is prime if and only if there is no previous prime less
than or equal to the square root of *j* that divides it.

```text
To write a C-Linda program using this approach, we need to build a vector in tuple space; each element of the vector will be defined by the
```
invocation of an is\_prime function. The loop

```text
for(i = 2; i < LIMIT; ++i) {
 eval("primes", i, is\_prime(i));
```
}
creates such a vector. As discussed in section 3.2, each tuple-element of the vector is labeled with its index. We can now read the *jth* element
of the vector by using

```text
rd("primes", j, ? ok)
```
The program is almost complete. The is\_prime(SomeIndex) function will involve reading each element of the distributed vector through

```text
the square root of *i* and, if the corresponding element is prime and divides *i*, returning zero; thus
limit = sqrt((double) SomeIndex) + 1;
for (i = 2; i < limit; ++i) {
 rd("primes", i, ? ok);
 if (ok && (SomeIndex%i == 0)) return 0;
```
}

```text
return 1;
(Note: in practice it might be cheaper for the *ith* process to compute all primes less than root of *i* itself, instead of reading them via rd. But
```
we're not interested in efficiency at this stage.)
The only remaining problem is producing output. Suppose the program is intended to count all primes from 1 through LIMIT. Easily done:
we simply read the distributed vector and count *i *if* i*'s entry is 1. The complete program is shown in figure 5.1.**Figure 5.1****Prime finder: Result parallelism**#**define**LIMIT 1000
real\_main()

```text
{**int**count = 0, i, is\_prime(), ok;**for**(i = 2; i <= LIMIT; ++i) eval("primes", i, is\_prime(i));**for**(i = 2; i <= LIMIT; ++i) {**rd**("primes", i, ? ok);**if**(ok) ++count);
```
 }

```text
 printf("%d.\n", count);
```
}

```text
is\_prime(me)**int**me;
```

```text
{**int**i, limit, ok;**double**sqrt(); limit = sqrt((**double**) me) +
1;**for**(i = 2; i < limit; ++i) {**rd**("primes", i, ? ok);**if**(ok && (me%i
== 0))**return**0; }**return**1; }
```
### 5.2 Using abstraction to get an efficient
version This program is concise and elegant, and it was easy to develop. It
derives parallelism from the fact that, once we know whether *k* is prime, we
can determine the primality of all numbers from *k*+1 through *k*. But it's
potentially highly inefficient: it creates large numbers of processes and
requires relatively little work of each. We can use abstraction to produce a
more efficient, agenda-parallel version. We reason as follows. 1. Instead of
building a live vector in tuple space, we'll use a passive vector, and create
worker processes. Each worker will choose some block of vector elements and fill
in the entire block. "Determine all primes from 2001 through 4000" is a typical
task. Tasks should be assigned in order: the lowest block is assigned first,
then the next-lowest block and so forth. If we've filled in the bottom block and
the highest prime it contains is *k*, we can compute in parallel all blocks up
to the block containing *k*. How to assign tasks in order? We could build a
distributed queue of task assignments, but there's an easier way. All tasks are

```text
identical in kind; they differ only in starting point. So we can use a single
```
tuple as a next-task pointer, as we discuss in the matrix multiplication example
in section 3.2. Idle workers withdraw the next-task tuple, increment it and then
reinsert it, so the next idle worker will be assigned the next block of integers
to examine. In outline, each worker will execute while(1) { in("next task", ?

```text
start); out("next task", start + GRAIN); <find all primes from start to start +
```
GRAIN> } GRAIN is the size of each block. The value of GRAIN, which is a
programmer-defined constant over each run, determines the granularity or task
size of the computation. GRAIN, in other words, is the granularity knob for this
application. The actual code is more involved than this: workers check for the
termination condition, and leave a marker value in the next-task tuple when they
find it. (See the code in figures 5.2 and 5.3 for details.)**Figure 5.2****Prime
finder: Agenda parallelism (master)**real\_main(argc,

```text
argv)**int**argc;**char**\* argv[];**{ int**eot, first\_num, i, length,
new\_primes[GRAIN], np2;**int**num, num\_prices, num\_workers, primes[MAX],
p2[MAX];**int**worker(); num\_workers = atoi(argv[1]);**for**(i = 0; i <
num\_workers; ++i)**eval**("worker", worker()); num\_primes =
init\_primes(primes, p2); first\_num = primes[num\_primes-1] + 2;**out**("next
task", first\_num); eot = 0;*/\* Becomes 1 at "end of table"—i.e.., table
complete. \*/***for**(num = first\_num; num < LIMIT; num += GRAIN)
{**in**("result", num, ? new\_primes:length);**for**(i = 0; i < length; ++i,
++num\_primes) { primes[num\_primes] = new\_primes[i];**if**(!eot)**{**np2 =
new\_primes[i]\* new\_primes)[i];**if**(np2 > LIMIT)**{**eot = 1; np2 = -1;**}
out**("primes", num\_primes, new\_primes[i], np2);**} } }**/\* *"? int" means
"match any int; throw out the value"* \*/**for**(i = 0; i < num\_workers;
++i)**in**("worker", ?**int**); printf("count: %d\n", num\_primes); }**Figure
```
5.3****Prime finder: Agenda parallelism (worker)**worker() {**int**count, eot,

```text
i, limit, num, num\_primes, ok, start;**int**my\_primes[GRAIN], primes[MAX],
p2[MAX]; num\_primes = init\_primes(primes, p2); eot = 0;**while**(1)
{**in**("next task", ? num);**if**(num == -1) {**out**("next task",
-1);**return**;**}**limit = num + GRAIN;**out**("next task", (limit > LIMIT) ?
-1 : limit);**if**(limit > LIMIT) limit = LIMIT: start = num;**for**(count = 0;
num < limit; num += 2) {**while**(!eot && num > p2[num\_primes-1])
```
{**rd**("primes", num\_primes, ?primes[num\_primes],

```text
?p2[num\_primes]);**if**(p2[num\_primes] < 0) eot = 1;**else**++num\_primes;**}
for**(i = 1, ok = 1; i < num\_primes; ++i) {**if**(!num%primes[i])) { ok =
0;**break**;**} if**(num < p2[i]) break;**} if**(ok) { my\_primes[count] = num;
++count;**} }**/\* *Send the control process any primes found.*
\*/**out**("result", start, my\_primes:count); } } 2. We've accomplished
```
"abstraction" and we could stop here. But since the goal is to produce an
efficient program, there's another obvious optimization. Instead of storing a
distributed bit-vector with one entry for each number within the range to be
searched, we could store a distributed *table* in which all primes are recorded.
The *ith* entry of the table records the *ith* prime number. The table has many
fewer entries than the bit vector, and is therefore cheaper both in space and in
access time. (To read all primes up to the square root of *j* will require a
number of accesses proportional not to√*j* but to the number of primes through
√*j*.) A worker examining some arbitrary block of integers doesn't know* a
priori *how many primes have been found so far, and therefore can't construct
table entries for new primes without additional information. We could keep a
primes count in tuple space, but it's also reasonable to allow a master process
to construct the table. We will therefore have workers send their

```text
newly-discovered batches of primes to the master process; the master process
```
builds the table. Workers attach batches of primes to the end of an in-stream,
which in turn is scanned by the master. Instead of numbering the stream using a
sequence of integers, they can number stream elements using the starting integer
of the interval they've just examined. Thus the stream takes the form ("result",

```text
start, FirstBatch); ("result", start+GRAIN, SecondBatch);
```

```text
("result", start+(2\* GRAIN,) ThirdBatch); ... The master scans the stream by
executing for (num = first\_num; num < LIMIT; num += GRAIN) { in("result", num,
? new\_primes); <record the new batch for eventual output>; <construct the
distributed primes table>; } This loop dismantles the stream in order, ining the
```
first element and assigning it to the variable new\_primes, then the second
element and so on. The master's job is now to record the results and to build

```text
the distributed primes table. The workers send prime numbers in batches; the
```
master disassembles the batches and inserts each prime number into the
distributed table. The table itself is a standard distributed array of the kind
discussed previously. Each entry takes the following form ("primes", i, <ith
prime>, <ith prime squared>) We store the square of the *ith* prime along with
the prime itself so that workers can simply read, rather than having to compute,
each entry's square as they scan upwards through the table. For details, see
figure 5.3. 3. Again, we could have stopped at this point, but a final
optimization suggests itself. Workers repeatedly grab task assignments, then set
off to find all primes within their assigned interval. To test for the primality

```text
of *k*, they divide *k* by all primes through the square root of *k*; to find
```
these primes, they refer to the distributed *primes* table. But they could save
repeated references to the distributed global table by building local copies.
Global references (references to objects in tuple space) are more expensive than
local references. Whenever a worker reads the global *primes* table, it will
accordingly copy the data it finds into a local version of the table. It now
refers to the global table only when its local copy needs extending. This is an
optimization similar in character to the *specialization* we described in
section 2: it saves global references by creating multiple local structures. It
isn't "full specialization", though, because it doesn't eliminate the global
data structure, merely economizes global references. Workers store their local
tables in two arrays of longs called primes and p2 (the latter holds the square
of each prime). The notation object: count in a Linda operation means "the first

```text
count elements of the aggregate named object"; in an in or a rd statement, ?
```
object: count means that the size of the aggregate assigned to object should be
returned in count.
### 5.3 Comments on the agenda version This version of the
program is substantially longer and more complicated than the original
result-parallel version. On the other hand, it performs well in several
widely-different environments. On one processor of the shared-memory Sequent
Symmetry, a sequential C program requires about 459 seconds to find all primes
in the range of 1 to three million. Running with twelve workers and the master
on thirteen Symmetry processors, the C-Linda program in figures 5.2 and 5.3and
does the same job in about 43 seconds, for a speedup of about ten and a half
relative to the sequential version, giving an efficiency of about 82%. One
processor of an Intel Intel iPSC/2 requires about 421 seconds to run the

```text
sequential C program; one master and sixty-three workers running on all
```
sixty-four nodes of our machine require just under 8 seconds, for a speedup of
about fifty two and a half and an efficiency of, again, roughly 82%. (The iPSC/2
is a so-called *hypercube*—a collection of processors each equipped with local
memory, arranged in such a way that each one "sits" at one corner of an *n*
dimensional binary cube. Communication links run over the edges of the cube to
each processor's *n*-1 neighbors.) If we take the same program and increase the
interval to be searched in a task step by a factor of 10 (this requires a change
to one line of code: we define GRAIN to be 20,000), the same code becomes a very
coarse-grained program that can perform well on a local area network. Running on
eight Ethernet-connected IBM RT's under Unix, we get roughly a 5.6-times speedup
over sequential running time, for an efficiency of about 70%. Somewhat lower
efficiencies on coarser-grained problems are still very satisfactory on local
area nets. Communication is far more expensive on a local area net than in a
parallel computer, and for this reason networks are problematic hosts for
parallel programs. They are promising nonetheless because, under some
circumstances, they can give us something for nothing: many computing sites have
compute-intensive problems, lack parallel computers but have networks of
occasionally underused or (on some shifts) idle workstations. Converting wasted
workstation cycles into better performance on parallel programs is an attractive
possibility. In comparing the agenda to the result-parallel version, it's
important to keep in mind that the more complicated and efficient program was
produced by applying a series of simple transformations to the elegant original.
So long as a programmer understands the basic facts in this domain—how to build
live and passive distributed data structures, which operations are relatively
expensive and which are cheap—the transformation process is conceptually
uncomplicated, and it can stop at any point. In other words, programmers with
the urge to polish and optimize (*i.e*., virtually all expert programmers) have
the same kind of opportunities in parallel as in conventional programming. Note
that for this problem, agenda parallelism is probably less natural than result
parallelism. The point here is subtle but is nonetheless worth making. The most
natural agenda-parallel program for primes finding would probably have been
conceived as follows: apply *T* in parallel to all integers from 1 to *limit*,
where *T* is simply "determine whether *n* is prime". If we understand these
applications of *T* as completely independent, we have a program that will work,
and is highly parallel. It's not an attractive solution, though, because it is
blatantly wasteful: in determining whether *j* is prime, we can obviously make
use of the fact that we know all previous primes through the square root of *j.*
The master-workers program we developed *on the basis of the result-parallel
version* is more economical in approach, and we regard this version as a "made"
rather than a "born" distributed data structure program.
### 5.4 Specialist
parallelism Primes finding had a natural result parallel solution, and we
derived an agenda parallel solution. There's a natural specialist parallel
solution as well. The Sieve of Eratosthenes is a simple prime-finding algorithm
in which we imagine passing a stream of integers through a series of sieves: a
2-sieve removes multiples of 2, a 3-sieve likewise, then a 5-sieve and so forth.
An integer that has emerged successfully from the last sieve in the series is a
new prime. It can be ensconced in its own sieve at the end of the line. We can
design a specialist parallel program based on this algorithm. We imagine the
program as a pipeline that lengthens as it executes. Each pipe segment
implements one sieve (specializes, that is, in a single prime). The first pipe
segment inputs a stream of integers and passes the residue (a stream of integers
not divisible by 2) onto the next segment, which checks for multiples of 3 and
so on. When the segment at the end of the pipeline finds a new prime, it extends
the sieve by attaching a new segment to the end of the program. One way to write
this program is to start with a two-segment pipe. The first pipe segment

```text
generates a stream of integers; the last segment removes multiples of the
```
last-known prime. When the last segment (the "sink") discovers a new greatest
prime, it inserts a new pipe segment directly before itself in line. The newly
inserted segment is given responsibility for sieving what had formerly been the
greatest prime. The sink takes over responsibility for sieving the *new*
greatest prime. Whenever a new prime is discovered, the process repeats. First,
how will integers be communicated between pipe segments? We can use a
single-source, single-sink in-stream. Stream elements look like ("seg",
<destination>, <stream index>, <integer>) Here, destination means "next pipe

```text
segment"; we can identify a pipe segment by the prime it's responsible for. Thus
```
a pipe segment that removes multiples of 3 expects a stream of the form ("seg",
3, <stream index>, <integer>) How will we create new pipe segments? Clearly, the

```text
"sink" will use eval; when it creates a new segment, the sink detaches its own
```
input stream and plugs this stream into the newly-created segment. Output from
the newly-created segment becomes the sink's new input stream. The details are
shown in figure 5.4.**Figure 5.4****Prime finder: Specialist

```text
parallelism**real\_main() {**eval**("source", source());**eval**("sink",
sink()); } source() {**int**i, out\_index=0;**for**(i = 5; i < LIMIT; i +=
2)**out**("seg", 3, out\_index++, i);**out**("seg", 3, out\_index, 0); } sink()
{**int**in\_index=0, num, pipe\_seg(), prime=3, prime\_count=2;**while**(1)
{**in**("seg", prime, in\_index++, ? num);**if**(!num)**break**;**if**(num %
prime) { ++prime\_count;**if**(num\* num < LIMIT) {**eval**("pipe seg",
pipe\_seg(prime, num, in\_index)); prime = num; in\_index = 0**} }
}**printf("count: %d.\n", prime\_count); } pipe\_seg(prime, next, in\_index)
{**int**num, out\_index=0;**while**(1) {**in**("seg", prime, in\_index++, ?
num);**if**(!num)**break**;**if**(num % prime)**out**("seg", next, out\_index++,
num);**} out**("seg", next, out\_index, num); } The code in Figure 5.4 produces
```
as output merely a count of the primes discovered. It could easily have
developed a table of primes, and printed the table. There's a more interesting

```text
possibility as well. Each segment of the pipe is created using eval; hence each
```
segment turns into a passive tuple upon termination. Upon termination (which is
signaled by sending a 0 through the pipe), we could have had each segment yield
its prime. In other words, we could have had the program collapse upon
termination into a data structure of the form ("source", 1, 2) ("pipe seg", 2,
3) ("pipe seg", 3, 5) ("pipe seg", 4, 7) ... ("sink", MaxIndex, MaxPrime) We
could then have walked over and printed out this table. This solution allows
less parallelism than the previous one. To see why, consider the result parallel
algorithm: it allowed simultaneous checking of all primes between *k*+1 and *k*
for each new prime *k*. Suppose there are *p* primes in this interval for some
*k*. The previous algorithm allowed us to discover all *p* simultaneously, but
in this version they are discovered one at a time, the first prime after *k*
causing the pipe to be extended by one stage, then the next prime, and so on.
Because of the pipeline, "one at a time" means a rapid succession of

```text
discoveries; but the discoveries still occur sequentially. The
```
specialist-parallel solution isn't quite as impractical as the result-parallel
version, but it is impressively impractical nonetheless. Since both of these
codes create fairly large numbers of processes, we tested them using a
thread-based C-Linda implementation on the Encore Multimax. (Note: Linda's eval
makes use of the underlying operating system in creating new processes.
C-Linda's semantics don't require that any particular kind of process be

```text
created; either "heavy-weight" or "light-weight" processes will do. Light-weight
```
processes, often called *threads*, are faster to create and more efficiently
managed than standard heavy-weight processes, and Linda does *not* require the
added services that heavy-weight processes provide. Hence threads are, in

```text
general, the implementation vehicle of choice for eval; but they aren't
```
universally available. They are particularly hard to come by in
current-generation distributed-memory environments. The running versions of both
programs differ trivially from the ones shown in the figures.) Both programs
needed about the same amount of time (1.4sec) to search the range from 1 to 1000
for primes on a "minimal" number of processors. (One represents the minimal
number for the specialist code, but the result code requires a minimum of six
processors. On fewer than six processors, our thread system is unable to handle
the blizzard of new processes.) The specialist code showed good relative speedup
through 4 processors (.35sec). The result code didn't speed up at all. So it
looks like the specialist code did well—right? Wrong. The sequential code needed

```text
only .03sec to examine the first thousand integers; in other words, the
```
specialist code on four processors ran over ten times *slower* than the
sequential code. This result is an instructive demonstration of the phenomenon
we discussed in the previous chapter—the fact that a program can show good
*relative* speedup—may run faster on many processors than on one—without ever
amortizing the "overhead of parallelization" and achieving *absolute* speedup.
We expect technology to move in a direction that makes finer-grained programming
styles more efficient. This is a welcome direction for several reasons.
Fine-grained solutions are often simpler and more elegant than coarser-grained

```text
approaches, as we've discussed; larger parallel machines, with thousands of
```
nodes and more, will in some cases require finer-grained programs if they are to
keep all their processors busy. But the coarser-grained techniques are virtually
guaranteed to remain significant as well. For one thing, they will be important
when parallel applications run on loosely-coupled nodes over local- or
wide-area-networks. (Whiteside and Leichter have shown that a Linda system
running on fourteen VAXes over a local area network can, in one significant case
at least, beat a Cray [ WL88]. This Cray-beating Linda application is in
production use at Sandia National Laboratory in Livermore.) Coarser-grained
techniques will continue to be important on "conventional" parallel computers as
well, so long as programmers are required or inclined to find
maximally-efficient versions of their programs. For this problem, our
specialist-parallel approach is clearly impractical. Those are the breaks. But
readers should keep in mind that exactly the same program structure *could* be
practical if each process had more computing to do. In some related problem
areas this would be the case. Furthermore the dynamic, fine-grained character of
this program makes it an interesting but not altogether typical example of the
message-passing *genre*. A static, coarse-grained message-passing program (of
the sort we described in the context of the *n*-body problem, for example) would
be programmed using many of the same techniques, but would be far more
efficient.
### 5.5 Conclusions In the primes example, one approach is the
obvious practical choice. But it's certainly *not* true that, having canvassed

```text
the field, we've picked the winner and identified the losers; that's not the
```
point at all. The performance figures quoted above depend on the Linda system
and the parallel machine we used. Most important, they depend on the character
of the primes problem. Agenda-parallel algorithms programmed under the

```text
master-worker model are often but not always the best stopping point; all three
```
methods can be important in developing a good program. Discovering a workable
solution may require some work and diligence on the programmer's part, but no
magic and nothing different in kind from the sketch-and-refine effort that's
typical of all serious programming. All that's required is that the programmer
understand the basic methods at his disposal, and have a programming language
that allows him to say what he wants.
### 5.6 Exercises 1. The primes-finder
will run faster if it uses "striking out" instead of division—instead of
dividing by *k*, it steps through an array and marks as non-prime every *kth*
element. Re-implement the agenda-parallel program using this approach. 2. In the
agenda-parallel primes-finder, what limit does the value of GRAIN impose on
achievable speedup? 3. We transformed the result parallel primes-finder into an
efficient program by abstraction to agenda parallelism. One aspect of the result

```text
version's inefficiency was its too-fine granularity; but it's possible to build
```
a coarser-grained* result* parallel version of this code too. Implement a
variable-granularity result-parallel version. 4. In the specialist-parallel
primes-finder, the pipe segment responsible for sieving out multiples of 3 is
heavily overloaded. We expect that large backlogs of candidates will await
attention from this process. Design a new version of the specialist-parallel
program in which pipe segments can be replicated—in particular, multiple copies
of the 3-sieve run simultaneously at the head of the pipeline. ° 5. At the start
of the industrial revolution, the British cotton industry faced the same kind of
bottleneck as the specialist-parallel primes finder: "It took three or four
spinners to supply one weaver with material by the traditional method, and where
the fly-shuttle speeded up the weavers' operations the shortage of yarn became
acute [Dea69, p. 86]." The answer was (in essence) exactly what we suggested in

```text
question 4; the solution hinged on one of the most famous gadgets in engineering
```
history. How was the problem solved? What was the gadget?
