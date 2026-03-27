# FastRead

Speed reader that displays one word at a time with a focal point for optimal recognition. Single Rust binary, ~3MB.

**[Try it in your browser](https://merv1n34k.github.io/fastread/)**

## Install

Download the binary from [releases](https://github.com/merv1n34k/fastread/releases), or build and install from source:

```
make build
make install
```

## Usage

1. Paste text in the input area
2. Click the reader area to focus
3. Press Space to start

| Key | Action |
|-----|--------|
| Space | Play / pause |
| Left / Right | Previous / next word |
| Up / Down | Increase / decrease WPM |
| T | Toggle dark / light theme |
| Esc | Unfocus reader |

## Text preprocessing

Input text is cleaned for distraction-free reading:

- Brackets, markdown formatting (`*_~#`) removed
- Em-dashes and `--` split into separate words
- Ellipsis removed
- Repeated punctuation collapsed (`!!!` → `!`)
- Quotes preserved

## License

Distributed under MIT License. See `LICENSE` for more.
