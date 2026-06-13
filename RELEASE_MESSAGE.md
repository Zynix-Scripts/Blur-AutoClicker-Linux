# v3.7.0 | Linux Port

> [!IMPORTANT]
> This is a Linux build and as such will NOT work on Windows.

This release ports upstream **v3.7.0** to Linux, bringing the latest features and fixes to the Linux fork.

## New
- Added Sequence picking instead of a timer.
- Added Sequence picking showing dots where clicks will happen. They only show if you change the value, and will disappear after a few seconds.
- Added 1000 CPS mode with warning (click duration is clamped to 1% at >500cps and <99% at >50cps to allow those speeds).
- Added Keyboard auto-press support using the Linux uinput backend.
- Added Sequence clicking with configurable clicks per point and overlay-based point picking.
- Added System tray icon with Show/Quit menu.
- Added Confirm dialogs for Reset & Clear Stats actions.
- Added Custom Stop Zone failsafe.

## Fix
- Removed dynamic adjustment of the panel size of the hotkey field in simple mode to prevent a scroll bar from showing up.
- Refactored double click timing to work correctly with click duration / duty cycle.
- Fixed a Linux compile issue in the keyboard auto-press backend.

## Assets
- `BlurAutoClicker Linux_3.7.0_amd64.deb`
- `BlurAutoClicker Linux-3.7.0-1.x86_64.rpm`
- `BlurAutoClicker-3.7.0-x86_64.tar.gz` (portable archive)

> [!NOTE]
- Always on Top works on X11; on pure Wayland use your compositor's window rules.
- The overlay-based sequence picker works on both X11 and Wayland.
- Keyboard auto-press requires access to `/dev/uinput`. Make sure your user is in the `input` group.

> [!WARNING]
> `.deb` and `.rpm` packages are provided for convenience but may need testing on your specific distribution. If they don't install or run, use the portable `.tar.gz` or report it in the Blur Discord.
