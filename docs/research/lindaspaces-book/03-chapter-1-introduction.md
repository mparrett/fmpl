# 1 Introduction
This introduction addresses three topics: what we propose to do in this book, why we claim it's important and how we propose to do it.
### 1.1 What?
One way to solve a problem fast is to break the problem into pieces, and arrange for all the pieces to be solved simultaneously. The more
pieces, the faster the job goes—up to a point where the pieces become too small to make the effort of breaking-up and distributing worth the
bother. These simple, obvious observations underlie all of parallel programming. A "parallel program" is a program that uses the
breaking-up and handing-out approach to solve large or difficult problems. This book's master plan is to transform "ordinary" programmers
into parallel programmers.
Parallelism, we claimed, is one way to solve problems fast. Actually, it is *the* way. Throughout human problem-solving history, complex
engineering and organizational problems have been attacked and mastered by using parallelism. Complex organizations are managed, large
buildings built and formidable enemies defeated by bunches of workers, not by isolated actors. A watch, a steam engine or a factory is built
up out of many simultaneously-active components.

```text
Parallelism is the norm; purely *sequential* problem solving is the anomalous restriction. In the early history of computing, the anomaly
```
developed for good reasons. Processors were expensive, and throwing many of them at one problem rarely made sense. More important,
people mistook programming for the mere transcription of algorithms into machine-executable form. We understand now that building
software machinery is an organizational and engineering challenge of the first order.
All of which is not to say that sequential programming is no longer important. It will remain the technique of choice for solving small

```text
problems; it is the indispensable basis for the parallel techniques we discuss in this book. It remains fundamental. When you learn how to
```
paint, you don't forget how to draw. But no serious programmer will want to restrict his technical capabilities to black and white now that
color has arrived.
Early in the "parallel computing age" (around ten years ago, let's say), software pundits used to claim that it would be hard to find programs
that could be "parallelized". After all, most ordinary programs are conceived in terms of *sequences* of activities: "do step 1, then do step
2,..." How often would this kind of program prove to be transformable into a parallel code—"do the following *n *steps* at the same time*?" As
things turned out, it's hard to find a large problem that *can't* benefit from parallelism. Not all algorithms benefit to the same extent, of course,
but algorithms that don't parallelize can often be replaced by algorithms that work better. Parallelism sees active ("production") use today in
numerical problem solving, simulation of large physical and engineering systems, graphics, database manipulation and other areas. Its
potential significance in the near future is enormous. We'll pursue these points below.
Parallel programs are intended for execution on many processors simultaneously. Each processor works on one piece of the problem, and
they all proceed together. In the best case, *n* processors focused on a single problem will solve it *n* times faster than any single processor. We
can't routinely achieve this "ideal linear speedup" in practice, but parallelism is a proven way to run a large variety of real and important
programs fast.
The many processors on which a parallel program executes may all be packed into one box (yielding a " multiprocessor" or "parallel
computer") or they may be separate, autonomous machines connected by a network. A multiprocessor might encompass a few processors or

```text
thousands; its processors might be inter-connected using any of a large number of schemes. The network might be local or wide-area; the
```
computers in the network might be anything. Note, however, that this book isn't about parallel programming "for hypercube-connected
multiprocessors" or "on Ethernet-connected local area nets" or "for the EnnuiTec SX (models 38 through 42)". It deals with parallel
programming *in general*, using *any* kind of processor ensemble. Our only restriction is that we focus on "asynchronous
parallelism"—parallelism in which (roughly speaking) each separate process or locus of activity can do whatever it wants. "Synchronous
parallelism" is a specialized form in which all activities proceed in lock step, doing essentially the same thing at the same time. We discuss
this specialized form only in passing.**The main focus, and two side-issues**. We concentrate mainly on parallel programming for current-generation parallel machines and
networks. Such environments usually encompass anywhere from several through several hundred processors. Inter-processor communication
is fast in some of these settings (the shared-memory multiprocessors), but fairly slow in others and *very* slow in some. A
slow-communication environment can be a challenging one for parallel program development, for reasons we will discuss (and which are
probably obvious in any case). Nonetheless, many useful and powerful platforms for parallel computing *are* slow-communication

```text
environments. The program development techniques we present in this book aren't speculative; they are designed to take real conditions into
```
account. Programmers who master them will be prepared for a wide variety of environments, including hostile ones.
The book has two subsidiary focuses as well. Neither is treated in depth, but we glance at both repeatedly.
One is* massive parallelism *(massive *asynchronous* parallelism specifically). Parallel machines encompassing ten thousand powerful
processors or more are now in design. Building software for these machines will be a major technical challenge. In fact, merely figuring out
what to *do* with them will be a challenge: what kind of applications will be capable of consuming hundreds of billions of instructions per
second? These issues are unlikely to be central to the near-term future of programming, as smaller-scale asynchronous parallelism certainly
will be. But they are fascinating and important nonetheless. The last four chapters each comment on the implications for massive parallelism
of the program structures they present.
The other subsidiary focus is *coordination*, a term we explain below. In general, there are many reasons to build software in the form of
multi-process ensembles. Achieving speedup through parallelism is only one. The last chapter discusses multi-process programs that are *not* for speedup.
Ultimately, the questions raised by coordinated software fall off the map of computer science altogether. After all, the world at large is a
mere asynchronous ensemble of asynchronous ensembles. Sociological, economic, mechanical and biological systems can be approached in
these terms. Human ensembles can be supported by some of the same software structures that process ensembles rely on. One way to treat
the art and science of coordinated software is precisely as a microcosmic introduction to a much broader topic, the analysis of coordinated
systems in general. Coordinated software is a laboratory for the study of how a single undertaking is synthesized out of many separate
activities.
These broader issues are not the main point here. We discuss them only in the exercises. Hard-nosed software-engineering types are
welcome to ignore them altogether. But readers who are willing to exercise their imaginations may find in this topic (as in massive
parallelism) lots to think about.**The programming language.**Parallel programming requires the use of a* computing language *and a* coordination language *. Broadly
speaking, the* coordination language *is the glue that allows us to build a unified program out of many separate activities, each specified

```text
using a* computing language *. A computing language (Fortran, C, Lisp,...) allows us to compute values and manipulate local data objects; a
```
coordination language must allow us to create simultaneous activities, and must allow those activities to communicate with each other. It's
possible to subsume the functions of a standard computing language *and* a coordination language within a single all-in-one super-language.
We prefer for many reasons to rely on a standard computing language plus a (separate) coordination language. In this book, the computing

```text
language we use is C; the coordination language is Linda. We assume familiarity with C but not with Linda. (Readers needn't be expert C
```
programmers. It should be an easy transition from any Algol-based language—Pascal, etc.—to a level of expertise sufficient for this book.)
C and Linda jointly form a good basis for this book because they are powerful, simple, efficient languages that have been implemented on a
wide range of machines. But this book is not *about* C or Linda, any more than a generic "Introductory Programming with Pascal" book is
about Pascal. The techniques to be discussed are applicable in any language environment comparable in expressive power to the C-Linda
combination.
### 1.2 Why
Parallelism will become, in the not too distant future, an essential part of every programmer's repertoire. *Coordination*—a general
phenomenon of which parallelism is one example—will become a basic and widespread phenomenon in computer science. Every
programmer, software engineer and computer scientist will need to understand it. We discuss these claims in turn.
#### 1.2.1 Parallelism
Parallelism is a proven way to run programs fast. Everybody who has an interest in running his programs fast has an interest in
parallelism—whether he knows it or not.
The most powerful computer at any given time must, by definition, be a parallel machine. Once you have taken your best shot and built the
fastest processor that current technology can support, two of them are faster. Now consider the intermediate and the bottom levels—roughly
speaking, the level of time-shared computers or file servers, and of workstations and personal computers (these categories are rapidly
fuzzing together, of course). Manufacturers are increasingly discovering that, having provided the basic hardware and software infrastructure
for a computer—the devices, communication busses, power supply, terminal, operating system and so on—it doesn't necessarily make sense
to stop at one processor. Adding more processors to the same box is often a highly efficient way to buy more power at low incremental cost.
First-generation parallel machines were generally aimed at the high-through-medium range of the market. Machines like the BBN Butterfly,
Intel iPSC, Sequent Balance and Symmetry, Alliant FX/8, Encore Multimax, NCUBE and others belong in this category. More recent
machines are often more powerful and relatively less expensive. This category includes (for example) new multiprocessors from Silicon
Graphics, Hewlett Packard/Apollo and Zenith, machines based on the Inmos Transputer processor (from Meiko, Cogent, Chorus and others)
and many more. In early 1990 the market is bracing for an onslaught of "parallel PC's," based on the Intel processors that IBM uses in its
personal computer line. These are designed in the first instance as file servers for workstation networks. But no wide-awake observer has
failed to notice that they will be good platforms for parallel applications too. Look for specialty software houses to start selling
shrink-wrapped parallel applications for these machines within the next few years—graphics packages, math and statistics libraries, database
searchers, financial and market models, fancy spreadsheets. (We'll say more about applications below.) Thereafter the big software houses
will jump on the bandwagon, and parallelism will truly be "mainstream."
Now, forget about parallel computers. Computer networks are, perhaps, an even more important basis for parallelism.
Processors that are packed into a single box can usually communicate with each other at much greater speeds than processors connected in a
local or wide-area network. Still, the principle is the same. We can take a hard problem, break it into pieces and hand one piece to each of *n* processors, whether the processors occupy one box or are strung out across the country. Some parallel programs that run well on parallel
computers fail to perform well on networks. Many programs can be made to perform well in both kinds of environment. We'll discuss the
issues in detail in the following chapters.
We'll start, again, at the high end. The argument is in essence the same as before. If you have a problem that is hard enough to defeat the
fastest current-generation supercomputer (whether it's a parallel machine or not), your next step is a *network* of supercomputers. Clearly, the
ultimate computing resource at any given time isn't the fastest supercomputer, it's *all* the fastest supercomputers—or as many as you can find
on a single network—focused simultaneously on one problem, acting in effect like a single parallel machine.
The typical modern office or lab environment—many workstations or personal computers, networked together—is another promising
environment for parallelism, arguably the most promising of all. If you sum up total computing power over all nodes of a typical local area
network, you often wind up with a significant power pool. The more powerful the nodes and the more of them, the greater the power reserve.

```text
Of course, the power pool is cut up into bite-sized desk-top chunks; the only way to get at the* whole thing *is to run a parallel program on
```
many nodes simultaneously.
If there are times of day when most nodes are idle, we can treat these idle nodes as a temporary parallel computer. But even when most
nodes are working, experience suggests that (on average) many of them aren't working very hard. Users who run compute-intensive
problems also read their mail, edit files, think, flirt, fulminate, eat lunch, crumple up paper and toss it at the wastebasket and perform various
other compute-unintensive activities. Let's say you're a medium-sized organization and you own a network of one hundred workstations or
personal computers. Let's say your machines are busy during normal working hours half time, on average. The real busy-ness number may
be a lot lower. But assuming an average busy-ness of one half, your organization is the proud owner of an (on average) fifty-processor
parallel computer, during the working day. Congratulations. Note that, whenever your network expands or the workstations are upgraded,
your power pool grows more formidable.
Sophisticated workstation users will want (and will get) software tools that allow them to run on unused cycles from all over the network.

```text
(Such tools are already entering the commercial market.) Once again, software houses will build applications. And these trends aren't by any
means inconsistent with the eventual rise of parallel workstations. Each user may have a parallel computer on his desk; *whatever* his desktop
```
machine, it can use the network as a sounding-board to amplify its own compute power. It can gather up unused cycles from all over the
network and throw them at the task in hand. A parallel program (if it is well-designed) has this special "expandability" property that no
sequential program does: the more processors you happen to have, the more you can throw at the problem.
But what will organizations do with their network power pools?—with parallel machines in general? Do we really need all of this power?
Yes.
New power is needed so that we can run current program types better and faster, and to support basically new types of software as well.
Vastly more powerful machines will solve larger numerical problems in more detail and with greater accuracy, run better simulations of
larger and more complex systems, produce more detailed and sophisticated scientific visualizations. Scientists and engineers will rely on
these techniques to progress on problems like computational molecular dynamics, weather prediction and climate modeling, aerospace
system design and a formidable list of others.
Of course, computational power-lust isn't restricted to science and engineering. Parallel machines will support sophisticated graphics and
high-quality voice and handwriting recognition. The most important extra-scientific use of parallelism will probably center on databases:
parallelism will support fast searches through huge piles of data using sophisticated search criteria. Further, parallelism will enormously

```text
expand the boundaries of what fits inside of "real time"—animation, "artificial reality," avionics will benefit; so will high-level monitor
```
programs designed to sift through torrents of incoming data for significant patterns. There are many other examples. Some are discussed in
the following chapters.
#### 1.2.2 Coordination
Thus far, we've focused on the first part of our claim: that parallelism will become an essential part of every programmer's repertoire. We
now turn to the second: that *coordination*—a general phenomenon of which parallelism is one example—will become a central topic in
computer science.
In studying parallelism, you are studying one kind of coordinated programming. (Many parallelism techniques are applicable *verbatim* to
other kinds of coordinated applications.) In studying coordinated programming, you are studying a basic underpinning of all computing.
We use the term *coordination* to refer to the process of* building programs by gluing together active pieces *. Each "active piece" is a process,
task, thread or any other locus of execution independent of the rest. (Executing "independently" means executing concurrently and
asynchronously with the rest.) To "glue active pieces together" means to gather them into an ensemble in such a way that we can regard the
ensemble itself as the program—all our glued-together pieces are in some sense working on the same problem. The "glue" must allow these
independent activities to communicate and to synchronize with each other exactly as they need to. A* coordination language *provides this
kind of glue.
Coordination is important conceptually because it can be treated as one of the two orthogonal axes that jointly span "software space." To
have a useful computer system, you must have computation *and* coordination. Every computation must ultimately be connected to the
outside world.* Connected to *entails* coordinated with *: the computer and its human user are two "active, independent pieces" that must be
assembled into a coordinated ensemble if computation is to have any meaning.
This simple observation has several consequences. One way to view an operating system is precisely as an* ad hoc *coordination language.

Any user-plus-computer is a coordinated ensemble, one special instance of the
sort of thing that this book is about. Further, the fact that a coordinated
ensemble can include both software processes and human "processes" suggests the
existence of a new species of software ("new" meaning not that examples are
unknown, but that the species hasn't been mapped out and explored in general).
We might call it "Turingware": processes and people encompassed within a single
structure, knit together by ordinary coordinated-programming techniques. Within
the ensemble, processes and people don't know whether they're dealing with other
people or other processes. (The "Turing Test" for machine intelligence hinges on
making people and processes indistinguishable beneath a uniform interface.)
Systems like the Unix command interpreter anticipate some aspects of
"Turingware" when they allow terminal screens (thus ultimately human users) and
ordinary files to be interchanged for some purposes. In full-blown Turingware,

```text
however, a human isn't a mere data terminus; he can be one node in a far-flung
```
network. Certain modules of the system (to put things another way) are
implemented by people. The goal is three-fold. First, to use the methods of
parallel programming to build fast, flexible and powerful information-exchange

```text
frameworks (what we call coordination frameworks in later chapters) for human
```
use. Second, to apply the architectural and organizational insights of parallel
programming to the coordination of *human* ensembles. Third, to change
software's status—to allow its seamless incorporation into human organizations,
in such a way that a program may (for example) ask me for data, reach some
conclusions and pass them along to other people. Human-type work gets done by
humans, computer-type work is done by "software robots," and they all work
merrily together. Exercises in chapters 8 and 9 discuss some examples. Operating
system designers have always been concerned with coordination (under one name or
another). Early work on parallel programming was heavily influenced by operating
system design for exactly this reason. But coordination is becoming a phenomenon
of increasing interest to programmers in general. A* parallel program *and a*
distributed system *are both coordinated programs. Parallel programs use

```text
concurrency to run fast; a distributed system—a distributed database, mail
```
system or file service, for example—uses concurrent processes because of the*
physical distribution *of the machines it manages. (A distributed mailer running
on fifty workstations may encompass fifty separate processes, one per machine.
But the point isn't to run fifty times faster than a one-machine mailer.) The
active pieces that make up a coordinated program may all be expressed in one
language, or in many different languages. A coordinated program may, in other
words, be a l* anguage-heterogeneous *program. The active pieces that make up a
coordinated program may be separated either by space or by time. If two
processes run at the same time on different machines, and the first needs to
send information to the second, the information must be sent through space. But
if two processes run on the *same* machine, one on Tuesday and the other on
Thursday, we may have a logically identical coordination problem. The Tuesday
process needs to send information to the Thursday process, only through time
instead of through space. The result is, still, a coordinated program.
"Time-wise" coordination languages usually take the the form of file systems or
databases, and we are not used to thinking of "time-wise" process ensembles in
the same way we think of space-wise ensembles. But the principles are the same.
Readers will discover that many of the coordination techniques developed in the
context of parallelism are logically applicable without modification to
file-system or database problems as well. In sum, coordination issues have

```text
always been fundamental to operating systems; and clearly they are central to
```
parallel programming. But in the future, users will be tempted with increasing
frequency and for all sorts of reasons to build large programs by gluing smaller
ones together. This approach complements a whole collection of current trends.

```text
Computer networks are ubiquitous; it's natural and desirable to treat them as
```
*integrated* systems. It's quite clear that no single programming language is
destined to predominate, that many languages will continue in use for the
foreseeable future (and probably forever). Hitching mixed-language programs
together is an increasingly obvious and desirable expedient. Special-purpose
hardware is proliferating: if I have a graphics machine, a database server and a
general-purpose "compute server," an application that requires all three
services will run, naturally, on all three machines. And of course, the world's
processor supply continues to explode. An environment in which processors are
cheap and plentiful *demands* coordinated programming. In a world that has more
processors at any given moment than it has running applications, a program that
insists on staying holed up inside a single processor is going to look downright
anti-social.
### 1.3 How? Chapter two lays the conceptual basis for parallelism
and coordination by presenting three basic paradigms for parallel programming.
In chapter three we introduce Linda, and then present the basic programming

```text
techniques (particularly the basic* data structures *for coordination) that
```
underlie all three fundamental approaches. Chapter four completes the groundwork
by discussing debugging and performance measurement. In chapter five, the
details come together: we show how to develop a simple parallel program under
all three basic approaches. Chapters six, seven and eight explore the three
basic paradigms in greater detail. They focus on case studies of "real"
applications, chapter six on a database problem, chapter seven on a matrix
computation and chapter eight on a kind of network problem. Chapter nine
discusses coordinated programming in the broader sense. It presents some
classical coordination problems, and then discusses a distributed system. The
appendix is a C-Linda programmer's manual. The exercises, we emphasize, are an
integral part of the text. Whether you plan to work through them or not, it
makes no sense to read the book without reading the exercise sections.
Programming students who have no access to a parallel computing environment
might learn a great deal, we think, by doing these problems using a uniprocessor
simulator. (If worst comes to worst, they can be treated as paper-and-pencil
exercises.) Programming students who *do* have a parallel environment will
emerge as* bona fide* parallel programmers if they pursue them seriously. All
the program examples and exercises are based on C-Linda, and the best way to
approach this book is with a C-Linda implementation at hand. Implementations are

```text
available on a variety of parallel machines; a Linda simulator runs on a number
```
of standard workstations. (C-Linda is distributed commercially by Scientific
Computing Associates in New Haven, Connecticut. They can be reached at (203)
777-7442, or via email at info@lindaspaces.com.) But if you don't have Linda,
can't get it or don't want it, there are other ways to approach this book. As
we've noted, the programming problems can be treated as paper-and-pencil
exercises. And the principles involved hold good in any language.
