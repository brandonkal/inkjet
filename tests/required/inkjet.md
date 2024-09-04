## required_val

**OPTIONS**

- val
  - flag: --val
  - type: string
  - required

```bash
echo "Value: $val"
```

```powershell
param (
    $in = $env:val
)
Write-Output "Value: $in"
```

## another (necessary)

```
echo You entered $necessary
```
