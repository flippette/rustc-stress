# `rustc-stress`

stress testing with rustc.

## usage

```
rustc-stress -p <project_dir> -c <cores> -m <seq|par>
```

where:

- `project_dir` is a directory containing cargo projects
- `cores` is a comma-separated list of physical cores
- `seq|par` is either the `seq`uential or `par`allel stressing mode

## using with temp dirs

`rustc-stress` does not handle tempdirs itself, so you have to do this manually
(for now).

example: using with tmpfs (`/tmp`) on linux to avoid disk writes

```
rm -rf /tmp/stress && cp -r <projects> /tmp/stress && rustc-stress -p /tmp/stress
```
