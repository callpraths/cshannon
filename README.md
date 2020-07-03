[![Build Status](https://travis-ci.com/callpraths/cshannon.svg?branch=master)](https://travis-ci.com/github/callpraths/cshannon)
[![fuzzit](https://app.fuzzit.dev/badge?org_id=callpraths-gh)](https://app.fuzzit.dev/orgs/callpraths-gh/dashboard)

# cshannon

    This is a pet project.
    All plans are funny money,
    code quality is (hopefully) reasonable,
    and readability is prioritized over efficiency.

A library of some early compression algorithms based on replacement schemes.

This library implements the standard [Huffman coding] scheme and two
precursors to the Huffman scheme often called [Shannon-Fano coding].

[Huffman coding]: https://en.wikipedia.org/wiki/Huffman_coding
[Shannon-Fano coding]: https://en.wikipedia.org/wiki/Shannon%E2%80%93Fano_coding

# Usage

cshannon provides a binary that can be used for compression / decompression
at the command line and a library that can be integrated into other projects.

Run `cshannon --help` to see the command-line options for the binary.

The easiest way to use cshannon library is:
```
use cshannon::{Args, run};

run(Args{
    command: "compress",
    input_file: "/path/to/input_file",
    output_file: "/path/to/output_file",
    tokenizer: "byte",
    encoding: "fano",
});
```