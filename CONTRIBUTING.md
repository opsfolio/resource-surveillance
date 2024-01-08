# Contributing to Surveilr

All commits to `main` should first go through a PR. All CI checks should pass
before merging in a PR and the PR titles should follow
[Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).

## Development environment

Contributing to Surveilr requires that you have Rust and Cargo installed, along with
some additional dependencies:

- [Just](https://just.systems/man/en/chapter_1.html)
- [Deno](https://docs.deno.com/runtime/manual/getting_started/installation)
  
## Testing

### Unit Tests

Unit tests attempt to written functions of `surveilr`.

```shell
just test
```

When writing unit tests, aims to keep the scope small with a minimal amount of
setup required.

### End To End Tests

These are tests executed against a running instance of `surveilr`.

```shell
just test-e2e
```

## Pushing your code

Before pushing, ensure your code is well formatted by running

```shell
just fmt
```

and then

```shell
just lint
```
