# nananiji-calculator

Generate "(mathematical) nananiji  expression" for any given integers.

## Getting Started

### Installing

```bash
$ git clone git://github.com/komori-n/nananiji-calculator.git
$ cargo install --path .
```

### Usage

You can calculate nananiji expression for any integers.

```bash
$ nananiji-calculator 3463
((2+2*7)*227-((22-7)+(22*7))) = 3463
$ nananiji-calculator -- -3463
((((2-2)/7)-(2+2*7)*227)+((22-7)+(22*7))) = -3463
```

You can also calculate hanshin expression or kyojin expression.

```bash
$ nananiji-calculator -l hanshin 3463
(((3+3-4)+(3/3+4))*(3+3/4)*(33*4)-(3+3-4)) = 3463
$ nananiji-calculator -l kyojin 3463
(((2-(6-4))-((2+6)/4))*(((2-6)*4)-(26/4)*264)-(2/(6-4))) = 3463
```

For more information, see command help.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
