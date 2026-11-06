# Security

## Reporting a problem

If you found a way AniBeam could harm the people who use it, report it privately: open the
repo's **Security** tab and click **Report a vulnerability**, or go straight to
<https://github.com/marcusbandit/AniBeam/security/advisories/new>. Only the maintainers see
what you send there.

Please do not open a public issue, discussion or pull request about it first. A public
report shows everyone the problem before a fixed build exists.

Useful things to include: the AniBeam version (`anibeam --version` prints it), the
platform, what an attacker needs (a file in the library, a feed or endpoint repo they control,
a machine on the same tailnet), and the steps that show it. Leave out your own tokens,
AniList or MyAnimeList credentials and home folder paths.

## What happens next

- You get a reply on the report within a week, saying whether it reproduces.
- A confirmed problem is fixed in the temporary private fork GitHub makes for the advisory,
  and ships in a release before the advisory goes public. The advisory credits you unless you
  would rather it did not.
- If it turns out not to be a security problem, you are told why, and asked to file it as a
  normal issue if it is still a bug.

## What counts

Anything that lets someone other than the user read their data, run code on their machine,
or act as them: the tracker sign-ins, the secrets file, the Share over Tailscale host, update
installs, endpoint repo signatures, and files AniBeam reads from the library or downloads.

AniBeam only supports its newest release and the nightly track. A problem in an older version
is fixed in the next one.
