# cshannon

Compression and Decompression a la Shannon's algorithm.

This is a pet project. All plans are funny money, code quality is (hopefully)
reasonable, and readability is prioritized over efficiency.

## Sprints

* ~~Encode and decode sub-commands added. Encoding, decoding is trivial (input is
  mirrored as output).~~
  * ~~Integration test added to ensure round-trip results in no-diff.~~
* ~~Tokenizer:~~
  * ~~Input is tokenized into a stream of source language tokens.~~
    * ~~tokens == {english word, byte, unicode ... (to support hindi)}~~
  * ~~Token stream is transformed back to text.~~
* ~Model builder: Relative frequencies are computed for the input.~
* Code:
  * Way to collect code.Text -> vec<u8>
* Coder:
  * Create from model (start with trivial code)
  * Serialize to vec<u8>x
  * Create from vec<u8>
  * coder[token.Token] -> code.Letter
  * coder[code.Letter] -> token.Token
* File IO
  * Given vec<u8> for Coder and Code, write to file.
  * Given file, read vec<u8> for Coder and Code.
* Shanon coder:
  * Shannon coding scheme is computed.
* end-to-end encode:
    * A second pass on the input is used to code the input.
      * Coding scheme is included _at the beginning of output_.
* end-to-end decode:
  * Using the coding scheme included in the input, complressed stream is decoded
    back to source token stream, and then to source text via Tokenizerrr.
