# Preface
This book is the raw material for a hands-on, "workshop" type course for
undergraduates or graduate students in parallel programming. It can also serves

```text
as the core of a more conventional course; and it might profitably be read (we
```
hope and believe) by any professional or researcher who needs an up-to-date
synthesis of this fast-growing, fast-changing and fast-maturing field. By a
"workshop" course we mean a course in which student projects play a major part.

```text
The exercise sections at the end of each chapter are integral to the text;
```
everyone who consults this book should (at least) read them. Problems in

```text
chapters 2 through 5 lay the groundwork; the exercise sections in the last four
```
chapters each center on a detailed investigation of a real and substantial
problem. For students who pursue them seriously, these programming problems will
require time, care and creativity. In most cases they lack stereotyped
solutions. Discussion of student efforts can and ought to play a significant
part in class meetings. The programming examples and exercises use C-Linda

```text
(Linda is a registered trademark of Scientific Computing Associates.); C-Linda
```
running on a parallel machine or a network is the ideal lab environment for the
workshop course we've described. A C-Linda simulator running on a standard
workstation is an adequate environment. Relying on some other parallel language
or programming system is perfectly okay as well. The called-for translations
between the book and the lab environment might be slightly painful (particularly
if the non-Linda parallel language you choose is any variant of the ubiquitous
message-passing or remote-procedure-call models), but these translation
exercises are always illuminating, and anyway, they build character. The "more
conventional" course we mentioned would deal with parallel systems in general.
Parallel *software* is still the heart and soul of such a course, but teachers
should add some material to what is covered in this book. We would arrange the
syllabus for such a course as follows:

```text
1. *Introduction and basic paradigms.* (The first two chapters of this text.)
```
2. *Machine architectures for parallelism*. We'd use chapter 21 of Ward and
   Halstead's *Computation Structures* [WH90], or part 3 of
Almasi and Gottlieb's *Highly Parallel Computing* [AG89].

```text
3. *Linda, and parallel programming basics*. (Chapter 3.)
```

```text
4. *Parallel languages and systems other than Linda; two special-purpose models
```
   of parallelism: data-parallelism and systolic systems.*
The most comprehensive and up-to-date overview of parallel languages and systems
is Bal, Steiner and Tanenbaum's survey paper on "Programming languages for
distributed computing systems" [BST89]. (A brief survey appears in section 2.6
of this book.) Hillis and Steele's paper on "Data parallel algorithms" [HS86] is

```text
a good introduction to data parallelism; Kung and Leiserson's paper on "Systolic
arrays (for VLSI)" [KL79] is the classic presentation of systolic programming.
```
This section ought to make a point of asking (among other questions) how Linda
differs from a horde of competitors. The Bal *et al*. paper discusses Linda in
context, as does Ben-Ari's *Principles of Concurrent and Distributed
Programming* [BA90] (which is mainly about Ada but has good discussions of occam
and Linda as well), the parallel languages chapter of Gelernter and
Jagannathan's *Programming Linguistics*[GJ90], and the authors' paper on "Linda
in context" [CG89].
5. *Writing parallel programs*: the rest of the text.
