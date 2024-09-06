## build//default

> A test to check implicit execution of default when calling inkjet without arguments.

```
echo "expected output"
```

## echo (name) (optional=default) (not_required?) -- (extras...?)

> Echo something interactively

**OPTIONS**

- flag: --num |number| A number
- flag: --required -r |string| required This must be specified
- flag: --any |string| Anything you want

```bash
echo "Hello $name! Optional arg is \"$optional\". Number is \"$num\". Required is \"$required\". Any is \"$any\". extras is \"$extras\""
```

## extras (extra...?)

> Test multiple optional values for extra

```
echo "Hello $extra"
```
