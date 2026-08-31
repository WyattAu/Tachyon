------------------------------- MODULE Diff3Merge -------------------------------
\* Three-way merge algorithm (diff3) for conflict resolution in the Tachyon
\* knowledge management system.
\*
\* Properties verified:
\*   P1 NoDataLoss:        If only one side changed a region, that change appears.
\*   P2 BothSame:          Identical changes on both sides merge cleanly.
\*   P3 ConflictDetection: Divergent changes to the same region produce conflict.
\*   P4 Idempotent:        Merge(base, base, base) == base.
\*
\* Verification status: Unverified (pending TLC installation)
\* To verify:  tlc Diff3Merge.tla
\* To prove:   tlaps Diff3Merge.tla
--------------------------------------------------------------------------------------

EXTENDS Naturals, Sequences, TLC

CONSTANTS MaxLines,    \* Maximum lines per document for model checking
          LineSet      \* Set of possible line contents

ASSUME MaxLines > 0
ASSUME LineSet \subseteq STRING

(***************************************************************************)
(* Types                                                                  *)
(***************************************************************************)

MergeStatus == {"Clean", "Conflicted"}

MergeResult == [status: MergeStatus, content: Seq(LineSet), conflictCount: Nat]

(***************************************************************************)
(* Diff computation                                                        *)
(*                                                                          *)
(* A diff between two sequences produces a list of hunks. Each hunk is     *)
(* either a Match(region) or a Change(removed, added).                     *)
(***************************************************************************)

Hunk == UNION {
    [type |-> "Match",    baseStart: Nat, baseEnd: Nat],
    [type |-> "Change",   baseStart: Nat, baseEnd: Nat,
                         oursStart: Nat, oursEnd: Nat, oursLines: Seq(LineSet),
                         theirsStart: Nat, theirsEnd: Nat, theirsLines: Seq(LineSet)]
}

(***************************************************************************)
(* Helper: compare two sequences line by line, producing hunks              *)
(***************************************************************************)

LinesEqual(seq1, i, seq2, j) ==
    IF i > Len(seq1) \/ j > Len(seq2) THEN FALSE
    ELSE seq1[i] = seq2[j]

ComputeHunks(base, ours, theirs) ==
    LET Compute(baseIdx, oursIdx, theirsIdx, hunks) ==
        IF baseIdx > Len(base) THEN hunks
        ELSE
            IF baseIdx <= Len(ours) /\ baseIdx <= Len(theirs)
               /\ ours[oursIdx] = base[baseIdx]
               /\ theirs[theirsIdx] = base[baseIdx] THEN
                \* All three match
                IF hunks /= <<>> /\ Head(hunks).type = "Match" THEN
                    Compute(baseIdx + 1, oursIdx + 1, theirsIdx + 1,
                            [[Head(hunks) EXCEPT !.baseEnd = baseIdx]]
                            \o Tail(hunks))
                ELSE
                    Compute(baseIdx + 1, oursIdx + 1, theirsIdx + 1,
                            <<[type |-> "Match", baseStart |-> baseIdx, baseEnd |-> baseIdx]>>
                            \o hunks)
            ELSE
                \* One or both sides differ from base
                LET oursStart == oursIdx
                    oursEnd == IF oursIdx <= Len(ours) /\ ours[oursIdx] = base[baseIdx]
                                THEN oursIdx - 1
                                ELSE oursIdx
                    oursChanged == SubSeq(ours, oursStart, oursEnd)
                    theirsStart == theirsIdx
                    theirsEnd == IF theirsIdx <= Len(theirs) /\ theirs[theirsIdx] = base[baseIdx]
                                  THEN theirsIdx - 1
                                  ELSE theirsIdx
                    theirsChanged == SubSeq(theirs, theirsStart, theirsEnd)
                    baseEnd == IF oursIdx <= Len(ours) /\ ours[oursIdx] = base[baseIdx]
                                THEN baseIdx - 1
                                ELSE IF theirsIdx <= Len(theirs) /\ theirs[theirsIdx] = base[baseIdx]
                                     THEN baseIdx - 1
                                     ELSE baseIdx
                IN
                    Compute(baseEnd + 1,
                            oursEnd + 1,
                            theirsEnd + 1,
                            <<[type |-> "Change",
                               baseStart |-> baseIdx,
                               baseEnd |-> baseEnd,
                               oursStart |-> oursStart,
                               oursEnd |-> oursEnd,
                               oursLines |-> oursChanged,
                               theirsStart |-> theirsStart,
                               theirsEnd |-> theirsEnd,
                               theirsLines |-> theirsChanged]>>
                            \o hunks)
    IN Compute(1, 1, 1, <<>>)

(***************************************************************************)
(* Merge: apply hunks to produce a merged result                            *)
(***************************************************************************)

MergeHunks(hunks, base) ==
    LET MergeH(hunks, base, result, conflicts) ==
        IF hunks = <<>> THEN
            [status |-> IF conflicts > 0 THEN "Conflicted" ELSE "Clean",
             content |-> result,
             conflictCount |-> conflicts]
        ELSE
            LET h == Head(hunks)
            IN
                IF h.type = "Match" THEN
                    MergeH(Tail(hunks), base,
                           result \o SubSeq(base, h.baseStart, h.baseEnd),
                           conflicts)
                ELSE
                    \* h.type = "Change"
                    IF h.oursLines = <<>> /\ h.theirsLines = <<>> THEN
                        \* Both sides deleted same region
                        MergeH(Tail(hunks), base, result, conflicts)
                    ELSE IF h.oursLines = h.theirsLines THEN
                        \* Both sides made identical change
                        MergeH(Tail(hunks), base,
                               result \o h.oursLines, conflicts)
                    ELSE IF h.oursLines = SubSeq(base, h.baseStart, h.baseEnd) THEN
                        \* Only theirs changed
                        MergeH(Tail(hunks), base,
                               result \o h.theirsLines, conflicts)
                    ELSE IF h.theirsLines = SubSeq(base, h.baseStart, h.baseEnd) THEN
                        \* Only ours changed
                        MergeH(Tail(hunks), base,
                               result \o h.oursLines, conflicts)
                    ELSE
                        \* True conflict: both changed differently
                        MergeH(Tail(hunks), base,
                               result \o h.oursLines \o <<">>>>>>> THEIRS">> \o h.theirsLines,
                               conflicts + 1)
    IN MergeH(hunks, base, <<>>, 0)

Diff3Merge(base, ours, theirs) ==
    LET hunks == ComputeHunks(base, ours, theirs)
    IN  MergeHunks(hunks, base)

(***************************************************************************)
(* VARIABLES                                                              *)
(***************************************************************************)

VARIABLES base, ours, theirs, mergeResult

(***************************************************************************)
(* Init                                                                   *)
(***************************************************************************)

Init ==
    /\ base = <<>>
    /\ ours = <<>>
    /\ theirs = <<>>
    /\ mergeResult = Diff3Merge(base, ours, theirs)

(***************************************************************************)
(* Type Invariant                                                         *)
(***************************************************************************)

TypeInvariant ==
    /\ base \in Seq(LineSet)
    /\ ours \in Seq(LineSet)
    /\ theirs \in Seq(LineSet)
    /\ Len(base) <= MaxLines
    /\ Len(ours) <= MaxLines
    /\ Len(theirs) <= MaxLines
    /\ mergeResult.status \in MergeStatus
    /\ mergeResult.content \in Seq(LineSet)
    /\ mergeResult.conflictCount \in Nat

(***************************************************************************)
(* Actions                                                                *)
(***************************************************************************)

SetBase(content) ==
    /\ content \in Seq(LineSet)
    /\ Len(content) <= MaxLines
    /\ base' = content
    /\ mergeResult' = Diff3Merge(content, ours, theirs)
    /\ UNCHANGED <<ours, theirs>>

SetOurs(content) ==
    /\ content \in Seq(LineSet)
    /\ Len(content) <= MaxLines
    /\ ours' = content
    /\ mergeResult' = Diff3Merge(base, content, theirs)
    /\ UNCHANGED <<base, theirs>>

SetTheirs(content) ==
    /\ content \in Seq(LineSet)
    /\ Len(content) <= MaxLines
    /\ theirs' = content
    /\ mergeResult' = Diff3Merge(base, ours, content)
    /\ UNCHANGED <<base, ours>>

(***************************************************************************)
(* Next-state relation                                                    *)
(***************************************************************************)

Next == \/ \E content \in Seq(LineSet): SetBase(content)
         \/ \E content \in Seq(LineSet): SetOurs(content)
         \/ \E content \in Seq(LineSet): SetTheirs(content)
         \/ UNCHANGED <<base, ours, theirs, mergeResult>>

Spec == Init /\ [][Next]_<<base, ours, theirs, mergeResult>>

(***************************************************************************)
(* Properties                                                             *)
(***************************************************************************)

NoDataLoss ==
    [](\A line \in LineSet:
        \A i \in 1..Len(base):
            \A j \in 1..Len(ours):
                \A k \in 1..Len(theirs):
                    (ours[j] /= base[i] /\ ours[j] = line /\ theirs = base) =>
                        line \in SubSeq(mergeResult.content, 1, Len(mergeResult.content)))

BothSame ==
    []((ours = theirs) => mergeResult.status = "Clean")

BothSamePreservesContent ==
    [](ours = theirs => mergeResult.content = ours)

ConflictDetection ==
    [](\A i \in 1..Len(base):
        \A j \in 1..Len(ours):
            \A k \in 1..Len(theirs):
                (ours /= theirs
                    /\ ours /= base
                    /\ theirs /= base
                    /\ ours[j] /= base[i]
                    /\ theirs[k] /= base[i])
                    => mergeResult.conflictCount >= 1)

Idempotent ==
    [](base = ours /\ ours = theirs => mergeResult.content = base /\ mergeResult.status = "Clean")

MergeResultConsistent ==
    [](mergeResult = Diff3Merge(base, ours, theirs))

CleanMergeNoConflicts ==
    [](mergeResult.status = "Clean" => mergeResult.conflictCount = 0)

ConflictedMergeHasConflicts ==
    [](mergeResult.status = "Conflicted" => mergeResult.conflictCount > 0)

=============================================================================
