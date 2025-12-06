# Changelog

## 0.3.0

### Added

- Unstable: Aggregate API.
- `prelude` module.
- `CollectorByRef` and `CollectorByMut`.
- `(Max:Min)::by(_key)()`.

### Changed

- `()`, `Vec`, `String` and collections in the standard library now implements
  `IntoCollector` instead of `(Ref)Collector`.
- `IntoCollector` for `&(Sync)Sender`.
- `BetterCollect`'s methods now receive an `IntoCollector` instead.
- `(Ref)Collector` are now dyn compatible.
- `Copied` is renamed to `Copying`.
- `Cloned` is renamed to `Cloning`.
- `Then` is renamed to `Combine`.

### Fixed

- Feature flags.

### Deprecated

- `(Max|Min)By(Key)::new()`.
- `Copied`.
- `Cloned`.
- `Then`.

## 0.2.2

### Fixed

- Unintended recursion in `Take::collect_many()`.
- `Unzip::collect(_ref)()` should not return `Break` early on one returning `Break` to be consistent
  with `Unzip::collect_many()` and `Unzip::collect_then_finish()`.

## 0.2.1

### Added

- `Collector::skip()`.
- `(Ref)Collector` implementations for `&mut C`, where `C: (Ref)Collector`.
- `IntoCollector`.
- `Collector`s for `std::sync::mpsc`.
- `unstable` feature flag.

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
