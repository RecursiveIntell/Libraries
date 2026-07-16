# Agent output contract

Every agent returns:

1. task/issue IDs and exact branch/head;
2. changed files;
3. contract-level semantic change;
4. commands with pass/fail/skipped/blocked and receipt paths;
5. acceptance gate mapping;
6. residual risks and scope deviations;
7. rollback strategy/commands/preserved data;
8. reviewer focus.

Forbidden completion language: “done”, “fixed”, “all good”, or “release-ready” without the structured
evidence above. Only Hermes closes issues after merge and post-merge validation.
