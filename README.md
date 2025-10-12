

# CITREELO

This is a basic [ROBDD](https://en.wikipedia.org/wiki/Binary_decision_diagram)-based [symbolic model checker](https://en.wikipedia.org/wiki/Model_checking#Symbolic_model_checking) for [Computational Tree Logic](https://en.wikipedia.org/wiki/Computation_tree_logic).

This work is mainly an interpretation of [the presentation of CTL symbolic model checking from the course of Roberto Sebastiani](http://disi.unitn.it/rseba/DIDATTICA/fm2024/SLIDES/06-SymbolicMc_handouts.pdf).

I wrote this out of curiosity and it has not been extensively tested (besides the tests in "/src/test/").

I use [biodivine-lib-bdd](https://github.com/sybila/biodivine-lib-bdd) as as a backend for the [ROBDDs](https://en.wikipedia.org/wiki/Binary_decision_diagram).

The supported [CTL](https://en.wikipedia.org/wiki/Computation_tree_logic) operators are:
- &, |, !, =>, <=>, AX, EX, AF, EF, AG, EG, AU, EU

To compute BDDs representing sets of states satisfying CTL formulae, all these operators directly correspond to operations on BDDs i.e., we do not use translation using a minimal set of operators e.g. "AX p -> !EX(!p)".


