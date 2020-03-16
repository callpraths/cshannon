# cshannon

Compression and Decompression a la Shannon's algorithm.

This is a pet project. All plans are funny money, code quality is (hopefully)
reasonable, and readability is prioritized over efficiency.

## Sprints

* ~~Encode and decode sub-commands added. Encoding, decoding is trivial (input is
  mirrored as output).~~
  ~~* Integration test added to ensure round-trip results in no-diff.~~
* Tokenizer: 
  * Input is tokenized into a stream of source language tokens.
    * tokens == {english word, byte, unicode ... (to support hindi)}
  * Token stream is transformed back to text.
* Encoder:
  * Model builder: Relative frequencies are computed for the input.
  * Coding scheme builder: Shannon coding scheme is computed.
  * Generic Coder:
    * A second pass on the input is used to code the input.
      * Coding scheme is included _at the beginning of output_.
* Generic Decoder:
  * Using the coding scheme included in the input, complressed stream is decoded
    back to source token stream, and then to source text via Tokenizerrr.
