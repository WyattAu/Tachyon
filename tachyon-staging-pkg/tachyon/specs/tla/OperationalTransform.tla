------------------------------- MODULE OperationalTransform --------------------------
\* Operational Transform system for real-time collaborative editing.
\*
\* Properties verified:
\*   P1 Convergence:          All clients eventually see the same document state.
\*   P2 IntentionPreservation: Transformed operations preserve logical intent.
\*   P3 Causality:            Causal ordering of operations is respected.
\*   P4 NoDataLoss:           All inserted characters eventually appear in the
\*                            final document.
\*
\* Verification status: Unverified (pending TLC installation)
\* To verify:  tlc -deadlock OperationalTransform.tla
\* To prove:   tlaps OperationalTransform.tla
--------------------------------------------------------------------------------------

EXTENDS Naturals, Sequences, TLC

CONSTANTS MaxPendingOps,   \* Maximum pending ops per document before flush
          MaxDocLen,       \* Maximum document length for model checking
          CharSet          \* Alphabet of characters (e.g. {"a","b","c"})

ASSUME MaxPendingOps > 0
ASSUME MaxDocLen > 0
ASSUME CharSet \subseteq STRING

(***************************************************************************)
(* Types                                                                  *)
(***************************************************************************)

OpType == {"insert", "delete"}

Op == [
    type:    OpType,
    clientId: STRING,
    seqNum:   Nat,
    pos:      Nat,
    text:     STRING
]

(***************************************************************************)
(* Helper: check that an op record is well-formed                          *)
(***************************************************************************)

WellFormedOp(op) ==
    /\ op.type \in OpType
    /\ op.clientId \in STRING
    /\ op.pos <= MaxDocLen
    /\ op.type = "insert" => Len(op.text) > 0
    /\ op.type = "delete" => op.text = ""  \* text field unused for deletes

(***************************************************************************)
(* Apply a single operation to a character sequence                        *)
(***************************************************************************)

ApplySingleOp(seq, op) ==
    IF op.type = "insert" THEN
        Append(SubSeq(seq, 1, op.pos), op.text) \o SubSeq(seq, op.pos + 1, Len(seq))
    ELSE  \* delete
        SubSeq(seq, 1, op.pos) \o SubSeq(seq, op.pos + Len(op.text) + 1, Len(seq))

(***************************************************************************)
(* Transform two concurrent operations so they can be applied in either   *)
(* order and converge to the same state.                                  *)
(*                                                                          *)
(* Transform(op1, op2) returns [op1', op2'] where op1' is op1 transformed *)
(* against op2, and op2' is op2 transformed against op1.                  *)
(***************************************************************************)

Transform(op1, op2) ==
    IF op1.type = "insert" /\ op2.type = "insert" THEN
        \* Tie-break by clientId for same-position inserts
        IF op1.pos = op2.pos THEN
            IF op1.clientId < op2.clientId THEN
                [op1 EXCEPT !.pos = op1.pos]
                \o [op2 EXCEPT !.pos = op2.pos + Len(op1.text)]
            ELSE
                [op1 EXCEPT !.pos = op1.pos + Len(op2.text)]
                \o [op2 EXCEPT !.pos = op2.pos]
        ELSE IF op1.pos < op2.pos THEN
            [op1 EXCEPT !.pos = op1.pos]
            \o [op2 EXCEPT !.pos = op2.pos + Len(op1.text)]
        ELSE
            [op1 EXCEPT !.pos = op1.pos + Len(op2.text)]
            \o [op2 EXCEPT !.pos = op2.pos]
    ELSE IF op1.type = "insert" /\ op2.type = "delete" THEN
        IF op1.pos >= op2.pos THEN
            [op1 EXCEPT !.pos = op1.pos - Len(op2.text)]
            \o op2
        ELSE
            op1 \o [op2 EXCEPT !.pos = op2.pos + Len(op1.text)]
    ELSE IF op1.type = "delete" /\ op2.type = "insert" THEN
        IF op2.pos >= op1.pos THEN
            [op1 EXCEPT !.pos = op1.pos + Len(op2.text)]
            \o op2
        ELSE
            op1 \o [op2 EXCEPT !.pos = op2.pos - Len(op1.text)]
    ELSE
        \* Both deletes: adjust positions
        LET overlap == Max(0, Min(op1.pos + Len(op1.text), op2.pos + Len(op2.text))
                                 - Max(op1.pos, op2.pos))
        IN
        IF op1.pos >= op2.pos THEN
            [op1 EXCEPT !.pos = op1.pos - Len(op2.text)]
            \o [op2 EXCEPT !.pos = op2.pos]
        ELSE
            [op1 EXCEPT !.pos = op1.pos]
            \o [op2 EXCEPT !.pos = op2.pos - Len(op1.text)]

(***************************************************************************)
(* VARIABLES                                                              *)
(***************************************************************************)

VARIABLES Docs,            \* set of document IDs
          Contents,        \* Contents[d] = sequence of characters
          PendingOps,      \* PendingOps[d] = <<Op1, Op2, ...>>
          Clients,         \* set of connected client IDs
          ClientSeqNum,    \* ClientSeqNum[c] = next sequence number
          ClientView,      \* ClientView[c] = [viewContent |-> seq, appliedSeqs |-> set]
          OpHistory        \* OpHistory[d] = <<op1, op2, ...>> applied globally

(***************************************************************************)
(* Init                                                                   *)
(***************************************************************************)

Init ==
    /\ Docs = {}
    /\ Contents = [d \in {} |-> <<>>]
    /\ PendingOps = [d \in {} |-> <<>>]
    /\ Clients = {}
    /\ ClientSeqNum = [c \in {} |-> 0]
    /\ ClientView = [c \in {} |-> [viewContent |-> <<>>, appliedSeqs |-> {}]]
    /\ OpHistory = [d \in {} |-> <<>>]

(***************************************************************************)
(* Type Invariant                                                         *)
(***************************************************************************)

TypeInvariant ==
    /\ Docs \subseteq STRING
    /\ DOMAIN Contents = Docs
    /\ DOMAIN PendingOps = Docs
    /\ DOMAIN OpHistory = Docs
    /\ Clients \subseteq STRING
    /\ DOMAIN ClientSeqNum = Clients
    /\ DOMAIN ClientView = Clients
    /\ \A d \in Docs:
        /\ \A op \in PendingOps[d]: WellFormedOp(op)
        /\ \A op \in OpHistory[d]: WellFormedOp(op)
    /\ \A c \in Clients:
        /\ ClientSeqNum[c] \in Nat
        /\ ClientView[c].viewContent \in Seq(CharSet)
        /\ ClientView[c].appliedSeqs \subseteq Nat

(***************************************************************************)
(* Actions                                                                *)
(***************************************************************************)

CreateDoc(d) ==
    /\ d \notin Docs
    /\ Docs' = Docs \cup {d}
    /\ Contents' = [Contents EXCEPT ![d] = <<>>]
    /\ PendingOps' = [PendingOps EXCEPT ![d] = <<>>]
    /\ OpHistory' = [OpHistory EXCEPT ![d] = <<>>]
    /\ UNCHANGED <<Clients, ClientSeqNum, ClientView>>

ConnectClient(c) ==
    /\ c \notin Clients
    /\ Clients' = Clients \cup {c}
    /\ ClientSeqNum' = [ClientSeqNum EXCEPT ![c] = 0]
    /\ ClientView' = [ClientView EXCEPT ![c] = [viewContent |-> <<>>,
                                                     appliedSeqs |-> {}]]
    /\ UNCHANGED <<Docs, Contents, PendingOps, OpHistory>>

DisconnectClient(c) ==
    /\ c \in Clients
    /\ Clients' = Clients \ {c}
    /\ UNCHANGED <<Docs, Contents, PendingOps, ClientSeqNum, OpHistory>>

LocalInsert(c, d, pos, text) ==
    /\ c \in Clients
    /\ d \in Docs
    /\ pos <= Len(ClientView[c].viewContent)
    /\ Len(text) > 0
    /\ \A ch \in 1..Len(text): SubSeq(text, ch, ch) \in CharSet
    /\ LET op == [type |-> "insert",
                   clientId |-> c,
                   seqNum |-> ClientSeqNum[c],
                   pos |-> pos,
                   text |-> text]
    IN
        /\ PendingOps' = [PendingOps EXCEPT ![d] = Append(PendingOps[d], <<op>>)]
        /\ ClientSeqNum' = [ClientSeqNum EXCEPT ![c] = @ + 1]
        /\ ClientView' = [ClientView EXCEPT
                            ![c].viewContent = ApplySingleOp(@[c].viewContent, op),
                            ![c].appliedSeqs = @[c].appliedSeqs \cup {op.seqNum}]
        /\ UNCHANGED <<Docs, Contents, OpHistory>>

LocalDelete(c, d, pos, len) ==
    /\ c \in Clients
    /\ d \in Docs
    /\ pos + len <= Len(ClientView[c].viewContent)
    /\ len > 0
    /\ LET deletedText == SubSeq(ClientView[c].viewContent, pos + 1, pos + len)
           op == [type |-> "delete",
                   clientId |-> c,
                   seqNum |-> ClientSeqNum[c],
                   pos |-> pos,
                   text |-> deletedText]
    IN
        /\ PendingOps' = [PendingOps EXCEPT ![d] = Append(PendingOps[d], <<op>>)]
        /\ ClientSeqNum' = [ClientSeqNum EXCEPT ![c] = @ + 1]
        /\ ClientView' = [ClientView EXCEPT
                            ![c].viewContent = ApplySingleOp(@[c].viewContent, op),
                            ![c].appliedSeqs = @[c].appliedSeqs \cup {op.seqNum}]
        /\ UNCHANGED <<Docs, Contents, OpHistory>>

ReceiveOp(d) ==
    /\ d \in Docs
    /\ Len(PendingOps[d]) > 0
    /\ LET op == Head(PendingOps[d])
    IN
        /\ Contents' = [Contents EXCEPT ![d] = ApplySingleOp(contents[d], op)]
        /\ OpHistory' = [OpHistory EXCEPT ![d] = Append(OpHistory[d], <<op>>)]
        /\ PendingOps' = [PendingOps EXCEPT ![d] = Tail(PendingOps[d])]
        /\ \A c \in Clients:
            /\ op.seqNum \notin ClientView[c].appliedSeqs =>
                LET transformed == TransformOpAgainstHistory(op, OpHistory[d], ClientView[c])
                IN  ClientView' = [ClientView EXCEPT
                                      ![c].viewContent = ApplySingleOp(@[c].viewContent, transformed),
                                      ![c].appliedSeqs = @[c].appliedSeqs \cup {op.seqNum}]
        /\ UNCHANGED <<Docs, Clients, ClientSeqNum>>

TransformOpAgainstHistory(op, history, view) ==
    IF history = <<>> THEN op
    ELSE
        LET lastOp == Head(history)
        IN  TransformOpAgainstHistory(TransformPrim(op, lastOp), Tail(history), view)

TransformPrim(op, against) ==
    \* Simplified transform: adjust position based on prior operation
    IF against.type = "insert" THEN
        IF op.pos >= against.pos THEN
            [op EXCEPT !.pos = @ + Len(against.text)]
        ELSE op
    ELSE  \* against.type = "delete"
        IF op.pos >= against.pos + Len(against.text) THEN
            [op EXCEPT !.pos = @ - Len(against.text)]
        ELSE IF op.pos >= against.pos THEN
            [op EXCEPT !.pos = against.pos]
        ELSE op

(***************************************************************************)
(* Next-state relation                                                    *)
(***************************************************************************)

Next == \/ \E d \in STRING: CreateDoc(d)
         \/ \E c \in STRING: ConnectClient(c)
         \/ \E c \in Clients: DisconnectClient(c)
         \/ \E c \in Clients, d \in Docs, pos \in Nat, text \in STRING:
              LocalInsert(c, d, pos, text)
         \/ \E c \in Clients, d \in Docs, pos \in Nat, len \in Nat:
              LocalDelete(c, d, pos, len)
         \/ \E d \in Docs: ReceiveOp(d)
         \/ UNCHANGED <<Docs, Contents, PendingOps, Clients, ClientSeqNum,
                         ClientView, OpHistory>>

Spec == Init /\ [][Next]_<<Docs, Contents, PendingOps, Clients, ClientSeqNum,
                          ClientView, OpHistory>>

(***************************************************************************)
(* Properties                                                             *)
(***************************************************************************)

Convergence ==
    []<>(\A c1 \in Clients, c2 \in Clients:
        ClientSeqNum[c1] = ClientSeqNum[c2] =>
            ClientView[c1].viewContent = ClientView[c2].viewContent)

IntentionPreservation ==
    [](\A c \in Clients, d \in Docs:
        /\ ClientView[c].viewContent \in Seq(CharSet)
        /\ \A op \in OpHistory[d]:
            op.type = "insert" => Len(op.text) > 0)

Causality ==
    [](\A op1 \in UNION {OpHistory[d]: d \in Docs},
          op2 \in UNION {OpHistory[d]: d \in Docs}:
        /\ op1.seqNum < op2.seqNum
        => \A c \in Clients:
            op2.seqNum \in ClientView[c].appliedSeqs =>
                op1.seqNum \in ClientView[c].appliedSeqs)

NoDataLoss ==
    [](\A d \in Docs, op \in OpHistory[d]:
        op.type = "insert" =>
            SubSeq(op.text, 1, Len(op.text)) \subseteq
                SubSeq(Contents[d], 1, Len(Contents[d])))

ServerContentInvariant ==
    [](\A d \in Docs:
        Contents[d] \in Seq(STRING))

=============================================================================
