# BackMeUp

## Prerequisites

### macOS
After running the setup script, grant `backup_program` the **Input Monitoring** permission:

1. Open `System Preferences`.
2. Select `Security & Privacy`.
3. In the `Privacy` tab, select `Input Monitoring`.
4. Add `backup_program` to the list of authorized applications.

### Linux
The `zenity` package must be installed for the program to function properly.  
If you want to compile the project, the following libraries are also required:

- ALSA development files:
    - Ubuntu/Debian:
        - `libasound2-dev`
    - Fedora:
        - `alsa-lib-devel`
- `libx11-dev`
- `libxi-dev`
- `libxtst-dev`

## Launching the Program

### Windows
1. Open `cmd` with administrator privileges.
2. Run the setup script using the command `./setup_windows.bat`.

### macOS/Linux
1. Open `Terminal`.
2. Run the setup script using the command `./setup_macos_linux.sh`.

## Backup Configuration
When the program starts, you will need to configure the following parameters:

1. **Source Path:** Specify the directory or disk from which you want to copy files.
2. **Destination Path:** Specify the directory or disk where files will be copied.
3. **Backup Type:**
    - **Directory:** Copies the entire folder specified in the source path.
    - **Selective:** Allows you to select specific file formats to copy (e.g., `.jpg`, `.txt`, etc.).
    - **Full-Disk:** Copies all data from the disk specified in the source path.

## Starting the Backup
To start the backup, perform the following gesture using the mouse:

    Draw a rectangle on the screen:
    - Start from the **bottom-left corner** of the screen.
    - Move to the **top-left corner**.
    - Move to the **top-right corner**.
    - Go down to the **bottom-right corner**.
    - Finally, complete the shape by moving to the **bottom-left corner** once again.

Once the command is recognized, a sound will confirm the action and a confirmation window will appear on the screen.

## Confirming, Canceling, or Reconfiguring the Backup
After the confirmation window appears, you can:

1. **Confirm the Backup:**
    - Perform a slide gesture **from left to right**, starting at the **bottom-left corner** and ending at the **bottom-right corner**.
    - A sound will confirm the selection.

2. **Cancel the Backup:**
    - Perform a slide gesture **upward**, starting at the **bottom-left corner** and moving to the **top-left corner**.
    - A sound will confirm the cancellation.

3. **Reconfigure the Backup:**
    - Draw a diagonal **from the bottom-left corner** to the **top-right corner**.
    - There will be no sound; instead, the initial configuration window will be shown again.

## Cleanup
To remove the program from the system, run the cleanup utility `uninstall_service`.