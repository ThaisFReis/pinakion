pragma circom 2.1.6;

// Pinakion — the claim of one seat (DEC-015, T-034 spike; T-036 finishes it).
// GPL-3.0: this directory includes circomlib (see circuits/LICENSE, DEC-021).
//
// Ported from the author's role_membership.circom of 2026-10-25 (DEC-017):
// same leaf, same Merkle path; the nullifier becomes y and actionHash becomes
// claimer.
//
// Statement: "I know s such that Poseidon(s) is a leaf of the tree with root
// `root`, and y = Poseidon(seed, s)". `seed` comes from the drand round the
// lock fixed; `claimer` is the seat account the proof is sent from, so a proof
// copied from the mempool is useless to anyone else.
include "../node_modules/circomlib/circuits/poseidon.circom";

template PinakionClaim(depth) {
    // public, in this order: the contract assembles the same vector
    signal input root;
    signal input seed;
    signal input y;
    signal input claimer;

    // private
    signal input s;
    signal input pathElements[depth];
    signal input pathIndices[depth];

    component leaf = Poseidon(1);
    leaf.inputs[0] <== s;

    component ticket = Poseidon(2);
    ticket.inputs[0] <== seed;
    ticket.inputs[1] <== s;
    y === ticket.out;

    signal cur[depth + 1];
    signal left[depth];
    signal right[depth];
    component node[depth];
    cur[0] <== leaf.out;

    for (var i = 0; i < depth; i++) {
        pathIndices[i] * (1 - pathIndices[i]) === 0;
        // index 0: current node is the left child; index 1: it is the right child
        left[i] <== cur[i] + pathIndices[i] * (pathElements[i] - cur[i]);
        right[i] <== pathElements[i] + pathIndices[i] * (cur[i] - pathElements[i]);
        node[i] = Poseidon(2);
        node[i].inputs[0] <== left[i];
        node[i].inputs[1] <== right[i];
        cur[i + 1] <== node[i].out;
    }

    root === cur[depth];

    // claimer enters a constraint so it is part of the statement.
    signal claimerSquare;
    claimerSquare <== claimer * claimer;
}

component main { public [root, seed, y, claimer] } = PinakionClaim(6);
