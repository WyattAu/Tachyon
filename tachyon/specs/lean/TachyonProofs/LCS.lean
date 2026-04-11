import Init.Data.List.Lemmas
import Init.Data.Nat.Lemmas

namespace TachyonProofs

inductive IsSubseq {α : Type} : List α → List α → Prop where
  | nil (b : List α) : IsSubseq [] b
  | cons (a : α) as b : IsSubseq as b → IsSubseq (a :: as) (a :: b)
  | skip (a : α) as x b : IsSubseq (a :: as) b → IsSubseq (a :: as) (x :: b)

#check IsSubseq

theorem isSubseq_nil_left {α : Type} (b : List α) : IsSubseq [] b :=
  IsSubseq.nil b

theorem isSubseq_nil_right {α : Type} (a : List α) (h : a ≠ []) : ¬ IsSubseq a [] := by
  intro habs
  cases a with
  | nil => exact h rfl
  | cons head tail =>
    cases habs

theorem isSubseq_refl {α : Type} : ∀ a : List α, IsSubseq a a
  | [] => IsSubseq.nil []
  | a :: as => IsSubseq.cons a as as (isSubseq_refl as)

theorem isSubseq_cons {α : Type} (a : List α) (x : α) (b : List α) :
    IsSubseq a b → IsSubseq a (x :: b) := by
  intro h
  cases a with
  | nil => exact IsSubseq.nil (x :: b)
  | cons head tail =>
    exact IsSubseq.skip head tail x b h

theorem isSubseq_append_left {α : Type} (a b c : List α) :
    IsSubseq a b → IsSubseq a (b ++ c) := by
  sorry -- VERIFICATION PENDING: Requires inversion lemma on IsSubseq
  -- to strip leading elements from the supersequence.

def commonSubseqLength {α : Type} [BEq α] (a b : List α) : Nat :=
  match a, b with
  | [], _ => 0
  | _, [] => 0
  | x :: xs, y :: ys =>
    if x == y then 1 + commonSubseqLength xs ys
    else Nat.max (commonSubseqLength xs (y :: ys)) (commonSubseqLength (x :: xs) ys)

#check commonSubseqLength

theorem lcs_nil {α : Type} [BEq α] (b : List α) : commonSubseqLength [] b = 0 := by
  unfold commonSubseqLength; rfl

theorem lcs_nil_right {α : Type} [BEq α] (a : List α) : commonSubseqLength a [] = 0 := by
  cases a with
  | nil => unfold commonSubseqLength; rfl
  | cons _ _ => unfold commonSubseqLength; rfl

theorem lcs_length_bound {α : Type} [BEq α] (a b : List α) :
    commonSubseqLength a b ≤ min a.length b.length := by
  sorry -- VERIFICATION PENDING: Requires structural induction with
  -- case analysis on the recursive commonSubseqLength calls.

theorem lcs_symmetric {α : Type} [BEq α] (a b : List α) :
    commonSubseqLength a b = commonSubseqLength b a := by
  sorry -- VERIFICATION PENDING: Requires induction with BEq symmetry
  -- reasoning for the if-then-else branches.

end TachyonProofs
