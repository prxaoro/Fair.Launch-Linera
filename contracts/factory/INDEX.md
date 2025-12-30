# Fair Launch Factory Contract - Documentation Index

Welcome to the Fair Launch Factory contract documentation. This contract is the core registry and launcher for the Fair Launch platform on Linera blockchain.

## Quick Start

1. **Read the Architecture**: Start with [README.md](README.md)
2. **See Examples**: Check [EXAMPLES.md](EXAMPLES.md)
3. **Deploy**: Follow [DEPLOYMENT.md](DEPLOYMENT.md)
4. **Review Implementation**: See [SUMMARY.md](SUMMARY.md)

## Documentation Structure

### Core Documentation

#### [README.md](README.md) - Architecture & Usage (396 lines)
- Contract architecture and design patterns
- Key components (State, Contract, Service)
- Security features
- Usage examples
- Data models
- Error handling
- Testing guide
- Performance considerations
- Deployment overview

**Read this first** to understand the overall architecture.

#### [EXAMPLES.md](EXAMPLES.md) - Code Examples (747 lines)
- Token creation examples (basic, advanced, minimal)
- GraphQL queries (8+ complete examples)
- Frontend integration (TypeScript/React)
- Python client examples
- Error handling patterns
- Batch operations
- Testing examples

**Use this** when you need copy-paste code examples.

#### [DEPLOYMENT.md](DEPLOYMENT.md) - Deployment Guide (509 lines)
- Prerequisites and setup
- Build process
- Deployment steps
- Multi-chain deployment
- Configuration
- Monitoring and alerting
- Integration testing
- Troubleshooting
- Upgrade procedures

**Follow this** for step-by-step deployment.

#### [SUMMARY.md](SUMMARY.md) - Implementation Details (366 lines)
- Complete overview of what was built
- Architecture decisions
- Security features
- API surface
- Performance characteristics
- Testing coverage
- Code quality metrics
- Dependencies
- Success criteria verification

**Read this** to understand implementation choices.

#### [CHECKLIST.md](CHECKLIST.md) - Verification Checklist
- 150+ requirement checkpoints
- Functional requirements verification
- Security checklist
- Testing checklist
- Production readiness sign-off

**Use this** to verify completeness.

### Source Code

#### [src/lib.rs](src/lib.rs) - Module Exports (25 lines)
Entry point exporting all public modules.

#### [src/state.rs](src/state.rs) - State Management (417 lines)
- `FactoryState` struct with Views
- Token registration and retrieval
- Creator registry
- Pagination
- Validation logic
- Error types
- 12+ unit tests

**Core data layer** - handles all storage operations.

#### [src/contract.rs](src/contract.rs) - Contract Logic (322 lines)
- `FactoryContract` implementation
- `CreateToken` operation
- Microchain spawning via `open_chain()`
- Cross-chain message handling
- Bonding curve validation
- Authentication
- 4 unit tests

**Business logic layer** - handles operations and messages.

#### [src/service.rs](src/service.rs) - GraphQL Service (317 lines)
- `FactoryService` implementation
- 8 GraphQL queries
- Pagination support
- Search functionality
- Statistics aggregation
- 1 unit test

**Query layer** - provides GraphQL API.

#### [src/tests.rs](src/tests.rs) - Test Suite (376 lines)
- 15+ comprehensive tests
- State operations
- Validation edge cases
- Pagination boundaries
- Complete lifecycle tests
- Integration scenarios

**Quality assurance** - ensures correctness.

### Build Tools

#### [build.sh](build.sh) - Build Script
Compiles contract to optimized WASM binary.

```bash
./build.sh
```

#### [test.sh](test.sh) - Test Script
Runs tests, clippy, and format checks.

```bash
./test.sh
```

#### [Cargo.toml](Cargo.toml) - Package Manifest
Dependencies and build configuration.

## Reading Guide

### For Developers Integrating the Factory

1. Read [README.md](README.md) sections:
   - Architecture
   - Usage
   - Data Models

2. Study [EXAMPLES.md](EXAMPLES.md):
   - GraphQL queries for your use case
   - Integration examples (TypeScript/Python)
   - Error handling patterns

3. Review [src/service.rs](src/service.rs):
   - Available queries
   - Response formats

### For Deploying the Contract

1. Follow [DEPLOYMENT.md](DEPLOYMENT.md) step-by-step:
   - Prerequisites
   - Build process
   - Deployment steps
   - Verification

2. Use [build.sh](build.sh) and [test.sh](test.sh)

3. Refer to troubleshooting section if issues arise

### For Understanding the Implementation

1. Read [SUMMARY.md](SUMMARY.md) for overview

2. Study source code in order:
   - [src/state.rs](src/state.rs) - Data layer
   - [src/contract.rs](src/contract.rs) - Business logic
   - [src/service.rs](src/service.rs) - Query layer

3. Review [src/tests.rs](src/tests.rs) for usage patterns

4. Check [CHECKLIST.md](CHECKLIST.md) for completeness

### For Security Auditing

1. Review [README.md](README.md) - Security Features section

2. Study [src/state.rs](src/state.rs):
   - `validate_metadata()` function
   - Input validation

3. Review [src/contract.rs](src/contract.rs):
   - `validate_curve_config()` function
   - Authentication checks
   - Message handling

4. Check [src/tests.rs](src/tests.rs):
   - Edge case tests
   - Validation tests

5. See [DEPLOYMENT.md](DEPLOYMENT.md) - Security Considerations

## Key Features Highlighted

### Multi-Chain Architecture
Each token gets its own microchain via `open_chain()`:
```rust
// In src/contract.rs
let (message_id, chain_id) = self
    .runtime
    .open_chain(ownership, token_application_id, ())
    .map_err(|e| {
        ContractError::ChainCreationFailed(format!("Failed to open chain: {}", e))
    })?;
```

### Message Tracking
Guaranteed delivery with `.with_tracking()`:
```rust
self.runtime
    .prepare_message(Message::TokenCreated { ... })
    .with_tracking()
    .send_to(token_chain_id);
```

### Rich Queries
8+ GraphQL queries for comprehensive data access:
- List all tokens (paginated)
- Get specific token
- Filter by creator
- Search by name/symbol
- Recent launches
- Graduated tokens
- Factory statistics

### Comprehensive Validation
Multi-layer validation ensures data integrity:
```rust
// Metadata validation
fn validate_metadata(metadata: &TokenMetadata) -> Result<(), FactoryError> {
    if metadata.name.trim().is_empty() { ... }
    if metadata.symbol.trim().is_empty() { ... }
    if metadata.name.len() > 100 { ... }
    // URL validation, etc.
}
```

## Statistics

### Code Metrics
- **Source Code**: 1,457 lines (5 files)
- **Documentation**: 2,018 lines (4 files)
- **Tests**: 376 lines (15+ test cases)
- **Total**: 3,475 lines

### Deliverables
- ✅ 5 production-ready source files
- ✅ 4 comprehensive documentation files
- ✅ 2 build/test utility scripts
- ✅ 15+ unit and integration tests
- ✅ Complete GraphQL API (8 queries)
- ✅ Multi-language examples (Rust, TypeScript, Python)

### Requirements Coverage
- ✅ 7/7 core requirements
- ✅ 150+ verification checkpoints
- ✅ 100% completion rate

## Support

### Documentation Issues
If you find unclear documentation, please:
1. Check all documentation files
2. Review code examples
3. See troubleshooting in DEPLOYMENT.md

### Contract Issues
For bugs or improvements:
1. Check CHECKLIST.md for known items
2. Review test suite for expected behavior
3. See error handling in code

### Deployment Help
For deployment issues:
1. Follow DEPLOYMENT.md step-by-step
2. Check troubleshooting section
3. Verify prerequisites

## Next Steps

1. **New to the project?** → Read [README.md](README.md)
2. **Want to integrate?** → See [EXAMPLES.md](EXAMPLES.md)
3. **Ready to deploy?** → Follow [DEPLOYMENT.md](DEPLOYMENT.md)
4. **Need technical details?** → Check [SUMMARY.md](SUMMARY.md)

## License

MIT - See repository root for full license.

## Version

Factory Contract v0.1.0 - Production Ready
Compatible with Linera SDK 0.15.8

---

**Last Updated**: 2025-12-29
**Status**: Production Ready ✅
