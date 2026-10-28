# packaging/linux

- `nfpm.yaml.in` — nfpm config template shared by the deb and the rpm (dependencies per format)
- `postinst.sh` — package post-install: system registration only (never touches `pcscd.socket`)
- `prerm.sh` — package pre-remove: system unregistration on real removals only
- `signlocal.desktop.in` — desktop entry template
- `signlocal.metainfo.xml.in` — AppStream metadata template
- `signlocal.svg` — placeholder icon (TODO(gustavo): final logo)
