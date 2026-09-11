# Desktop-control references

`cua-driver/` contains plugin-maintained guidance adapted from
[Cua](https://github.com/trycua/cua). Keep its [MIT license](cua-driver/LICENSE.md)
and attribution when editing. The executable is installed separately.

The [desktop-control entrypoint](../skills/desktop-control/SKILL.md) owns plugin
routing, setup, and evidence handoff. Load its linked mechanics references on
demand; they are documentation, not another skill to install.

## Maintenance

Edit these documents with the plugin. Keep shared action rules in
[WORKFLOW.md](cua-driver/WORKFLOW.md), lifecycle and authorization in
[RUNTIME.md](cua-driver/RUNTIME.md), and host-specific details in the platform
guides. Check changed examples against the intended driver's live schemas and
behavior rather than assuming every installation supports the same features.

From the plugin directory, check that the documentation's links and anchors
resolve after relocation:

```bash
python3 -m unittest discover -s tests -p 'test_desktop_control.py'
```

This static packaging check needs no Cua installation, personal skill directory,
GUI access, or network. It does not certify live desktop or recording behavior.
