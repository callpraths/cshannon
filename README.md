[![Build Status](https://travis-ci.com/callpraths/cshannon.svg?branch=master)](https://travis-ci.com/github/callpraths/cshannon)

# cshannon

Compression and Decompression a la Shannon's algorithm.

This is a pet project. All plans are funny money, code quality is (hopefully)
reasonable, and readability is prioritized over efficiency.

## TODO

### Implemenation

* Implement Fano encoding scheme
* Implement Huffman encoding scheme

### Hygiene

* Create crate and releases.

### Learn

* Get help from rust-users etc to make the `Tokens` trait cleaner.
* micro-benchmarks: Write some, make it faster!
* fuzz testing: Write some, find bugs!

### Blog

* Write Blog post about use of private tests for encapsulated packages
* Concretize ideas for data viz blog post comparing Fano, Shannon and Huffman
  encodings
  * [extra credit] WASM compilation of cshannon to allow users to input text.

### Refactors

* Accept vector of pairs in `model::with_frequencies()`  to reduce boilerplate
  at call sites.
* Accept vector of pairs to create `Encoding` in tests and implement `Eq` to
  reduce assertion boilerplate in tests.
* Deduplicate cumulative probability computation in `model::balanced_tree` vs
  `model::fano`
* Make `model::fano::Window` more readable by replacing tuple with struct.