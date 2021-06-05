## [0.3.0] - 2021-06-05
### Changed
- Renamed `Generator::generate_exact` to `Generator::generate_one` for generating a single string from a Generator
- Renamed `Generator::values` to `Generator::generate_all` for iterating over a Generator's range
- Renamed `Iter` to `IterString` for string iteration over a Generator's range, returned by `Generator::generator_all`

### Added
- `Generator::visit_one` to provide each part, in order, as a `&str`. For cases where the entire string isn't necessarily needed at once or at all, this avoids allocating and freeing memory.
- [quickcheck](https://github.com/BurntSushi/quickcheck) testing
- Additional tests, including of `Generator::regex`