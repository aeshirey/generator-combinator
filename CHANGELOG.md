## [0.4.0] - 2022-04-16
### Changed
- Made `rand` optional by default; use `features = ["with_rand"]` to enable the `random` function.
- `visit_one` now passes ownership of the `String` instead of a `&str`. It doesn't need to maintain ownership, and if the caller wants it, this may avoid unnecessary clones.
- Changes for Clippy.

### Added
- `impl From<String> for Generator`: you could previously say `let g : Generator = "hello".into();` for `&str` and can now do so with a `String`
- `Add<T>`, `AddAssign<T>`, `BitOr<T>`, and `BitOrAssign<T>` implementations for `char`, `&str`, and `String`; for example:
   ```rust
   let mut g = Generator::from("hello");
   g |= "salut";
   g += ',';
   g += " ";
   g += Generator::from("world") | "tout le monde" | "🌎";
   g += String::from("!");
   ```
- Extra test for a pattern that exceeds `u128` capacity.
- Two example programs in the `/examples/` folder.
- `Generator::Empty` variant that is basically a no-op and is also the value returned by the new `impl Default for Generator`.

### Fixed
- `BitOrAssign` for two OneOf variants incorrectly included an extra digit:
   ```rust
   let mut g = oneof!('a', 'b');
   g += oneof!('x', 'y');
   ```

## [0.3.0] - 2021-06-05
### Changed
- Renamed `Generator::generate_exact` to `Generator::generate_one` for generating a single string from a Generator
- Renamed `Generator::values` to `Generator::generate_all` for iterating over a Generator's range
- Renamed `Iter` to `IterString` for string iteration over a Generator's range, returned by `Generator::generator_all`

### Added
- `Generator::visit_one` to provide each part, in order, as a `&str`. For cases where the entire string isn't necessarily needed at once or at all, this avoids allocating and freeing memory.
- [quickcheck](https://github.com/BurntSushi/quickcheck) testing
- Additional tests, including of `Generator::regex`