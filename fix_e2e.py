#!/usr/bin/env python3
"""Replace exponential backoff sleep in e2e_test.rs with immediate retry pattern"""

lines = open('tests/integration/e2e_test.rs.backup', 'r').readlines()

# Lines 207-210 (1-indexed) = indices 206-209
before = lines[:206]
after = lines[210:]

replacement = [
    '                Err(_) if login_attempt < 10 => {\n',
    '                    // Account not yet created - retry immediately (polling pattern replaces backoff)\n',
    '                    continue;\n',
    '                }\n'
]

with open('tests/integration/e2e_test.rs', 'w') as f:
    f.writelines(before + replacement + after)

print('✓ E2E test updated: Removed exponential backoff sleep at line 209')
