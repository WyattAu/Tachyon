------------------------------- MODULE ReviewStateMachine -------------------------
\* Document review workflow state machine for the Tachyon knowledge management
\* system.
\*
\* States:
\*   Pending         -> Approved, Rejected, ChangesRequested, Cancelled
\*   ChangesRequested -> Approved, Rejected, Cancelled
\*   Approved, Rejected, Cancelled are terminal states.
\*
\* Properties verified:
\*   P1 NoInvalidTransitions:  Terminal states have no outgoing transitions.
\*   P2 IdempotentCancel:      Cancelling an already-cancelled review is a no-op.
\*   P3 ApprovalChain:         A review must go through Pending before Approved.
\*   P4 SingleResolution:      Only one non-terminal state at any time.
\*
\* Verification status: Unverified (pending TLC installation)
\* To verify:  tlc ReviewStateMachine.tla
\* To prove:   tlaps ReviewStateMachine.tla
--------------------------------------------------------------------------------------

EXTENDS Naturals, Sequences, TLC

CONSTANTS ReviewSet   \* Set of all review IDs

ASSUME ReviewSet \subseteq STRING

(***************************************************************************)
(* Types                                                                  *)
(***************************************************************************)

ReviewState == {"Pending", "Approved", "Rejected", "ChangesRequested", "Cancelled"}

TerminalStates == {"Approved", "Rejected", "Cancelled"}

NonTerminalStates == {"Pending", "ChangesRequested"}

(***************************************************************************)
(* VARIABLES                                                              *)
(***************************************************************************)

VARIABLES ReviewStatus,       \* ReviewStatus[r] \in ReviewState
          ReviewHistory,      \* ReviewHistory[r] = sequence of past states
          ReviewsEverApproved \* set of reviews that have been Approved

(***************************************************************************)
(* Init                                                                   *)
(***************************************************************************)

Init ==
    /\ ReviewStatus = [r \in ReviewSet |-> "Pending"]
    /\ ReviewHistory = [r \in ReviewSet |-> <<"Pending">>]
    /\ ReviewsEverApproved = {}

(***************************************************************************)
(* Type Invariant                                                         *)
(***************************************************************************)

TypeInvariant ==
    /\ DOMAIN ReviewStatus = ReviewSet
    /\ DOMAIN ReviewHistory = ReviewSet
    /\ ReviewsEverApproved \subseteq ReviewSet
    /\ \A r \in ReviewSet:
        /\ ReviewStatus[r] \in ReviewState
        /\ ReviewHistory[r] \in Seq(ReviewState)
        /\ Len(ReviewHistory[r]) >= 1
        /\ Head(ReviewHistory[r]) = "Pending"
        /\ ReviewStatus[r] = LastState(ReviewHistory[r])

(***************************************************************************)
(* Helper                                                                 *)
(***************************************************************************)

LastState(hist) ==
    IF Len(hist) = 0 THEN "Pending"
    ELSE hist[Len(hist)]

(***************************************************************************)
(* Transitions                                                            *)
(***************************************************************************)

TransitionApprove(r) ==
    /\ r \in ReviewSet
    /\ ReviewStatus[r] \in {"Pending", "ChangesRequested"}
    /\ ReviewStatus' = [ReviewStatus EXCEPT ![r] = "Approved"]
    /\ ReviewHistory' = [ReviewHistory EXCEPT ![r] = Append(@[r], <<"Approved">>)]
    /\ ReviewsEverApproved' = ReviewsEverApproved \cup {r}

TransitionReject(r) ==
    /\ r \in ReviewSet
    /\ ReviewStatus[r] \in {"Pending", "ChangesRequested"}
    /\ ReviewStatus' = [ReviewStatus EXCEPT ![r] = "Rejected"]
    /\ ReviewHistory' = [ReviewHistory EXCEPT ![r] = Append(@[r], <<"Rejected">>)]
    /\ UNCHANGED ReviewsEverApproved

TransitionRequestChanges(r) ==
    /\ r \in ReviewSet
    /\ ReviewStatus[r] = "Pending"
    /\ ReviewStatus' = [ReviewStatus EXCEPT ![r] = "ChangesRequested"]
    /\ ReviewHistory' = [ReviewHistory EXCEPT ![r] = Append(@[r], <<"ChangesRequested">>)]
    /\ UNCHANGED ReviewsEverApproved

TransitionCancel(r) ==
    /\ r \in ReviewSet
    /\ ReviewStatus[r] \in {"Pending", "ChangesRequested", "Cancelled"}
    /\ ReviewStatus' = [ReviewStatus EXCEPT ![r] = "Cancelled"]
    /\ ReviewHistory' = [ReviewHistory EXCEPT
                            ![r] = IF ReviewStatus[r] = "Cancelled"
                                    THEN @[r]
                                    ELSE Append(@[r], <<"Cancelled">>)]
    /\ UNCHANGED ReviewsEverApproved

(***************************************************************************)
(* Next-state relation                                                    *)
(***************************************************************************)

Next == \/ \E r \in ReviewSet: TransitionApprove(r)
         \/ \E r \in ReviewSet: TransitionReject(r)
         \/ \E r \in ReviewSet: TransitionRequestChanges(r)
         \/ \E r \in ReviewSet: TransitionCancel(r)
         \/ UNCHANGED <<ReviewStatus, ReviewHistory, ReviewsEverApproved>>

Spec == Init /\ [][Next]_<<ReviewStatus, ReviewHistory, ReviewsEverApproved>>

(***************************************************************************)
(* Properties                                                             *)
(***************************************************************************)

NoInvalidTransitions ==
    [](\A r \in ReviewSet:
        ReviewStatus[r] \in TerminalStates =>
            []~(ReviewStatus' /= [ReviewStatus EXCEPT ![r] = ReviewStatus[r]]
                  /\ ReviewStatus[r] = ReviewStatus'[r]))

NoInvalidTransitionsSimple ==
    [](\A r \in ReviewSet:
        ReviewStatus[r] \in TerminalStates =>
            UNCHANGED ReviewStatus[r])

IdempotentCancel ==
    [](\A r \in ReviewSet:
        ReviewStatus[r] = "Cancelled" =>
            (ReviewStatus'[r] = "Cancelled")
            /\ (ReviewHistory'[r] = ReviewHistory[r]))

ApprovalChain ==
    [](\A r \in ReviewSet:
        ReviewStatus[r] = "Approved" =>
            "Pending" \in ReviewHistory[r]
            /\ "ChangesRequested" \notin ReviewHistory[r] \ {ReviewHistory[r]})

ApprovalChainPrecise ==
    [](\A r \in ReviewSet:
        ReviewStatus[r] = "Approved" =>
            \E i \in 1..Len(ReviewHistory[r]):
                /\ ReviewHistory[r][i] = "Pending"
                /\ \A j \in 1..(i-1):
                    ReviewHistory[r][j] \in NonTerminalStates)

SingleResolution ==
    [](\A r \in ReviewSet:
        ReviewStatus[r] \in TerminalStates =>
            Cardinality({s \in DOMAIN ReviewHistory[r]:
                ReviewHistory[r][s] \in TerminalStates}) = 1)

HistoryMonotonic ==
    [](\A r \in ReviewSet:
        Len(ReviewHistory[r]) <= Len(ReviewHistory'[r]))

=============================================================================
