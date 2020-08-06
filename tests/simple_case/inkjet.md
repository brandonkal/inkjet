## build//default

> A test to check implicit execution of default when calling inkjet without arguments.

```
echo "expected output"
```

## echo (name) (optional=default)

> Echo something interactively

**OPTIONS**

- flags: --num |number| A number

```
echo "Hello $name! Optional arg is $optional. Number is $num"
```

## extras (extra...?)

> Test multiple optional values for extra

```
echo "Hello $extra"
```
