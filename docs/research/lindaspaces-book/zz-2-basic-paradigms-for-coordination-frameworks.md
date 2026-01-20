# 2
Basic Paradigms for Coordination Frameworks
### 2.1
Introduction How do we build programs using parallel algorithms? On a spectrum of approaches, three primary points deserve special mention. We can use *result parallelism*, *agenda parallelism*or *specialist parallelism*, terms we define. We refer to these three as the *basic paradigms*for parallelism. They establish three distinct ways of thinking about parallelism—of designing parallel programs, but also (for that matter) of analyzing parallelism wherever it occurs, even if the parallel actors are people or hardware instead of software. Corresponding to these basic paradigms are three parallel *programming methods*—practical techniques for translating concepts into working programs. We can use message passing, distributed data structures or live data structures. Each programming method involves a different view of the role of processes and the distribution of data in a parallel program. These basic paradigms and programming methods aren't provably the only ones possible. But empirically they cover all examples we have encountered in the research literature and in our own programming experience. A *coordination framework*is the organizing strategy for a parallel program—the part of the program for which you rely in a coordination language. Our goal here is to explain the conceptual paradigms and the programming methods that apply to coordination frameworks, and the mapping between them.
### 2.2

```text
Paradigms and methods How to write parallel programs? For each paradigm there is a natural programming method; each method relates to the others in well-defined ways. (In other words, programs using method *x* can be transformed into programs using method *y* by following well-defined steps.) We will therefore develop the following approach to parallel programming: To write a parallel program, (1) choose the paradigm that is most natural for the problem, (2) write a program using the method that is most natural for that paradigm, and (3) if the resulting program isn't acceptably efficient, transform it methodically into a more efficient version by switching from a more-natural method to a more-efficient one. First we explain the paradigms—result, agenda and specialist parallelism. Then we explain the methods: live data structures, distributed structures and message passing. Finally we discuss the relationship between concepts and methods, and give an example.
```
#### 2.2.1

```text
The paradigms We can envision parallelism in terms of a program's *result*, a program's *agenda of activities*or of an *ensemble of specialists*that collectively constitute the program. We begin with an analogy. Suppose you want to build a house. Parallelism—using many people on the job—is the obvious approach. But there are several different ways in which parallelism might enter. First, we might envision parallelism by starting with the finished product, the *result*. The result can be divided into many separate components—front, rear and side walls, interior walls, foundation, roof and so on. After breaking the result into components, we might proceed to build all components simultaneously, assembling them as they are completed; we assign one worker to the foundation, one to the front exterior wall, one to each side wall and so on. All workers start simultaneously. Separate workers set to work laying the foundation, framing each exterior wall, building a roof assembly. They all proceed in parallel, up to the point where work on one component can't proceed until another is finished. In sum, each worker is assigned to *produce one piece of the result*, and they all work in parallel up to the natural restrictions imposed by the problem. This is the *result parallel*approach. At the other end of the spectrum, we might envision parallelism by starting with the crew of workers who will do the building. We note that house-building requires a collection of separate skills: we need surveyors, excavators, foundation-builders, carpenters, roofers and so on. We assemble a construction crew in which each skill is represented by a separate specialist worker. They all start simultaneously, but initially most workers will have to wait around. Once the project is well underway, however, many skills (hence many workers) will be called into play simultaneously: the carpenter (building forms) and the foundation-builders work together and concurrently, the roofer can be shingling while the plumber is installing fixtures and the electrician is wiring, and so on. Although a single carpenter does all the woodwork, many other tasks will overlap and proceed simultaneously with his. This approach is particularly suited to *pipelined* jobs—jobs that require the production or transformation of a series of identical objects. If we're building a group of houses, carpenters can work on one house while foundation-builders work on a second and surveyors on a third. But this strategy will often yield parallelism even when the job is defined in terms of a single object, as it does in the case of the construction of a single house. In sum, each worker is assigned to *perform one specified kind of work*, and they all work in parallel up to the natural restrictions imposed by the problem. This is the *specialist parallel*approach. Finally, we might envision parallelism in terms of an agenda of activities that must be completed in building a house. We write out a list of tasks; we assemble a collection of workers. Each worker grabs some task and gets to work. When he's finished, he grabs another task. Workers have no particular identities—no special commitment to any part of the project. They do whatever needs doing. The *agenda of tasks*is in control of the action: workers may need to consult it repeatedly. The tasks on the agenda might occur in a single unordered collection (we tell the workers "grab any task from this list"), or there might be a sequence ("first do all these tasks, then start on those"). We need a foundation (foundation building might be represented by a collection of tasks on the agenda); then we need a frame; then we need a roof; then we need wallboard and perhaps plastering, and so on. We assemble a work team of generalists, each member capable of performing any construction step. First, everyone pitches in and builds the foundation; then, the same group sets to work on the framing; then they build the roof; then some of them work on plumbing while others (randomly chosen) do the wiring, and so on. In sum, each worker is assigned to *pick a task from the agenda and do that task—and repeat, until the job is done*, and they all work in parallel up to the natural restrictions imposed by the problem. This is the *agenda parallel*approach. The boundaries between the three paradigms can sometimes be fuzzy, and we will often mix elements of several paradigms in getting a particular job done. A specialist approach might make secondary use of agenda parallelism, for example, by assigning a team of workers to some specialty—the team of carpenters, for example, might execute the "carpentry agenda" in agenda-parallel style. It's nonetheless an essential point that *these three paradigms represent three clearly separate ways of**thinking**about the problem:* Result parallelism focuses on the shape of the finished product; specialist parallelism focuses on the make-up of the work crew; agenda parallelism focuses on the list of tasks to be performed. We've talked about house building; our next (and final) topic is software. But the ubiquitous applicability of these paradigms to all sorts of human endeavors is thought provoking. We pursue this point in the exercises. How do the three paradigms apply to software? Clearly
1. We can plan a parallel application around the data structure yielded as the ultimate result; we get parallelism by
```
computing all elements of the result simultaneously.
2. We can plan an application around a particular agenda of tasks, and then assign many workers to execute the tasks.
3. We can plan an application around an ensemble of specialists connected into a logical network of some kind.

```text
Parallelism results from all nodes of the logical network (all the specialists) being active simultaneously. How do we know what kind of parallelism—what paradigm—to use? Consider the house-building analogy again. In effect, all three classes are (or have been) used in building houses. Factory-built housing is assembled at the site using pre-built modules—walls, a roof assembly, staircases and so on; all these components were assembled separately and (in theory) simultaneously back at the factory. This is a form of result parallelism in action. "Barn raisings" evidently consisted of a group of workers turning its attention *en masse* to each element on a list of required tasks—a form of agenda parallelism. But some form of specialist parallelism, usually with secondary agenda parallelism, seems like the most natural choice for house-building: each worker (or team) has a specialty, and parallelism arises in the first instance when many separate specialties operate simultaneously, secondarily when the many (in effect) identical workers on one team cooperate on the agenda. In software as well, certain paradigms tend to be more natural for certain problems. The choice depends on the problem to be solved. In some cases, one choice is immediate. In others, two or all three paradigms might be equally natural. This multiplicity of choices might be regarded as confusing or off-putting; we'd rather see it as symptomatic of the fact that parallelism is in many cases so abundant that the programmer can take his choice about how to harvest it. In many cases, the easiest way to design a parallel program is to think of the resulting data structure—*result parallelism*. The programmer asks himself (1) is my program intended to produce some multiple-element data structure as its result (or can it be conceived in these terms)? If so, (2) can I specify exactly how each element of the resulting structure depends on the rest, and on the input? If so, it's easy (given knowledge of the appropriate programming methods) to write a result-parallel program. Broadly speaking, such a program reads as follows: "Build a data structure in such-and-such a shape; attempt to determine the value of all elements of this structure simultaneously, where the value of each element is determined by such-and-such a computation. Terminate when all values are known." It may be that the elements of the result structure are completely independent—no element depends on any other. If so, all computations start simultaneously and proceed in parallel. It may also be that some elements can't be computed until certain other values are known. In this case, all element-computations *start* simultaneously, but some immediately get stuck. They remain stuck until the values they rely on have been computed, and then proceed. Consider a simple example: we have two *n*-element vectors, *A *and* B*, and need to compute their sum *S*. A result parallel program reads as follows: "Construct an *n*-element vector *S*; to determine the *ith* element of *S*, add the *ith* element of *A* to the *ith* element of *B*". The elements of *S* are completely independent. No addition depends on any other addition. All additions accordingly start simultaneously, and go forward in parallel. More interesting cases involve computations in which there are dependencies among elements of the result data structure. We discuss an example in the next section. Result parallelism is a good starting point for any problem whose goal is to produce a series of values with predictable organization and inter-dependencies, but not every problem meets this criterion. Consider a program that produces output whose shape and format depend on the input—a program to format text or translate code in parallel, for example, whose output may be a string of bytes and (perhaps) a set of tables, of unpredictable size and shape. Consider a program in which (conceptually) a *single* object is transformed repeatedly—an LU decomposition or linear programming problem, for example, in which a given matrix is repeatedly transformed in place. Consider a program that's executed not for value but for effect—a realtime monitor-and-control program, or an operating system, for example. Agenda parallelism is a versatile approach that adapts easily to many different problems. The most flexible embodiment of this type of parallelism is the master-worker paradigm. In a master-worker program, a master process initializes the computation and creates a collection
of identical worker processes. Each worker process is capable of performing any step in the computation. Workers seek a task to perform, perform the selected task and repeat; when no tasks remain, the program (or this step) is finished. The program executes in the same way no matter how many workers there are, so long as there is at least one. The same program might be executed with one, ten and 1000 workers in three consecutive runs. If tasks are distributed on the fly, this structure is naturally load-balancing: while one worker is tied up with a time-consuming task, another might execute a dozen shorter task assignments. For example: suppose we have a database of employee records, and we need to identify the employee with (say) the lowest ratio of salary to dependents. Given a record *Q*, the function *r(Q)* computes this ratio. The agenda is simple: "apply function *r* to all records in the database; return the identity of the record for which *r* is minimum." We can structure this application as a master-worker program in a natural way: the master fills a bag with data objects, each representing one employee record. Each worker repeatedly withdraws a record from the bag, computes *r* and sends the result back to the master. The master keeps track of the minimum-so-far, and when all tasks are complete, reports the answer.*Specialist parallelism* involves programs that are conceived in terms of a logical network. They arise when an algorithm or a system to be modeled is best understood as a network in which each node executes a relatively autonomous computation, and inter-node communication follows predictable paths. The network may reflect a physical model, or the logical structure of an algorithm (as in a pipelined or systolic computation, for example). Network-style solutions are particularly transparent and natural when there is a physical system to be modeled. Consider a circuit simulator, for example, modeled by a parallel program in which each circuit element is realized by a separate process. There are also problems that partition naturally into separate realms of responsibility, with clearly-defined inter-communication channels; further on we discuss a "cooperating experts" type of heuristic monitor that uses this kind of organization. In chapter 5, we discuss a pipeline type of algorithm, an algorithm understood as a sequence of steps applied to a stream of input values, with each stage of the pipe transforming a datum and handing it forward. For example: suppose a nation-wide trucking company needs to produce a large number of estimates for travel time between two points, given current estimates for road conditions, weather and traffic. We might design a specialist-parallel programs as follows: we embody a map of the continental U.S. in a logical network; each state is represented by its own node in the network. The Wyoming node is responsible for staying up-to-date on travel conditions in and expected transit time through Wyoming, and so forth. To estimate travel time from New Hampshire to Arkansas, we plan out a route, and include a representation of this route within a data object representing a truck. We hand the "truck" to New Hampshire, which estimates its travel time through New Hampshire and then hands the truck to the next state along its route. Eventually the "truck" reaches Arkansas, which prints out the final estimate for its transit time. Note that large numbers of trucks may be moving through the logical network at any one time. We conclude this survey of paradigms by mentioning two special classes that we won't deal with further, "data parallelism" and"speculative parallelism" (sometimes called "or-parallelism"). Data parallelism is a restricted kind of agenda parallelism: it involves a series of transformations each applied to all elements of a data structure simultaneously. If we start with an agenda of activities in which each item requires that a transformation be applied to a data structure, the agenda-parallel program we'd derive would in effect be an example of data parallelism. Empirically, data parallelism is usually associated with synchronous machines (*e.g*. MPP [Gil79], the Connection Machine [HS86]), and is accordingly tied to an implementation in which transformations are applied to all elements of some data structure not merely concurrently but *synchronously*—at each instant, each active worker is applying the same step of the same transformation to its own assigned piece of the structure. In this book, our focus is restricted to techniques that are used on general-purpose *asynchronous* parallel machines. (This focus can be taken as arbitrary, but there's a reason for it. At present synchronous or SIMD machines are rare and expensive; asynchronous machines can be built cheaply, and are increasingly widespread.) In "speculative parallelism", often associated with logic programming but also significant in (for example) parallel algorithms for heuristic search (*e.g.* parallel alpha-beta search on game trees [MC82]), a collection of parallel activities is undertaken with the understanding that some may ultimately prove to be unnecessary to the final result. Whenever a program's structure includes clauses like "try *x*, and if *x* fails, try *y*" (and so on through a list of other alternatives), we can get parallelism by working on *x*, *y* and any other alternatives simultaneously. If and when *x *fails,* y*is already underway. We understand this under our schematization as another special form of agenda parallelism—many workers are thrown simultaneously into the completion of a list of tasks, with the understanding that ultimately, only one of the results produced will be incorporated in the finished product.
```
#### 2.2.2

```text
The programming methods In message passing, we create many concurrent processes, and enclose every data structure within some process; processes communicate by exchanging messages. In message passing methods, no data objects are shared among processes. Each process may access its own local set of private data objects only. In order to communicate, processes must send data objects from one local space to another; to accomplish this, the programmer must explicitly include send-data and receive-data operations in his code (figure 2.1).
```

At the other extreme, we dispense with processes as conceptually-independent
entities, and build a program in the shape of the data structure that will
ultimately be yielded as the result. Each element of this data structure is
implicitly a separate process, which will turn into a data object upon

```text
termination. To communicate, these implicit processes don't exchange messages;
```
they simply "refer" to each other as elements of some data structure. Thus if

```text
process *P* has data for *Q*, it doesn't send a message to *Q*; it terminates,
```
yielding a value, and *Q* reads this value directly. These are "live data
structure" programs (figure 2.2). The message passing and live data structure
approaches are similar in the sense that in each, all data objects are

```text
distributed among the concurrent processes; there are no global, shared
```
structures. In message passing, though, processes are created by the programmer
*explicitly*, they communicate *explicitly* and may send values *repeatedly* to
other processes. In a live data structure program, processes are created
*implicitly* in the course of building a data structure, they communicate
*implicitly* by referring to the elements of a data structure, and each process
produces only a *single* datum for use by the rest of the program. Details will
become clear as we discuss examples. Between the extremes of allowing all data
to be absorbed into the process structure (message passing) or all processes to
melt into a data structure (live data structures), there's an intermediate
strategy that maintains the distinction between a group of data objects and a
group of processes. Many processes share direct access to many data objects or
structures. Because shared data objects exist, processes may communicate and
coordinate by leaving data in shared objects. These are "distributed data
structure" programs (figure 2.3).
### 2.3

```text
Where to use each? It's clear that result parallelism is naturally expressed in a live data structure program. For example: returning to the vector-sum program, the core of such an application is a live data structure. The live structure is an *n* element vector called *S*; trapped inside each element of *S* is a process that computes *A *[* i*] + *B *[* i*] for the appropriate *i*. When a process is complete, it vanishes, leaving behind only the value it was charged to compute. Specialist parallelism is a good match to message passing: we can build such a program under message passing by creating one process for each network node, and using messages to implement communication over edges. For example: returning to the travel-time program, we implement each node of the logical network by a process; trucks are represented by messages. To introduce a truck into the network at New Hampshire, we send New Hampshire a "new truck" message; the message includes a representation of the truck's route. New Hampshire computes an estimated transit time and sends another message, including both the route and the time-en-route-so-far, to the next process along the route. Note that, with lots of trucks in the network, many messages may converge on a process simultaneously. Clearly, then, we need some method for queuing or buffering messages until a process can get around to dealing with them. Most message passing systems have some kind of buffering mechanism built in. Even when such a network model exists, though, message passing will sometimes be inconvenient in the absence of backup-support from distributed data structures. If every node in the network needs to refer to a collection of global status variables, those globals can only be stored (absent distributed data structures) as some node's local variables, forcing all access to be channeled through a custodian process. Such an arrangement can be conceptually inept and can lead to bottlenecks. Agenda parallelism maps naturally onto distributed data structure methods. Agenda parallelism requires that many workers set to work on what is, in effect, a single job. In general, any worker will be willing to pick up any task. Results developed by one worker will often be needed by others, but one worker usually won't know (and won't care) what the others are doing. Under the circumstances, it's far more convenient to leave results in a distributed data structure, where any worker who wants them can take them, than to worry about sending messages to particular recipients. Consider also the dynamics of a master-worker program—the kind of program that represents the most flexible embodiment of agenda parallelism. We have a collection of workers and need to distribute tasks, generally on the fly. Where do we keep the tasks? Again, a distributed data structure is the most natural solution. If the tasks on the agenda are strictly parallel, with no necessary ordering among them, the master process can store task descriptors in a distributed *bag* structure; workers repeatedly reach into the bag and grab a task. In some cases, tasks should be started in a certain order (even if many can be processed simultaneously); in this case, tasks will be stored in some form of distributed queue or stream. For example: we discussed a parallel database search carried out in terms of the master-worker model. The bag into which the master process drops employee records is naturally implemented as a distributed data structure—as a structure, in other words, that is directly accessible to the worker processes and the master.
```
### 2.4
An example Consider a naive *n*-body simulator: on each iteration of the simulation, we calculate the prevailing forces between each body and all the rest,

```text
and update each body's position accordingly. (There is a better ( *O *(* n*)) approach to solving the *n*-body problem, developed by Rokhlin and Greengard of Yale [GR87]; the new algorithm can be parallelized, but to keep things simple, we use the old approach as a basis for this discussion.) We'll consider this problem in the same way we considered house building. Once again, we can conceive of result-based, agenda-based and specialist-based approaches to a parallel solution. We can start with a result-based approach. It's easy to restate the problem description as follows: suppose we're given *n* bodies, and want to run *q* iterations of our simulation; compute a matrix *M* such that *M *[*i*,* j*] is the position of the *ith* body after the *jth* iteration. The zeroth column of the matrix gives the starting position, the last column the final position, of each body. We've now carried out step 1 in the design of a live data structure. The second step is to define each entry in terms of other entries. We can write a function *position(i, j)* that computes the position of body *i* on iteration *j*; clearly,*position(i,j)* will depend on the positions of each body at the previous iteration—will depend, that is, on the entries in column *j-1* of the matrix. Given a suitable programming language, we're finished: we build a program in which *M *[* i,j*] is defined to be the value yielded by *position(i,j)*. Each invocation of *position* constitutes an implicit process, and all such invocations are activated and begin execution simultaneously. Of course, computation of the second column can't proceed until values are available for the first column: we must assume that, if some invocation of *position* refers to *M *[*x*,* y*] and *M *[*x*,* y*] is still unknown, we wait for a value, and then proceed. Thus the zeroth column's values are given at initialization time, whereupon all values in the first column can be computed in parallel, then the second column and so forth (figure 2.4).
```

```text
(Note that, if the forces are symmetric, this program does more work than
```
necessary, because the force between *A *and* B* is the same as the force
between *B *and* A*. This is a minor problem that we could correct, but our goal
here is to outline the simplest possible approach.) We can approach this problem
in terms of agenda parallelism also. The task agenda states "repeatedly apply
the transformation *compute next position*to all bodies in the set". To write
the program, we might create a master process and have it generate *n* initial
task descriptors, one for each body. On the first iteration, each worker in a
group of identical worker processes repeatedly grabs a task descriptor and
computes the next position of the corresponding body, until the pile of task
descriptors is used up (and all bodies have advanced to their new positions).
Likewise for each subsequent iteration. A single worker will require time

```text
proportional to *n* 2 to complete each iteration; two workers together will
```
finish each iteration in time proportional to *n* 2/2, and so on. We can store
information about each body's position at the last iteration in a distributed
table structure, where each worker can refer to it directly (figure 2.5).
Finally we might use a specialist-parallel approach: we create a series of
processes, each one specializing in a single body—that is, each responsible for
computing a single body's current position throughout the simulation. At the
start of each iteration, each process informs each other process by message of

```text
the current position of its body. All processes are behaving in the same way; it
```
follows that, at the start of each iteration, each process *sends data to*but
also *receives data from* each other process. The data included in the incoming
crop of messages is sufficient to allow each process to compute a new position
for its body. It does so, and the cycle repeats (figure 2.6). (A similar but
slightly cleaned up—*i.e*., more efficient—version of such a program is
described by Seitz in [ Sei85].)
### 2.5

```text
A methodology: how do the three techniques relate? The methodology we're developing requires (1) starting with a paradigm that is natural to the problem, (2) writing a program using the programming method that is natural to the paradigm, and then (3) if necessary, transforming the initial program into a more efficient variant that uses some other method. If a natural approach also turns out to be an efficient approach, then obviously no transformation is necessary. If not, it's essential to understand the relationships between the techniques and the performance implications of each. After describing the relationships in general, we discuss one case of this transformation-for-efficiency in some detail. **The relationships.**The main relationships are shown in figure 2.7. Both live data structures and message passing center on* captive data objects *—every data object is permanently associated with some process. Distributed data structure techniques center on *delocalized* data objects, objects not associated with any one process, freely floating about on their own. We can transform a live data structure or a message passing program into a distributed structure program by using *abstraction*: we cut the data objects free of their associated processes and put them in a distributed data structure instead. Processes are no longer required to fix their attention on a single object or group of objects; they can range freely. To move from a distributed structure to a live data structure or a message passing program, we use *specialization*: we take each object and bind it to some process.
```

It's clear from the foregoing that live data structures and message passing are
strongly related, but there are also some important differences. To move from
the former to the latter we need to make communication *explicit*, and we may
optionally use *clumping*. A process in a live-data-structure program has no
need to communicate information explicitly to any other process. It merely
terminates, yielding a value. In a message passing program, a process with data
to convey must execute an explicit "send message" operation. When a
live-data-structure process requires a data value from some other process, it

```text
references a data structure; a message passing process will be required to
```
execute an explicit "receive message" operation. Why contemplate a move from
live data structures to message passing, if the latter technique is merely a

```text
verbose version of the former? It's not; message passing techniques offer an
```
added degree of freedom, which is available via "clumping". A process in a
live-data-structure program develops a value and then dies. It can't live on to
develop and publish another value. In message passing, a process can develop as
many values as it chooses, and disseminate them in messages whenever it likes.
It can develop a whole series of values during a program's lifetime. Hence
"clumping": we may be able to let a single message passing process do the work
of a whole collection of live-data-structure processes.**Using abstraction and
then specialization to transform a live data structure program.**Having
described some transformations in the abstract, what good are they? We can walk
many paths through the simple network in figure 2.7, and we can't describe them
all in detail. We take up one significant case, describing the procedure in

```text
general and presenting an example; we close the section with a brief examination
```
of another interesting case. Suppose we have a problem that seems most naturally
handled using result parallelism. We write the appropriate live data structure
program, but it performs poorly, so we need to apply some transformations.
First, why discuss this particular case? When the problem is suitable, a live
data structure program is likely to be rather easy to design and concise to
express. It's likely to have a great deal of parallelism (with the precise
degree depending, obviously, on the size of the result structure and the
dependencies among elements). But it may also run poorly on most
current-generation parallel machines, because the live-data-structure approach
tends to produce* fine-grained *programs—programs that create a large number of
processes, each one of which does relatively little computing. Concretely, if
our resulting data structure is (say) a ten thousand element matrix, this
approach will implicitly create ten thousand processes. There's no reason in
theory why this kind of program can't be supported efficiently, but on most
current parallel computers there are substantial overheads associated with
creating and coordinating large numbers of processes. This is particularly true
on distributed-memory machines, but even on shared-memory machines that support
lightweight processes, the potential gain from parallelism can be overwhelmed by
huge numbers of processes each performing a trivial computation.

```text
If a live data structure program performs well, we're finished; if it doesn't, a
```
more efficient program is easily produced by *abstracting* to a distributed data
structure version of the same algorithm. We replace the live data structure with
a passive one, and raise the processes one level in the conceptual scheme: each
process* fills in *many elements, rather than *becoming* a single element. We
might create one hundred processes, and have each process compute one hundred
elements of the result. The resulting program is coarser-grained than the
original—the programmer decides how many processes to create, and can choose a
reasonable number. We avoid the overhead associated with huge numbers of
processes. This second version of the program may still not be efficient enough,
however. It requires that each process read and write a single data structure,
which must be stored in some form of logically shared memory. Accesses to a
shared memory will be more expensive than access to local structures. Ordinarily

```text
this isn't a problem; distributed data structure programs can be supported
```
efficiently even on distributed-memory (* e.g.*, hypercube) machines. But for
some communication-intensive applications, and particularly on distributed
memory machines, we may need to go further in order to produce an efficient
program. We might produce a maximally-efficient third version of the program by
using *specialization* to move from distributed data structures to message
passing. We break the distributed data structure into chunks, and hand each
chunk to the process with greatest interest in that chunk. Instead of a shared
distributed data structure, we now have a collection of local data structures,
each encapsulated within and only accessible to a single process. When some
process needs access to a "foreign chunk"—a part of the data structure that it
doesn't hold locally—it must send a message to the process that does hold the
interesting chunk, asking that an update be performed or a data value returned.
This is a nuisance, and usually results in an ugly program. But it eliminates
direct references to any shared data structures. Under this scheme of things, we
can see a neat and well-defined relationship among our three programming
methods. We start with an elegant and easily-discovered but potentially
inefficient solution using live data structures, move on via abstraction to a
more efficient distributed data structure solution, and finally end up via
specialization at a low-overhead message passing program. (We might
alternatively have gone directly from live data structures to message passing
via "clumping".) There's nothing inevitable about this procedure. In many cases
it's either inappropriate or unnecessary. It's inappropriate if live data
structures are *not* a natural starting point. It's unnecessary if a live data
structure program runs well from the start. It's partially unnecessary if
abstraction leads to a distributed data structure program that runs well—in this
case, there's nothing to be gained by performing the final transformation, and
something to be lost (because the message passing program will probably be
substantially more complicated than the distributed data structure version).
It's also true that message passing programs are not always more efficient than

```text
distributed data structure versions; often they are, but there are cases in
```
which distributed data structures are the optimal approach.**An example.**For
example, returning to the *n*-body simulator: we discussed a live-data structure

```text
version; we also developed distributed data structure and message passing
```
versions, independently. We could have used the live-data structure version as a
basis for abstraction and specialization as well. Our live data structure
program created *n*×*q* processes, each of which computed a single invocation of
*position* and then terminated. We can create a distributed data structure
program by *abstraction*. i is now a distributed data structure—a passive
structure, directly accessible to all processes in the program. Its zeroth

```text
column holds the initial position of each body; the rest of the matrix is blank.
```
We create *k* processes and put each in charge of filling in one band of the
matrix. Each band is filled-in column-by-column. In filling-in the *jth* column,
processes refer to the position values recorded in the* j-*1 *st* column. We now

```text
have a program in which number-of-processes is under direct programmer control;
```
we can run the program with two or three processes if this seems reasonable (as
it might if we have only two or three processors available). We've achieved
lower process-management overheads, but the new program was easy to develop from
the original, and will probably be only slightly less concise and
comprehensible. Finally we can use *specialization* to produce a
minimal-overhead message passing program. Each process is given one band of *M*

```text
to store in its own local variable space; *M* no longer exists as a single
```
structure. Since processes can no longer refer directly to the position values
computed on the last iteration, these values must be disseminated in messages.

```text
At the end of each iteration, processes exchange messages; messages hold the
```
positions computed by each process on the last iteration. We've now achieved low
process-management overhead, and also eliminated the overhead of referring to a
shared distributed data structure. But the cost is considerable: the code for
this last version will be substantially more complicated and messier than the
previous one, because each process will need to conclude each iteration with a
message-exchange operation in which messages are sent, other messages are
received and local tables are updated. We've also crossed an important
conceptual threshold: communication in the first two solutions was conceived in
terms of* references to data structure *, a technique that is basic to all
programming. But the last version relies on message passing for
communication—thus substituting a new kind of operation that is conceptually in
a different class from standard programming techniques.**When to abstract and
specialize?**How do we know whether we need to use abstraction, or to move

```text
onward to a message passing program? The decision is strictly pragmatic; it
```
depends on the application, the programming system and the parallel machine.
Consider one concrete datum: using C-Linda on current parallel machines,
specialization leading to a message passing program is rarely necessary. Most
problems have distributed data structure solutions that perform well. In this
context, though, abstraction to a distributed data structure program usually
*is* necessary to get an efficient program.**Another path through the network:
abstraction from message passing.**When live-data-structure solutions are
natural, they may involve too many processes and too much overhead, so we use
abstraction to get a distributed data structure program. It's also possible for
a message passing, network-style program to be natural, but to involve too many
processes and too much inter-process communication—in which case we can use
abstraction, again, to move from* message passing *to distributed data
structures. Suppose, for example, that we want to simulate a ten thousand
element circuit. It's natural to envision one process for each circuit element,
with processes exchanging messages to simulate the propagation of signals
between circuit elements. But this might lead to a high-overhead program that
runs poorly. Abstraction, again, allows us to create fewer processes and put
each in charge of one segment of a distributed data structure that represents
the network state as a whole.

In sum, there are many paths that a programmer might choose to walk though the
state diagram in figure 2.7. But the game itself is simple: start at whatever
point is most natural, write a program, understand its performance and then, if
necessary, follow the "efficiency" edges until you reach an acceptable stopping
place.
### 2.6

```text
Where are the basic techniques supported? Although it's our intention in this book to discuss programming techniques, not programming systems, a brief guide to the languages and systems in which the basic techniques occur may be helpful. Before we discuss some possibilities from the programming methodology point of view, we need to address the basic character of these proposals. The bulk of published work on languages and systems for parallelism deals with* integrated parallel or distributed programming languages *or with* operating systems *that support parallel or distributed programming. Linda, on the other hand, is a* coordination language *. The distinction is important. An integrated parallel language is a complete, new programming language with built-in support for coordination. An* operating system *that supports coordination isn't (of course) a language at all. It supplies a collection of utility routines (to perform message passing, for example) to be invoked by user programs in the same way that I/O or math-library routines might be invoked. These routines get no (or essentially no) compiler support. A* coordination language **is* a language, but one that embodies a coordination model *only*. A new compiler (for example, a C-Linda compiler) supports a *combined* language environment, consisting of a computing language plus a coordination language. There are relatively few coordination languages on the market, but Linda isn't the only one. Dongarra and Sorenson's Schedule [DSB88] is another example. Schedule in turn is related to Babb's work on coarse-grain dataflow [Bab84]. Strand is an example that approaches the problem from a logic-programming viewpoint [FT89]. We expect to see many more examples in coming years. Message passing is by far the most widespread of the basic models; it occurs in many different guises and linguistic contexts. The best-known of message passing languages is Hoare's influential fragment CSP [Hoa78], which inspired a complete language called occam [May83]. CSP and occam are based on a radically tight-knit kind of message passing: both the sending and the receiving of a message are *synchronous* operations. A process with a message to send blocks until the designated receiver has taken delivery. CSP and occam are *static* languages as well: they don't allow new processes to be created dynamically as a program executes. CSP and occam are for these reasons not expressive enough to support the full range of message-passing-type programs we discuss here. Monitor and remote-procedure-call languages and systems are another subtype within the message passing category (with a qualification we note below). In these systems, communication is modeled on procedure call: one process communicates with another by invoking a procedure defined within some other process or within a passive, globally-accessible module. This kind of quasi-procedure-call amounts to a specialized form of message passing: arguments to the procedure are shipped out in one message, results duly returned in another. The qualification mentioned above is that, in certain cases, systems of this sort are used for quasi-distributed data structure programs. A global data object can be encapsulated in a module, then manipulated by remotely-invoked procedures. (The same kind of thing is possible in any message passing system, but it's more convenient given a procedure-style communication interface.) Why *quasi*-distributed data structures? As we understand the term, a distributed data structure is directly accessible to many parallel processes *simultaneously*. (Clearly we may sometimes need to enforce sequential access to avoid corrupting data, but in general many read operations may go forward simultaneously, and many write operations that affect separate and independent parts of the same structure may also proceed simultaneously – for example, many independent writes to separate elements of a single matrix.) Languages in this class support data objects that are global to many processes, but in general they allow processes one-at-a-time access only. Nor do they support plain distributed data objects; a global object must be packaged with a set of access procedures. Monitors were first described by Hoare [Hoa74], and they have been used as a basis for many concurrent programming languages—for example Concurrent Pascal [Bri75], Mesa [LR80], Modula [Wir77]. (A *concurrent* language, unlike a parallel language, assumes that multiple processes inhabit the same address space. Fairly recently they have been revived for use in parallel programming, in the form of parallel object-oriented programming languages (e.g Emerald [JLHB88].) A form of remote procedure call underlies Ada [Ada82]; Birrell and Nelson's RPC kernel [BN84] is an efficient systems-level implementation. Another variant of message passing centers on the use of *streams*: senders (in effect) append messages to the end of a message stream; receivers inspect the stream's head. This form of communication was first proposed by Kahn [Kah74], and it forms the basis for communication in most concurrent logic languages (*e.g.* Concurrent Prolog [Sha87], Parlog [Rin88]) and in functional languages extended with constructs for explicit communication (*e.g.*[Hen82]). Message passing of one form another appears as a communication method in many other parallel languages (for example in Poker [Sny90]) and in many operating systems: for example the V kernel [CZ85], Mach [You87], Amoeba [MT86]. Distributed data structures are less frequently encountered. The term was introduced in the context of Linda [ CGL86]. Distributed data structures form the *de facto* basis of a number of specialized Fortrans that revolve around parallel do-loops, for example Jordan's Force system [Jor86]. In this kind of system, parallelism is created mainly by specifying parallel loops—loops in which iterations are executed simultaneously instead of sequentially. Separate loop-iterations communicate through distributed structures that are adaptations of standard Fortran structures. Distributed data structures in one form of another are central in Dally's CST [ Dal88], Bal and Tanenbaum's Orca [BT87] and Kale's Chare Kernel [Kal89], Browne's CODE [Bro90] and are supported in Multilisp [Hal85] as well. Live data structures are a central technique in several languages that support so-called non-strict data structures—data structures that can be accessed before they are fully defined. Id Nouveau [NPA86], Multilisp and Symmetric Lisp [GJL87] are examples. This same idea forms the implicit conceptual basis for the large class of functional languages intended for parallel programming (for example Sisal [LSF88] or Crystal
```

[Che86]). Programs in these languages consist of a series of equations
specifying values to be bound to a series of names. One equation may depend on

```text
the values returned by other equations in the set; we can solve all equations
```
simultaneously, subject to the operational restriction that an equation
referring to a not-yet-computed value can't proceed until this value is
available. The equivalent program in live-data-structure terms would use each
equation to specify the value of one element in a live data structure.
### 2.7

```text
Exercises The following is the only exercise set that doesn't involve programming. It's somewhat quirky, and not typical of the others. It can serve either as a basis for informal discussion or as a standard question set. Other exercise sets include an occasional "general coordination" question along these lines too; when they occur in other chapters, they are marked out by a prefixed °. The paradigms we've discussed in this chapter are by no means limited to software.*Virtually all human organizations and virtually all machines* use parallelism. We can analyze the sorts of parallelism they use in exactly the same terms we've used for software structures. Why would we want to do this? Does it do any good to identify one house-building technique as "result" and another as "agenda" parallelism? Perhaps. There are two sorts of potential benefit. First, this kind of identification may be a useful way to develop and solidify the *mental models*(or the "intuitions") that underlie the programmer's approach to parallelism. Mental models are crucially important in our approach to programming (as to any other machine-building activity). Consider the quasi-physical model that underlies most programmer's idea of a "pushdown stack"—the model or mental picture of a heap of objects, with new objects added to and removed from the top. The physical model makes it easier for beginners to understand what stacks are. It continues in many cases to be useful to expert programmers as they conduct a mental search for the right data structure. Not every programmer relies on quasi-physical models, of course. But in our observation, many of the best and most creative programmers do. Building analogies between software structures and "real" structures is useful insofar as it promotes the formation of this kind of mental imagery. There may be another benefit as well. Is there such a thing as the science of coordination?—the study of asynchronous systems in general? Such a study would attempt to characterize *in general* the trade-offs between (say) result- and specialist-parallelism, whether the systems at issue were biological, economic, software or whatever. It would attempt to use software (that most plastic, most easily-worked medium for machine-building) as a laboratory for the study of coordination phenomena, but it would also be willing to transfer good coordination strategies from (say) biological to economic domains. The authors can't say for sure whether such a science exists or not. (Those who are interested in its potential existence should read Thomas Malone's"What is Coordination Theory," [Mal88]; they should probably consult Norbert Wiener's *Cybernetics*[Wie48] too, while they are at it.) If such a science *does* exist—if these sorts of investigations do prove *fruitful* and not merely interesting—the study of basic paradigms for coordination frameworks will have a significant role to play in it.
```
1. Human physiology and most internal-combustion engines rely on specialist *and*(in a sense) on agenda parallelism. Give examples and
explain.
2. A nineteenth century German businessman noted (uneasily) that "... the incredibly elaborate division of labour diminish[es] the strength
and intelligence which is required among the masses... [cited by Hobsbawm, Hobs69 p.65]." Translate this into software terms: in what sense does specialist parallelism lead to simpler, less-capable processes than agenda parallelism? But this argument is in some sense counter-intuitive—informally, "specialists" are often assumed to be *more* capable than non-specialists. Are there two species of specialist parallelism?
3. A simple result in electronics states the following: when we connect two resistors in series, the total resistance is the sum of the individual

```text
resistances. When we connect them in parallel, the combined resistance is *R *1* R* 2/*R* 1+*R* 2, where *R* 1 and *R* 2 are the values of the two resistances. The formula for resistors in parallel becomes simple and intuitive when we express it in terms of *conductance*, which is the reciprocal of resistance. When we connect resistors in parallel, their combined *conductance* is equal to the sum of the individual conductances. (For a discussion, see for example [HH80].) Now, consider a collection of software processes of unequal capabilities: some work faster than others (perhaps because some are running on faster computers than others). We can build a specialist-parallel program (specifically a pipeline program) by connecting processes in series; we can build an agenda-parallel program (specifically a master-worker program, with dynamic task assignment) by connecting the processes in parallel. (*a*) Explain. (*b*) Define "software resistance" (or "conductance") in such a way that we can characterize the behavior of these two software structures using the same expressions that describe the electronics.
```
4. Harpsichord and organ engineers typically rely both on specialist and on agenda parallelism ("agenda" again in the broader sense we used
above). Piano builders restrict themselves to agenda parallelism. Explain.
5. In what sense can we describe a coral reef, a city or a human vascular system as the products of result parallelism? In what sense can we
describe a typical military assault against entrenched defenders in these terms? Again, we're using a somewhat more general understanding of the result paradigm. Explain.

```text
6. Pipeline programs represent one version of specialist parallelism. Pipelines have several characteristics; the linear arrangement of their
```
process graphs is only one. Describe a subset of specialist parallelism that includes pipelined programs as one instance, but also includes programs with non-linear process graphs. State as precisely as possible what characteristics a problem should have in order to profit from this style of solution.
7. Malone[Mal88, p. 8] writes that "several organizational theorists have identified a tradeoff between coordination and 'slack resources.'
Organizations with slack resources (such as large inventories or long product development times) can use them as a 'cushion' to reduce the need for close coordination." On the other hand, some American companies have adopted the Japanese "just in time" manufacturing system, in which inventories are minimal or non-existent, and a steady parade of delivery trucks supplies parts to a factory exactly when they are needed. Clearly both approaches have advantages. What are the software-structure analogs of these two alternatives?
