#!/usr/bin/env python3
"""
File Orchestrator - Dependency Checker
Verifies all prerequisites before installation
"""

import subprocess
import shutil
import sys
import os
import platform

# ANSI Colors for output
class Colors:
    RED = '\033[0;31m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    BLUE = '\033[0;34m'
    BOLD = '\033[1m'
    NC = '\033[0m'  # No Color

# Enable ANSI colors on Windows
if platform.system() == 'Windows':
    os.system('')

ERRORS = 0
WARNINGS = 0


def print_header():
    """Print the script header."""
    print(f"{Colors.BLUE}╔════════════════════════════════════════════╗{Colors.NC}")
    print(f"{Colors.BLUE}║   🔍 Dependency & Prerequisites Check      ║{Colors.NC}")
    print(f"{Colors.BLUE}╚════════════════════════════════════════════╝{Colors.NC}")
    print()


def run_command(cmd, capture_output=True):
    """Run a command and return the output."""
    try:
        result = subprocess.run(
            cmd,
            capture_output=capture_output,
            text=True,
            shell=True if platform.system() == 'Windows' else False
        )
        return result.returncode == 0, result.stdout.strip() if result.stdout else ""
    except Exception:
        return False, ""


def get_version(cmd):
    """Get version string from a command."""
    try:
        if platform.system() == 'Windows':
            result = subprocess.run(
                f"{cmd} --version",
                capture_output=True,
                text=True,
                shell=True
            )
        else:
            result = subprocess.run(
                [cmd, "--version"],
                capture_output=True,
                text=True
            )
        if result.returncode == 0:
            return result.stdout.strip().split('\n')[0]
        return None
    except Exception:
        return None


def check_command(cmd, name, install_hint, required=True):
    """Check if a command exists and return status."""
    global ERRORS, WARNINGS
    
    path = shutil.which(cmd)
    
    if path:
        version = get_version(cmd)
        print(f"{Colors.GREEN}✓{Colors.NC} {Colors.BOLD}{name}{Colors.NC} - Found")
        if version:
            print(f"  {Colors.BLUE}→{Colors.NC} {version}")
        return True
    else:
        if required:
            print(f"{Colors.RED}✗{Colors.NC} {Colors.BOLD}{name}{Colors.NC} - {Colors.RED}NOT FOUND (REQUIRED){Colors.NC}")
            print(f"  {Colors.YELLOW}→{Colors.NC} Install: {install_hint}")
            ERRORS += 1
            return False
        else:
            print(f"{Colors.YELLOW}⚠{Colors.NC} {Colors.BOLD}{name}{Colors.NC} - {Colors.YELLOW}NOT FOUND (OPTIONAL){Colors.NC}")
            print(f"  {Colors.BLUE}→{Colors.NC} Install: {install_hint}")
            WARNINGS += 1
            return False


def check_rust_toolchain():
    """Check Rust toolchain components."""
    global ERRORS
    
    if not shutil.which("rustc"):
        return
    
    print()
    print(f"{Colors.BLUE}📦 Checking Rust Toolchain...{Colors.NC}")
    
    # Check cargo
    if shutil.which("cargo"):
        version = get_version("cargo")
        print(f"{Colors.GREEN}✓{Colors.NC} Cargo - {version}")
    else:
        print(f"{Colors.RED}✗{Colors.NC} Cargo - NOT FOUND")
        ERRORS += 1
    
    # Check rustup
    if shutil.which("rustup"):
        version = get_version("rustup")
        if version:
            print(f"{Colors.GREEN}✓{Colors.NC} Rustup - {version}")
        
        # Check active toolchain
        try:
            if platform.system() == 'Windows':
                result = subprocess.run(
                    "rustup show active-toolchain",
                    capture_output=True,
                    text=True,
                    shell=True
                )
            else:
                result = subprocess.run(
                    ["rustup", "show", "active-toolchain"],
                    capture_output=True,
                    text=True
                )
            if result.returncode == 0:
                toolchain = result.stdout.strip()
                print(f"  {Colors.BLUE}→{Colors.NC} Active toolchain: {toolchain}")
        except Exception:
            pass
    else:
        print(f"{Colors.YELLOW}⚠{Colors.NC} Rustup - NOT FOUND (recommended for updates)")


def check_system_libs():
    """Check system libraries."""
    global ERRORS, WARNINGS
    
    print()
    print(f"{Colors.BLUE}📚 Checking System Libraries...{Colors.NC}")
    
    system = platform.system()
    
    if system == "Linux":
        # Check pkg-config
        if shutil.which("pkg-config"):
            print(f"{Colors.GREEN}✓{Colors.NC} pkg-config - Found")
            
            # Check for required GUI libraries
            gui_libs = ["x11", "xext", "xcursor", "xrandr", "xi"]
            missing_libs = []
            
            for lib in gui_libs:
                try:
                    result = subprocess.run(
                        ["pkg-config", "--exists", lib],
                        capture_output=True
                    )
                    if result.returncode == 0:
                        print(f"{Colors.GREEN}✓{Colors.NC} lib{lib} - Found")
                    else:
                        print(f"{Colors.RED}✗{Colors.NC} lib{lib} - NOT FOUND")
                        missing_libs.append(f"lib{lib}-dev")
                except Exception:
                    pass
            
            if missing_libs:
                print()
                print(f"{Colors.YELLOW}Missing GUI libraries. Install with:{Colors.NC}")
                print(f"  {Colors.BOLD}sudo apt install {' '.join(missing_libs)}{Colors.NC}")
                ERRORS += 1
        else:
            print(f"{Colors.YELLOW}⚠{Colors.NC} pkg-config - NOT FOUND (cannot verify libraries)")
            WARNINGS += 1
        
        # Check for fontconfig
        if shutil.which("pkg-config"):
            try:
                result = subprocess.run(
                    ["pkg-config", "--exists", "fontconfig"],
                    capture_output=True
                )
                if result.returncode == 0:
                    print(f"{Colors.GREEN}✓{Colors.NC} fontconfig - Found")
                else:
                    print(f"{Colors.RED}✗{Colors.NC} fontconfig - NOT FOUND")
                    print(f"  {Colors.YELLOW}→{Colors.NC} Install: sudo apt install libfontconfig1-dev")
                    ERRORS += 1
            except Exception:
                pass
    
    elif system == "Darwin":
        print(f"{Colors.GREEN}✓{Colors.NC} macOS detected - GUI libraries built-in")
    
    elif system == "Windows":
        print(f"{Colors.GREEN}✓{Colors.NC} Windows detected - GUI libraries built-in")


def check_disk_space():
    """Check available disk space."""
    global ERRORS, WARNINGS
    
    print()
    print(f"{Colors.BLUE}💾 Checking Disk Space...{Colors.NC}")
    
    try:
        if platform.system() == 'Windows':
            import ctypes
            free_bytes = ctypes.c_ulonglong(0)
            ctypes.windll.kernel32.GetDiskFreeSpaceExW(
                ctypes.c_wchar_p(os.getcwd()),
                None,
                None,
                ctypes.pointer(free_bytes)
            )
            available_mb = free_bytes.value // (1024 * 1024)
        else:
            stat = os.statvfs('.')
            available_mb = (stat.f_bavail * stat.f_frsize) // (1024 * 1024)
        
        if available_mb > 1000:
            print(f"{Colors.GREEN}✓{Colors.NC} Available space: {available_mb} MB")
        elif available_mb > 500:
            print(f"{Colors.YELLOW}⚠{Colors.NC} Available space: {available_mb} MB (low)")
            WARNINGS += 1
        else:
            print(f"{Colors.RED}✗{Colors.NC} Available space: {available_mb} MB (insufficient)")
            print(f"  {Colors.YELLOW}→{Colors.NC} Recommended: At least 500 MB free")
            ERRORS += 1
    except Exception as e:
        print(f"{Colors.YELLOW}⚠{Colors.NC} Could not determine disk space: {e}")
        WARNINGS += 1


def check_usb_support():
    """Check USB drive detection capability."""
    global ERRORS, WARNINGS
    
    print()
    print(f"{Colors.BLUE}🔌 Checking USB Support...{Colors.NC}")
    
    system = platform.system()
    
    if system == "Linux":
        if os.path.isdir("/dev/disk/by-id"):
            print(f"{Colors.GREEN}✓{Colors.NC} USB device detection available")
        else:
            print(f"{Colors.YELLOW}⚠{Colors.NC} Cannot access /dev/disk/by-id")
            print(f"  {Colors.BLUE}→{Colors.NC} May need elevated permissions for drive detection")
            WARNINGS += 1
        
        # Check if user is in plugdev group
        try:
            result = subprocess.run(["groups"], capture_output=True, text=True)
            if "plugdev" in result.stdout:
                print(f"{Colors.GREEN}✓{Colors.NC} User in 'plugdev' group - can access USB devices")
            else:
                print(f"{Colors.YELLOW}⚠{Colors.NC} User not in 'plugdev' group")
                print(f"  {Colors.BLUE}→{Colors.NC} Add with: sudo usermod -aG plugdev $USER")
                print(f"  {Colors.BLUE}→{Colors.NC} Then logout/login for changes to take effect")
                WARNINGS += 1
        except Exception:
            pass
    
    elif system == "Darwin":
        if os.path.isdir("/Volumes"):
            print(f"{Colors.GREEN}✓{Colors.NC} USB mount point available (/Volumes)")
        else:
            print(f"{Colors.RED}✗{Colors.NC} Cannot access /Volumes")
            ERRORS += 1
    
    elif system == "Windows":
        # On Windows, USB drives are accessible via drive letters
        print(f"{Colors.GREEN}✓{Colors.NC} Windows drive detection available")
        
        # List available drives
        try:
            import string
            available_drives = []
            for letter in string.ascii_uppercase:
                drive = f"{letter}:\\"
                if os.path.exists(drive):
                    available_drives.append(f"{letter}:")
            if available_drives:
                print(f"  {Colors.BLUE}→{Colors.NC} Available drives: {', '.join(available_drives)}")
        except Exception:
            pass


def print_summary():
    """Print the summary of checks."""
    global ERRORS, WARNINGS
    
    print()
    print(f"{Colors.BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{Colors.NC}")
    print(f"{Colors.BOLD}Summary:{Colors.NC}")
    print()
    
    if ERRORS == 0 and WARNINGS == 0:
        print(f"{Colors.GREEN}✨ All checks passed! You're ready to install.{Colors.NC}")
        print()
        print(f"{Colors.BLUE}Next step:{Colors.NC}")
        if platform.system() == 'Windows':
            print(f"  Run: {Colors.BOLD}python start.py{Colors.NC}")
        else:
            print(f"  Run: {Colors.BOLD}./start.sh{Colors.NC}")
        return 0
    
    elif ERRORS == 0:
        print(f"{Colors.YELLOW}⚠️  {WARNINGS} warning(s) found{Colors.NC}")
        print(f"{Colors.GREEN}✓ All required dependencies are installed{Colors.NC}")
        print()
        print(f"{Colors.BLUE}You can proceed, but some features may be limited.{Colors.NC}")
        print()
        print(f"{Colors.BLUE}Next step:{Colors.NC}")
        if platform.system() == 'Windows':
            print(f"  Run: {Colors.BOLD}python start.py{Colors.NC}")
        else:
            print(f"  Run: {Colors.BOLD}./start.sh{Colors.NC}")
        return 0
    
    else:
        print(f"{Colors.RED}✗ {ERRORS} error(s) found{Colors.NC}")
        if WARNINGS > 0:
            print(f"{Colors.YELLOW}⚠️  {WARNINGS} warning(s) found{Colors.NC}")
        print()
        print(f"{Colors.RED}Please install required dependencies before proceeding.{Colors.NC}")
        return 1


def main():
    """Main function."""
    print_header()
    
    print(f"{Colors.BOLD}Checking Required Dependencies...{Colors.NC}")
    print()
    
    # Determine install hints based on OS
    system = platform.system()
    if system == 'Windows':
        rust_install = "https://www.rust-lang.org/tools/install (download rustup-init.exe)"
        git_install = "https://git-scm.com/download/win"
    elif system == 'Darwin':
        rust_install = "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        git_install = "xcode-select --install"
    else:
        rust_install = "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        git_install = "sudo apt install git"
    
    # Required dependencies
    check_command("rustc", "Rust Compiler", rust_install, required=True)
    check_command("cargo", "Cargo (Rust Package Manager)", rust_install, required=True)
    
    # Optional but recommended
    print()
    print(f"{Colors.BOLD}Checking Optional Dependencies...{Colors.NC}")
    print()
    check_command("git", "Git", git_install, required=False)
    check_command("rustup", "Rustup (Rust Toolchain Manager)", rust_install, required=False)
    
    # Additional checks
    if shutil.which("rustc"):
        check_rust_toolchain()
    
    check_system_libs()
    check_disk_space()
    check_usb_support()
    
    # Print summary and exit
    return print_summary()


if __name__ == "__main__":
    sys.exit(main())
