Phase 00 focus: package/source closure.

Run package scanners first. If any referenced file is missing, restore it or restore the correct test fixture from the handoff. Do not delete references/tests to make the scanner green.

Required proof:
- package integrity scanner passes;
- source cross-reference scanner passes;
- agency eval fixture validates;
- p21 verify script reaches expected state.
