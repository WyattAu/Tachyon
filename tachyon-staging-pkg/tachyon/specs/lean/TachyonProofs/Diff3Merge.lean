import Init.Data.List.Lemmas
import Init.Data.Nat.Lemmas

namespace TachyonProofs

inductive LineDiff where
  | keep (content : String)
  | insert (content : String)
  | delete (content : String)
  deriving Repr, BEq

inductive MergeClassification where
  | clean
  | conflicted
  deriving Repr, BEq

def classifyChange (base ours theirs : String) : MergeClassification :=
  if base == ours && base == theirs then .clean
  else if ours == theirs then .clean
  else if base == ours then .clean
  else if base == theirs then .clean
  else .conflicted

#check classifyChange

-- mergeLines uses DecidableEq (List String) so that split on if-then-else
-- produces proof terms (a = b) rather than Bool hypotheses (a == b = true).
-- This makes formal verification tractable while preserving semantic
-- equivalence with the Rust implementation (which uses PartialEq/BEq).
def mergeLines (base ours theirs : List String) : List String :=
  if base = ours ∧ base = theirs then base
  else if base = ours then theirs
  else if base = theirs then ours
  else if ours = theirs then ours
  else base ++ ours ++ theirs

#check mergeLines

-- When all three inputs are identical, mergeLines returns the input unchanged.
theorem merge_identical (a : List String) : mergeLines a a a = a := by
  unfold mergeLines
  split
  · rfl
  · split
    · rfl
    · -- ¬(a = a) is False; all remaining ifs auto-reduced to conflict branch
      exact False.elim (absurd rfl ‹_›)

-- When only ours changed (base = theirs), mergeLines returns ours (= b).
theorem merge_only_ours_changed (a b : List String) : mergeLines a b a = b := by
  unfold mergeLines
  split
  · exact ‹a = b ∧ a = a›.1
  · -- ¬(a = b ∧ a = a)
    split
    · exact absurd ⟨‹a = b›, rfl⟩ ‹¬(a = b ∧ a = a)›
    · -- ¬(a = b). Remaining: if a = a then b else if b = a then b else a ++ b ++ a
      split
      · rfl
      · -- ¬(a = a). Remaining: if b = a then b else a ++ b ++ a
        split
        · rfl
        · exact False.elim (absurd rfl ‹_›)

-- When only theirs changed (base = ours), mergeLines returns theirs (= b).
theorem merge_only_theirs_changed (a b : List String) : mergeLines a a b = b := by
  unfold mergeLines
  split
  · exact ‹a = a ∧ a = b›.2
  · -- ¬(a = a ∧ a = b). Remaining: if a = a then b else ...
      -- a = a always true, so this branch fires immediately.
    split
    · rfl
    · exact False.elim (absurd rfl ‹_›)

-- When ours and theirs are the same, mergeLines returns ours (= b).
theorem merge_both_same (a b : List String) : mergeLines a b b = b := by
  unfold mergeLines
  split
  · exact ‹a = b ∧ a = b›.1
  · -- ¬(a = b ∧ a = b)
    split
    · exact absurd ⟨‹a = b›, ‹a = b›⟩ ‹¬(a = b ∧ a = b)›
    · -- ¬(a = b). Remaining ifs (a = b) auto-reduce. Left: if b = b then b else ...
      split
      · rfl
      · exact False.elim (absurd rfl ‹_›)

-- The merge result length is bounded by a generous upper bound.
-- Holds trivially since the conflict case (concatenation) is the longest.
theorem merge_length_bound (a b c : List String) :
    (mergeLines a b c).length ≤ max (max a.length b.length) c.length + a.length + b.length := by
  unfold mergeLines
  split
  · omega
  · split
    · omega
    · split
      · omega
      · split
        · omega
        · -- True conflict case: a ++ b ++ c
          -- All conditions were false, so we get the concatenation
          have h1 : (a ++ b).length = a.length + b.length := @List.length_append String a b
          have h2 : (a ++ b ++ c).length = (a ++ b).length + c.length := @List.length_append String (a ++ b) c
          omega

end TachyonProofs
