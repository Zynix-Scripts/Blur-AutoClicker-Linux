# v3.7.2 | Linux Port

> [!IMPORTANT]
> This is a Linux build and as such will NOT work on Windows.

This release ports upstream **v3.7.2** to Linux, including all v3.7.1 and v3.7.2 fixes and features that work on Linux.

## New
- Added Sequence picking instead of a timer.
- Added Sequence picking showing dots where clicks will happen. They only show if you change the value, and will disappear after a few seconds.
- Added 1000 CPS mode with warning (click duration is clamped to 1% at >500cps and <99% at >50cps to allow those speeds).
- Added Keyboard auto-press support using the Linux uinput backend.
- Added Sequence clicking with configurable clicks per point and overlay-based point picking.
- Added System tray icon with Show/Quit menu.
- Added Confirm dialogs for Reset & Clear Stats actions.
- Added Custom Stop Zone failsafe.
- Added scrolling on drop-down fields.
- Added Shift and Shift + Ctrl scrolling on number fields for bigger increments (5 and 10).
- Added "Check for update" button in Settings.
- Added changelog in the Settings page.

## Fixed
- Added scroll blocking for Sequence clicking items so wheel events don't change input values.
- Removed text weight difference for light mode.
- Added bundled font to the overlay so it no longer defaults to Arial.
- Fixed hotkey self-triggering during auto-clicks by flagging synthetic input and filtering it out in hotkey detection.
- Fixed stop reason repeating when switching between simple and advanced mode.
- Removed dynamic adjustment of the panel size of the hotkey field in simple mode to prevent a scroll bar from showing up.
- Refactored double click timing to work correctly with click duration / duty cycle.
- Fixed Linux keyboard auto-press compile issue.

## Assets
- `BlurAutoClicker Linux_3.7.2_amd64.deb`
- `BlurAutoClicker Linux-3.7.2-1.x86_64.rpm`
- `BlurAutoClicker-3.7.2-x86_64.tar.gz` (portable archive)

> [!NOTE]
- Always on Top works on X11; on pure Wayland use your compositor's window rules.
- The overlay-based sequence picker works on both X11 and Wayland.
- Keyboard auto-press requires access to `/dev/uinput`. Make sure your user is in the `input` group.

> [!WARNING]
> `.deb` and `.rpm` packages are provided for convenience but may need testing on your specific distribution. If they don't install or run, use the portable `.tar.gz` or report it in the Blur Discord.
