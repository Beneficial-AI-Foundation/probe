namespace LeanMicro

def add (a b : Nat) : Nat :=
  a + b

def double (x : Nat) : Nat :=
  add x x

theorem add_comm (a b : Nat) : add a b = add b a := by
  simp [add, Nat.add_comm]

structure Point where
  x : Nat
  y : Nat

def Point.origin : Point :=
  { x := 0, y := 0 }

def Point.translate (p : Point) (dx dy : Nat) : Point :=
  { x := add p.x dx, y := add p.y dy }

theorem double_eq_add_self (n : Nat) : double n = add n n := by
  rfl

class HasSize (α : Type) where
  size : α → Nat

instance : HasSize Point where
  size p := add p.x p.y

theorem sorry_example (n : Nat) : n + 0 = n := by
  sorry

end LeanMicro
