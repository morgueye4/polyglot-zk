pragma circom 2.1.6;

template SumCircuit () {
    signal input a;
    signal input b;
    signal output c;

    c <== a + b;
    c === 28;

}

component main = SumCircuit();