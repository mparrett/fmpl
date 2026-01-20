# 3 Basic Data Structures
We've discussed paradigms and programming methods. We turn now to the practical question: how do we build working parallel programs?
In this section we sketch implementations of the pieces out of which parallel programs are constructed.
We will start with a systematic investigation of distributed data structures. We will give an overview of the most important kinds of
distributed structures, when each is used and how each is implemented. This first part of the discussion should equip readers with a
reasonable tool-kit for building distributed data structure programs. Of course, we intend to discuss all three programming methods. But the
other two are easily derived from a knowledge of distributed data structures, as we discuss in the following sections. We arrive at
message-passing by restricting ourselves to a small and specialized class of distributed structures. We arrive at live-data-structures by
building distributed structures out of processes instead of passive data objects.
We start with an overview of our coordination language.
### 3.1 Linda
Linda consists of a few simple operations that embody the "tuple space" model of parallel programming. A base language with the addition
of these tuple-space operations yields a parallel-programming dialect. To write parallel programs, programmers must be able to create and
coordinate multiple execution threads. Linda is a model of process creation and coordination that is *orthogonal* to the base language in

```text
which it's embedded. The Linda model doesn't care *how* the multiple execution threads in a Linda program compute what they compute; it
```
deals only with how these execution threads (which it sees as so many black boxes) are created, and how they can be organized into a
coherent program. The following paragraphs give a basic introduction. Linda is discussed in greater detail, and contrasted with a series of
other approaches, in [CG89].
The Linda model is a *memory* model. Linda memory (called *tuple space*) consists of a collection of logical tuples. There are two kinds of

```text
tuples. Process tuples are under active evaluation; data tuples are passive. The process tuples (which are all executing simultaneously)
```
exchange data by generating, reading and consuming data tuples. A process tuple that is finished executing turns into a data tuple,
indistinguishable from other data tuples.
It's important to note that *Linda is a model, not a tool.*A *model*(or *paradigm*) represents a particular way of thinking about a problem. It can
be *realized*(or *instantiated *or* implemented*) in many different ways and in many different contexts. A software *tool*, on the other hand, is a
working system that can be used to solve problems. We will be discussing a system called C-Linda, which is a tool—a piece of software that
supports parallel programming. C-Linda is one *realization* of the Linda model. There are many other realizations, and these others aren't
necessarily compatible with (needn't even closely resemble) C-Linda.

```text
Some realizations are designed as platforms for operating systems; they include multiple tuple spaces, and place restrictions on tuple format
```
in the interests of efficiency in the absence of compiler support [ Lel90]. Others, in a Lisp environment, replace Linda's matching protocol
with Prolog-style unification [AJ89]. Others use Linda-style operations to supply blackboard-style communication within Prolog [ ACD90].
Others treat Linda's idea of a tuple as an extension to a Pascal-style type system [ BHK88]. Others integrate Linda into an object-oriented

```text
environment [MK88]; and there are many other projects ongoing, designed to build Linda-like systems for databases, for image-processing,
```
for visualization, in many different Lisp and Prolog environments, and so on.

```text
To summarize: Scheme and Common Lisp differ dramatically, but they're both realizations of the *Lisp* paradigm; Simula 67, Smalltalk and
```
C++ differ even more radically, but they're all realizations of *object-oriented programming*. The C-Linda we discuss in this book is one
realization of the Linda paradigm.
C-Linda has four basic tuple-space operations, out, in, rd and eval, and two variant forms, inp and rdp.

```text
out(*t*) causes tuple *t* to be added to tuple space; the executing process continues immediately. A tuple is a series of typed values, for example
("a string", 15.01, 17, x),
or
(0, 1).
in(*s*) causes some tuple *t* that matches anti-tuple *s* to be withdrawn from tuple space. An anti-tuple is a series of typed fields; some are values
(or "actuals"), others are typed place-holders (or "formals"). A formal is prefixed with a question mark. For example,
("a string", ? f, ? i, y).
The first and last field are *actuals*; the middle two fields are *formals*. Once in(*s*) has found a matching *t*, the values of the actuals in *t* are
```
assigned to the corresponding formals in *s*, and the executing process continues. If no matching *t* is available when in(*s*) executes, the
executing process suspends until one is, then proceeds as before. If many matching *t* s are available, one is chosen arbitrarily.

```text
rd(*s*) is the same as in(*s*), with actuals assigned to formals as before,
```
except that the matched tuple remains in tuple space. Predicate versions of in
and rd, inp and rdp, attempt to locate a matching tuple and return 0 if they

```text
fail; otherwise they return 1, and perform actual-to-formal assignment as
```
described above. (If and only if it can be shown that, irrespective of relative
process speeds, a matching tuple must have been added to tuple space before the
execution of inp or rdp, and cannot have been withdrawn by any other process
until the inp or rdp is complete, the predicate operations are *guaranteed* to
find a matching tuple.) eval(*t*) is the same as out(*t*), except that *t* is

```text
evaluated after rather than before it enters tuple space; eval implicitly
```
creates one new process to evaluate each field of *t*. When all fields have been
completely evaluated, *t* becomes an ordinary passive tuple, which may be ined
or read like any other tuple. A tuple exists independently of the process that
created it, and in fact many tuples may exist independently of many creators,
and may collectively form a data structure in tuple space. It's convenient to
build data structures out of tuples because tuples are referenced associatively,
somewhat like the tuples in a relational database. Examples: executing the out
statements out("a string", 15.01, 17, x) and out(0, 1) causes the specified
tuples to be generated and added to tuple space. An in or rd statement specifies
a template for matching: any values included in the in or rd must be matched

```text
identically; formal parameters must be matched by values of the same type. (It's
```
also possible for formals to appear in tuples, in which case a matching in or rd
must have a type-consonant value in the corresponding position.) Consider the
statement in("a string", ? f, ? i, y) Executing this statement causes a search
of tuple space for tuples of four elements, first element "a string", last
element equal to the value bound to y, and middle two elements of the same types
as variables f and i respectively. When a matching tuple is found it is removed,
the value of its second field is assigned to f and its third field to i. The
read statement, for example rd("a string", ? f, ? i, y) works in the same way,
except that the matched tuple is not removed. The values of its middle two
fields are assigned to f and i as before, but the tuple remains in tuple space.
A tuple created using eval resolves into an ordinary data tuple. Consider the
statement eval("e", 7, exp(7)). It creates a three-element "live tuple", and

```text
continues immediately; the live tuple sets to work computing the values of the
```
string "e", the integer 7 and the function call exp(7). The first two

```text
computations are trivial (they yield "e" and 7); the third ultimately yields the
```
value of *e* to the seventh power. Expressions that appear as arguments to eval
inherit bindings from the environment of the eval-executing process *only* for
whatever names they cite explicitly. Thus, executing eval("Q", f(x,y))
implicitly creates two new processes, which evaluate "Q" and f(x,y)
respectively. The process evaluating f(x,y) does so in a context in which the
names f, y and x have the same values they had in the environment of the process
that executed eval. The names of any variables that happen to be free in f, on
the other hand, were *not* cited explicitly by the eval statement, and no
bindings are inherited for them. The statement rd("e", 7, ? value)) might be
used to read the tuple generated by the previous eval, once the live tuple has
resolved to a passive data tuple—*i.e.*, once the necessary computing has been
accomplished. (Executed before this point, it blocks until the active
computation has resolved into a passive tuple.)
### 3.2 The basic distributed
data structures We can divide conventional "undistributed" data structures into
three categories: (1) structures whose elements are identical or
indistinguishable, (2) structures whose elements are distinguished by name, (3)
structures whose elements are distinguished by position. It's useful to
sub-divide the last category: (3 *a*) structures whose elements are
"random-accessed" by position, (3 *b*) structures whose elements are accessed
under some ordering. In the world of sequential programming, the first category
is unimportant. A *set* of identical or indistinguishable elements qualifies for
inclusion, but such objects are rare in sequential programming. Category 2
includes records, objects instantiated from class-definitions, sets and
multi-sets with distinguishable elements, associative memories, Prolog-style
assertion collections and other related objects. Category 3 *a* consists mainly
of arrays and other structures stored in arrays, 3 *b* includes lists, trees,
graphs and so on. Obviously the groupings aren't disjoint, and there are
structures that can claim membership in several. The distributed versions of
these structures don't always play the same roles as their sequential analogs.
Furthermore, factors with no conventional analogs can play a major role in
building distributed structures. *Synchronization* concerns arising from the
fact that a distributed structure is accessible to many asynchronous processes
simultaneously form the most important example. Notwithstanding, every
conventional category has a distributed analog.
### 3.3 Structures with
identical or indistinguishable elements. The most basic of distributed data
structures is a lock or semaphore. In Linda, a counting semaphore is precisely a
collection of identical elements. To execute a *V* on a semaphore "sem",

```text
out("sem"); to execute a *P*, in("sem") To initialize the semaphore's value to
```
*n*, execute out("sem") *n* times. Semaphores aren't heavily used in most
parallel applications (as opposed to most concurrent systems), but they do arise

```text
occasionally; we elaborate in the next section. A *bag* is a data structure that
```
defines two operations: "add an element" and "withdraw an element". The elements
in this case needn't be identical, but they are treated in a way that makes them
indistinguishable. Bags are unimportant in sequential programming, but extremely
important to parallel programming. The simplest kind of replicated-worker
program depends on a bag of tasks. Tasks are added to the bag using out("task",
TaskDescription) and withdrawn using in("task", ? NewTask) Suppose we want to
turn a conventional loop, for example for ( <loop control> ) <something> into a
parallel loop—all instances of *something* execute simultaneously. This
construct is popular in parallel Fortran variants. One simple way to do the
transformation has two steps. First we define a function something() that
executes one instance of the loop body and returns, say, 1. Then we rewrite the
loop: for ( <loop control> ) eval("this loop", something(<iteration-specific

```text
arg>); for ( <loop control> ) in("this loop", 1); We have, first, created *n*
processes; each is an active tuple that will resolve, when the function call
something() terminates, to a passive tuple of the form ("this loop", 1). Second,
```
we collect the *n* passive result tuples. These *n* may be regarded as a bag, or
equivalently as a single counting semaphore which is V'ed implicitly by each
process as it terminates. A trivial modification to this example would permit
each iteration to "return" a result.**Name-accessed structures**Parallel
applications often require access to a collection of related elements
distinguished by name. Such a collection resembles a Pascal record or a C
struct. We can store each element in a tuple of form (name, value) To read such

```text
a "record field", processes use rd(name, ? val); to update it, in(name, ? old);
out(name, new) As always, the synchronization characteristics of distributed
```
structures distinguish them from conventional counterparts. Any process
attempting to read a distributed record field while it is being updated will
block until the update is complete and the tuple is reinstated. Processes

```text
occasionally need to wait until some event occurs; Linda's associative matching
```
makes this convenient to program. For example, some parallel applications rely
on "barrier synchronization": each process within some group must wait at a

```text
barrier until all processes in the group have reached the barrier; then all can
```
proceed. If the group contains *n* processes, we set up a barrier called
barrier-37 by executing out("barrier-37", n)

Upon reaching the barrier point, each process in the group executes (under one

```text
simple implementation) in("barrier-37", ? val); out("barrier-37", val-1);
rd("barrier-37", 0) That is: each process decrements the value of the field
```
called barrier-37, and then waits until its value becomes 0.**Position-accessed
structures**Distributed arrays are central to parallel applications in many
contexts. They can be programmed as tuples of the form (* Array name, index
fields, value *). Thus ("V", 14, 123.5) holds the the fourteenth element of

```text
vector *V*; ("A", 12, 18, 5, 123.5) holds one element of the three-dimensional
```
array *A*, and so forth. For example: one way to multiply matrices *A* and *B*,
yielding *C*, is to store *A* and *B* as a collection of rectangular blocks, one
block per tuple, and to define a task as the computation of one block of the
product matrix. Thus *A* is stored in tuple space as a series of tuples of the

```text
form ("A", 1, 1, <first block of A>) ("A", 1, 2, <second block of A>) ... and
```
*B* likewise. Worker processes repeatedly consult and update a* next-task
*tuple, which steps though the product array pointing to the next block to be
computed. If some worker's task at some point is to compute the* i, jth *block
of the product, it reads all the blocks in *A*'s *ith* row band and *B*'s *jth*

```text
column band, using a statement like for (next=0; next<ColBlocks; next++) rd("A",
i, next, ? RowBand[next]) for *A* and similarly for *B*; then, using RowBand and
```
ColBand, it computes the elements of *C*'s* i, jth *block, and concludes the
task step by executing out("C", i, j, Product) Thus "C" is a distributed array
as well, constructed in parallel by the worker processes, and stored as a series
of tuples of the form ("C", 1, 1, <first block of C>) ("C", 1, 2, <second block
of C>). It's worth commenting at this point on the obvious fact that a
programmer who builds this kind of matrix multiplication program is dealing with
two separate schemes for representing his data, the standard array structures of
his base language and a tuple-based array representation. It would be simple in
theory to demote the tuple-based representation to the level of assembler
language generated by the compiler: let the compiler decide which arrays are

```text
accessed by concurrent processes, and must therefore be stored in tuple space;
```
then have the compiler generate the appropriate Linda statements. Not hard to
do—but would this be desirable? We tend to think not. First, there are

```text
distributed data structures with no conventional analogs, as we've noted; a
```
semaphore is the simplest example. It follows that parallel programmers won't be
able to rely exclusively on conventional forms, and will need to master some new
structures regardless of the compiler. But it's also the case that the dichotomy
between* local memory *and* all other memory *is emerging as a fundamental

```text
attribute (arguably *the* fundamental attribute) of parallel computers. Evidence
```
suggests that programmers can't hope to get good performance on parallel
machines without grasping this dichotomy and allowing their programs to reflect
it. This is an obvious point when applied to parallel architectures without
physically-shared memory. Processors in such a machine have much faster access
to data in their local memories then to data in another processor's local
memory—non-local data is accessible only via the network and the communication
software. But hierarchical memory is also a feature of shared-memory
architectures. Thus an observation like the following, which deals with the BBN
Butterfly shared-memory multiprocessor: [A]lthough the Uniform System [a
BBN-supplied parallel programming environment] provides the illusion of shared
memory, attempts to use it as such do not work well. Uniform System programs
that have been optimized invariably block-copy their operands into local memory,
do their computation locally, and block-copy out their results... This being the
case, it might be wise to optimize later-generation machines for very high
bandwidth transfers of large blocks of data rather than single-word reads and
writes as in the current Butterfly. We might end up with a computational model
similar to that of LINDA [...], with naming and locking subsumed by the
operating system and the LINDA**in, read**and**out**primitives implemented by
very high speed block transfer hardware [Ols86]. Because the dichotomy between
local and non-local storage appears to be fundamental to parallel programming,
programmers should (we believe) have a high-level, language-based model for
dealing with non-local memory. Tuple space provides such a model. Returning to
position-accessed distributed data structures: synchronization properties can
again be significant. Consider a program to compute all primes between 1 and
*n*(we examine several versions of this program in detail in chapter 5). One
approach requires the construction of a distributed table containing all primes
known so far. The table can be stored in tuples of the form ("primes", 1, 2)

```text
("primes", 2, 3) ("primes", 3, 5)
```

```text
... A worker process may need the values of all primes up to some maximum; it
```
reads upwards through the table, using rd statements, until it has the values it
needs. It may be the case, though, that certain values are still missing. If all
table entries through the *kth* are needed, but currently the table stops at *j*
for* j<k *, the statement rd("primes", j + 1, ? val) blocks – there is still no*
j+*1 *st* element in the table. Eventually the* j+*1 *st* element will be
computed, the called-for tuple will be generated and the blocked rd statement
unblocks. Processes that read past the end of the table will simply pause, in
other words, until the table is extended. Ordered or linked structures make up
the second class of position-accessed data structures. It's possible to build

```text
arbitrary structures of this sort in tuple space; instead of linking components
```
by address, we link by logical name. If *C*, for example, is a *cons* cell
linking *A* and *B*, we can represent it as the tuple ("C", "cons", cell), where
cell is the two-element array ["A", "B"]. If "A" is an atom, we might have ("A",
"atom", value) For example: consider a program that processes queries based on
Boolean combinations of keywords over a large database. One way to process a
complex query is to build a parse tree representing the keyword expression to be

```text
applied to the database; each node applies a sub-transformation to a stream of
```
database records produced by its inferiors—a node might *and* together two
sorted streams, for example. All nodes run concurrently. A Linda program to
accomplish this might involve workers executing a series of tasks that are in

```text
effect linked into a tree; the tuple that records each task includes "left",
```
"right" and "parent" fields that act as pointers to other tasks [ Nar88] . Graph

```text
structures in tuple space arise as well; for example, a simple shortest-path
```
program [ GCCC85] stores the graph to be examined one node per tuple. Each
node-tuple has three fields: name of the node, an array of neighbor nodes (Linda
supports variable-sized arrays in tuples), and an array of neighbor
edge-lengths. These linked structures have been fairly peripheral in our
programming experiments to date. But there *is* one class of ordered structure
that is central to many of the methods we've explored, namely streams of various
kinds. There are two major varieties, which we call in-streams and read-streams.
In both cases, the stream is an ordered sequence of elements to which
arbitrarily-many processes may append. In the in-stream case, each one of
arbitrarily-many processes may, at any time, remove the stream's head element.
If many processes try to remove an element simultaneously, access to the stream
is serialized arbitrarily at runtime. A process that tries to remove from an
empty stream blocks until the stream becomes non-empty. In the read-stream case,
arbitrarily-many processes read the stream simultaneously: each reading process
reads the stream's first element, then its second element and so on. Reading
processes block, again, at the end of the stream. In- and read-streams are easy
to build in Linda. In both cases, the stream itself consists of a numbered
series of tuples: ("strm", 1, val1) ("strm", 2, val2) ... The index of the last
element is kept in a tail-tuple: ("strm", "tail", 14) To append NewElt to

```text
"strm", processes use in("strm", "tail", ? index); /\* consult tail pointer \*/
out("strm", "tail", index+1); out("strm", index, NewElt); /\* add element \*/ An
```
in-stream needs a head-tuple also, to store the index of the head value (*

```text
i.e.*, the next value to be removed); to remove from the in-stream "strm",
processes use in("strm", "head", ? index); /\* consult head pointer \*/
out("strm", "head", index+1); in("strm", index, ? Elt); /\* remove element \*/
(Note that, when the stream is empty, blocked processes will continue in the
```
order in which they blocked. If the first process to block awaits the *jth*
tuple, the next blocked process will be waiting for the* j+*1 *st*, and so on.)
A read-stream dispenses with the head-tuple. Each process reading a read-stream

```text
maintains its own local index; to read each element of the stream, index = 1;
<loop> { rd("strm", index++, ? Elt);
```

... } As a specialization, when an in-stream is consumed by only a single
process, we can again dispense with the head-tuple, and allow the consumer to
maintain a local index. Similarly, when only a single process appends to a
stream we can dispense with the tail tuple, and the producer can maintain a
local index. In practice, various specializations of in- and read-streams seem
to appear more often than the fully-general versions. The streams we've
discussed so far are* multi-source, multi-sink *streams: many processes can add

```text
elements (multi-source) and remove or read-elements (multi-sink). Often,
```
however, single-source or single-sink streams are sufficient. Consider, for
example, an in-stream with a single consumer and many producers. Such a stream
occurs in one version of the prime-finding program we'll discuss: worker

```text
processes generate a stream, each of whose elements is a block of primes; a
```
master process removes each element of the stream, filling in a primes-table as
it goes. Consider an in-stream with a single producer and many consumers. In a
traveling salesman program (written by Henri Bal of the Vrije Universiteit in
Amsterdam), worker processes expand sub-trees within the general search tree,
but these tasks are to be performed not in random order but in a particular

```text
optimized sequence. A master process writes an in-stream of tasks; worker
```
processes repeatedly remove and perform the head task. (This structure
functions, in other words, as a distributed queue.) Consider a read-stream with
a single producer and many consumers. In an LU-decomposition program [ BCGL88],
each worker on each iteration reduces some collection of columns against a pivot

```text
value. A master process writes a stream of pivot values; each worker reads the
```
stream.
#### 3.3.1 Message passing and live data structures We can write a
message-passing program by sharply restricting the distributed data structures
we use: in general, a message passing program makes use only of streams. The
tightly-synchronized message passing protocols in CSP, occam and related
languages represent an even more drastic restriction: programs in these

```text
languages use no distributed structures; they rely only (in effect) on isolated
```
tuples. It's simple, then, to write a message passing program. First, we use
eval to create one process for each node in the logical network we intend to

```text
model. Often we know the structure of the network beforehand; the first thing
```
the program does, then, is to create all the processes it requires. In some

```text
cases the shape of a logical network changes while a program executes; we can
```
use eval to create new processes as the program runs. Having created the
processes we need, we allow processes to communicate by writing and reading
message streams. Live data structure programs are also easy to write given the
passive distributed structures we've discussed. Any distributed data structure
has a live as well as a passive version. To get the live version, we simply use
eval instead of out in creating tuples. For example: we've discussed streams of
various kinds. Suppose we need a stream of processes instead of passive data
objects. If we execute a series of statements of the form eval("live stream", i,

```text
f(i)), we create a group of processes in tuple space: ("live stream", 1,
```
<computation of f(1)>) ("live stream", 2, <computation of f(2)>) ("live stream",
3, <computation of f(3)>) ... If f is the function "factorial" (say), then this
group of processes resolves into the following stream of passive tuples: ("live
stream", 1, 1) ("live stream", 2, 2) ("live stream", 3, 6) ... To write a live
data structure program, then, we use eval to create one process for each element
in our live structure. Each process executes a function whose value may be
defined in terms of other elements in the live structure. We can use ordinary rd
or in statements to refer to the elements of such a data structure. If rd or in
tries to find a tuple that's still under active computation, it blocks until
computation is complete. Thus a process that executes rd("live stream", 1, ? x)
blocks until computation of f(1) is complete, whereupon it finds the tuple it's
looking for and continues.
### 3.4 Exercises 1. Write a set of four routines to
support the creation and manipulation of various stream types. create accepts
one argument, describing the type of stream to be created. You should support
three types: single-source multi-sink in-streams, multi-source single-sink
in-stream and multi-source multi-sink in-streams. Thus create(SMS) creates a
single-source, multi-sink in-stream. create yields a result (it can be an
integer) that will henceforth be used as a stream identifier. Thus we can
execute NewStrm = create(SMS). The put routine accepts two arguments: a stream
id, and an integer to put on the stream. (We'll assume that all streams have

```text
integer elements only.) Thus, put(NewStrm, j); (Clearly, the fact that strm is
```
an SMS-type stream must be recorded somewhere, so that put can do the right
thing.) get accepts a stream identifier and returns an integer, if one is

```text
available: elt = get(NewStrm); ( get may block, of course.) The close routine
```
accepts a stream identifier and "closes" the designated stream. (You'll need to
establish for yourself, based on your implementation, what it means to close a
stream.) Why can't read-streams be (conveniently) supported under this scheme?*
Subsidiary question:*assuming that you are storing open-stream information in a
tuple, in what sense have you used C-Linda to implement a "distributed
object?"—"object" in the sense of the word used by object-oriented languages? If
C were itself an "object-oriented" language, would your implementation be
simpler? (If you don't know what "object-oriented programming" is, (*a*) you
must have spent the last five years in Western Samoa, (*b*) you can skip this
subsidiary question, (*c*) if you want to find out what it is, start with*
Simula Begin *by Birtwistle* et al.*[BDMN79] .) 2. Build a distributed hash
table in tuple space. The hash table will hold elements of some uniform,
arbitrary type (pick whatever type you want). It will will have *k* buckets,
each represented by (at least) one tuple. Buckets must capable of holding more
than one element. You must insure that no part of the table is damaged in the

```text
event that many processes try to add elements simultaneously; you should also
```
allow for as much concurrency as possible among users of the table. (It's
clearly *not* satisfactory, for example, to allow only one process at a time to
access the table.) 3. (*a*) Build a set of routines to create and manipulate
vectors in tuple space. NewVec = vector(n,k), creates an *n* element vector with
each element initialized to *k*. Assume that vectors have integer elements. i =

```text
rdvec(NewVec, j) returns the *jth* element of NewVec; update(NewVec, j, i)
```
resets the *jth* element of NewVec to *i*. (*b*) Your implementation of *a*
probably has the following behavior: if many processes try to update some
element simultaneously, one will succeed immediately, another will wait for the

```text
first (will wait one update-time), a third will wait two update-times, and so
```
on. Suppose that we want the following behavior instead: updating a vector
element takes constant time. The cost does *not* rise as more processes attempt
to update simultaneously. (As before, if many processes attempt to update
simultaneously, the final state of the vector will reflect *one* of the update

```text
values; the rest will be lost.) Implement this new version. (*c*) Your
```
implementation probably has the following behavior: the time it takes to execute
vector increases linearly with *n*. Write a new version in which vector's
execution time increases with the log of *n*(so long as there are enough
processors to allow all processes to execute simultaneously). 4. Implement
Lisp-style lists in tuple space. Your implementation should support cons , car

```text
and cdr ; atoms are stored in tuple space using whatever representation you
want. cons yields a result that identifies a new list (or cons cell); this
```
identifier can serve as an argument to car , cdr or to cons itself. Note that it
must be possible for *many* processes to "car-cdr" down the same list
simultaneously. 5. Suppose you have a server process that executes function *S*

```text
for its clients: they invoke the server, sending an argument *x*; the server
```
sends back the value of* S(x)*. (In the course of evaluating *S* it might
perform any ordinary server activity. This server process might be a remote file
system, or a print server, or a location server, or a date-and-time server or
whatever. You don't need to worry about what the server actually does.) (*a*)
Write two routines, ServerS() and S(x). We create the server by executing

```text
eval("Server S", ServerS());
```

```text
Processes invoke the service it provides by executing result = S(x); Assume that
S() yields an integer. Note that S() behaves like a* remote procedure call *:
any process can invoke this routine; information is passed to some other process
```
as a consequence, and a result is eventually returned to the invoking process

```text
(which remains blocked until the result arrives). In writing procedure S(), make
```
sure that each invoking process gets the right answer back (* i.e *. gets the
result that was intended for *it*). (A good way to do this involves a
multi-source single-sink in-stream.) (*b*) Write a routine LoadS() which returns
the current "load average" on server S , defined as "the number of requests
awaiting service from S."' (*c*) S might be a computational rather than a
device-related service: it might (say) convert a printer file from one format to
another, or remove noise from an image, or optimize a compiled program. If this
kind of service is overloaded, we can start another identical server process

```text
(running, presumably, on some other processor). Write a routine that "pushes" a
```
new instance of ServerS() on a "live stack" (a stack whose elements are
processes) when the load average exceeds some figure, and pops the stack when
the load average drops. Popping the stack will require that instances of

```text
ServerS() be aware of the existence of the stack; this routine should be
```
rewritten in such a way that it terminates (or pops itself) when told to do so.
A request for service should be handled by the first available server. Build
"hysteresis" into your stack-management routine: it should resist
overly-frequent pushes and pops. (*d*) Again assuming a "computational" server,

```text
return to your original version of ServerS and modify it as follows: instead of
```
receiving a request, acting on it and sending back the result, ServerS sends
back a *process* that *computes* the requested result. Thus, clients send data

```text
values to the server; the server sends processes back. The interface routine,
S(x), should *not* change.* Massive parallelism:*6. If you had a parallel
```
machine with tens of thousands of processors and very cheap inter-processor
communication, you might want to build active instead of passive data structures

```text
routinely. Write a new version of the vector routine discussed in question 3;
```
the new routine creates "live vectors." The vector is realized as a tree of

```text
processes; vectors implement three operations: "update," "sum" and "sort."
update(NewVec, j, i); works as before. i = sum(NewVec) causes NewVec to sum its
```
elements and return the answer. The summing operation should require time that
is logarithmic in the length of the vector. sort(NewVec) causes NewVec to sort
itself, again in logarithmic time. Note that the structure you build will be
wildly inefficient in current environments. (But it may be a harbinger of the

```text
future.) ° 7. You can use distributed data structures to organize processes; to
```
what extent are the same structures useful (in principle) for organizing people?
Specifically: suppose you had a tuple space that could be accessed
interpretively. You can write arbitrarily-many routines that put tuples into

```text
this space, or withdraw them; these routines never have to be linked together
```
into a single program. Now, consider writing some routines that are invoked by
users directly from an operating system command shell, and that deal with this
tuple space. How would you write routines that (*a*) send another user mail,

```text
(*b*) post notices to a bulletin board on any topic (the topic is designated by
a character string), and search the bulletin board for a notice on some topic;
(*c*) allow many users to withdraw task assignments from a stream, add new
```
assignments to the stream and monitor the current task backlog (* i.e.*the
length of the stream)? (Topics that relate to these questions are discussed in
greater detail in chapter 9.) ° 8. Design and implement a distributed data
structure that represents an (abstract) market. You can decide in advance which
goods are for sale (this may be a market in,* e.g.*, commodities *A*, *B*,*C
*and* D*). The market implements two operations: buy one unit of something, and
sell one unit, at the current prevailing price. In other words, it provides
operations like buy(A) and sell(B). buy blocks if there are no sellers, sell if
there are no buyers. Each commodity has some initial price. Thereafter, prices
change dynamically and automatically, depending on supply and demand. If a buyer
shows up and finds exactly one seller waiting, the price doesn't change. If a
seller shows up and finds one buyer, the price, again, doesn't change. If a

```text
buyer finds more than one seller waiting, the price drops; if a seller finds
```
more than one buyer, it rises. (Use whatever function you want to adjust prices.
If there is more than one buyer, for example, you might set the new price equal
to *P*+ .01(*L*- 1)*P*, where *P* is the old price and* L* is the length of the
buyer's line.) Prices of each commodity are tracked in a distributed data
structure. Your market must be able to support simultaneous invocations of buy

```text
and sell by many processes; action in one commodity should not exclude
```
simultaneous action in other commodities. (This data structure is the basis for
a further exercise in chapter 9.)
