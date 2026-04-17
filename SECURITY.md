# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 1.x.x   | Yes       |

## Reporting a Vulnerability

Please report security vulnerabilities by opening a **private** GitHub Security Advisory at:

  https://github.com/AIWithHitesh/unilang/security/advisories/new

Do **NOT** open a public issue for security vulnerabilities. Public disclosure
before a fix is available puts all users at risk.

## Response Timeline

- Acknowledgement within **48 hours** of submission.
- Fix or mitigation within **14 days** for critical issues.
- Coordinated public disclosure once a fix is available.

## Scope

The following are in scope for security reports:

- The UniLang VM (`crates/unilang-runtime/`)
- The UniLang compiler pipeline (lexer, parser, semantic analysis, codegen)
- The UniLang CLI (`crates/unilang-cli/`)
- The driver ecosystem (`crates/unilang-driver-*/`)
- The standard library and builtin functions

Out-of-scope items include: third-party dependencies (report those upstream),
documentation typos, and issues that require physical access to the host machine.

## Security Model

For a full description of the VM's security guarantees and how to run untrusted
code safely, see [docs/security/SECURITY_MODEL.md](docs/security/SECURITY_MODEL.md).
