# 7 Matrices: Starting with Result
In the previous chapter, we referred to a string-comparison algorithm for DNA sequences. The details of the algorithm weren't important, but
we need to consider them now. The algorithm happens to be fairly typical of a broad class of important numerical problems: it requires the
computation of a matrix in such a way that each matrix element depends either on the input data or on other, already-computed elements of

```text
the matrix. (Not all DNA comparison algorithms work this way; but many algorithms in many domains do.) We'll use this algorithm as the
```
basis for our discussion of problems with natural result-parallel solutions.
In practical terms, we noted that the DNA database problem had two different kinds of solution: we could perform many sequential
comparisons simultaneously, or we could parallelize each individual comparison. In this chapter we investigate the second approach. We
then discuss, briefly, a third approach that incorporates elements from both the others. As in the previous chapter, the real topic (of course)

```text
isn't DNA sequence comparison; it's one significant type of parallel programming problem, its immediate *natural* solution, and the orderly
```
transition from the *natural* solution to an *efficient* one—in other words, from the right starting place to a good stopping point.
### 7.1 Big issues *Main themes:*
*Load balance*is, again, crucial to good performance. The need for good load balance motivates our transformation from a
result- to an agenda-parallel strategy.*Granularity control*is the other crucial issue. Here, we use the size of a matrix sub-block (see below) as a granularity knob .*Special considerations:*
*Matrix sub-blocking*is a powerful technique for building efficient matrix computations.*Logical inter-dependencies*among sub-computations need to be understood and taken into account in building efficient
programs. Here, we focus on an application with "wavefront" type dependencies.
### 7.2 Problem: the wavefront computation of a matrix
Given a function *h *(* x,y*), we need to compute a matrix *H* such that the value in the *ith *row,* jth*column of *H*—that is, the value of *H *[* i,j*] (in
standard programming language notation) or *Hi,j*(in mathematical notation)—is the function *h* applied to *i *and* j*, in other words *h *(* i,j*).
The value of *h *(* i,j*) depends on three other values: the values of *h *(* i-*1*, j*), of *h *(* i, j-* 1) and of *h *(* i-*1*, j-* 1). To start off with, we're given the value
of *h*(0 *, j*) for all values of *j*, and of *h *(* i,* 0) for all values of *i*. These two sets of values are simply the *two strings that we want to compare*. In
other words, we can understand the computation as follows: draw a two-dimensional matrix. Write one of the two comparands across the top

```text
and write the other down the left side. Now fill in the matrix; to fill in each element, you need to check the element on top, the element to
```
the left, and the element on top and to the left.
### 7.3 The result-parallel approach
We noted in chapter two that *result parallelism is a good starting point for any problem whose goal is to produce a series of values with
predictable organization and inter-dependencies*. What we've described is exactly such a problem.
Our computation fills in the entries of an *m*× *n* matrix (where *m* is, arbitrarily, the length of the shorter sequence and *n* is the length of the
longer). Using result parallelism, we create a live data structure containing *m*× *n* processes, one for each element of the matrix. To do so we

```text
will execute a series of eval statements; to create the *i, jth*element of the live data structure, we execute a statement like
eval("H", i, j, h(i,j));
```
Here, h() is the function referred to above, charged with computing the *i, jth* entry on the basis of entries in the previous counter-diagonal of
the matrix. Each process waits for its input values by blocking on rd statements. In general, the process that's computing entry (*i,j*) of the
matrix will start out by executing

```text
rd("H", i-1, j, ?val1);
rd("H", i, j-1, ?val2);
rd("H", i-1, j-1, ?val3);
```
It now performs an appropriate computation using values val1, val2 and val3 and completes, turning into the data tuple

```text
("H", i, j, *the i,jth value of the matrix*).
```

It's typical of the result paradigm that a simple outline like this is very
close to the actual code. We need only add one detail. The matrix described
above is the core of the computation, but for this problem the answer really
isn't the matrix—it's the maximum value in the matrix. So we need to adjust
slightly the concept of a matrix element and how to compute one. An element will
now have a value field and a max field. After we have computed an element's
value, we compute its max field. To do so, we simply take the maximum of (1) the
max fields of each input element and (2) the newly-computed value of *this*
element. It should be clear that the max field of the lower-right entry of the
matrix will hold the maximum of *all* values in the matrix. The result parallel
code appears in figure 7.1 and 7.2. An entry's value actually consists of a

```text
triple (d, p, q), with d measuring similarity proper; these plus a max slot
```
constitute the fields of each matrix entry. As discussed above, the answer

```text
(*i.e.*, the maximum d value in the matrix) will be propagated to the max field
```
of the lower-right entry. It is this entry that is ined at the end of
real\_main.**Figure 7.1****A result-parallel wavefront matrix computation: main
routine***/\* Adopted from O. Gotoh "An Improved Algorithm for Matching
Biological Sequences", J. Mol. Biol. (162:pp705-708). This code is complete
except for the constant MAX and the match\_weight table \*/***typedef

```text
struct**entry {**int**d, ma, p, q; } ENTRY\_TYPE; ENTRY\_TYPE zero\_entry = { 0,
0, 0, 0,};**#define**ALPHA 4 */\* indel penalty. \*/***#define**BETA 1 */\*
extension penalty. \*/***char**side\_seq[MAX], top\_seq[MAX]; real\_main(argc,
argv)**int**argc;**char**\*\* argv; { ENTRY\_TYPE compare(),
max\_entry;**int**i, j;**int**side\_len, top\_len; side\_len =
get\_target(argv[1], side\_seq); top\_len = get\_target(argv[2],
top\_seq);**for**(i = 0; i < side\_len; ++i)**for**(j = 0; j < top\_len;
++j)**eval**("H", i, j, compare(i,j,side\_seq[i], top\_seq[j]));**in**("H",
side\_len-1, top\_len-1, ? max\_entry); printf("max: %d", max\_entry.max);
```
}**Figure 7.2**The compare routine ENTRY\_TYPE compare(i, j, b\_i, b\_j) {

```text
ENTRY-TYPE d, p, q; ENTRY-TYPE me;**int**t; d = p = q =
zero\_entry;**if**(i)**rd**("H", i-1, j, ? q);**if**(j)**rd**("H", i, j-1, ?
p);**if**(i && j)**rd**("H", i-1, j-1, ? d);
```

```text
me.d = d.d + match\_weights[b\_i&0xF][b\_j&0xF];**if**(me.d < 0) me.d = 0; me.p
= p.d - ALPHA; t = p.p - BETA;**if**(me.p < t) me.p = t; me.q = q.d - ALPHA; t =
q.q - BETA;**if**(me.q < t) me.q = t;**if**(me.p > me.d) me.d = me.p;**if**(me.q
> me.d) me.d = me.q;*/\* Remember overall max. \*/* me.max = me.d;**if**(d.max >
me.max) me.max = d.max;**if**(p.max > me.max) me.max = p.max;**if**(q.max >
me.max) me.max = q.max;**return**me; }
```
### 7.4 A result => agenda transformation
Our result parallel solution is simple and elegant, but its granularity is too
fine to allow efficient execution on most current-generation asynchronous
multiprocessors. For starters, we are using three input values to feed one very
small computation. There is another cause of inefficiency as well, stemming from
the data dependencies. Consider the comparison of two 4-base sequences—the
computation of a 4 × 4 matrix. At the starting point, only the upper-left matrix
element can be computed. Once this is done, the elements to its right and
beneath it (that is, the elements along the next counter-diagonal) can be
computed. In general, the enabled computations sweep down in a wavefront from
the upper-left element to the lower-right. Suppose we look at the computation in
terms of a series of discrete time steps. During every step, everything that can
be computed *is* computed. We see that one element can be computed during the
first time step, two during the second, three during the third, and so on up
through the length of the longest diagonal. Thus, it's not until time step *K*
that we will have enough enabled tasks to keep *K* processors busy. The same
phenomenon occurs as the computation winds down. Opportunities for parallel
execution diminish until, at the last time step, only one element remains to be
computed. These start-up and shut-down phases limit the amount of speedup we can
achieve, and the efficiency with which we can use our processors. For example,
if we were to compare two length-*n* sequences using *n* processors, the best
speedup we could manage would be approximately *n*/2 (we are thus in effect
throwing away half our processors) – and this* ignores any other source of
overhead *. In practice, additional overhead is likely to reduce our achieved
efficiency to considerably less than 50%. We'll have to address both the
communication-to-computation ratio, and the start-up and shut-down costs. A good
way to address both will be in terms of a transformation to an agenda-parallel
approach. We can transform the result parallel solution into an agenda-parallel
approach using essentially the same strategy we used in the primes-finder
example. We group the elements to be computed into bunches, and we create a
collection of worker processes. A single task is to compute all elements in a
bunch. A bunch of elements will be a sub-block of the matrix, as we'll describe

```text
below. This is the basic strategy, and it's fairly simple; but we need to deal
```
with a number of details before we can make the strategy work effectively.
####
7.4.1 Granularity We'll start with the issue of communication. In general, we
want to do as much work as possible per communication event. Our strategy will
be to enlarge each separate computation—to let the computation of many matrix
elements, not just one, be the basic step. We can borrow a well-known technique
of computational linear algebra in order to accomplish this. Many computations
on matrices have the useful property that the computation can be easily
rewritten to replace matrix elements with matrix sub-blocks, where a sub-block
is merely a sub-matrix of elements contained within the original matrix. The
wavefront computation we described above can easily be rephrased in terms of
sub-blocks. We need to know how to handle each sub-block individually, and then
how to handle *all* sub-blocks in the aggregate. Individually, each sub-block is
treated in exactly the same way as the original matrix as a whole. In the
aggregate, we can compute sub-blocks wavefront-by-wavefront, in exactly the same
way we compute individual elements. A given sub-block is computed on the basis
of known values for the three sub-blocks immediately above, to the left, and
above and to the left. When we say "known values," though, we don't need to know
these *entire* sub-blocks, merely the upper block's lower edge, the leftward
block's right edge, and the bottom-right corner of the upper-left block. To
eliminate the need for this one datum from the upper-left block, we can define
blocks in such a way that that they overlap their neighbors by one row or
column. Hence we can compute a new sub-block on the basis of one row (from
above) and one column (from the left).

Assuming for the moment that we have square *n*×*n* sub-blocks, we have two
communication events each involving *n*+1 pieces of data supporting the *n* 2
computations that are required to fill-in a sub-block. We can control
granularity by controlling the size of *n*. The ratio of communication to
computation falls as we increase the size of the sub-blocks. Of course,
increasing sub-block size has an obvious disadvantage as well. Consider the
extreme case. If the sub-block is the entire matrix, we succeed in eliminating
communication costs altogether—and with them, all available parallelism! As we
increase the block size, we improve prospects for "local" efficiency (each task
has less relative overhead) but we may reduce "global" efficiency by producing
too few tasks to keep all processors busy. Nonetheless, we now have a
granularity knob that we can adjust to give a reasonable tradeoff between the
two extremes of many small tasks with relatively large communication overheads
and one large task with no communication overhead.
#### 7.4.2 An agenda-style
solution We *could* build a result-parallel program based on sub-block rather
than single-element computations (see the exercises). But there's an efficiency
problem in our computation that is conveniently handled by switching to a
master-worker model. Our computation has start-up and shut-down phases. Having
divided our matrix into sub-blocks, we could create one process for each
sub-block (that is, we could build a live data structure). But many of these
processes would be inactive for much of the time. In general, the parallelism we
can achieve is limited by the length of the longest diagonal (* i.e.*, the
length of the shorter of the two sequences being compared): at no time can there
be more enabled computations than elements in the longest diagonal. But if this
length is *k*, and we have *k* processors, all *k* processors will be active
only during the middle of the computation. We can get better efficiencies (at
the cost of some degree of *absolute* speedup, of course) by executing with
fewer than *k* processors. If efficiency is important, then, we will run with
far fewer processors than there are sub-blocks in the matrix. We face, once
again, a load-balancing problem. We could write a result-parallel program under
the assumption that the underlying system will balance enabled processes
correctly among available processors. We might begin with a random assignment of
processes to processors, and simply move processes around at runtime from
heavily- to lightly-burdened processors. But this is difficult to accomplish on
parallel processors that lack shared memory—for example on hypercubes,
Transputer arrays or local area networks. In such distributed-memory
environments, moving a process requires copying the entire process image from
one address space to another. This *can* be done—but at the expense of some
complex operating system support, and the obvious runtime costs of moving
process images around. A more conservative approach is to* abstract,*and thereby
make the switch to an agenda-parallel, master-worker scheme. A task will be the
computation of a single sub-block (with a qualification to be discussed below).
The naturally load-balancing properties of this software structure work in favor
of an efficient implementation.* Massive parallelism:*Some of the assumptions
behind these arguments will almost certainly be falsified on future massively
parallel asynchronous machines. These machines should provide communication
support that's fast enough to relieve us from many of our current concerns about
fine-grained programs. In such a setting, program development could have stopped
several paragraphs ago, with the pure result solution. Result parallelism seems
like an excellent way to express matrix computations for massively parallel
computers. Given a sufficiently large parallel machine, throwing away processors
in the interests of strong absolute performance might be an excellent strategy.
If*D *(the length of the longest diagonal) represents the maximum achievable
degree of parallelism, you may insist on getting *D*-way parallelism whatever
efficiencies you achieve—despite the fact (in other words) that many processors
are idle during start-up and shut-down. Even if you have processors to burn,
though, you won't necessarily want to waste them needlessly. You may be
unwilling to use *more* than *D* processors. To program such a machine, you
might build a length-*D* vector of processes. Each process in the vector
computes every element in its row. It leaves a stream of tuples behind,
representing the first element in its row, the second element and so on. The
second process in the vector can begin as soon as the first process's stream is
one element long, and so on. The resulting program is more complex than the
result-parallel version given above, but it's simple and elegant nonetheless.

```text
(See the exercises.) Another possible approach is to apply existing automatic
```
transformation techniques to our pure result program, producing in effect the
same sort of *D*-process program we've just discussed. The transformation
techniques developed by Chen in the context of her Crystal compiler [Che86]
might be applicable here. (These techniques are currently targeted at
functional-language programs, and functional languages are unacceptable for our

```text
purposes; they may support a neat solution to the problem discussed here, but
```
outside the bounds of this chapter they fall flat. Like parallel do-loop

```text
languages, they are too inflexible to be attractive tools; we want to solve many
```
sorts of problems, not just one. *But*—Chen's techniques may well be applicable
outside of the constricting functional language framework.)
#### 7.4.3 What
should the sub-blocks look like? So, we've designed a master-worker program
where a task is the computation of a single matrix sub-block. We must now
consider what a sub-block should look like. Notice that, if we are comparing two
sequences of *different* lengths, potential efficiency improves. The
shorter-sized sequence determines the maximum degree of parallelism. But
whereas, for a square result matrix, full parallelism is achieved during a
single time step only (namely the time-step during which we compute the elements
along the longest counter-diagonal), a rectangular matrix sustains full
parallelism over many time steps. The difference between the lengths of the
longer and the shorter sequence is the number of additional time steps during
which maximum parallelism is sustained. Suppose our two sequences are length
*m*(the shorter one) and *n*, and suppose we are executing the program with *m*
workers. The number of time steps for parallel execution, *tpar*, is given by 2
*m*+ (*n*- (*m*- 1)), so for* m, n *>> 1 *tpar*≈*m*+*n*. The sequential time,
*tseq*, is given by *m*\**n*, and thus speedup, *S*, is *tseq */* tpar*= *mn
*/*m*+* n*. Note that if *n >> m*, *S ≈ m.* In other words, if *n >> m*, we get
essentially perfect speedup (*m* workers yielding a speedup of *m*), and perfect
efficiency. This result is intuitively simple and reasonable. Picture a short

```text
(in height), wide (in width) matrix—a matrix for which *n *>>* m*. It has *many*
```
longest counter-diagonals (where a square matrix has only one). Each

```text
counter-diagonal is *m* long; whenever we are computing a counter-diagonal, we
```
can use all *m* workers at once, and thus get *m*-fold speedup. If we define the
*aspect ratio*, *α*, of a matrix to be the ratio of its width to its height,
then *α = n/m *and* S* = (*α*/*α*+ 1) *m*. So, if *α* happens to be 10 (one
sequence is ten times longer than the other), efficiency can be as high as 90%.
Combining this observation with the concept of blocking yields a mechanism for
controlling start-up and shut-down costs. We've been assuming that sub-blocks
are square (although we do have to allow for odd shaped blocks along the right
edge and the bottom). But they don't have to be. We have an additional,
important degree of freedom in designing this program: the aspect ratio of a
sub-block (as opposed to that of the whole matrix). We can use non-square
sub-blocks to produce a blocked matrix just high enough to make use of all
available workers, but long enough to reduce start-up and shut-down
inefficiencies to an acceptable level. In other words, we can set the aspect
ratio of a matrix (in its blocked form) by adjusting the aspect ratio of its
sub-blocks. First, we choose an ideal height for our sub-blocks. If our blocked
matrix is exactly *W* high (where *W* is the number of worker processes), then
each worker can compute a single row of the blocked matrix. To achieve a
*W*-high blocked matrix, we set the height of each sub-block to *m*(the length
of the shorter sequence, hence the height of the original matrix) divided by
*W*. We now need to determine a good width for each sub-block. Let's say that we
aim for an efficiency of 90%. For a maximum-achievable efficiency of 90%, *α*(as
we saw above) should be ≈ 10. It follows that each row of the blocked matrix
should have 10 *W* elements. Hence, the width of of a sub-block should be the
length of the longer sequence divided by 10 *W*. Why not shoot for 95%
efficiency? *Pop quiz*: What *α* corresponds to 95% efficiency? 99% efficiency?
Keep in mind that the number of communication events (transfers of a sub-block's
bottom edge from one worker to the worker directly below) grows linearly with
*α*. Too much communication means bad performance.
#### 7.4.4 Task scheduling A
task—the computation of a sub-block—is enabled once the sub-blocks to its left
and above it have been computed. (Sub-blocks along the top and the leftmost rim

```text
depend on one neighbor only, of course; the upper-left sub-block depends on no
```
others. ) Clearly, we can't merely dump task-descriptors for every sub-block
into a bag at the start of the computation, and turn the workers loose. We must
ensure that workers grab a task-descriptor only when the corresponding task is
enabled. We *could* begin with a single enabled task (the computation of the
upper-left block), and arrange for workers to generate other tasks dynamically
as they become enabled. (See the exercises.) Another solution is (in some ways)
simpler. After a worker computes a sub-block, it proceeds to compute the next
sub-block to the right. Thus, the first worker starts cruising across the top
band of the matrix, computing sub-blocks. As soon as the upper-left sub-block is
complete, the second worker can start cruising across the second band. When this
second worker is done with its own left-most sub-block, the third worker starts
cruising across the third band, and so on.

This scheme has several advantages. We've simplified scheduling, and reduced
task-assignment overhead: once a worker has been given an initial task
assignment, the assignment holds good for an entire row's worth of blocks. We
have also reduced inter-process communication overhead. When a block is
complete, its right and bottom edges will, in general, be required by other
computations. Under our band-cruising scheme, though, there's no need to drop a
newly-computed right-edge into tuple space. No other worker will ever need to

```text
pick it up; the worker that generated it will use it. We present the code for
```
this version in figures 7.3 and 7.4. The partitioning of the matrix into
sub-blocks of nearly equal height is a bit obscure. Since the number of workers
may not evenly divide the lengths of the sequences, we must provide for slightly
non-uniform blocks. We could treat the left-over as a lump forming one final row
and column of blocks smaller than the rest. But this would leave the worker
assigned to the last row with less work than all the rest, leading to an
unnecessary efficiency loss. Instead, we increment the height (or width—see the
worker's code) by 1, and use this larger value until all the excess has been
accounted for, at which point we decrement the height to its "correct"
value.**Figure 7.3****Wavefront: The agenda-parallel version

```text
(master)****char**side\_seq[MAX], top\_seq[MAX]; real\_main(argc,
argv)**char**\*\* argv; {**char**\* sp; side\_len = get\_target(argv[1],
side\_seq); top\_len = get\_target(argv[2], top\_seq); num\_workers =
atoi(argv[3]); aspect\_ratio = atoi(argv[4]); /\**Set up*. \*/**for**(i = 0; i <
num\_workers; ++i)**eval**("worker", compare());**out**("top sequence",
top\_seq:top\_len); height = side\_len / num\_workers; left\_over = side\_len -
(height\* num\_workers); ++height;**for**(i = 0, sp = side\_seq; i <
num\_workers; ++i, sp += height) {**if**(i == left\_over)
--height;**out**("task", i, num\_workers, aspect\_ratio, sp:height); } real\_max
= 0;**for**(i = 0; i < num\_workers; ++i) {**in**("result", ? max);**if**(max >
real\_max) real\_max = max; } print\_max(real\_max); }**Figure 7.4****Wavefront:
The agenda-parallel version (worker)****char**side\_seq[MAX], top\_seq[MAX];*/\*
```
Note: MAX can differ from main.\*/**/\* Work space for a vertical slice of the
similarity matrix. \*/* ENTRY\_TYPE col\_0[MAX+2], col\_1[MAX+2],

```text
\*cols[2]={col\_0,col\_1}; ENTRY\_TYPE top\_edge[MAX]; compare() { SIDE\_TYPE
left\_side, top\_side;**rd**("top sequence", ? top\_seq:top\_len);
top\_side.seg\_start = top\_side.seq\_start;**in**("task", ? id, ? num\_workers,
? aspect\_ratio, ? side\_seq:height); left\_side.seg\_start = side\_seq;
left\_side.seg\_end = left\_side.seg\_start + height;*/\* Zero out column
buffers. \*/***for**(i = 0; i <= height+1; ++i)
cols[0][i]=cols[1][i]=ZERO\_ENTRY; max = 0; blocks = aspect\_ratio\*
num\_workers; width = top\_len / blocks; left\_over = top\_len -
(width\*blocks); ++width;*/\* Loop across top sequence, stride is width of a
sub-block. \*/***for**(block\_id = 0) block\_id < blocks; ++block\_id,
top\_side.seg\_start += width) {**if**(block\_id == left\_over) -- width;
top\_side.seg\_end = top\_side.seg\_start + width;**if**(id)*/\* Get top edge
```
from the worker "above". \*/***in**("top edge", id, block\_id, ?

```text
top\_edge:);**else***/\* If nothing above, use zero. \*/***for**(i = 0; i <
width; ++i) top\_edge[i] = ZERO\_ENTRY; similarity(&top\_side, &left\_side,
cols, top\_edge, &max);*/\* Send "bottom" edge (in reality the overwritten
```
top\_edge). \*/***if**((id+1) < num\_workers)**out**("top edge", id+1,

```text
block\_id, top\_edge:width);**} out**("result", max); } The worker code uses the
similarity() routine from the previous chapter. We can now account for its extra
```
arguments. It's designed to accept a pointer to a max value (for recording the
maximum entry computed), working buffers (cols), a description of the sequence
segments that label the left and top sides of a sub-block, and a vector of
entries forming the top edge of the sub-block. During the computation, this
vector is overwritten with a new set of values defining the *bottom* edge of the
sub-block. The cols buffer is used in analogous fashion: initially cols[0]
points to a buffer holding the left edge, finally it points to the buffer
holding the right edge of the sub-block. Thus, when similarity() completes, we
know the maximum similarity value within the sub-block, and the values that
define the right and bottom edges. The program's performance is summarized by
the speedup graph in figure 7.5 (as usual, the abscissa reports the number of
workers—the number of processes is one greater). We ran tests on the Encore
Multimax and the Intel iPSC2, again running a larger problem on the larger Intel
machine. On both machines,*α*(the aspect ratios) was 10. Again, speedup is
relative to a sequential C program running on one processor of the machine in
question.

On the Multimax, our test problem compared a sequence of length 3500 against

```text
itself. (Big problems are what we want to parallelize; here, a big problem is a
```
big sequence [not a big database]. The largest sequence in just one subset of
the standard compendium of genetic sequence data is, at present, over 70,000

```text
bases long; there's nothing implausible about a length 3500 sequence.) The
```
Multimax sequential time for this problem was 258 seconds... or 247 seconds.
Here we have an example of the memory effects discussed in chapter 4. These
large sequences are "cache busters" taken in their entirety, but will fit in
cache when they're broken up into pieces. The higher sequential time (258
seconds) is the figure actually reported by the sequential code. The lower time
is calculated by extrapolation from the sequential database-search times—recall
that, for the problems discussed in the previous chapter, the sequences in the
database and the target sequences are all "small" compared with our current test
case. Which time should we use in assessing performance? The former is of more
interest to the program's user. But the latter corrects for memory effects, and
thus makes it somewhat easier to assess our analysis and verify our
understanding of this code's performance. Setting *α* to 10 should yield a
maximum-achievable speedup that is 90% of ideal. And indeed, using the lower

```text
(memory-effect corrected) time, our measured efficiencies are within 5% of this
```
model through 10 workers on the Multimax and 40 workers on the iPSC/2. (Using
the larger time, efficiencies*exceed *the "maximum achievable" figure on both
machines for small numbers of workers.) The 16-worker time for the Multimax was
18 seconds. Using a 7000-base self-comparison problem, sequential times for the

```text
iPSC/2 were 776 and 758 seconds; 60 workers finished the computation in 16
```
seconds. On both machines, there is a tail-off in efficiency as the number of
workers increases. We can account for this tail-off if we think about the size
of each block as a function of the number of workers. The number of blocks is

```text
*α* times the square of the number of workers; thus, the size of each block, and
```
accordingly the amount of work per communication event,* falls as the square of
the number of workers *. It is the inexorable drag arising from this
steadily-sinking task granularity that shows up in the observed performance
degradation.
### 7.5 Back to the original database problem We've developed a
reasonable approach to parallel wavefront computations. Suppose, though, that we
need to perform a *series* of wavefront computations—not merely a single one in
isolation. Why should a series of computations be necessary? For one, consider
our original DNA-database problem. Our goal was to compare a target to every
sequence in the database, not merely to one other sequence. One way to speedup a
series of events is (of course) to speedup each event in the series. In this
sense, parallelizing each individual comparison, using the techniques developed
in this chapter, is one way to speedup the database search as a whole. But once
we've decided to perform a whole series of comparisons, a further significant
optimization to our wavefront technique suggests itself. The idea is simply to
overlap the shut-down phase for one comparison with the start-up phase for the
next. Processors that would normally lie idle while the last few sub-blocks of
one comparison were being computed can be set to work on the first few
sub-blocks of the next comparison. As a result, we pay *once* for start-up and
*once* for shut-down over the *entire* database search. The conversion to
overlapped execution is simple in principle, but we must take care to avoid
several problems. First, there need only be two sequences (or actually, parts of

```text
two sequences) in tuple space at any point; again, we must assume that our
```
database is too large to fit into core, and that we must play it out gradually.
While some workers finish up one comparison, the rest soldier on to begin the
next. Carrying out this clutter-control strategy requires some additional
synchronization between the master and the workers. Our code uses an index tuple
to assign each worker an identification tag for the duration of a particular
sequence comparison. (The id is, in effect, the index of the row for which the
worker will be responsible. ) The worker who picks up the last id

```text
(num\_workers-1) informs the master that all workers have signed on to the most
```
recent comparison. Only then does the master set up the *next* comparison. The
second problem is a bit more subtle. If we are not careful to distinguish top
edge tuples, the possibility exists that tuples from two different comparisons
might be mixed up. A speedy first worker might romp through the first row of one
comparison strewing top edge tuples for the second row in its wake, and then zip
on to the first row of the next comparison. If the worker handling the second
row of the first comparison is relatively slow, it may see a mixture of top edge
tuples, some from the first comparison and some from the second. Thus, block

```text
coordinates alone are insufficient labels for top edge data; we need to include
```
a task\_id as well. This change is easily handled by adding a task\_id field to
the top edge tuple. The code, reflecting these comments, is given in figures 7.6
and 7.7.**Figure 7.6****Overlapped database search (master)****char**dbe[MAX +

```text
HEADER], dbs=dbe + HEADER, target[MAX]; real\_main(argc, argv)**char**\*\* argv;
{**char**\* dbsp; t\_length = get\_target(argv[1], target); open\_db(argv[2]);
num\_workers = atoi(argv[3]);*/\* Set up \*/***for**(i = 0; i < num\_workers;
++i)**eval**("worker", compare());**out**("top sequence", num\_workers,
target:t\_length);**while**(d\_length = get\_seq(dbe)) {**out**("id", 0); height
= d\_length / num\_workers; left\_over = d\_length - (height\* num\_workers);
++height;**for**(i = 0, dbsp = dbs; i < num\_workers; ++i, dbsp += height)
{**if**(i == left\_over) --height;**out**("task", i, ++task\_id,
get\_db\_id(dbe), dbsp:height);**} in**("started");**} for**(i = 0; i <
num\_workers; ++i)**out**("id", -1);*/\* Gather maxes local to a given worker
and compute global max. \*/* real\_max = 0;**for**(i = 0; i < num\_workers; ++i)
{**in**("result", ? db\_id, ? max); (max > real\_max) ? (real\_max=max,
real\_max\_id=db\_id):0; } print\_max(db\_id, real\_max); }**Figure
```
7.7****Overlapped database search (worker)***/\* side\_seq, top\_seq, col\_0,
etc., are the same as in figure 7.3. \*/* compare() { SIDE\_TYPE left\_side,

```text
top\_side;**rd**("top sequence", ? num\_workers, ? top\_seq:top\_len); width =
top\_len /num\_workers; left\_over = top\_len - (width\* num\_workers);
local\_max = 0;**while**(1) {**in**("id", ?id);**if**(id == -1)**break**;
((id+1) == num\_workers) ?**out**("started") :**out**("id", id+1);**in**("task",
id, ? task\_id, ? db\_id, ? side\_seq:height);**if**(height == 0)**break**;
top\_side.seg\_start = top\_seq; left\_side.seg\_start = side\_seq;
left\_side.seg\_end = left\_side.seg\_start + height;**for**(i = 0; i <=
height+1; ++i) cols[0][i]=cols[1][i]=ZERO\_ENTRY;*/\* Loop across top sequence,
stride is width of a sub-block. \*/*++width; max = 0;**for**(block\_id = 0;
block\_id < num\_workers; ++block\_id, top\_side.seg\_start += width)
{**if**(block\_id == left\_over) --width; top\_side.seg\_end =
top\_side.seg\_start + width;**if**(id)*/\* Get top edge from the worker
```
"above". \*/***in**("top edge", task\_id, id, block\_id, ?

```text
top\_edge);**else***/\* If nothing above, use zero. \*/***for**(i = 0; i <
width; ++i) top\_edge[i] = ZERO\_ENTRY; similarity(&top\_side, &left\_side,
cols, top\_edge, &max);*/\* Send "bottom" edge (in reality the overwritten
```
top\_edge). \*/***if**((id+1) < num\_workers)**out**("top edge", task\_id, id+1,

```text
block\_id, top\_edge; width); } (max > local\_max) ? (local\_max=max,
local\_max\_id=db\_id):0; }**out**("result", local\_max\_id, local\_max); }*/\*
```
Exercise: why can't we make "started" == "id", num\_workers ? \*/* We noted in
the previous section that our blocking approach has a drawback: task granularity
falls as the square of the number of workers. This means the code is likely to
be efficient only when dealing with relatively large sequences or running with
relatively few workers. In our test database, short sequences predominate.
Figure 7.8 compares the speedup using the overlapping code to speedup for the
final version of the code in the previous chapter, using the test case described
there for the Multimax. As the number of workers increases, speedup is
acceptable at first, then levels off, and then actually begins (at 16 workers)
to*decrease *. Nonetheless, for a modest number of workers, overlapping *does*
work.*α* is 1 here, and yet we manage a speedup of over 9 with 12 workers,
*versus* an expected speedup of 6 using the wavefront code "as is" to process a
series of comparisons.
### 7.6 And the denouement: hybrid search
We've made a number of changes to the original code that have resulted in a much more efficient program. We can also see that this code
doesn't suffer from many of the problems that the agenda version did. In particular because it parallelizes each comparison we needn't worry
about the issues stemming from the multiple comparison approach of the agenda version: we only need one sequence (or a few sequences)
from the database at any given time, we don't need to order the sequences to improve load balancing, and long comparisons will not lead to
idle workers. So why bother with the other version? In a word, efficiency. Doing comparisons in chunks *must* involve more overhead than
doing them "whole". Ideally we would like to pay that overhead only when necessary, and use the parallel comparisons of the previous

```text
chapter (as opposed to *parallelized* comparisons) by default. Fortunately, it's possible to write a code that combines the best parts of both.
```
Actually, we've already done it (almost). The hybrid approach follows from a simple observation: if we could fix the block size by making it
relatively large, comparisons involving long sequences would still be blocked (and therefore be "internally" parallelized), while short
sequences would fit into one block, with many such comparisons taking place in parallel. In other words, we would use the more efficient
approach where it worked best: with moderate size comparisons. We would use the more expensive approach where it was needed: for very
long sequences.
To do this we need to modify our previous code to manage tasks explicitly—we can no longer guarantee that all workers are meshing
together in the same way that they did for the overlapped code, in which sub-blocking was adjusted to ensure every worker had a slice of
every comparison. We can still exploit row affinity, so this boils down to handling the first sub-block of each row as task that when created
is "incomplete" and that becomes enabled when the the top-edge is computed.
We must now choose an appropriate block size. If sub-blocks are too large, the largest piece of work may still dominate the rest of the
computation. If they are too small, we defeat our attempt to use more efficient whole-sequence (*non*-sub-blocked) comparisons whenever
possible.
To simplify things, let's assume that the target sequence always defines the top row of the comparison matrix and that there is at least one
really large sequence that needs blocking (a worst case assumption). The largest possible chunk of work, then, is the computation of a row of
sub-blocks. The size of this chunk* in toto *is* T \*B*, where *T*(the width) is the length of the target, and *B*(the height) is the height of one

```text
block. To this point, we've neglected start-up and shut-down costs; if we add these in, the *last* piece of work this size cannot be completed
```
any earlier than the time needed to compute about 2 *TB* entries, establishing a lower limit for the parallel runtime. If the total number of
workers is *W*, we would like maximum speedup to be *W*, or in other words
*S = W = DT/*2* TB = D/*2* B,*where *D* is the size of the database. This in turn implies that *B = D/ 2W*.
Note that this constraint is easily expressed algebraically and straightforward to compute. Thus it's no problem to design a code that
dynamically adapts to information about the number of processors (which implies the desired speedup) and the size of the database. Once
again, we have a knob that can be used to adjust the granularity of tasks to suit the resources at hand.
The code, assuming a constant block size, is presented in figures 7.9 and 7.10.

```text
**Figure 7.9****Hybrid database search (master)****char**dbe[MAX + HEADER], \* dbs = dbe+HEADER, target[MAX];
real\_main(argc, argv)**char**\*\* argv;
{**char**\* dbsp;
 t\_length = get\_target(argv[1], target);
 open\_db(argv[2]);
 num\_workers = atoi(argv[3]);
 lower\_limit = atoi(argv[4]);
 upper\_limit = atoi(argv[5]);*/\* Set up \*/***for**(i = 0; i < num\_workers; ++i)**eval**("worker", compare());**out**("top sequence", target:t\_length);**out**("index", 1);**while**(d\_length = get\_seq(dbe)) {
 blocks = (d\_length+BLOCK-1) /BLOCK;**if**(blocks > t\_length) blocks = t\_length;
 height = d\_length / blocks;
 left\_over = d\_length - (height\* blocks);
 ++height;**for**(i = 0, dbsp = dbs; i < blocks; ++i, dbsp += height) {**if**(i == left\_over) --height;**out**("task", ++task\_id, i, blocks, get\_db\_id(db\_id), dbsp:height);**}
 if**(++ tasks > upper\_limit);*/\* Too many tasks, get some results. \*/***do in**("task done");**while**(--tasks > lower\_limit);
 }*/\* Poison tasks. \*/***for**(i = 0; i < num\_workers; ++i)**out**("task", ++task\_id, -1,0,0,"":0);
 close\_db();**while**(tasks--)**in**("task done");*/\* Clean up \*/**/\* Gather maxes local to a given worker and compute global max. \*/* real\_max = 0;**for**(i = 0; i < num\_workers; ++i) {**in**("result", ? db\_id, ? max);
 (max > real\_max) ? (real\_max=max, real\_max\_id=db\_id):0;
 }
 print\_max(db\_id, real\_max);
}*/\* Remark: BLOCK could be a function of \DB\ and num\_workers: block = |DB| /(2\* nw) \*/*
```

```text
**Figure 7.10****Hybrid database search (worker)***/\* side\_seq, top\_seq, col\_0, etc., are the same as in figure 7.3. \*/* compare()
{
 SIDE\_TYPE left\_side, top\_side;**rd**("top sequence", ? top\_seq:top\_len);
```

```text
local\_max = 0;**while**(1) {**in**("index", ? task\_id);**out**("index",
task\_id+1);**in**("task", task\_id, ? id, ? blocks, ? db\_id, ?
side\_seq:height);**if**(id == -1)**break**; top\_side.seg\_start = top\_seq;
left\_side.seg\_start = side\_seq; left\_side.seg\_end = left\_side.seg\_start +
height;**for**(i = 0; i <= height+1; ++i) cols[0][i]=cols[1][i]=ZERO\_ENTRY;*/\*
```
Loop across top sequence, stride is width of a sub-block. \*/* width = top\_len

```text
/blocks; left\_over = top\_len - (width\* blocks); ++width; max =
0;**for**(block\_id = 0; block\_id < blocks; ++block\_id, top\_side.seg\_start
+= width) {**if**(block\_id == left\_over) --width; top\_side.seg\_end =
top\_side.seg\_start + width;**if**(id)*/\* Get top edge from the worker
```
"above". \*/***in**("top edge", task\_id, id, block\_id, ?

```text
top\_edge);**else***/\* If nothing above, use zero. \*/***for**(i = 0; < width;
++i) top\_edge[i] = ZERO\_ENTRY; similarity(&top\_side, &left\_side, cols,
top\_edge, &max);*/\* Send "bottom" edge (in reality the overwritten
```
top\_edge).*\*/**if**((id+1) < blocks)**out**("top edge", task\_id, id+1,

```text
block\_id, top\_edge:width); } (max > local\_max) ? (local\_max=max,
local\_max\_id=db\_id):0;**out**("task done"); }**out**("result",
local\_max\_id, local\_max); } Once again, we present performance data for the
```
Multimax and iPSC/2. First, to establish that this code is competitive with last
chapter's simpler database code, figure 7.11 compares speedups for this program
and the the chapter 6 program using the same test database as before. As
expected, there's virtually no difference. To illustrate the problem we're
addressing, figure 7.12 presents speedups for these same two programs on the
iPSC/2 when our test database is augmented by a single 19,000-base sequence. The

```text
19 Kbase whopper imposes a maximum speedup of about 9 on the database code; the
```
speedup data clearly demonstrate this upper bound. The hybrid version, on the
other hand, takes the big sequence in stride. It delivers close to the same
speedup as it did in the previous case.
### 7.7 Conclusions
Many other computations can be approached using the techniques described in the chapter. Any problem that can be described in terms of a

```text
recurrence relation belongs in this category. Matrix arithmetic falls into this category; we discuss**matrix multiplication**in the exercises.
The result itself needn't be a matrix; we might, for example, build a parallel program in the shape of a tree that develops dynamically as the
program runs. Tree-based algorithms like**traveling salesman**and**alpha-beta search**are candidates for this kind of result parallelism; so
```
are parsers of various kinds.
### 7.8 Exercises
1.**Matrix multiplication**is a problem that can be treated in roughly the same way as string comparison (although it's much simpler, with no
inter-element dependencies). Multiplying a *p*×*q *matrix* A*by a *q*×*r *matrix* B*yields a *p*×*r *matrix* C*, where the value of *C*[* i,j *] is the inner
product of the *ith* row of *A* and the *jth* column of *B*. To get the inner product of two vectors, multiply them pairwise and add the products:
that is, add the product of the two first elements plus the product of the two second elements and so on.
Write a result-parallel matrix multiplication routine (it assumes that the two matrices to be multiplied are stored one element per tuple).
Clearly, all elements of the result can be computed simultaneously. In principle, the multiplications that are summed to yield each inner

```text
product could also be done simultaneously—but don't bother; they can be done sequentially. Now, transform your result-parallel code into
```
an efficient master-worker program that uses sub-blocking. (The input matrices will now be assumed to be stored one block per tuple, and
the result will be generated in this form as well.)
2. (*a*) Write the vector-based result parallel string-comparison program described in the "massive parallelism" discussion above. Each

```text
process can turn into a data tuple, before it does so creating a new process to compute the next element in its row; or, each row can be
```
computed by a single process that creates a stream of data tuples.

```text
(*b*) We've assumed that a "live vector" is created as a first step. The vector's first element starts computing immediately; the other elements
are blocked, but unblock incrementally as the computation proceeds. Write a (slightly) different version in which you begin with a* single* process (which computes the first row of the result); the first process creates the second process as soon it's finished computing one element;
```
the second process creates the third, and so on.
3. Write a result-parallel string comparison program with adjustable granularity. (Each process will compute an entire sub-block of the
result.)

```text
4. Write a different version of the agenda-parallel string comparison program. Workers don't cruise along rows, computing each sub-block;
```
instead, they grab any enabled task from tuple space. An enabled task is a sub-block whose input elements (the values along its left and
upper edge) have already been computed. Compare the code-complexity and (if possible) the performance of the two versions. How would
this strategy affect the overlapped and hybrid searches?
