# Main branch protection

Status (2026-11-02): **active ruleset verified**. The maintainer imported the
template; GitHub reports [Protect main](https://github.com/Piyush-xo-19/Anti-ai-guard-/rules/26384686)
Active, targeting refs/heads/main with deletion, force-push, PR/thread and required
check rules. Committing a template alone does not enable protection. The activation
instructions below remain useful for another repository or recreating the rule.

## Intended rules

Import [.github/rulesets/main-protection.json](../.github/rulesets/main-protection.json).
It applies only to refs/heads/main, blocks force pushes/deletion, requires a PR,
requires resolved review threads and requires these exact checks:

| Check | Expected GitHub App |
| --- | --- |
| Windows fixtures (windows-2022) | GitHub Actions, integration 15368 |
| Windows fixtures (windows-2025) | GitHub Actions, integration 15368 |
| Shared contracts (Linux; no collector support claim) | GitHub Actions, integration 15368 |
| CodeRabbit | CodeRabbit, integration 347564 |

Names and integration IDs were read from this repository's check runs. There are
four unique contexts, even though push and PR triggers can display seven runs.
Keep the names synchronized with the workflow and app configuration.

The branch must be up to date before merging, and there are no bypass actors.
After main changes, merge the new base into a PR branch, push it and wait for the
fresh checks. Required human approvals are zero so a sole maintainer is not
locked out of authoring their own PRs; PR/check/thread requirements still apply.
Increase human approval requirements once a suitable independent reviewer is
available. This template neither enables signed-commit rules nor linear history;
the current workflow uses merge commits.

GitHub's accepted-check semantics are not a substitute for our review agreement:
inspect the actual CodeRabbit conclusion and findings. A cancelled/skipped review
is not approval under CONTRIBUTING.md. Do not bypass a missing or failed review.

## One-time activation by the repository admin

1. Open [repository ruleset settings](https://github.com/Piyush-xo-19/Anti-ai-guard-/settings/rules).
2. Choose New ruleset, then Import a ruleset.
3. Select this repository's .github/rulesets/main-protection.json file.
4. Review the target main branch, Active enforcement, empty bypass list, PR and
   check requirements, then create the ruleset.
5. Verify the ruleset is Active and applies to main; check the
   [public rules page](https://github.com/Piyush-xo-19/Anti-ai-guard-/rules).

Do not test force-push or deletion by attempting them against main. Read the
active rule settings and confirm the PR merge box reports the required checks.
A successful import/upload alone is not proof of activation; verify the saved
ruleset and enforcement state.

GitHub documents [rulesets](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/about-rulesets),
[JSON import](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/managing-rulesets-for-a-repository#importing-a-ruleset)
and the [REST schema](https://docs.github.com/en/rest/repos/rules#create-a-repository-ruleset).
For a public repository, this does not require upgrading just to use a branch
ruleset. Never create/share a personal token in chat to enable these settings.
