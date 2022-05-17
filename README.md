# lcs-png-diff
Generate diff bitmap(PNG) by the Longest Common Subsequence algorithm. This project is inspired by [LCS-IMAGE-DIFF](https://crates.io/crates/lcs-image-diff). But it combined [LCS-IMAGE-DIFF](https://crates.io/crates/lcs-image-diff) and its dependency algorithm crate [LCS-DIFF](https://crates.io/crates/lcs-diff) into one.

Here is another well-written [article](https://florian.github.io/diffing/) about LCS diff algorithm.

## Features

- Only supports PNG images as input and output
- Supports a simple command-line interface
- Supports batch diff operation

## Example

- Diff single pair of png files

``` bash
lcs-png-diff \
    -b path/to/before.png \
    -a path/to/after.png \
    -d path/to/diff/result.png
```

- Diff multiple pairs of png files

``` bash
lcs-png-diff \
    -j path/to/pair.json
```

## The JSON schema of the batch diff operation input

``` json
{
    "$schema": "http://json-schema.org/draft-04/schema#",
    "type": "array",
    "items": [
        {
            "type": "object",
            "properties": {
                "before": {
                    "type": "string"
                },
                "after": {
                    "type": "string"
                },
                "result": {
                    "type": "string"
                }
            },
            "required": [
                "before",
                "after"
            ]
        }
    ]
}
```
`result` property is optional. If omitted, the result png file will be generated in the exact location of the before png file with the base name appended "_result". For example:
```json
[
  {
    "before": "tests/fixtures/pricing.png",
    "after": "tests/fixtures/pricing_after.png"
  },
  {
    "before": "tests/fixtures/slider.png",
    "after": "tests/fixtures/slider_after.png"
  },
  {
    "before": "tests/fixtures/text-area.png",
    "after": "tests/fixtures/text-area_after.png"
  }
]
```
Using the above JSON content as batch operation input will generate three files named pricing_result.png, slider_result.png, and text-area_result.png in relative path `tests/fixtures/fixtures/`.

## Benchmark

```
cargo criterion
```

## LICENSE

The MIT License (MIT)

Copyright (c) 2022 @jianliao

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
