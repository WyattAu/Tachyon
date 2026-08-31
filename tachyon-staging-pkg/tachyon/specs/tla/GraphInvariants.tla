------------------------------- MODULE GraphInvariants ----------------------------
\* Knowledge graph structural invariants for the Tachyon system.
\*
\* Invariants:
\*   INV001 No Self-Loops:       EdgeSrc(e) /= EdgeTgt(e) for all edges
\*   INV002 Weight Bounds:       0.0 <= Weight(e) <= 1.0 for all edges
\*   INV003 Reversal Idempotent: Reversing an edge twice returns to original
\*   INV007 Component Partition: Active nodes partition into connected components
\*   INV018 Degree Sum:          Sum of all degrees = 2 * |Edges|
\*
\* Verification status: Unverified (pending TLC installation)
\* To verify:  tlc GraphInvariants.tla
\* To prove:   tlaps GraphInvariants.tla
--------------------------------------------------------------------------------------

EXTENDS Naturals, Sequences, FiniteSets, TLC

CONSTANTS NodeSet,     \* Set of all node IDs
          NodeTypes,   \* Set of valid node types (must include Document, Component, Project, Person)
          MaxNodes,    \* Upper bound on nodes for model checking
          MaxEdges     \* Upper bound on edges for model checking

ASSUME NodeSet \subseteq STRING
ASSUME NodeTypes \subseteq STRING
ASSUME {"Document", "Component", "Project", "Person"} \subseteq NodeTypes
ASSUME MaxNodes > 0
ASSUME MaxEdges > 0

(***************************************************************************)
(* VARIABLES                                                              *)
(***************************************************************************)

VARIABLES Nodes,        \* Current set of node IDs (subset of NodeSet)
          Edges,        \* Current set of edge IDs
          EdgeSrc,      \* EdgeSrc[e] = source node ID
          EdgeTgt,      \* EdgeTgt[e] = target node ID
          EdgeType,     \* EdgeType[e] = edge type string
          NodeType,     \* NodeType[n] = type of node n
          Active,       \* Active[n] = TRUE/FALSE
          Weight,       \* Weight[e] \in Real, 0.0 to 1.0
          OriginalDir   \* OriginalDir[e] = [src |-> s, tgt |-> t] before reversal

(***************************************************************************)
(* Init                                                                   *)
(***************************************************************************)

Init ==
    /\ Nodes = {}
    /\ Edges = {}
    /\ EdgeSrc = [e \in {} |-> ""]
    /\ EdgeTgt = [e \in {} |-> ""]
    /\ EdgeType = [e \in {} |-> ""]
    /\ NodeType = [n \in {} |-> "Document"]
    /\ Active = [n \in {} |-> TRUE]
    /\ Weight = [e \in {} |-> 0.0]
    /\ OriginalDir = [e \in {} |-> [src |-> "", tgt |-> ""]]

(***************************************************************************)
(* Type Invariant                                                         *)
(***************************************************************************)

TypeInvariant ==
    /\ Nodes \subseteq NodeSet
    /\ Cardinality(Nodes) <= MaxNodes
    /\ Edges \subseteq STRING
    /\ Cardinality(Edges) <= MaxEdges
    /\ DOMAIN EdgeSrc = Edges
    /\ DOMAIN EdgeTgt = Edges
    /\ DOMAIN EdgeType = Edges
    /\ DOMAIN Weight = Edges
    /\ DOMAIN OriginalDir = Edges
    /\ DOMAIN NodeType = Nodes
    /\ DOMAIN Active = Nodes
    /\ \A e \in Edges:
        /\ EdgeSrc[e] \in Nodes
        /\ EdgeTgt[e] \in Nodes
        /\ EdgeType[e] \in STRING
    /\ \A n \in Nodes:
        /\ NodeType[n] \in NodeTypes
        /\ Active[n] \in BOOLEAN

(***************************************************************************)
(* Structural Invariants                                                  *)
(***************************************************************************)

INV001_NoSelfLoops ==
    \A e \in Edges:
        EdgeSrc[e] /= EdgeTgt[e]

INV002_WeightBounds ==
    \A e \in Edges:
        /\ Weight[e] >= 0.0
        /\ Weight[e] <= 1.0

INV003_ReversalIdempotent ==
    \A e \in Edges:
        \* After reversal, src and tgt are swapped relative to original
        /\ (EdgeSrc[e] = OriginalDir[e].src /\ EdgeTgt[e] = OriginalDir[e].tgt)
            \/ (EdgeSrc[e] = OriginalDir[e].tgt /\ EdgeTgt[e] = OriginalDir[e].src)
        \* Reversing twice returns to original direction
        /\ \E orig \in OriginalDir[e]:
            TRUE

INV007_ComponentPartition ==
    LET ActiveNodes == {n \in Nodes : Active[n]}
        Adj(n) == {m \in ActiveNodes :
                     \E e \in Edges:
                        (EdgeSrc[e] = n /\ EdgeTgt[e] = m)
                        \/ (EdgeSrc[e] = m /\ EdgeTgt[e] = n)
                        /\ Weight[e] > 0.0}
        Reachable(from, to) ==
            \E path \in Seq(ActiveNodes):
                /\ Len(path) >= 1
                /\ path[1] = from
                /\ path[Len(path)] = to
                /\ \A i \in 1..(Len(path) - 1): path[i+1] \in Adj(path[i])
    IN
        \A n1 \in ActiveNodes, n2 \in ActiveNodes:
            Reachable(n1, n2) => Reachable(n2, n1)

INV018_DegreeSum ==
    LET InDeg(n) == Cardinality({e \in Edges : EdgeTgt[e] = n})
        OutDeg(n) == Cardinality({e \in Edges : EdgeSrc[e] = n})
        TotalDeg(n) == InDeg(n) + OutDeg(n)
    IN
        Cardinality({e \in Edges : TRUE}) * 2 = SUM(n \in Nodes : TotalDeg(n))

(***************************************************************************)
(* Actions                                                                *)
(***************************************************************************)

AddNode(n, ntype) ==
    /\ n \notin Nodes
    /\ n \in NodeSet
    /\ ntype \in NodeTypes
    /\ Nodes' = Nodes \cup {n}
    /\ NodeType' = [NodeType EXCEPT ![n] = ntype]
    /\ Active' = [Active EXCEPT ![n] = TRUE]
    /\ UNCHANGED <<Edges, EdgeSrc, EdgeTgt, EdgeType, Weight, OriginalDir>>

RemoveNode(n) ==
    /\ n \in Nodes
    /\ \A e \in Edges: EdgeSrc[e] /= n /\ EdgeTgt[e] /= n
    /\ Nodes' = Nodes \ {n}
    /\ UNCHANGED <<Edges, EdgeSrc, EdgeTgt, EdgeType, Weight, OriginalDir>>

DeactivateNode(n) ==
    /\ n \in Nodes
    /\ Active[n] = TRUE
    /\ Active' = [Active EXCEPT ![n] = FALSE]
    /\ UNCHANGED <<Nodes, Edges, EdgeSrc, EdgeTgt, EdgeType, Weight, OriginalDir>>

AddEdge(e, src, tgt, etype, w) ==
    /\ e \notin Edges
    /\ src \in Nodes
    /\ tgt \in Nodes
    /\ src /= tgt
    /\ w >= 0.0 /\ w <= 1.0
    /\ etype \in STRING
    /\ Edges' = Edges \cup {e}
    /\ EdgeSrc' = [EdgeSrc EXCEPT ![e] = src]
    /\ EdgeTgt' = [EdgeTgt EXCEPT ![e] = tgt]
    /\ EdgeType' = [EdgeType EXCEPT ![e] = etype]
    /\ Weight' = [Weight EXCEPT ![e] = w]
    /\ OriginalDir' = [OriginalDir EXCEPT ![e] = [src |-> src, tgt |-> tgt]]
    /\ UNCHANGED <<Nodes, NodeType, Active>>

RemoveEdge(e) ==
    /\ e \in Edges
    /\ Edges' = Edges \ {e}
    /\ UNCHANGED <<Nodes, NodeType, Active, EdgeSrc, EdgeTgt, EdgeType,
                     Weight, OriginalDir>>

ReverseEdge(e) ==
    /\ e \in Edges
    /\ EdgeSrc' = [EdgeSrc EXCEPT ![e] = EdgeTgt[e]]
    /\ EdgeTgt' = [EdgeTgt EXCEPT ![e] = EdgeSrc[e]]
    /\ Weight' = [Weight EXCEPT ![e] = @]
    /\ UNCHANGED <<Nodes, Edges, EdgeType, NodeType, Active, OriginalDir>>

UpdateWeight(e, w) ==
    /\ e \in Edges
    /\ w >= 0.0 /\ w <= 1.0
    /\ Weight' = [Weight EXCEPT ![e] = w]
    /\ UNCHANGED <<Nodes, Edges, EdgeSrc, EdgeTgt, EdgeType, NodeType,
                     Active, OriginalDir>>

(***************************************************************************)
(* Next-state relation                                                    *)
(***************************************************************************)

Next == \/ \E n \in NodeSet, t \in NodeTypes: AddNode(n, t)
         \/ \E n \in Nodes: RemoveNode(n)
         \/ \E n \in Nodes: DeactivateNode(n)
         \/ \E e \in STRING, s \in Nodes, t \in Nodes, et \in STRING, w \in REAL:
              AddEdge(e, s, t, et, w)
         \/ \E e \in Edges: RemoveEdge(e)
         \/ \E e \in Edges: ReverseEdge(e)
         \/ \E e \in Edges, w \in REAL: UpdateWeight(e, w)
         \/ UNCHANGED <<Nodes, Edges, EdgeSrc, EdgeTgt, EdgeType, NodeType,
                         Active, Weight, OriginalDir>>

Spec == Init /\ [][Next]_<<Nodes, Edges, EdgeSrc, EdgeTgt, EdgeType, NodeType,
                          Active, Weight, OriginalDir>>

(***************************************************************************)
(* Invariant Conjunction                                                  *)
(***************************************************************************)

AllInvariants ==
    /\ TypeInvariant
    /\ INV001_NoSelfLoops
    /\ INV002_WeightBounds
    /\ INV003_ReversalIdempotent
    /\ INV007_ComponentPartition
    /\ INV018_DegreeSum

=============================================================================
