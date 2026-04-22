> [!WARNING]
> This does not currently work and is still a WIP. the current code here is windows only until I update this branch again (which will be very soon.)
# Blur Auto Clicker

<div align="center">
    <img src="https://github.com/Blur009/Blur-AutoClicker/blob/main/public/V3.0.0_UI.png" width="600"/>
</div>
<p align="center"><em>An accuracy and performance focused auto clicker</em></p>


## Why blur made it

A lot of the most popular auto clickers like OP Auto Clicker and Speed Auto Clicker are pretty inaccurate at higher speeds. Setting CPS to 50 might give you 40.. or 60. Technically this is not an issue since they are still clicking _fast_, but I am a perfectionist and I wanted something that could actually click at the CPS I set it to, even at higher speeds. So I made this.

Additionally, many auto clickers have 1 good feature but are missing the other features I want. My auto clicker combines all the featurs I have seen in other auto clickers, and even adds some of my own ideas.

Performance is also a heavy focus of this. While version 3 and onwards run on a web based UI, The total RAM usage is around 50mb and I intend for it to never go above 100mb.

---

## Features

<div align="center">
    <img src="https://github.com/Blur009/Blur-AutoClicker/blob/main/public/30s_500cps_Speed_Test.png" width="600"/>
</div>
<p align="center"><em>Blur Auto Clicker reaching 500 CPS steadily (windows limit¹)</em></p>

Simple Mode:
- On / Off Indicator (blur logo turns green when active)
- Individual mouse button settings (left, right, middle)
- Hold / Toggle activation modes
- customizable hotkeys

Advanced Mode (includes all simple mode features plus):
- adjustable click timing (duty cycle)
- Speed Range Mode (randomizes CPS within a range)
- Corner and edge stopping (turns off when mouse is in corners or near edges of the screen)
- Click and Time limits (stop after certain amount of clicks or time)
- Double clicks
- Position Clicking (you can pick a position where the mouse will move to and click.)
- Clicks can be adjusted to per Second, Minute, Hour, or even Day
  
Other Features:
- click stats (total clicks, clicks per second, etc)
  
---

## Installation

1. download from releases  
2. run the installer
3. run the program (default location is `%localappdata%/BlurAutoClicker/BlurAutoClicker.exe`)

config and stats are stored in `%appdata%/BlurAutoClicker`

Info: If you are on version 2.1.2 or below, delete the old executable (the installer will not delete it for you).
The old Config and Stats files will unfortunately not be compatible with the new versions (3.0.0 and above), so they will be deleted upon launching the application.

---

## license

GPL v3 https://www.gnu.org/licenses/gpl-3.0.en.html#license-text

## Support the main project
Ko-fi: https://ko-fi.com/blur009

You can also support the linux porting project by starring the repository and sharing it with your friends :). Thank you for your support!

## Other Info
### Windows
1. Windows has a limit of around 500 CPS for mouse events. This is because Windows timer resolution is limited to about 1ms at minimum (1000cps) but windows also needs to do other things, so the practical limit is around 800cps, but since I can not guarantee that performance on every computer, I set the limit to 500cps.
### Linux
TBD
