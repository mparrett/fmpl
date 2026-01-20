# 9 Coordinated Programming
### 9.1 And now for something completely different
We noted in the introduction that our topic in the broader sense was *coordinated programming*. We can use the term *coordination* refer to
the process of *building programs by gluing together active pieces*. Each "active piece" is a process, task, thread or any other locus of
execution independent of (and asynchronous with) the rest. *Gluing active pieces together*means gathering them into an ensemble in such a
way that we can regard the ensemble itself as the program. The gluing process requires that we allow independent activities to communicate
and to synchronize with each other exactly as they need to. This definition is vague and broad, which is the whole point.
Parallel programming is one important sub-topic within this general category. But as we noted, many other program types fall into the
"coordination" class as well.*Distributed systems,* which use many processes because they encompass many separate and autonomous
computers, are coordinated programs. The term *concurrent system* is sometimes used in the special sense of a multi-process program running on a single processor. (Operating systems for conventional machines are often concurrent systems; they use multiple processes in the
interests of a clean and simple software structure, and to support user-level time-sharing or multi-programming.) Concurrent systems are
clearly *coordinated* systems as well.*Time-coordinated systems*are coordinated programs; mixed-language programs may qualify as well.

We can't cover this entire ground in one chapter. We focus on a single problem that is simultaneously a *distributed system*, a *concurrent
operating system*and a *time-coordinated system*problem. We prepare for this discussion by considering two "classical problems" in
coordinated programming,*dining philosophers*and the *readers-writers problem*. Neither problem is terribly important in itself, but they both
deal in an abstract way with issues that are typical of this broader class of applications.

```text
Note that concurrency *is already intrinsic*to the applications we discuss in this chapter. We don't have to decide where to put it; we merely
```
need to figure out how to deal with it—how to glue a bunch of concurrent activities into a satisfactory ensemble.
#### 9.1.1 Two classical problems

```text
The "dining philosophers" problem goes like this: a round table is set with some number of plates (traditionally five); there's a single
```
chopstick between each two plates, and a bowl of rice in the center of the table. Philosophers think, then enter the room, eat, leave the room

```text
and repeat the cycle. A philosopher can't eat without two chopsticks in hand; the two he needs are the ones to the left and the right of the
```
plate at which he is seated. If the table is full and all philosophers simultaneously grab their left chopsticks, no right chopsticks are available
and deadlock ensues. To prevent deadlock, we allow only four philosophers (or one less than the total number of plates) into the room at any
one time.
The dining philosophers problem was posed originally by Dijkstra, and has long been used to test the expressivity of new concurrent,
parallel or coordination languages. (In older discussions of the problem, for example Hoare's treatment in his "Concurrent Sequential
Processes [Hoa78]," there was a bowl of spaghetti in the center of the table and a *fork* between every two plates. But the problem in this
form was simply too hard for the average computer science student, who was rarely able to figure out why a philosopher or anyone else
shouldn't be able to make do with one fork in a pinch. In a seminal contribution, Ringwood [ Rin88] realized that, although a single fork is in
most cases perfectly serviceable, a single chopstick is another matter. We use Ringwood's version here, but of course this still leaves several
questions unanswered—why, for example, philosophers are such unsanitary people that they're willing to grab someone else's chopstick and
chow down without a second thought, and whether it's really fair to assume that the typical philosopher knows how to eat rice with *any* number of chopsticks. And of course, even the new and improved formulation leaves the functional programmers still trying to figure out
how you can eat rice without actually consuming any of it. In any case, further development of the problem statement should be regarded as
an exercise. Mail your contributions to Professor Ringwood in London.)
The software problem in this scenario is the following: write a program that has five concurrent processes (the philosophers) and five
"resources" (the chopsticks). Each process executes an infinite loop in which it is guaranteed, at regular intervals, to need a particular set of
two resources from among the five. The ensemble must be put together in such a way that each a process gets the resources it needs
eventually—no process or group of processes ever gets stuck and waits indefinitely.
Dining philosophers is an abstraction of problems that arise when concurrent processes contend for a fixed collection of resources. The
resources might be access to protected parts of the operating system, to devices, or to arbitrary shared data objects. In the problem to be
discussed in the next section, for example, concurrent processes contend for access to a set of objects representing appointment calendars.
#### 9.1.2 A solution

```text
There are Num philosophers in total; we'll only let Num - 1 of them into the dining room at the same time. If were to allow all Num
```
philosophers to be in the room at the same time, it becomes possible for all philosophers to grab their left chopsticks simultaneously, and
then—deadlock. No philosopher can eat, because none has more than one chopstick. No philosopher will relinquish a chopstick: you are not
allowed to put down your chopsticks until you are done eating. With no more than Num - 1 philosophers in the room, on the other hand,
there's guaranteed to be at least *one* philosopher who is able to make use both of his left and of his right chopstick.

```text
(We can show that this is true by using a simple combinatorial argument based on the pigeonhole principle. Consider *n -* 1
```
bins and *n* balls. In every distribution of the *n* balls among the *n -* 1 bins, there must be at least one bin that winds up
holding at least two balls. Now if we associate each bin with a philosopher and each ball with a chopstick, it's clear that the
set of all legal distributions of chopsticks to philosophers is a subset of the possible distributions of balls into bins—a
chopstick can be grabbed only by the fellow on its right or on its left, and hence the corresponding ball can only land in one
of the corresponding two bins. Since every distribution in the original set must assign two balls to one bin, it follows that
every distribution in any *subset* of the original set must also assign two balls to one bin, and we conclude that at least one
philosopher will be guaranteed to succeed in grabbing two chopsticks.)
So long as one philosopher can eat, deadlock is impossible. When the eating philosopher is done, he puts his chopsticks down, and they
become available to the philosophers to his left and right. (In principle, a philosopher could put down his chopsticks, leave the room, then
race back in again and grab both chopsticks before either of his stunned colleagues has made a move. We will return to this possibility.)

```text
A solution to the problem now becomes straightforward (figure 1). We'll allow only Num - 1 philosophers into the room at a time; to
```
accomplish this, we create Num - 1 "tickets" and set them adrift in tuple space. We require a philosopher to grab a ticket (using in) before
he enters the room and starts contending for chopsticks, and to release his ticket (using out) when he's done eating and ready to leave. Each

chopstick is represented by a tuple; philosophers grab chopsticks using in and release them using out.**Figure 9.1****Dining philosophers**phil(i)
 int i;
```
{

```text
 while(1) {
 think ();
 in(in"room ticket")
 in("chopstick", i);
 in("chopstick", (i+i)%Num);
 eat();
 out("chopstick", i);
 out("chopstick",(i+i)%Num);
 out("room ticket");
```
 }
}

```text
initialize()
```
{

```text
 int i;
 for (i = 0; i < Num; i++) {
 out("chopstick", i);
 eval(phil(i);
 if (i < (Num-1)) out("room ticket");
```
 }
}
To return now to the speedy philosopher problem: careful readers will notice that, if the Linda implementation is "unfair"—if it can
repeatedly bypass one process blocked on in in favor of others—the Linda solution allows indefinite overtaking or livelock. A slow
philosopher could remain blocked on an in("room ticket") statement while a speedy one repeatedly outs a room ticket and then grabs it

```text
again, leaving the slow philosopher still blocked; likewise with respect to the in("chopstick", ...) operation. We need to assume, then, that
```
our Linda systems are "fair," which is a common assumption in dealing with non-deterministic programming environments. We must
assume that, if matching tuples are available, a process blocked on in or rd will eventually have a matching tuple delivered to it. It will *not* be indefinitely bypassed while other blocked processes are continued.**9.1.3The readers/writers problem**Suppose many processes share access to a complex data object (which is too large, we assume, to be conveniently stored in a single Linda
tuple). Processes are permitted direct access to the shared object, but only after they get permission to do so. The rules of access specify that

```text
many readers or a single writer may have access to the object, but not both; a constant stream of read-requests must furthermore not be
```
allowed to postpone satisfaction of a write request indefinitely, nor may a stream of write-requests indefinitely postpone reading.
The simplest and clearest way to solve this problem is to append each new read or write request to the end of a single queue . If the queue's

```text
head request is a read request, the requestor is permitted to proceed as soon as no writer is active; if the head request is a write request, it is
```
permitted to proceed as soon as neither readers nor a writer are active. When a reader or writer is given permission to proceed, its request is

```text
removed from the head of the queue; the requesting process reads or writes directly, and notifies the system when it is done.
```
Reader processes will execute

```text
startread(); *read*;
stopread();
```
and writers similarly. All readers and writers determine on their own when it is permissible to proceed. They do so by manipulating four
shared counters. When a process needs to read or write, it consults (and increments) the value of the rw-tail counter. Suppose the value of
this counter was *j*. The request process now waits until the value of the rw-head counter is *j*: when it is, this requestor is first on line and will
be the next process to be permitted access to the shared object. Before proceeding, though, it must wait for the value of the active-writers
counter to be 0—when and only when this counter is 0, no writers have access to the shared object. If the requesting process is *itself* a writer,
it must also wait for the active-readers counter to be 0. (Readers can share access with other readers, but writers share access with nobody.)
Finally, the requestor increments either active-writers or active-readers (depending on whether it intends to write or read), and increments
the rw-head counter to give the next waiting process a chance to await access at the head of the line.

```text
startread()—the routine that is executed as a preface to reading—is
startread()
```
{

```text
 rd("rw-head", incr("rw-tail"));
 rd("writers", 0); incr("active-readers");
 incr("rw-head");
```
}
incr is a routine that increments a shared counter stored in a tuple, ining the tuple, outing it after incrementing an integer count field, and
returning the former (that is, the unincremented) value of the integer counter:

```text
int incr(CounterName);
...
```
{

```text
 ...
 in(CounterName, ?value);
 out(CounterName, value + 1);
 return value;
}* Pop Quiz:*we've outlined the basics; complete the implementation of the readers-writers system.
```
### 9.2 The meeting maker
As our main example in this chapter we'll use an exercise rather than a real system. All the code we discuss can be built and executed (see
the exercises), but the result would be a demonstration or a prototype, not something that could be handed to users and put into service. A
genuinely usable version of the system we'll discuss would require some important facilities, having to do with persistent storage and
security, that are part of the Linda model conceptually but aren't supported by most current implementations.
Why study a prototype? First, this problem beautifully encapsulates exactly the range of issues that distinguish "coordinated" programs in
general from parallel applications specifically. Second, the prototype's basic routines and data structures—its software architecture—are
exactly what the real version would use. Studying the software architecture of the prototype exposes you to exactly the same logical issues
as a real system would raise. Third, the sort of application we discuss here is becoming increasingly important. Linda implementations will
soon provide the full complement of necessary features (some research implementations already address these issues). Other programming
languages—in particular, ones that are intended specifically for distributed systems and not for parallel applications—already support the
construction of this kind of system. In short, the topic is important, we can explore its logical ramifications in the context of a prototype, and
support for real implementations isn't far behind.
#### 9.2.1 The problem
Consider an organization in which each user has a workstation and all workstations are networked together—a standard, widespread
environment. Occasionally the people in this organization need to meet with each other. Everybody maintains a calendar listing the periods
of time during which he's generally willing to have meetings, as well as his current appointments.
We need to provide a package of routines that schedule meetings automatically*and *allow the participants in such meetings to discuss them
beforehand. In general terms, then, our software package will address two major issues. First, scheduling meetings is a nuisance, and it ought
to be automated if it can be. On the other hand, people generally don't like being handed edicts (especially by computers) to the effect that
they*will *present themselves at such-and-such a place and time. Automatic meeting-scheduling is a potentially significant convenience, but*only *if the participants have an opportunity to ask questions, make comments and bow out if need be. Of course, the conversation as well as
the scheduling should be software-supported.
A software-mediated conversation might give up a little something in charm and spontaneity to the old-fashioned variety (in which people
actually gather around and talk to each other). On the other hand, it should also abolish the spatial and temporal barriers that constrain
ordinary conversations. The participants in a software conversation don't need to share one place or one time. Users of electronic bulletin
boards are familiar with the advantages of such free-floating electronic exchanges. What we need for our appointment-maker is something
like an electronic bulletin board for every meeting—except that, in cases where all participants are logged on and joining in simultaneously,
each person's comments should be made known to the others immediately. The result should be a sort of "real time" bulletin board that can
function as a regular "archival" bulletin board as well.
To make the requirements precise, we'll trace through an example interaction with the system. We'll assume a multi-window display
environment.
Let's say that Robert wants to schedule a half-hour meeting with Clara and Felix between 9 and 11 tomorrow. He types his request in the try
to schedule window.

```text
The system consults Clara's and Felix's calendars; let's say they're both available at 9:30. All three calendars are updated accordingly. A new
```
message appears in the "calendar changes" window on all three displays, alerting Robert, Clara and Felix to the fact that they are scheduled
to meet tomorrow at 9:30. A "conversation stream" is created, and each participant is invited to type comments in the comments? window.

```text
Say Felix comments what about?; Robert replies the new manual. Felix then comments forget it, I'm not coming. But meet with Clara.
The schedule is changed accordingly. Clara, let's say, is not logged on; she hasn't participated in the conversation. Three hours later she
shows up. She notes the change to her calendar and reads the associated conversation; then she adds a comment of her own (OK, I'll be
```
there).
The "conversation stream" will continue to exist until the time of the meeting. When the participants are actively, simultaneously taking part,
each one's comments should be passed on to the rest immediately. But the conversation stream should also support communication between
participants whose contributions take place at separate times. The problem, in other words, has both "space-coordination" and
"time-coordination" aspects.
#### 9.2.2 The general form of a solution

```text
Each user's calendar is stored in a tuple. A calendar can be read, then, by means of a simple rd operation; to be updated, the old calendar
```
must be ined and the updated version outed.
A calendar tuple must include one field that identifies its owner, and a linear ordering must exist over these identification fields. If the

```text
identification fields are simply names (assuming everyone has a unique name), we can use alphabetical order; or they might be
```
integer-valued "employee identification numbers," or whatever. The ordering is important when some user needs to consult and update many
calendars in the course of scheduling a meeting. Calendars will always be ined in some fixed order, say lowest-to-highest, in order to
eliminate the possibility of deadlocks.

```text
(Why do we worry about deadlock? Suppose that Robert and Clara are simultaneously trying to schedule a meeting involving both of them.
Robert grabs his calendar tuple and Clara grabs hers; then Robert tries to in Clara's and Clara to grab Robert's. Both ins block, and we have
```
deadlock. Under an alphabetical ordering, on the other hand, Clara and Robert will both try to in Clara's calendar first. Whoever succeeds
will find Robert's calendar available—assuming that no-one else is also trying to schedule a meeting.)* Pop Quiz:*prove that ining tuples under some fixed ordering, as described above, in fact prevents deadlock. Consider the
conditions under which a cycle could arise in the resource graph that defines the problem. (For a discussion of resource
graphs and deadlock, see Tanenbaum [Tan87, p. 124].)
We'll use*daemon processes *to alert users that their calendars have been changed. A daemon process is, in general, a process that waits for
some condition to be true, takes some appropriate action in consequence, and repeats. (The awaited condition may be that*n *seconds have
passed since this daemon last woke up. Operating systems often use daemons of this sort—a daemon process may wake up every now and
then and flush the file buffers to disk, say, or recompute process priorities.) In our case, each user will have a daemon process that waits for
his calendar tuple to be changed. When a scheduling process changes a calendar, the calendar tuple is replaced with a "Just Changed" flag
raised in one of its fields. The appropriate daemon wakes up, removes the tuple, resets the "Just Changed" flag, replaces the tuple, and takes
some appropriate action (for example, it prints a message in the "Calendar Changes" window).
The conversation streams are merely multi-source, multi-sink read-streams of the kind discussed in chapter 3. Each tuple in the stream holds
a string representing a single comment. If a user of the system is logged on when his calendar changes, we can create a "listener" daemon for
him and attach it to the conversation stream that will be associated with the newly-scheduled meeting. The listener daemon rds (and prints
to the appropriate window) each newly-appended tuple in the conversation stream. When a user logs on much later, we can do exactly the
same thing—create a listener daemon to read and print each comment in a conversation that actually took place much earlier.
#### 9.2.3 Implementing the system
We'll discuss the basic structure of each of the main routines in the appointment-maker. In every case, the code we'll present follows
immediately from the description given.
First, consider the routine SetUpMeeting. It accepts as arguments a list of participants, a meeting-duration and the interval of time within

```text
which the meeting should take place. If it can schedule the meeting as requested, it goes ahead and does so; otherwise it returns failure, and
```
the user tries again. (Obviously we might want SetUpMeeting to return, in the event of failure, the*reason *a meeting couldn't be
scheduled—* e.g.*, "Doris isn't available until noon." This and many other features are easy to provide. Here, though, we're concerned only
with the basic coordination framework.)
A calendar tuple will take the following form:

```text
("calendar", id, theCalendar, modified)
id identifies the calendar's owner; theCalendar represents times-available and meetings scheduled (probably in the form of an array or a
struct); modified is set to TRUE whenever theCalendar is changed.
```
Suppose SetUpMeeting has been told to schedule a meeting whose participants are *A*,*B *and* C*. *A*,*B *and* C*might be names or some other

```text
identifiers; we'll assume that* A < B < C *in the identifier ordering. In outline, SetUpMeeting will execute
in("calendar", A, ?Calendars[A], ?modified);
in("calendar", B, ?Calendars[B], ?modified);
in("calendar", C, ?Calendars[C], ?modified);* attempt to set up the requested meeting;*if (* the attempt succeeds *) {* assign this meeting a number that
 is unique over all meetings pending throughout the system;
```
 alter each participant's calendar to reflect the

```text
 new meeting;*out("calendar", A, Calendars[A], TRUE);
 out("calendar", B, Calendars[B], TRUE);
 out("calendar", C, Calendars[C], TRUE);
} else* replace the calendar tuples without altering them;*When an attempt to schedule a meeting has succeeded, the meeting's initiator is responsible for setting up the associated conversation
```
stream. The SetUpConvStream routine merely executes

```text
out("conversation stream tail", MNUM, 1);
```
where MNUM is the new meeting's unique identifier.
Users want to be notified when their calendars have been changed. To accomplish this, they'll create a daemon that executes the routine
MonitorCalendar, in outline

```text
while (TRUE) {
 in("calendar", me, ?theNewCalendar, TRUE);
 out("calendar", me, theNewCalendar, FALSE);* compare theNewCalendar to the old calendar and
 figure out what's changed;
 print an appropriate notice to the Calendar Changes window;
```
 for every new meeting on the calendar, create

```text
 a Listener daemon;*}
The variable me is a formal parameter of the MonitorCalendar procedure; it identifies the user on whose behalf the daemon is working. To
```
create the daemon, we use

```text
eval("Monitor Calendar Daemon", MonitorCalendar(MyId));
```
Note that MonitorCalendar can't assume that only a *single* meeting has appeared on the calendar. When a calendar tuple with its last field
set to TRUE is dropped into tuple space, it's possible that another process intent on scheduling more meetings may grab it before
MonitorCalendar does. We might also be operating in a setting in which a user's daemons are destroyed when he logs off. In this case, a
newly fired-up MonitorCalendar may need to report a whole series of new meetings.
We turn now to the listener daemon. It executes a routine that scans a read-stream, checking each comment for a shut-down message.
We assume that the initiator of a meeting has the responsibility, once the scheduled hour has arrived, to append this "shut down" message to
the associated conversation stream. We won't discuss this aspect of the problem further here, except to note that the process of appending
shut-down messages can of course be automated—see the exercises. Appending a "shut down" to a comment stream has the effect of causing
any daemons scanning the stream to terminate. No further comments will be printed automatically to user displays. But it's important to note

```text
that the conversation stream *itself* is *not* destroyed. It continues to exist; it may be scanned at any time by any interested party. In other
```
words, conversations that precede meetings are part of the system's archives—they persist indefinitely in tuple space—until someone
decides to destroy them.
The listener daemon executes the following routine (in outline):

```text
void ListenTo(MtngNum); int MtngNum; { ... while (TRUE) { rd("conversation
stream", MtngNum, ++i, ?comment); if (* the comment is "shut down"*) return;
else* print the message in the appropriate window;*} } To create a daemon to
```
monitor a new conversation whose identifier is MNUM, each user executes

```text
eval("ListenerDaemon", ListenTo(MNUM)); Participants in the conversation must be
```
able to comment as well as to listen. The last of our basic routines allows
users to comment, by installing a new tuple at the end of the appropriate
conversation stream. The routine MakeAComment accepts the meeting number and the

```text
comment (in the form of a character string) as arguments: void
MakeAComment(MNUM, TheComment); int MNUM; char \* TheComment; { int index;
in("conversation stream tail", MNUM, ?index); out("conversation stream tail",
MNUM, index + 1); out("conversation stream", MNUM, index, TheComment); } Note
```
that, if many participants try to comment simultaneously, their comments are
appended to the stream in some arbitrary order determined by the sequence in
which they succeed in grabbing the "conversation stream tail" tuple.
#### 9.2.4
Characteristics of the solution: distribution, parallelism, concurrency and
time-coordination How can we characterize our solution in general? Why is this
example significant? First, we've designed a *distributed system* both in the
physical and in the logical sense. In the simple physical sense, a distributed
system runs in a host environment consisting of physically-dispersed machines.
We've assumed for this exercise that each user has his own workstation. (As we
noted in the first chapter, Linda has been implemented on local networks, and so
our contention that we've been designing software for a distributed environment
makes sense in practical terms.) A *logically* distributed system goes beyond
mere physical distribution: it allows the *logic* of the system, the decision
making, to be *shared* among participating machines, not centralized on one. In
the system we've described there is a function called SetUpMeeting, and any user
who wants to set up a meeting can execute this function *directly*, on his own
machine. (He needn't send a message to some particular machine charged with
scheduling everybody's meetings.) If eight different meetings are being
organized, then eight different machines must be executing their local copies of
the SetUpMeeting routine. Likewise, when a user needs to listen or add to a
conversation stream, he executes the necessary code on his own machine.
Logically-distributed systems are interesting for several reasons. They avoid
channeling activity through the potential bottleneck of a single machine. They
expand gracefully: adding more users to the system probably means that more
meetings will get scheduled, but does *not* necessarily mean that the load on

```text
any one machine will increase; each new machine on the network shares in the
```
execution of the meeting-maker system. And logically-distributed systems *may*
be more reliable—less prone to disruption when machines fail—than undistributed
systems. (In our case, though, the reliability of the system we described
depends on the reliability of the underlying Linda system. We can't make
assumptions about the *system's* (*i.e.*, *Linda's*) reliability on the basis of
a particular *application's* being logically distributed.) Logical distribution
is related to another important characteristic of our example, hinted at above:
it is a *parallel* program as well as a distributed one. It can make use of
concurrency not only to accommodate the fact of physical distribution, but to
improve performance. Suppose that, in a ten-user system, two users are trying to
schedule new meetings and the other eight are split into two groups of four,
each group conversing about an upcoming already-scheduled meeting. All ten
workstations can be active simultaneously. As a system gets larger, and it
becomes increasingly likely that many users will be planning meetings
simultaneously, it's fair to expect that parallelism will make an increasingly
significant contribution to good system performance (assuming, of course, that
the underlying Linda system is well implemented). Our meeting-maker is
*concurrent* as well, in the special sense discussed previously: using multiple
processes on a *single* processor in the interests of clean program structure.
When a user creates daemons, the daemon processes execute on his workstation,
sharing its processor among themselves and all the other processes that this
user may have started. When a single processor is shared among many
processes—for example, among a "monitor calendar" daemon, several "listener"
daemons and another process executing a command interpreter—we gain nothing in
terms of execution speed. Five processes running on a single machine give us no
better performance (in fact are guaranteed to give us slightly worse
performance) than a single process carrying out all five functions by itself.
But this kind of concurrency may be important, as it is here, because it
supports clean program structure. The daemon idea, and the concurrent-process
concept behind it, make it easy to design the "monitor calendar" and the
"listener" routines. These ideas lead, too, to a system that naturally and
dynamically adapts to a complex environment. In the system we designed, a single
user might be involved in three different conversations simultaneously. No
change or addition to the code is necessary to support this possibility—it
happens automatically when the need arises. Finally, the system we've designed
is *time-coordinated* as well as space-coordinated, as we've discussed above.
*n* users may participate in planning some meeting—one initiating it and all *n*
entering into the associated conversation—despite the fact that no more than one
of them is ever logged on at any given time. In short, this example exercises
virtually the entire armamentarium of multi-process programming. It is
significantly more general in its demands than the parallelism examples that

```text
most of the book centers on; and it is a good representative of the "coordinated
```
software"*genre* that, we've argued, can only grow steadily in significance.
*Massive parallelism:* Large organizations may own thousands or tens of
thousands of computers, usually interconnected in some way. Electronic mail and
file transfer services are generally available over large, far-flung networks,
but highly-integrated services like the meeting maker usually are not. They
certainly *will* be in the future, though. A ten-thousand person organization is

```text
*itself* a "massively parallel" system; if we give each worker a computer, and
```
connect the computers together, massive parallelism inevitably follows. The
meeting maker is in this sense one paradigm for an important class of massively
parallel systems. If the underlying implementation is sound, it should be
capable of scaling to enormous sizes. We close by underlining those aspects of
the exercise that require systems support beyond what most current Linda systems
can supply.*Persistent tuple spaces* are this exercise's most crucial
requirement. The fact that tuples persist indefinitely is part of Linda's
semantics, but in most current systems a tuple space is destroyed when all the
processes it contains have terminated. *Security* is vital: we can't allow
unauthorized tampering with the tuples that define this application (can't
allow, for example, one user to make arbitrary changes to another's calendar).
This application must run in *system *or* privileged* mode, and make use of a
*privileged* tuple space—one that is accessible only to system-level routines of
the sort we've described, not to arbitrary user-level code. Most current Linda
systems don't support privileged modes. Clearly *reliability* is another
important issue: tuple space must be implemented in such a way that important
tuples are unlikely to get lost even when machines fail. Persistent, secure and
reliable implementations are fully consistent with Linda's semantics—to the
programmer, in other words, they'll look essentially the same as any current
Linda system looks—but they require implementation-level support. The necessary
support is a major topic of research in our own group and in others.
### 9.3
Exercises 1. Implement the meeting-maker. (The main routines are outlined above,
but the details and the overall framework are, of course, missing.) If you're
using a simulator running on a single processor, test the system by creating
several concurrent "users" on your single processor—three is a reasonable
number. We will name these users "1", "2", and "3" (after which, having strained
our creative faculties to the limit, we will retire to the sofa for a short
nap). To avoid dealing with the creation and maintenance of lots of windows,
implement one input and one output process. The input process accepts
single-line commands like 1: schedule on Tues, betw 9 11, duration 0.5, with 3
and 3: comment meeting 3401, "what for?" The prefixes (1: or 3:) identify which
user is supposed to be doing the typing—*i.e.*, user 1 asks that a meeting be

```text
scheduled with user 3; user 3 comments on the meeting. (The input syntax doesn't
have to be exactly like this; you can use anything that does the job.) The input
```
process interprets these commands and invokes the appropriate routines. The
output process repeatedly withdraws tuples from the head of an in-stream and
prints their contents. SetUpMeeting, "monitor calendar" daemons and "listener"
daemons all send comments (by stuffing them in tuples and appending them to the
in-stream) to the output process for printing. Thus, the output process prints
comments that look like \*new meeting (id 3401) Tues 9 - 9:30 with 3\* or
\*comment re 3401: "what for?" -- 3\* These output lines may wind up being

```text
interspersed with input lines; that's alright. (Those are the breaks when you're
```
dealing with this kind of asynchronous system.) Note that you'll have to make
provision for initializing the system and creating each user's calendar in some
starting state.

2. (*a*) Design and implement the following extensions to the meeting maker:

```text
(*i*) *Best-effort scheduling*. If a requested meeting proves to be impossible to schedule, the system either looks for some other
```
time not far from the requested interval during which it *can* be scheduled, or it schedules a meeting during the requested
interval that omits some of (but not "too many" of) the specified attendees. It reports this best-effort result to the requestor,
and asks whether the modified meeting should be ratified or cancelled.

```text
(*ii*) *Calendar extension.* Toward the end of each month, the system automatically generates a next-month calendar tuple for
```
each user, initialized to the same schedule (with regard to acceptable meeting hours) as the previous month's. The user must
approve the new calendar tuple before it's installed. (Clearly, you now need a process that knows the current date.)

```text
(*iii*) *Regular meetings with query.* The user may request that a certain meeting be scheduled not once but daily, weekly,
```
monthly or whatever. After each occurrence of a regular meeting, the system asks the original scheduler whether the next
meeting should in fact go ahead as planned.

```text
(*iv*) *Priority meetings.* If a priority meeting is requested, the system attempts to override any existing appointments. First, it
```
checks what conflicts will arise if the priority meeting is scheduled as planned. Next, it attempts to reschedule any
preempted meetings, placing them as close as possible to their originally scheduled times. Then it reports the whole

```text
(tentative) arrangement to all parties to the priority meeting; if all agree, the meeting is scheduled; otherwise, the attempt
```
fails.

```text
(*v*) *Room scheduling.* Each user's calendar tuple notes how many people can be accommodated in his office. A "rooms
```
tuple" holds a schedule for every public meeting room. When a meeting has been scheduled that is too big for the
originator's office, the system picks a "convenient" public room, modifies the rooms tuple and reports the location as well as
the time to all participants.

```text
 (*b*) Think up five more nice extensions. Implement them.
 (*c*) Is your system starting to sound like something that people might want to pay money for? What are you waiting for?
```
3. Chandy and Misra [CM88] present the following problem:
The problem is to find the earliest meeting time acceptable to every member of a group of people. Time is integer valued
and nonnegative. To keep notation simple, assume that the group consists of three people, *F*, *G,*and *H*. Associated with
persons *F*, *G,*and *H* are functions *f, g, h*(respectively), which map times to times. The meaning of *f* is as follows (and the

```text
meanings of *g, h*follow by analogy). For any *t*, *f *(*t*)*≥ t*; person *F* can meet at time *f *(* t*) and cannot meet at any time *u *where* t
```
≤ u < f *(* t*). Thus *f *(* t*) is the earliest time at or after *t* at which person *F* can meet. [CM88, p.12].
Chandy and Misra discuss several ways of solving the earliest meeting time problem given functions *f, g*and *h*. In one approach, a proposed

```text
meeting time (initially 0) is passed in a circuit from one participant to the next; when *F* receives it, if the proposed time is *t*, *F* resets the
```
proposed time to *f *(* t*) (and then hands the proposal on to *G*, who does likewise, and so on). When the proposal makes a complete circuit
without changing, we're done. In another approach (somewhat modified from a strategy [CM88] describes), each participant proposes a

```text
meeting time; then each participant looks at the other proposals, and modifies his own proposal accordingly; the cycle repeats until all
```
proposals are the same.
Implement both approaches. (A good strategy for the second involves one single-source multi-sink read stream for each participant.)
4. **Operating systems**are sometimes structured as coordinated process
   ensembles. Write a simulated multi-process terminal input system
consisting of four concurrent processes plus a fifth for testing purposes. The processes and their functions are

```text
(*a*) tty: simulate a keyboard by generating a sequence of "input characters". The sequence is stored in tuple space as a stream. Characters can
```
be alphanumeric, or three special characters: BACKSPACE, LINEKILL or NEWLINE. You can represent these characters in any way you

```text
want (since your own program will be interpreting whatever character code you use). To simulate a user's typing ok<RETURN>, your
```
process generates

```text
 ("tty", 1, <o>, ... ) /\* you may need other fields \*/
 ("tty", 2, <k>, ... )
 ("tty", 3, <NEWLINE>, ... )
```
where <char> is your code for char. The tty process can generate arbitrary character streams for testing purposes.
We'll refer to the stream of characters generated by this process as the "tty stream". (Two processes, "echo" and "line-editor", will use the tty
stream as input).

```text
(*b*) echo: inspect each character in the tty stream. If it's an alphanumeric, print it. If it's a BACKSPACE, print a backspace. If it's a
```
LINEKILL, print *n* backspaces, where *n* is the number of characters printed and not backspaced-over since the last NEWLINE. If it's a
NEWLINE, print a newline.

```text
(*c*) line-editor: inspect each character in the tty stream. If it's an alphanumeric, add it to the end of the current-line buffer. If it's a
```

```text
BACKSPACE, erase the most recently-added character; if it's a LINEKILL, erase
```
all characters in the buffer. If it's a NEWLINE, remove the characters from the
line buffer one-by-one and put them into an "edited" stream, which is
structurally the same as the tty stream. For each new character added to the
stream, increment a global "CharsAvail" counter, which will be stored in its own
tuple. (*d*) supply-chars: wait for get-character requests, attempt to meet the
requests and return the results, by repeatedly executing the following outline:

```text
in("get", ?id, wanted); ... out("got", id, char-string, count); A client process
```
that wants 5 characters will generate a ("get", id, 5) request-tuple.
supply-chars returns the requested number of characters by removing them from

```text
the "edited" stream, updating the CharsAvail tuple as appropriate; it returns
```
the requested characters in a character string, together with a count of how
many there are (* i.e.*, how long the string is). If the client asks for 5 and
there are only 3 available, return all 3 together with a "count" of 3. (*e*) The
test-process should use a procedure (not a process) called readchar. readchar()

```text
accepts as arguments a character count and the address of a buffer; it deposits
```
characters in the specified buffer, and returns a character count. If I execute

```text
count = readchar(5, &buff); when three characters are available, the three
```
characters are placed in buff and 3 is returned. Assume that many different
processes may execute readchar concurrently (meaning that many different client
processes all want to read input from the same stream). ° 5. (*a*) In chapter 3,
you built a distributed data structure that realized an abstract market.
Beginning with the market structure, implement a "stock ticker" process that
prints out the price of each commodity after each transaction. You can implement
an "approximate ticker" that uses a changed flag of the sort discussed above

```text
(why is such a ticker "approximate"?), or you can implement an "exact ticker"
```
that is guaranteed to report each transaction. Test your system by creating a
collection of concurrent "trader" processes, each of which carries out a series
of transactions. (*b*) Build a restricted version of your market: only a single
commodity is traded (the ticker, again, reports its current price). Create a
collection of trader processes. Each one repeatedly generates a random number

```text
between 0 and 1. If the generated number is <*k* 1, the trader buys; if it's
```
>*k* 2, the trader sells (0 ≤*k* 1 ≤*k* 2 ≤ 1). Store the values of *k* 1 and
*k* 2 in a tuple, where they can be changed as the program runs. Does your
market respond to relative increases in supply (sellers) and demand (buyers) as
it ought to? (*c*) Build a new version of the market in which, as price rises,

```text
traders display an increased propensity to sell and not to buy; as it falls, the
```
tendency to buy increases, and to sell decreases. Can you build a
"self-stabilizing system," in which heightened demand or supply creates only a
short-term change in prices? (*d*) Finally, "customize" each trader. After each
buy-or-sell decision, traders look at the ticker. If the price has risen by more
than *U* percent over the last *t* trades, or fallen by more than *D* percent
over the last *t* trades, a trader's behavior changes. In the former case he
enters *speculate* mode, in the latter case *panic* mode. In *speculate* mode,
every rise in price *increases* a trader's propensity to buy. In *panic* mode,
every *fall* in price increases a trader's tendency to *sell*. In "normal" mode,
each trader's behavior is "self-stabilizing," as in part (*c*). The behavior
thresholds *U* and *D* are *different* for each trader. If a "boom" is a
consistent, self-sustaining rise in prices and a "panic" is a corresponding
fall, can you build a system in which random price fluctuations can

```text
(occasionally) induce booms or panics? ° 6. The previous exercise centered on an
```
"automatic" market—prices are determined by the number of buyers or sellers.
Implement a system modeled on the meeting maker that supports a more realistic
market, in which prices are set by negotiation among buyers and sellers. As in
the original chapter 3 exercise, your system implements the routines buy() and

```text
sell(). Now, however, the first seller or buyer of some commodity creates a
conversation stream (as discussed above); the stream exists for as long as there
```
are outstanding requests to sell or buy that commodity. Executing buy(A, 100)
causes the message "process Q offers $100" to be added to the relevant
conversation stream. (Q's identity is supplied automatically by his
"stockbroker" process.) sell works in a similar way. Traders must be able to add
new offers and bids to an existing stream. The new system should have a stock
ticker, as in the previous problem. (A more interesting ticker service would
report volume as well as price.) ° (* Turingware:*) 7. Consider an "expert
network" that represents a kind of generalization of the trellis architecture
discussed in chapter 8. An expert has ≥ 1* input streams *and a unique* output
stream *. An expert is notified whenever a value appears on any one of its input
streams. The expert may, in response, add a new value to its output stream

```text
(which may in turn cause other experts to be notified). (For example: if the
```
weather expert attaches a snowstorm prediction to its output stream, or the
mayor's office predicts a presidential visit, the highway department expert
attaches a mobilization plan to its output stream, which may trigger, in turn,
the costs-projection expert.) In many cases, these expert networks might be
naturally trellis-shaped. But unlike trellises, they may have cycles (and hence,

```text
among other things, can't support realtime performance projections); and their
```
data streams are explicitly *archival*. They may be *read* by any number of

```text
processes, but they are never *consumed*; they exist forever, creating a
```
permanent record of the system's career.

Design and implement this kind of system for some arbitrary domain, in such a
way that each expert can be *either* a process or a person. Persons should
interface to the system in the same way processes do. Specifically: they should
be notified (by a message printed on a terminal) whenever a new value is

```text
attached to one of their input streams; subsequently, they must be able to
invoke (directly from a command interpreter) a routine that attaches some
```
newly-computed value to their* own* output streams—whereupon other processes (or
people) that depend on these output streams must be notified in turn. (Note
that, to read several streams, you'll need to create one stream-reader process
for each. Each one of these processes reads a single stream, copying each
newly-arriving element onto an input stream that is scanned directly by a
decision process, or by a daemon representing a human decider. If this expedient
distresses you, what simple extension to Linda would solve the problem?) This
sort of system, in which persons and processes come together in a single
coordinated ensemble held together by distributed data structures, is a
harbinger of the future, a fitting conclusion to the chapter and this book, and
an introductory glance at a broad and fascinating new topic in computer science.
