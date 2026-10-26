# Tessera's wasm client

These two files are **Tessera's**, the author's earlier secret-ballot module (https://github.com/ThaisFReis/tessera), MIT (`LICENSE`, beside this file).

They were built unchanged from `cliente-wasm/` at commit `356c22fbf90ea396203ea411b463ae14a5480d59`, with Tessera's own command:

```bash
wasm-pack build --release --target web --out-dir pacote
```

The live case (`web/vivo/`) uses them to build the ring key, the anonymous ballot and the time-locked tally in the browser, exactly as Tessera's own dapp does.
