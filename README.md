## ebolg

My personal rust blog generator. Generate tailwind styled html from markdown files.

This is a work in progress and is not ready for use.

Your generated `tailwind.css` file should be at the root level of the target directory.

Example:
```
style/
  tailwind.css
```

## Usage

```bash
ebolg <FILE or DIRECTORY> <OUTPUT DIRECTORY>
```

## Examples 

```bash
ebolg . dist
```

```bash
ebolg README.md dist
```

```nix
nix run github:erictossell/ebolg -- . dist
```


