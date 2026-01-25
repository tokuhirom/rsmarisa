# Binary Compatibility Investigation

## Current Status

### Working
- ✓ All 314 library tests pass
- ✓ File size matches C++ (4144 bytes)
- ✓ Cache struct fixed (enum → union, 12 bytes)
- ✓ Key sorting works correctly (lexicographic order)
- ✓ Simple cases work: ["app", "apple"]

### Issues
- ✗ Crash pattern: Words starting with "a" after "a" exists
- ✗ Missing words in full test: "application", "banana", "band", "bank"
- ✗ Integer underflow in tail.rs:308: `let start_offset = offset - query_pos;`

## Test Results

### Passing Cases
```rust
["app", "apple"]  // ✓ Both found
```

### Failing Cases
```rust
["a", "app"]         // ✗ "a" found, "app" crashes
["a", "apple"]       // ✗ "a" found, "apple" crashes
["a", "app", "apple"] // ✗ "a" found, "app" crashes
```

## Root Cause Analysis

### The Problem
When tail.rs:308 executes `let start_offset = offset - query_pos;`, it assumes `offset >= query_pos`.
This assumption is violated, causing integer underflow.

### Why It Fails
1. "a" is inserted and becomes a terminal node
2. "app" is inserted, shares prefix "a" with first entry
3. Tail offset for "app" is calculated incorrectly (likely 0 or too small)
4. During lookup of "app":
   - Match 'a' in trie nodes → query_pos = 1
   - Follow link to tail with offset = 0 (or small value)
   - Try to calculate `start_offset = 0 - 1` → UNDERFLOW

### Expected Behavior
The tail should store the continuation "pp" for "app", and the offset should point to where "pp" begins in the tail buffer. The match function should then be able to calculate the correct start position.

## Code Flow

### Build Process
1. `build_trie_key()` creates two terminal vectors:
   - `terminals` - for current level (terminal node IDs)
   - `next_terminals` - for next level (tail offsets or next-trie terminals)

2. `build_current_trie_key()` processes keys, sets `terminals[orig_id] = node_id`

3. `build_next_trie_key()` either:
   - Builds next trie level (recursively), OR
   - Calls `tail.build()` which OVERWRITES `next_terminals` with tail offsets

4. `tail.build()` computes offsets: `offsets[orig_id] = tail_buffer_offset`

5. Back in `build_trie_key()`, `next_terminals` values are stored in `bases` and `extras`:
   ```rust
   self.bases[node_id] = (next_terminals[i] % 256) as u8;
   next_terminals[i] /= 256;
   self.extras.build(&next_terminals);
   ```

### Lookup Process
1. Traverse trie nodes, matching characters
2. When link_flag is true, extract offset from bases/extras
3. Call `tail.match(agent, offset)`
4. tail.match() does: `start_offset = offset - query_pos`
5. Match remaining string from tail buffer

## Next Steps

1. **Add debug tracing** to build_current_trie_key for ["a", "app"]:
   - Log how keys are grouped into w_ranges
   - Log what key_pos values are calculated
   - Log what gets stored in bases/link_flags
   - Log what goes into next_key_data

2. **Add debug tracing** to tail.build():
   - Log the sorting order of entries
   - Log calculated offsets for each entry
   - Log what gets stored in tail buffer

3. **Compare with C++** byte-by-byte:
   - Extract bases/extras/tail from both implementations
   - Identify exactly which values differ

4. **Write focused tests** for each component:
   - Test common prefix finding
   - Test w_range grouping
   - Test tail offset calculation
   - Test bases/extras encoding

## Known Differences from C++

### File Structure
Both implementations produce 4144 byte files, but content differs in:
- L2 extras (offset 0x023a)
- L3 louds (offset 0x02a1)
- L3 link_flags (offset 0x0318)
- L3 extras (offset 0x0361)
- L3 tail (offset 0x03a2)

### Key ID Assignment
C++ and Rust assign different key_ids to the same words, suggesting different trie structures are being built despite identical top-level sorting.

## References

- C++ implementation: `/home/tokuhirom/work/marisa-trie`
- Test data: 15 English words (a, app, apple, application, apply, banana, band, bank, can, cat, dog, door, test, testing, trie)
- Crash location: `src/grimoire/trie/tail.rs:308`
- Related tests: `tests/minimal_failing_test.rs`, `tests/basic_lookup_test.rs`
