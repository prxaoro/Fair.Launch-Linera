# Fair Launch - Deployment Information

## Local Network Status
- **Network**: Local Linera testnet
- **Validators**: 2 (ports 9001, 9002)
- **Chains**: 10 chains total
- **Default Chain**: `dfada58d53643b15bf79b9ceaeb0e57a00b801ad8d6ba657e2c1e8f3b1e38ac9`

## GraphQL Service
- **URL**: http://localhost:8080
- **GraphiQL IDE**: http://localhost:8080 (for testing queries)

## Deployed Contracts

### Factory Contract
- **Bytecode ID**: `0df3009aeb72512f598cd40e500e61f19a88fed045e75e0793c7edde4e5873c0346c32afe4e50cf09ffb9f4447941f7a903a500c16b8dcd8db778b88378a18c200`
- **Contract Size**: 194K
- **Service Size**: 928K
- **Block Height**: 9

### Token Contract
- **Bytecode ID**: `968b3b48a8bf6142fb92ec2b9d2ee11129190bd556df45213e7d6f0856b21b264734198c6b4a7d53ac12a022aeaed271305ec36b2371e55963e4b55954c1009800`
- **Contract Size**: 302K
- **Service Size**: 940K
- **Block Height**: 10

### Swap Contract
- **Bytecode ID**: `dc92b6f246072b0629c7adf5e17cd05ce75d8190ed4744eac290d8f71aff026427f152644e2e03976306879ba89fce54a63cc5f0727c2673b9ce0d48192161a700`
- **Contract Size**: 243K
- **Service Size**: 855K
- **Block Height**: 11

## Environment Variables

```bash
export LINERA_WALLET=/tmp/.tmpKRl5jk/wallet_0.json
export LINERA_KEYSTORE=/tmp/.tmpKRl5jk/keystore_0.json
export LINERA_STORAGE="rocksdb:/tmp/.tmpKRl5jk/client_0.db"
```

## Application IDs

### Factory Application
- **Application ID**: `ba329760710cc839fc3f99f0853c21861b11449b2f723e7c397fcb6ef24222d5`
- **Chain Height**: 12

### Token Application
- **Application ID**: `f08476beb66ad4128904dd0d1ae0809f31de70801107be7f24a28941f570f014`
- **Chain Height**: 13

### Swap Application
- **Application ID**: `70cca1cad5a260b6440b3b74d40924a8491e23d8bbc4dc53669450d7d391d65d`
- **Chain Height**: 14

## Next Steps

1. ✅ Local network running
2. ✅ Contracts compiled (WASM files ready)
3. ✅ Bytecode modules published to blockchain
4. ✅ Application instances created
5. ⏳ GraphQL service running on port 8080
6. ⏳ Update frontend to connect to GraphQL endpoint
7. ⏳ Remove mock data fallback
8. ⏳ Test end-to-end token creation and trading

## Commands Reference

### Start GraphQL Service
```bash
LINERA_WALLET=/tmp/.tmpKRl5jk/wallet_0.json \
LINERA_KEYSTORE=/tmp/.tmpKRl5jk/keystore_0.json \
LINERA_STORAGE="rocksdb:/tmp/.tmpKRl5jk/client_0.db" \
linera service --port 8080
```

### Query Wallet
```bash
LINERA_WALLET=/tmp/.tmpKRl5jk/wallet_0.json \
LINERA_KEYSTORE=/tmp/.tmpKRl5jk/keystore_0.json \
LINERA_STORAGE="rocksdb:/tmp/.tmpKRl5jk/client_0.db" \
linera wallet show
```

### Sync Blockchain
```bash
LINERA_WALLET=/tmp/.tmpKRl5jk/wallet_0.json \
LINERA_KEYSTORE=/tmp/.tmpKRl5jk/keystore_0.json \
LINERA_STORAGE="rocksdb:/tmp/.tmpKRl5jk/client_0.db" \
linera sync
```

## Troubleshooting

### getrandom Issue (RESOLVED ✅)
**Problem**: getrandom with "js" feature pulls in wasm-bindgen which Linera rejects
**Solution**:
- Downgraded to linera-sdk 0.15.7 (exact version match with microcard winner)
- Added `getrandom = { workspace = true }` to abi/Cargo.toml
- This forces all contracts to use workspace-defined getrandom with "custom" feature

### Deployment Issue (RESOLVED ✅)
**Problem**: "No such file or directory" when using `linera project publish-and-create`
**Solution**:
- Build both contract AND service binaries with `--features service`
- Use `linera publish-module` with explicit WASM file paths
- Must specify LINERA_KEYSTORE in addition to LINERA_WALLET and LINERA_STORAGE
