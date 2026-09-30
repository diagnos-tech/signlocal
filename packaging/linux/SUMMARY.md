# packaging/linux

- `nfpm.yaml.in` — nfpm config template shared by the deb and the rpm (dependencies per format)
- `postinst.sh` — package post-install: system registration and `pcscd.socket`
- `prerm.sh` — package pre-remove: system unregistration on real removals only
- `websign.desktop.in` — desktop entry template
- `websign.metainfo.xml.in` — AppStream metadata template
- `websign.svg` — placeholder icon (TODO(gustavo): final logo)
