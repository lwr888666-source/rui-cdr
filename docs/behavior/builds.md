# Build and package checks

The `build` workflow runs the complete workspace test suite on Linux,
then compiles the Windows, macOS, Linux and browser applications with
Rust 1.97.0 and the checked-in lockfile. The `Windows package` manual
entry point calls this same workflow, so pushes do not start duplicate
Windows builds. Failed jobs retain their dependency cache for retries.

Cargo packages and executable names are `rui` and `rui-cli`.
Native package scripts, installer source paths, the browser loader and
release artifact patterns use these names too. Shell scripts are checked
out with LF line endings and called through Bash from the workflow.

Unit tests treat native file and folder dialogs as cancelled. This lets
the dialog tests press Enter without opening a system dialog that needs
a person to dismiss it. File reads, writes and export conversions still
run normally. The cancellation path is covered by
`files::tests::system_dialogs_cancel_in_unit_tests_without_running_callbacks`.
Help-source checks normalize line endings before extracting topics.

Windows packaging verifies both output files and every required portable
member, including its nonzero size. Regression tests exercise valid,
missing and empty members and a missing installer. The real installer is
then run silently in a temporary directory on the Windows build machine.
Installed executable hashes must match the portable copies. Both command
line programs must convert an original SVG sample to PDF successfully,
and the installed copy is uninstalled afterwards.

Release publication requires the installer and portable archive to exist.
A build that has only compiled the programs has not passed these package
checks. The browser build separately verifies its Wasm size and version
substitutions before uploading the static page.
