## Checklist

- [ ] Signing changes reproduce `tests/vectors/vectors.json` byte-for-byte.
- [ ] Every public method added or changed has unit tests with mocked HTTP covering method, path, query, body, auth headers, and error mapping.
- [ ] Money-unit helpers remain 100% covered, including bad inputs, 6-decimal boundaries, huge values, and overflow returning errors instead of panicking.
- [ ] No floats are used for money.
- [ ] API errors preserve typed codes.
- [ ] Examples dry-run by default, and `SEND=1` refuses public test-vector keys.
- [ ] No secrets are committed; test keys are derived from public seeds only.
- [ ] README and CHANGELOG are updated when behavior changes.
