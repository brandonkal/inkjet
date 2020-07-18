#!/usr/bin/env inkjet

# main

> This is the main info

## echo

```
echo "Hello"
```

<!-- In this example, the contents below were imported from a seperate file. The h1 title `second` is parsed as an h2 and so on. -->

<!-- inkfile: ./tests/inkjet.md -->

# second

## echo (word=Second)

When imported, this command would be called as `inkjet second echo`.

> Echo the word provided

```
echo "$word"
```

# third

> A third file with a ping command

## ping

```
echo "pong"
```
