# Proposed FlatPark pull request

Status: prepared for David's approval; not submitted.

## Title

Allow Leaf's requested missing-font downloads

## Body

Leaf alpha.45 downloads missing fonts and their licenses after an explicit user
action, then opens the actual font file in the system viewer for installation.
This adds the network permission that workflow needs and describes it in the
English and Spanish permission text. Document access continues through portals.

The normal updater selected the official alpha.45 archive: SHA-256
138cdb67333384a086cd56190d5fe2f146c2202708fa6ac75751c8db31199203,
14,821,544 bytes. Its release notes and the incident entry are included. The
application bytes, wrapper and GNOME 51 runtime are unchanged.

Validation of commit 32c2bcaf41f3d99c4b967d29e1f1cac85792aa96:

| Check | Result |
| --- | --- |
| Descriptor audit | Passed |
| Pinned download and system-style unpack as root without capabilities | Passed |
| Build and install from a local signed repository using publish.sh --verify | Passed |
| Effective network permission; no static document or home grant | Passed |
| Per-app data survives separate processes | Passed |
| Poppins Regular, Bold, Italic and Bold Italic: download, real viewer Install, exact host bytes and live rendering | Passed |
| Normal startup, captured window and window tree | Passed; capture inspected |

[Installed package validation and evidence](https://github.com/frandavid100/leaf/actions/runs/43173821014/job/131033158433).
These checks used the unmodified upstream release, not the separate Leaf source
branch that fixes refreshing another tab's font. No personal GNOME OS or Bazaar
test is claimed.

When a user-font directory is created after a Flatpak has started, that process
can need to be closed and opened again to see the new directory. This change
does not add filesystem overrides or install a private font collection.
Leaf's follow-up tracks that first-use recovery separately in
[#279](https://github.com/frandavid100/leaf/issues/279).

This is an existing developer-approved listing. Prepared with AI assistance for
Leaf's maintainer.
