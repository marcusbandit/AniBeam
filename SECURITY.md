# Security

## Reporting a problem

If you found a way AniBeam could harm the people who use it, report it privately: open the
repo's **Security** tab and click **Report a vulnerability**, or go straight to
<https://github.com/marcusbandit/AniBeam/security/advisories/new>. Only you, the maintainers
and anyone you add to the report see what you send there.

Please do not open a public issue, discussion or pull request about it first. A public
report shows everyone the problem before a fixed build exists. One that lands in a public
issue anyway gets hidden, and you are asked to send it through the form.

Useful things to include: the AniBeam version (Settings, the Data tab, under About; on
Linux `anibeam --version` prints it too), the platform, what an attacker needs (a file in the library, a feed or endpoint repo they control,
a machine on the same tailnet), and the steps that show it. Leave out your own tokens,
AniList or MyAnimeList credentials and home folder paths.

## What happens next

- You get a reply on the report within a week, saying whether it reproduces.
- A confirmed problem gets a fix that says as little as it can about the problem. It lands on
  main like any other change, and the first nightly that carries it is promoted to a stable
  release right away instead of after the usual soak, or it ships as a hotfix for a shipped
  version. The advisory goes public once that release is out, and credits you unless you would
  rather it did not.
- If it turns out not to be a security problem, you are told why, and asked to file it as a
  normal issue if it is still a bug.

## What counts

Anything that lets someone other than the user read their data, run code on their machine,
or act as them: the tracker sign-ins, the secrets file, the Share over Tailscale host, update
installs, endpoint repo signatures, and files AniBeam reads from the library or downloads.

Supported are the newest desktop release and the Nightly track, and the newest phone release
(the `android-v` ones). A problem in an older version is fixed in the next one.
