# conjure

Build, deploy and manage sites and dashboards on
[Conjure](https://conjure.factory.ai), Factory's internal static-site platform.

The skill installs the `conjure` CLI when it is missing, walks the user through
the Okta browser sign-in, and stops if that sign-in does not complete. It then
builds and deploys the site, asking what to build when the request did not say.

Conjure is gated on Factory Okta, so the skill is only useful to Factory
employees. It is safe to publish here: it contains no credentials, and the CLI
it installs is already served publicly from `downloads.factory.ai`.

## Editing the skill

`skills/conjure/SKILL.md` is the source of truth. Edit it here, like every
other plugin in this repo.

It used to live in [`Factory-AI/conjure`](https://github.com/Factory-AI/conjure)
and be scaffolded into projects by `conjure init`. That copy was retired: a
scaffolded file is a frozen snapshot, and Droid ranks a project or user skill
file above a user-scope plugin skill, so it silently outranked this one and
served stale text with no warning. There is now exactly one copy.
