# Changelog

## 0.2.1

### Added

- `Collector::skip()`.

## 0.2.0

### Changed

- `[___]Ref` collectors have their constructors moved to their non-`Ref` verions.

### Fixed

- `Chain::collect_ref()` returning `Break` when the first collector stops.

## 0.1.2

### Added

- `Collector::unbatching()`.
- `Product`.
- Implementations for `Sum<Wrapping<Int>>` and `Product<Wrapping<Int>>`.
- `Fold(Ref)`.

### Changed

- Optimized `Take::collect_many()`.

### Fixed

- Added doc for `Funnel` (struct).

## 0.1.1

### Added

- `ConcatStr(ing)::with_buf()` that initializes `ConcatStr(ing)` with an initial string.
- `Sink`.
- `Collector::take_while()`.
- Doc for `RefCollector::funnel()`.

## 0.1.0

### Added

- This crate.
