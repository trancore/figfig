# figfig Terminal for JetBrains

This macOS adapter shares the focused terminal's cursor geometry with figfig. The CLI and popup helper must be installed separately. Terminal text and keystrokes stay in the IDE and zsh. Only composition length is observed to suppress suggestions during IME conversion.

Build against an installed IDE and its bundled JDK:

```sh
python3 scripts/build-jetbrains-plugin.py --ide '/Applications/IntelliJ IDEA.app'
```

Install `target/editors/figfig-terminal.zip` using Settings → Plugins → gear → Install Plugin from Disk. Restart the IDE and open a project and a local zsh terminal. Run `figfig doctor` in that terminal for diagnostics. Removing the plugin in Settings removes the adapter on restart.

The plugin targets platform build 243 or later with `org.jetbrains.plugins.terminal`. Classic and Reworked are targets; deprecated Experimental 2024 is excluded. See [compatibility](../../docs/reference/compatibility.md) for verified behavior and remaining real-IDE acceptance checks.

The adapter binds `jb-<IDE PID>.sock` in the same private runtime directory as figfig. An override of `FIGFIG_RUNTIME_DIR` must be supplied to both the IDE and figfig. The application service owns the socket and closes it on disposal. Requests have bounded size and timeouts. Swing geometry is captured on the EDT, with distinct component identities for separate terminal panes. JediTerm's zero-height position is converted into a row rectangle using font metrics.

Run the socket/geometry fixture against the real compiled adapter:

```sh
FIGFIG_JETBRAINS_IDE='/Applications/IntelliJ IDEA.app' python3 -m unittest discover -s tests -p jetbrains_test.py -v
```

The fixture supplies only terminal geometry, and uses a temporary Unix socket instead of a running IDE. Compilation uses the actual IDE platform libraries. GUI acceptance is a separate check.
