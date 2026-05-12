# api-merger

Universal API specification merging system for combining specifications across multiple formats.

## Usage

```rust
use api_merger::{SpecificationMerger, ConflictResolutionStrategy};
use api_core::SpecFormat;

let merger = SpecificationMerger::new(SpecFormat::OpenApi)
    .with_conflict_resolution(ConflictResolutionStrategy::FirstWins)
    .with_validation(true);

// Ready to merge specifications
```

## Features

- `SpecificationMerger` - Main merger with conflict resolution strategies
- `ConflictResolutionStrategy` - How to handle merge conflicts (FirstWins, LastWins, Error)
- `UnifiedApiSpec` - Unified specification holding any format
- `MergedApiSpec` - Result of merging with metadata
- `MergeConflict` - Information about merge conflicts
- Support for OpenAPI, GraphQL, and gRPC specification merging

## License

MIT
