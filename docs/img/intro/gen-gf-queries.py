"""Render the spqr encoding/gf multiplication slice three times, each with one
query result highlighted (pink: not a status colour). Nodes, edges and statuses come from
aeneas_spqr_1.6.0.json (probe-aeneas 0.21.0, spqr-verify 01245a1).
Usage: python3 gen-gf-queries.py  (writes gf-<query>.dot and .png)"""
import subprocess

N = {  # id: (label, verified, translated)
    "A": ("GF16 * &GF16\n(Mul::mul)", True, True),
    "B": ("GF16 *= &GF16\n(MulAssign::mul_assign)", True, True),
    "C": ("accelerated::mul", False, False),
    "D": ("accelerated::mul2", False, False),
    "E": ("accelerated::\nmul2_unreduced", False, False),
    "F": ("unaccelerated::mul", True, True),
    "G": ("unaccelerated::mul2", True, True),
    "H": ("unaccelerated::\npoly_mul", True, True),
    "I": ("reduce::poly_reduce", True, True),
}
E = ["A B", "B C", "B F", "C D", "D E", "D I", "F H", "F I", "G F"]
Q = {
    "entrypoints": {"A", "G"},
    "untranslated": {k for k, v in N.items() if not v[2]},
    "verified": {k for k, v in N.items() if v[1]},
}

for q, hit in Q.items():
    out = ['digraph G {', '  rankdir=TB; nodesep=0.35; ranksep=0.45;',
           '  node [fontname="Inter", fontsize=13, shape=box, style="filled,rounded", margin="0.15,0.08"];',
           '  edge [color="#777777", arrowsize=0.7];']
    for k, (label, _, _) in N.items():
        if k in hit:
            out.append(f'  {k} [label="{label}", fillcolor="#F8BBD0", color="#AD1457", penwidth=2.5];')
        else:
            out.append(f'  {k} [label="{label}", fillcolor="#F2F2F2", color="#BBBBBB", fontcolor="#777777"];')
    out += [f'  {e.split()[0]} -> {e.split()[1]};' for e in E]
    out.append('}')
    open(f"gf-{q}.dot", "w").write("\n".join(out) + "\n")
    subprocess.run(["dot", "-Tpng", "-Gdpi=150", f"gf-{q}.dot", "-o", f"gf-{q}.png"], check=True)
