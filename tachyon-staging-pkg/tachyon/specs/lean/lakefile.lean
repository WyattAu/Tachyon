import Lake
open Lake DSL

package TachyonProofs where
  leanOptions := #[⟨`autoImplicit, false⟩]

@[default_target]
lean_lib TachyonProofs
