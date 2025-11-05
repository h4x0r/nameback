# MSI Installer UI Improvements

## Problem Statement

The original MSI installer had poor user experience:
- No visible progress window during dependency installation
- Silent completion with no confirmation
- User confusion - "did it work?"
- No feedback about what was installed

## Solution Implemented

### 1. Added Full UI Dialog Sequence

**Changed:** `installer/nameback.wxs`

```xml
<UI>
  <!-- Use Mondo UI which shows progress, completion screen, and feature selection -->
  <UIRef Id="WixUI_Mondo" />

  <!-- Progress text for custom actions - [1] will be replaced with ActionData from script -->
  <ProgressText Action="InstallDependencies">Installing dependencies: [1]</ProgressText>
  <ProgressText Action="AddScoopToPath">Configuring system PATH: [1]</ProgressText>

  <!-- Customize the completion dialog -->
  <Publish Dialog="ExitDialog" Control="Finish" Event="DoAction" Value="LaunchApplication">
    WIXUI_EXITDIALOGOPTIONALCHECKBOX = 1 and NOT Installed
  </Publish>
</UI>
```

**Benefits:**
- Shows full installation wizard with Welcome, Feature Selection, Progress, and Completion screens
- Progress bar visible during dependency installation
- Completion dialog requires user to click "Finish"
- Optional checkbox to launch GUI immediately after install

### 2. Real-Time Progress Reporting

**Created:** `installer/install-dependencies-with-ui.ps1`

New PowerShell script that reports progress to MSI UI in real-time:

```powershell
function Report-Progress {
    param([string]$Message)

    # Log to file with timestamp
    Add-Content -Path $script:logFile -Value "[$timestamp] $Message"

    # Report to MSI using ActionData format
    # This appears in the progress dialog as "[1]" replacement
    Write-Host "1: $Message"
    [Console]::Out.Flush()
}
```

**How it works:**
1. PowerShell script captures nameback CLI output line-by-line
2. Filters for important messages (Installing, Downloading, etc.)
3. Reports each step to MSI via `Write-Host "1: <message>"`
4. MSI displays in progress dialog: "Installing dependencies: <message>"

### 3. Launch GUI Option

**Added:**
```xml
<Property Id="WIXUI_EXITDIALOGOPTIONALCHECKBOXTEXT" Value="Launch nameback GUI now" />
<Property Id="WixShellExecTarget" Value="[#namebackGUI]" />
<CustomAction Id="LaunchApplication" BinaryRef="Wix4UtilCA" DllEntry="WixShellExec" Impersonate="yes" />
```

**Benefits:**
- User can optionally launch the GUI immediately after installation
- Provides instant feedback that installation succeeded
- Smooth first-run experience

## User Experience Flow

### Before (Silent, Confusing):
1. User double-clicks MSI
2. Brief Windows Installer window appears
3. Window disappears
4. User confused: "Did it install? Where is it?"

### After (Visible, Clear):
1. User double-clicks MSI
2. Welcome screen appears
3. Feature selection screen (CLI, GUI)
4. Progress screen shows:
   - "Installing dependencies: Checking network connectivity..."
   - "Installing dependencies: Downloading exiftool..."
   - "Installing dependencies: Installing tesseract..."
   - etc.
5. Completion screen shows:
   - "Nameback has been installed successfully"
   - Checkbox: "Launch nameback GUI now"
   - "Finish" button
6. User clicks Finish (optionally launching GUI)

## Technical Details

### WixUI_Mondo vs Other UIs

**WixUI_Mondo:**
- Complete wizard UI with all screens
- Feature selection dialog (choose CLI and/or GUI)
- Progress bar during installation
- Completion dialog with customizable options
- No license file required (unlike WixUI_InstallDir)

**Alternatives considered:**
- `WixUI_Minimal` - Too minimal, no completion dialog
- `WixUI_InstallDir` - Requires license RTF file
- `WixUI_FeatureTree` - More complex than needed

### ActionData Progress Reporting

MSI's `ProgressText` element supports `[1]` placeholder for dynamic text:

```xml
<ProgressText Action="InstallDependencies">Installing dependencies: [1]</ProgressText>
```

PowerShell sends data via `Write-Host "1: <message>"`:
- MSI captures this output
- Replaces `[1]` with `<message>`
- Displays: "Installing dependencies: <message>"

This provides real-time feedback without complex COM interop.

### Log File Preservation

All progress messages are also logged to:
```
%TEMP%\nameback-msi-install-YYYYMMDD-HHMMSS.log
```

Benefits:
- Troubleshooting failed installations
- Verifying what was installed
- Support debugging

## Testing Checklist

Before release, verify:

- [ ] Installer shows Welcome screen on launch
- [ ] Feature selection allows choosing CLI/GUI
- [ ] Progress bar visible during installation
- [ ] Progress text updates in real-time during dependency install
- [ ] Messages like "Downloading exiftool..." appear
- [ ] Completion dialog shows success message
- [ ] "Launch nameback GUI now" checkbox works
- [ ] Clicking "Finish" closes installer
- [ ] If checkbox selected, GUI launches after Finish
- [ ] Log file created in %TEMP% directory

## Files Modified

1. **installer/nameback.wxs**
   - Added `<UIRef Id="WixUI_Mondo" />`
   - Added `<ProgressText>` elements for actions
   - Added launch checkbox configuration
   - Updated script reference to new UI-aware version

2. **installer/install-dependencies-with-ui.ps1** (NEW)
   - Replaces `install-dependencies.ps1` for MSI context
   - Real-time progress reporting via `Write-Host "1: <msg>"`
   - Filtered output (only important messages)
   - Full logging to temp file

## Future Enhancements

Potential improvements:
1. **Progress percentage** - Calculate actual % based on steps
2. **Dependency status dialog** - Show checkmarks for each installed dependency
3. **Retry on failure** - Allow user to retry failed dependency installs
4. **Custom branding** - Add banner/dialog background images
5. **Localization** - Support multiple languages

## References

- WiX Toolset Documentation: https://wixtoolset.org/docs/
- MSI ActionData: https://learn.microsoft.com/en-us/windows/win32/msi/actiondata
- WixUI Dialog Sets: https://wixtoolset.org/docs/tools/wixext/wixui/
