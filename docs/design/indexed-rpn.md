# Indexed Reverse Polish Notation, an Alternative to AST

*Source: https://burakemir.ch/post/indexed-rpn/*
*Date: 2025-12-12*

"Why study compiler construction? Because knowing how a programming language is specified
and implemented makes you a better programmer." I still remember these words,
pronounced as matter-of-fact introduction to an undergrad course of compilers.

Compiler engineers have come up with many useful programming techniques and representations.

Today, I want to write about one such technique, an alternative to Abstract Syntax Trees (ASTs).
Inspired by the parse tree representation in the [Carbon compiler](https://github.com/carbon-language/carbon-lang),
this post explains a way to represent parsed source code using a variation of Reverse Polish Notation (RPN),
in a contiguous array.

We call this **Indexed RPN**. Ordering program parts in a linear sequence very naturally leads to
machine interpretation, which is well-known for calculators but maybe a little less well-known
when there are scoped definitions and control flow structures.

This is by no means a new way of doing things, but with modern machines having plenty of memory,
there may have been less pressure to reach for techniques that memory-friendly.

## 1. From Arithmetic to Indices

Let's start with an arithmetic expression example: We want to represent (3 + 4) * 5.

In a standard AST, this is a tree of pointers. In a standard stack-machine RPN, this looks like 3 4 + 5 *.
Now we want something slightly different, because we want to operate on tree structure.
For example, if we deal with this expression in a compiler that translates and optimizes.
We want to be able to refer to specific sub-expressions later.

### The "Administrative Normal Form" Perspective

Before we look at the memory layout, let's imagine breaking up this expression by *naming* subexpression.
If we had local definitions in our language this would give us Administrative Normal Form (ANF).
In ANF, we give a name to every intermediate result:

```
// Source: (3 + 4) * 5

val t0 = 3
val t1 = 4
val t2 = add(t0, t1)
val t3 = 5
val t4 = mul(t2, t3)
```

The expression now takes a lot more to write, but for a compiler it is much more structured.
The arguments of the operations are always names, which makes the data flow and also the
order in which arguments get evaluated fully explicit. Here, `t2` depends entirely on `t0` and `t1`,
and `t0` is evaluated before `t1`.

### The In-Memory Representation

We don't want local definitions just yet, the above is just to motivate flattening of a tree structure.
If we store the instructions in a contiguous array (e.g., a vector or Vec), the **index** of the node becomes its name.

The internal names are merely indices into the sequence of nodes.

| Index (ID) | Node Kind | Operands / Data |
| --- | --- | --- |
| **0** | `IntLiteral` | `3` |
| **1** | `IntLiteral` | `4` |
| **2** | `BinaryOp` | `Add(lhs: 0, rhs: 1)` |
| **3** | `IntLiteral` | `5` |
| **4** | `BinaryOp` | `Mul(lhs: 2, rhs: 3)` |

This is similar to Reverse Polish Notation (RPN), but there is a difference. In standard RPN, there is an implicit stack from which `Add` consumes items blindly. In **Indexed RPN**, `Add` explicitly refers to indices 0 and 1. This provides a stable reference to every sub-expression, allowing us to traverse the code and locate nodes without necessarily having to build up a stack.

## 2. Dealing with "Let" and Scope

Let us make the language more realistic by adding local let-declarations and scoping.

```
// Source
let a = 10;
let b = a + 5;
```

Here we face a problem: Source variables (a, b) are different from our internal indices (0, 1, 2...). We need a node that bridges this gap — an **"Introducer"**.

### The Bind Node

We introduce a Bind node. This node represents the action of bringing a name into existence in the current scope.
Depending on the language you are working with, a binding may have semantic significance. For example, if references to the binding are objects of the language like Rust or C++ references.

| Index | Node Kind | Operands | Meaning |
| --- | --- | --- | --- |
| **0** | `IntLiteral` | `10` | The raw value 10. |
| **1** | `Bind` | `name: "a", val: 0` | **Introducer**: "a" exists, bound to Index 0. |
| **2** | `NameRef` | `ref: 1` | A reference back to the introducer node. |
| **3** | `IntLiteral` | `5` | The raw value 5. |
| **4** | `BinaryOp` | `Add(2, 3)` | Adds the NameRef and the Literal. |
| **5** | `Bind` | `name: "b", val: 4` | **Introducer**: "b" exists, bound to Index 4. |

### The Stack Returns (For Compilation)

In order to deal with this data, we traverse it but we will also want to build up a stack. You might ask: *If we flattened the tree, why do we need a stack?*

While the **storage** is flat, the **compilation process** requires a stack to handle scopes. Because let declarations can be nested, we cannot simply scan linearly and remember everything forever. We need to handle when names go *out* of scope (shadowing).

Let's add `BlockStart` and `BlockEnd` nodes that indicate nested blocks.

```
// Source Code
let x = 10;       // Outer 'x'
{
    let x = 20;   // Inner 'x' (shadows outer)
    print(x);     // Should print 20
}
print(x);         // Should print 10
```

### The resolve_names Algorithm

I am too lazy for full code examples, just the idea.

We use a SymbolTableStack during the resolution pass.
We iterate through the array once. We maintain a stack of scopes, where each scope maps a string name to an integer index.

```
function resolve_names(nodes):
    # A stack of scopes. Each scope is a Map: String -> Index
    scope_stack = [ new Map() ]

    for i, node in enumerate(nodes):

        match node.kind:
            case BlockStart:
                # Push a new, empty scope onto the stack
                scope_stack.push( new Map() )

            case BlockEnd:
                # Pop the top scope. Inner variables are forgotten.
                scope_stack.pop()

            case Bind(name, value_index):
                # Register the variable in the CURRENT (top) scope.
                current_scope = scope_stack.top()
                current_scope.set(name, i)

            case NameRef(name):
                # Look for the name, starting from the top scope down.
                target_index = find_in_stack(scope_stack, name)

                # PATCH THE NODE:
                # The node no longer holds "x". It holds the index (e.g., 4).
                node.resolved_index = target_index
```

After this pass, the stack is discarded. The IR is now "wired." Every variable usage points directly to the instruction that created it.

When representing source as AST, we would use an algebraic data type. One could use mutable data structures there, or build up a symbol table.

## 3. Breaking the Line: Control Flow

So far, execution has been linear: Index 0, then 1, then 2. But branching constructs like if, else, and while break this line.

In a tree-based AST, an If node has children pointers to "Then" and "Else" blocks. In our flat array, we may prefer to have in the same contiguous vector, instead of blocks
floating in separate memory. So we introduce **Jump** nodes.

### The Linear Layout

Consider this source:

```
if (a) { print(1); } else { print(2); }
print(3);
```

Here is the Indexed RPN layout. Note the use of `BrFalse` (Branch if False) and `Jmp` (Unconditional Jump).

| Index | Node Kind | Data | Explanation |
| --- | --- | --- | --- |
| **0** | `NameRef` | `"a"` | Load variable `a`. |
| **1** | `BrFalse` | `target: 5` | If `a` is false, jump to Index 5 (Else). |
| **2** | `Int` | `1` | Start of "Then" block. |
| **3** | `Print` | `2` |  |
| **4** | `Jmp` | `target: 7` | Jump over the "Else" block. |
| **5** | `Int` | `2` | Start of "Else" block (Target of node 1). |
| **6** | `Print` | `5` |  |
| **7** | `Int` | `3` | **Merge Point.** Execution continues here. |
| **8** | `Print` | `7` |  |

### Building It: Backpatching

When we emit the BrFalse instruction at index 1, we haven't written the Else block yet, so we don't know the target index.

It is quite straightforward to deal with that:

1. Emit BrFalse with a placeholder target. Save the index.
2. Emit the "Then" block.
3. Emit Jmp with a placeholder target. Save the index.
4. Mark the current index as the start of "Else". **Backpatch** (update) the BrFalse at index 1.
5. Emit the "Else" block.
6. Mark the current index as the end. **Backpatch** the Jmp at index 4.

This effectively flattens the logic of the program into a shape that mirrors how hardware executes instructions: predictable, linear memory access with explicit jumps.

## 4. Towards Interpretation and Code Gen

We have successfully flattened our source code. We have resolved variable names into absolute indices and lowered high-level control flow into jumps. Now comes the reward.

### The Interpreter: The "Big Switch"

Because our code is a flat array, we can come up with a Virtual Machine (VM) that looks exactly like a hardware CPU: it has an Instruction Pointer (ip) and a big loop.
As a reminder — I will never get tired of repeating this — the difference between a virtual machine and abstract machine is that a virtual machine
has *instructions*, whereas an abstract machine has *transitions*.

A translation to a low-level format and a virtual machine plays the role of an interpreter, which provides an implementation of our language. We *can* also use it to
specify the *operational semantics*, roughly: however you implement this language,
it should produce the same result as the reference interpreter. For "real" languages, often specification comes as an afterthought but there
are plenty of situations where one really would like to know how a piece of source code is supposed to behave. For example, to find out
if the "real" implementation is correct. Somehow, educated people who really should know better can parrot statements like "undefined behavior is all about compiler optimizations"
and completely ignore that "undefined behavior" is first and foremost a gap in the specification.

Back to our interpreter: we can do something really simple: since we used ANF, (where every node index represents a runtime value), we don't even need a runtime stack for intermediate calculations. We can simply map the nodes array to a parallel values array. A real implementation would not do this, but if we use an interpreter solely to specify behavior, this is sufficient, and we can defer optimizations.
Note that since we have already resolved names to indices, instructions like `Bind` or `BlockStart` are effectively metadata. The interpreter can simply skip them.

```
function run_vm(nodes):
    # Holds the runtime result of every node.
    values = new Array(size=len(nodes))
    ip = 0

    while ip < len(nodes):
        node = nodes[ip]

        match node.kind:
            case IntLiteral:
                values[ip] = node.raw_value
                ip += 1

            case Add:
                # Direct access by index! No stack popping needed.
                lhs_val = values[node.lhs_index]
                rhs_val = values[node.rhs_index]
                values[ip] = lhs_val + rhs_val
                ip += 1

            case BrFalse:
                if values[node.cond_index] == False:
                    ip = node.target_index # JUMP
                else:
                    ip += 1

            case Jmp:
                ip = node.target_index # Unconditional JUMP

            case _:
                 # Skip metadata nodes (Bind, BlockStart, etc.)
                 ip += 1
```

### The Code Generator

We could also do a source-to-source translation and generate C code. The Indexed RPN shines again, because the complexity of the source language is reduced quite a bit.
Since instructions are topologically sorted and dependencies are explicit, generating C can be as simple as a single for loop where every node becomes a temporary variable t{i}.

This is maybe not a great way to specify what a language means, but a clear implementation advantage of translating to an existing language is that one can build
on top of an existing whole compiler, with optimizations, native code generation backends. How exactly the semantics and runtime aspects of the source language
and the target language are connected is of course a design choice and can be wildly different.

```
function generate_c_code(nodes):
    output = StringBuilder()
    output.append("int main() {\n")

    for i, node in enumerate(nodes):
        # 1. Create a label for every instruction so Jumps can find it
        #    e.g., "L_0:", "L_1:", etc.
        output.append(f"L_{i}: ;\n")

        # 2. Create a variable name for this node's result
        #    e.g., "t_0", "t_1"
        var_name = f"t_{i}"

        match node.kind:
            case IntLiteral:
                # int t_0 = 10;
                output.append(f"    int {var_name} = {node.value};\n")

            case Add:
                # int t_2 = t_0 + t_1;
                lhs = f"t_{node.lhs_index}"
                rhs = f"t_{node.rhs_index}"
                output.append(f"    int {var_name} = {lhs} + {rhs};\n")

            case Print:
                # printf("%d\n", t_5);
                arg = f"t_{node.arg_index}"
                output.append(f"    printf(\"%d\\n\", {arg});\n")

            case BrFalse:
                # if (!t_1) goto L_5;
                cond = f"t_{node.cond_index}"
                target = f"L_{node.target_index}"
                output.append(f"    if (!{cond}) goto {target};\n")

            case Jmp:
                # goto L_7;
                target = f"L_{node.target_index}"
                output.append(f"    goto {target};\n")

    output.append("    return 0;\n}")
    return output.toString()
```

### Conclusion

People will always build more languages, especially domain-specific ones.
A realistic work-in-progress language that uses indexed RPN is [Carbon](http://github.com/carbon-language/carbon-lang/).

By moving from a tree to an **Indexed RPN**, we replace heap allocations with a single contiguous vector. What was recursive tree-walking of AST can in many cases become index lookups.
So there should be a lot less memory-traffic, and when programs get large, memory traffic can have a significant impact on performance.

If you are like me and build toy programming language implementations for fun, consider trying this out and see how it works for you!
