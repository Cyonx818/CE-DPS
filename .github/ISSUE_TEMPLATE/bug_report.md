---
name: Bug Report
about: Create a report to help us improve CE-DPS
title: ''
labels: bug
assignees: ''

---

**Bug Description**
A clear and concise description of what the bug is.

**Component**
- [ ] Methodology (phases, templates, documentation)
- [ ] Fortitude (knowledge management)
- [ ] Tools (quality-gates, phase-validator, fortitude-integration)
- [ ] Examples (authentication, API development)
- [ ] Documentation

**To Reproduce**
Steps to reproduce the behavior:
1. Go to '...'
2. Click on '....'
3. Scroll down to '....'
4. See error

**Expected Behavior**
A clear and concise description of what you expected to happen.

**Actual Behavior**
A clear and concise description of what actually happened.

**Screenshots/Output**
If applicable, add screenshots or command output to help explain your problem.

**Environment**
- OS: [e.g. Ubuntu 22.04, Windows 11, macOS 13]
- Rust Version: [e.g. 1.70.0]
- Python Version: [e.g. 3.11.0]
- Claude Code Version: [if applicable]

**Quality Gates Output**
If relevant, include output from:
```bash
cargo run --bin quality-gates
```

**Phase Validator Output**
If relevant, include output from:
```bash
python tools/phase-validator.py --phase [1|2|3]
```

**Additional Context**
Add any other context about the problem here.

**Workaround**
If you found a workaround, please describe it here.